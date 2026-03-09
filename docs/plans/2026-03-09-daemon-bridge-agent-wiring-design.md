# Daemon Bridge + Platform Agent Wiring

**Date:** 2026-03-09
**Status:** Approved
**Scope:** Wire daemon modules together, implement platform agent service, update CLI wire format

## Overview

The daemon (`crates/daemon/`) has all modules implemented but not wired together in `main.rs`. The platform's WebSocket handler (`/ws/daemon`) only handles ping/pong. This design connects the full chain: CLI/Mac App → daemon → platform → Dr. Bob agent → LiteLLM gateway → Kimi K2.5.

## Architecture

```
┌──────────┐  WS:/chat  ┌──────────────┐  WS:/ws/daemon  ┌───────────────┐  HTTP  ┌─────────┐
│  CLI /   │ ──────────► │   Daemon     │ ───────────────► │   Platform    │ ─────► │ Gateway │
│  Mac App │ ◄────────── │  (localhost) │ ◄─────────────── │  (Cloud Run)  │ ◄───── │ LiteLLM │
└──────────┘  broadcast  │              │  v1 ChatMessage  │               │        └─────────┘
                         │  - Redactor  │                  │  - ws_daemon  │            │
                         │  - ChatRelay │                  │  - DrBobAgent │            ▼
                         │  - REST API  │                  │  - Sessions   │     ┌───────────┐
                         │  - SQLite    │                  │  - Redis      │     │ Kimi K2.5 │
                         └──────────────┘                  └───────────────┘     └───────────┘
```

## Wire Format

All WS messages use ChatMessage v1 (from `chat_relay.rs`):

```json
{
  "v": 1,
  "id": "uuid",
  "ts": 1234567890000,
  "type": "user_message|agent_response|stream_chunk|stream_end|session_init|error",
  "payload": {
    "session_id": "uuid",
    "content": "...",
    "metadata": {}
  }
}
```

## Component Design

### 1. Daemon `main.rs` — Module Wiring

Initialize and connect all existing modules:

1. **Load config** from `~/.d1doctor/config.toml` (defaults if missing)
2. **Create Redactor** from `RedactionConfig`
3. **Open SQLite** via `local_db::open_database()`
4. **Create ChatRelay** — returns `(relay, cloud_rx)` channel pair
5. **Start Axum server** on `0.0.0.0:{daemon_port}`:
   - `GET /api/health` — existing REST handler
   - `GET /api/memory/search` — existing REST handler
   - `GET /api/connection/status` — existing REST handler
   - `WS /chat` — new handler bridging local clients to ChatRelay
6. **Spawn CloudWsClient** — connects to `config.orchestrator_url`
7. **Cloud writer task** — reads `cloud_rx`, redacts, sends to cloud WS
8. **Cloud reader task** — reads cloud WS, forwards to `relay.send_to_local()`
9. **Detect system profile** on startup (background)

### 2. Daemon `/chat` WebSocket Handler

New Axum WS handler for local client connections:

- On connect: `relay.subscribe_local()` for broadcast receiver
- On client message: parse as `ChatMessage`, redact content via `Redactor`, `relay.send_to_cloud(msg)`
- Spawn reader task: `broadcast_rx.recv()` → filter by `session_id` → forward to client WS
- On disconnect: drop broadcast subscription (automatic cleanup)

### 3. Platform `/ws/daemon` — Full Rewrite

**Auth handshake** (matches `cloud_ws.rs` client):

```
Daemon → AUTH { jwt, device_fingerprint }
Platform → verifies JWT against Supabase JWT secret
Platform → AUTH_OK or AUTH_FAIL { reason }
```

**Message routing**:

- `session_init` → Create `DrBobAgent(locale=payload.metadata.locale)`, register in session registry
- `user_message` → Lookup agent by session_id, call `agent_service.chat()`, stream response
- `HEARTBEAT` → Respond with `HEARTBEAT_ACK`
- Unknown → Respond with `error` ChatMessage

**Session registry** (in-memory + Redis):

- Key: `session_id` → `{ agent: DrBobAgent, history: list[Message], created_at, last_active }`
- In-memory dict for active sessions on this instance
- Redis backup for reconnection within TTL (30 min default)
- Cleanup on WS disconnect: persist to Redis, remove from memory

### 4. Platform Agent Service (new)

`orchestrator/src/services/agent_service.py`:

```python
class AgentService:
    def __init__(self, gateway_url: str, gateway_key: str):
        self.client = openai.AsyncOpenAI(base_url=gateway_url, api_key=gateway_key)

    async def chat_stream(self, agent: DrBobAgent, session_id: str,
                          user_message: str, history: list) -> AsyncIterator[ChatMessage]:
        messages = [
            {"role": "system", "content": agent.system_prompt},
            *history,
            {"role": "user", "content": user_message},
        ]
        stream = await self.client.chat.completions.create(
            model="kimi-k2-5",
            messages=messages,
            stream=True,
        )
        async for chunk in stream:
            token = chunk.choices[0].delta.content or ""
            if token:
                yield ChatMessage(type=stream_chunk, payload={session_id, content=token})
        yield ChatMessage(type=stream_end, payload={session_id, content=""})
```

### 5. CLI Update

Update `crates/cli/src/chat/connection.rs`:

- Replace `Envelope` with `ChatMessage` v1 wire format
- On connect: send `session_init` with locale
- `send_and_stream()`: send `user_message`, read `stream_chunk` messages, accumulate and print tokens incrementally
- On `stream_end`: finalize response
- On `error`: display error

Update `crates/cli/src/chat/display.rs`:

- Add incremental token printing (print each chunk as it arrives, no newline until stream_end)

### 6. Terraform

Add `GATEWAY_SERVICE_URL` to platform Cloud Run container:

```hcl
env {
  name  = "GATEWAY_SERVICE_URL"
  value = google_cloud_run_v2_service.gateway.uri
}
env {
  name  = "LITELLM_MASTER_KEY"
  value = var.LITELLM_MASTER_KEY
}
```

The platform already has `MOONSHOT_API_KEY` but doesn't need it directly — the gateway handles LLM provider keys. Platform only needs the gateway URL + master key for auth.

## Error Handling

| Error | Response | Session Impact |
|-------|----------|----------------|
| AUTH failure | `AUTH_FAIL` + close WS | No session created |
| No session_init | `error`: "Send session_init first" | Connection stays open |
| LLM timeout | `error`: timeout details | Session stays alive |
| LLM API error | `error`: API error details | Session stays alive |
| Rate limit | `error`: "Rate limit exceeded" | Session stays alive |
| Gateway unreachable | `error`: "Service unavailable" | Session stays alive |

## Changes Summary

| Component | File(s) | Change |
|-----------|---------|--------|
| Daemon main.rs | `crates/daemon/src/main.rs` | Wire all modules: config, redactor, SQLite, Axum, CloudWsClient |
| Daemon /chat handler | `crates/daemon/src/main.rs` | Axum WS handler bridging ChatRelay ↔ local clients |
| CLI connection | `crates/cli/src/chat/connection.rs` | Switch from Envelope to ChatMessage v1, session_init, streaming |
| CLI display | `crates/cli/src/chat/display.rs` | Incremental token printing for stream_chunk |
| Platform ws_daemon | `orchestrator/src/api/ws_daemon.py` | Full rewrite: AUTH, session registry, message routing |
| Platform agent_service | `orchestrator/src/services/agent_service.py` | New: DrBobAgent + gateway streaming LLM calls |
| Platform session_registry | `orchestrator/src/services/session_registry.py` | New: in-memory + Redis session storage |
| Terraform cloud_run.tf | `infra/terraform/cloud_run.tf` | Add GATEWAY_SERVICE_URL + LITELLM_MASTER_KEY to platform; remove unused PLATFORM_SERVICE_URL from gateway (breaks cycle) |

## Non-Goals (follow-up work)

- Command execution flow (Command → CommandResult) — daemon executor is built but not wired to platform yet
- Multi-agent delegation (Dr. Bob → Coder/Researcher/Planner) — agents exist but routing is not implemented
- MCP tool integration during agent execution
- Conversation history persistence to Supabase (Redis-only for now)
