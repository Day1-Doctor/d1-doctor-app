# Daemon Bridge + Agent Wiring — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Wire the daemon, platform, and CLI together so CLI users can chat with Dr. Bob via the full chain: CLI → daemon → platform → LiteLLM gateway → Kimi K2.5.

**Architecture:** The daemon listens on localhost:9876 for local WS connections (`/chat`), relays messages through a cloud WS to the platform (`/ws/daemon`). The platform authenticates daemons, manages sessions, and streams LLM responses back. All messages use ChatMessage v1 wire format.

**Tech Stack:** Rust (Axum, tokio-tungstenite) for daemon/CLI, Python (FastAPI, litellm) for platform, Terraform for deploy.

**Design doc:** `d1-doctor-app/docs/plans/2026-03-09-daemon-bridge-agent-wiring-design.md`

---

## Task 1: Fix Terraform Env Var Names (Platform D1_ Prefix)

The platform uses `env_prefix = "D1_"` in Pydantic Settings (`orchestrator/src/core/config.py:47`), but Terraform passes `GATEWAY_SERVICE_URL` and `LITELLM_MASTER_KEY` without the prefix. Also `MOONSHOT_API_KEY` is passed to platform but platform doesn't use it (gateway handles LLM keys).

**Files:**
- Modify: `d1-doctor-platform/infra/terraform/cloud_run.tf:60-67`
- Modify: `d1-doctor-platform/orchestrator/src/core/config.py:38-40`

**Step 1: Add `litellm_master_key` field to platform Settings**

In `orchestrator/src/core/config.py`, add after line 40:

```python
    litellm_master_key: str = ""
```

The existing `litellm_api_base` field (line 39) maps to `D1_LITELLM_API_BASE`. The new `litellm_master_key` maps to `D1_LITELLM_MASTER_KEY`.

**Step 2: Fix Terraform env var names**

In `d1-doctor-platform/infra/terraform/cloud_run.tf`, replace lines 52-67:

```hcl
      # MOONSHOT_API_KEY removed — platform doesn't need it; gateway handles LLM keys
      env {
        name  = "D1_LITELLM_API_BASE"
        value = "${google_cloud_run_v2_service.gateway.uri}/v1"
      }
      env {
        name  = "D1_LITELLM_MASTER_KEY"
        value = var.LITELLM_MASTER_KEY
      }
```

**Step 3: Verify locally**

Run: `cd d1-doctor-platform/infra/terraform && terraform plan`
Expected: Changes only to platform Cloud Run env vars (2 renamed, 1 removed).

**Step 4: Commit**

```bash
git add infra/terraform/cloud_run.tf orchestrator/src/core/config.py
git commit -m "fix: rename platform env vars to match D1_ prefix convention"
```

---

## Task 2: Platform Session Registry

New service managing in-memory + Redis-backed sessions for active daemon WebSocket connections.

**Files:**
- Create: `d1-doctor-platform/orchestrator/src/services/session_registry.py`
- Test: `d1-doctor-platform/orchestrator/tests/test_session_registry.py`

**Step 1: Write the failing test**

Create `orchestrator/tests/test_session_registry.py`:

```python
"""Tests for session registry."""

import pytest
from unittest.mock import AsyncMock, patch

from orchestrator.src.services.session_registry import SessionRegistry
from orchestrator.src.agents.dr_bob import DrBobAgent


class TestSessionRegistry:
    def setup_method(self):
        self.registry = SessionRegistry()

    def test_create_session(self):
        session_id = "sess-1"
        agent = DrBobAgent(locale="en")
        self.registry.create(session_id, agent)
        assert self.registry.get(session_id) is not None

    def test_get_missing_session_returns_none(self):
        assert self.registry.get("nonexistent") is None

    def test_remove_session(self):
        session_id = "sess-1"
        agent = DrBobAgent(locale="en")
        self.registry.create(session_id, agent)
        self.registry.remove(session_id)
        assert self.registry.get(session_id) is None

    def test_add_message_to_history(self):
        session_id = "sess-1"
        agent = DrBobAgent(locale="en")
        self.registry.create(session_id, agent)
        self.registry.add_message(session_id, {"role": "user", "content": "hello"})
        self.registry.add_message(session_id, {"role": "assistant", "content": "hi"})
        session = self.registry.get(session_id)
        assert len(session["history"]) == 2

    def test_get_returns_agent_and_history(self):
        session_id = "sess-1"
        agent = DrBobAgent(locale="zh-CN")
        self.registry.create(session_id, agent)
        session = self.registry.get(session_id)
        assert session["agent"].locale == "zh-CN"
        assert session["history"] == []

    @pytest.mark.asyncio
    async def test_persist_to_redis(self):
        mock_redis = AsyncMock()
        mock_redis.setex = AsyncMock()
        registry = SessionRegistry(redis=mock_redis, ttl=1800)
        agent = DrBobAgent(locale="en")
        registry.create("sess-1", agent)
        registry.add_message("sess-1", {"role": "user", "content": "hi"})
        await registry.persist("sess-1")
        mock_redis.setex.assert_called_once()

    @pytest.mark.asyncio
    async def test_restore_from_redis(self):
        import json
        mock_redis = AsyncMock()
        stored = json.dumps({
            "locale": "en",
            "model": "kimi-k2-5",
            "history": [{"role": "user", "content": "hello"}],
        })
        mock_redis.get = AsyncMock(return_value=stored)
        registry = SessionRegistry(redis=mock_redis)
        session = await registry.restore("sess-1")
        assert session is not None
        assert len(session["history"]) == 1
```

**Step 2: Run test to verify it fails**

Run: `cd d1-doctor-platform && pytest orchestrator/tests/test_session_registry.py -v`
Expected: FAIL — ImportError (module doesn't exist yet).

**Step 3: Write the implementation**

Create `orchestrator/src/services/session_registry.py`:

```python
"""In-memory + Redis session registry for daemon WebSocket connections.

Each session stores: the DrBobAgent instance, conversation history,
and timestamps. Sessions are kept in memory for active connections
and persisted to Redis on disconnect for reconnection within TTL.
"""

from __future__ import annotations

import json
import logging
import time
from typing import Any

from redis.asyncio import Redis

from ..agents.dr_bob import DrBobAgent

logger = logging.getLogger(__name__)

DEFAULT_TTL = 1800  # 30 minutes


class SessionRegistry:
    """Manages chat sessions for connected daemons."""

    def __init__(self, redis: Redis | None = None, ttl: int = DEFAULT_TTL):
        self._sessions: dict[str, dict[str, Any]] = {}
        self._redis = redis
        self._ttl = ttl

    def create(self, session_id: str, agent: DrBobAgent) -> None:
        """Create a new session with the given agent."""
        self._sessions[session_id] = {
            "agent": agent,
            "history": [],
            "created_at": time.time(),
            "last_active": time.time(),
        }
        logger.info("Session created: %s (locale=%s)", session_id, agent.locale)

    def get(self, session_id: str) -> dict[str, Any] | None:
        """Get session data, or None if not found."""
        session = self._sessions.get(session_id)
        if session:
            session["last_active"] = time.time()
        return session

    def add_message(self, session_id: str, message: dict[str, str]) -> None:
        """Append a message to the session history."""
        session = self._sessions.get(session_id)
        if session:
            session["history"].append(message)
            session["last_active"] = time.time()

    def remove(self, session_id: str) -> None:
        """Remove a session from memory."""
        self._sessions.pop(session_id, None)
        logger.info("Session removed: %s", session_id)

    async def persist(self, session_id: str) -> None:
        """Persist session to Redis for reconnection."""
        if not self._redis:
            return
        session = self._sessions.get(session_id)
        if not session:
            return
        data = json.dumps({
            "locale": session["agent"].locale,
            "model": session["agent"].model,
            "history": session["history"],
        })
        key = f"d1:session:{session_id}"
        await self._redis.setex(key, self._ttl, data)
        logger.info("Session persisted to Redis: %s", session_id)

    async def restore(self, session_id: str) -> dict[str, Any] | None:
        """Restore session from Redis."""
        if not self._redis:
            return None
        key = f"d1:session:{session_id}"
        raw = await self._redis.get(key)
        if not raw:
            return None
        data = json.loads(raw)
        agent = DrBobAgent(locale=data["locale"], model=data.get("model", "kimi-k2-5"))
        self.create(session_id, agent)
        session = self._sessions[session_id]
        session["history"] = data["history"]
        logger.info("Session restored from Redis: %s", session_id)
        return session
```

**Step 4: Run test to verify it passes**

Run: `cd d1-doctor-platform && pytest orchestrator/tests/test_session_registry.py -v`
Expected: All 7 tests PASS.

**Step 5: Commit**

```bash
git add orchestrator/src/services/session_registry.py orchestrator/tests/test_session_registry.py
git commit -m "feat: add session registry with in-memory + Redis persistence"
```

---

## Task 3: Platform Agent Service (LLM Streaming)

New service that calls the LiteLLM gateway to stream Dr. Bob responses.

**Files:**
- Create: `d1-doctor-platform/orchestrator/src/services/agent_service.py`
- Test: `d1-doctor-platform/orchestrator/tests/test_agent_service.py`

**Step 1: Write the failing test**

Create `orchestrator/tests/test_agent_service.py`:

```python
"""Tests for agent service — LLM streaming via gateway."""

import pytest
from unittest.mock import AsyncMock, MagicMock, patch

from orchestrator.src.services.agent_service import AgentService
from orchestrator.src.agents.dr_bob import DrBobAgent


class TestAgentService:
    def setup_method(self):
        self.service = AgentService(
            gateway_url="http://gateway:8080/v1",
            gateway_key="sk-test-key",
        )

    def test_build_messages(self):
        agent = DrBobAgent(locale="en")
        history = [{"role": "user", "content": "hi"}, {"role": "assistant", "content": "hello"}]
        messages = self.service._build_messages(agent, "new question", history)
        assert messages[0]["role"] == "system"
        assert messages[0]["content"] == agent.system_prompt
        assert messages[1] == {"role": "user", "content": "hi"}
        assert messages[2] == {"role": "assistant", "content": "hello"}
        assert messages[3] == {"role": "user", "content": "new question"}

    def test_build_messages_empty_history(self):
        agent = DrBobAgent(locale="en")
        messages = self.service._build_messages(agent, "hello", [])
        assert len(messages) == 2
        assert messages[0]["role"] == "system"
        assert messages[1] == {"role": "user", "content": "hello"}

    @pytest.mark.asyncio
    async def test_chat_stream_yields_chunks(self):
        agent = DrBobAgent(locale="en")

        # Mock litellm.acompletion streaming response
        chunk1 = MagicMock()
        chunk1.choices = [MagicMock()]
        chunk1.choices[0].delta.content = "Hello"

        chunk2 = MagicMock()
        chunk2.choices = [MagicMock()]
        chunk2.choices[0].delta.content = " world"

        chunk3 = MagicMock()
        chunk3.choices = [MagicMock()]
        chunk3.choices[0].delta.content = None

        async def mock_stream():
            for chunk in [chunk1, chunk2, chunk3]:
                yield chunk

        with patch("orchestrator.src.services.agent_service.litellm") as mock_litellm:
            mock_litellm.acompletion = AsyncMock(return_value=mock_stream())
            tokens = []
            async for token in self.service.chat_stream(
                agent=agent,
                session_id="sess-1",
                user_message="hi",
                history=[],
            ):
                tokens.append(token)

        # Should yield "Hello", " world", then stream_end
        assert len(tokens) == 3
        assert tokens[0]["type"] == "stream_chunk"
        assert tokens[0]["content"] == "Hello"
        assert tokens[1]["type"] == "stream_chunk"
        assert tokens[1]["content"] == " world"
        assert tokens[2]["type"] == "stream_end"

    @pytest.mark.asyncio
    async def test_chat_stream_handles_error(self):
        agent = DrBobAgent(locale="en")

        with patch("orchestrator.src.services.agent_service.litellm") as mock_litellm:
            mock_litellm.acompletion = AsyncMock(side_effect=Exception("Gateway timeout"))
            tokens = []
            async for token in self.service.chat_stream(
                agent=agent,
                session_id="sess-1",
                user_message="hi",
                history=[],
            ):
                tokens.append(token)

        assert len(tokens) == 1
        assert tokens[0]["type"] == "error"
        assert "Gateway timeout" in tokens[0]["content"]
```

**Step 2: Run test to verify it fails**

Run: `cd d1-doctor-platform && pytest orchestrator/tests/test_agent_service.py -v`
Expected: FAIL — ImportError.

**Step 3: Write the implementation**

Create `orchestrator/src/services/agent_service.py`:

```python
"""Agent service — streams LLM responses from the LiteLLM gateway.

Uses litellm.acompletion() with api_base pointed at our gateway.
The gateway routes to the correct LLM provider (Kimi K2.5 by default).
"""

from __future__ import annotations

import logging
from typing import Any, AsyncIterator

import litellm

from ..agents.dr_bob import DrBobAgent

logger = logging.getLogger(__name__)


class AgentService:
    """Handles LLM calls via the LiteLLM gateway proxy."""

    def __init__(self, gateway_url: str, gateway_key: str):
        self._gateway_url = gateway_url
        self._gateway_key = gateway_key

    def _build_messages(
        self,
        agent: DrBobAgent,
        user_message: str,
        history: list[dict[str, str]],
    ) -> list[dict[str, str]]:
        """Build the messages array for the LLM call."""
        return [
            {"role": "system", "content": agent.system_prompt},
            *history,
            {"role": "user", "content": user_message},
        ]

    async def chat_stream(
        self,
        agent: DrBobAgent,
        session_id: str,
        user_message: str,
        history: list[dict[str, str]],
    ) -> AsyncIterator[dict[str, Any]]:
        """Stream LLM response tokens.

        Yields dicts with keys: type ("stream_chunk"|"stream_end"|"error"),
        session_id, content.
        """
        messages = self._build_messages(agent, user_message, history)

        try:
            response = await litellm.acompletion(
                model=agent.model,
                messages=messages,
                stream=True,
                api_base=self._gateway_url,
                api_key=self._gateway_key,
            )
            async for chunk in response:
                token = chunk.choices[0].delta.content
                if token:
                    yield {
                        "type": "stream_chunk",
                        "session_id": session_id,
                        "content": token,
                    }
            yield {
                "type": "stream_end",
                "session_id": session_id,
                "content": "",
            }
        except Exception as e:
            logger.error("LLM streaming error for session %s: %s", session_id, e)
            yield {
                "type": "error",
                "session_id": session_id,
                "content": str(e),
            }
```

**Step 4: Run test to verify it passes**

Run: `cd d1-doctor-platform && pytest orchestrator/tests/test_agent_service.py -v`
Expected: All 4 tests PASS.

**Step 5: Commit**

```bash
git add orchestrator/src/services/agent_service.py orchestrator/tests/test_agent_service.py
git commit -m "feat: add agent service for streaming LLM calls via gateway"
```

---

## Task 4: Platform ws_daemon.py — Full Rewrite

Replace the ping/pong stub with AUTH handshake, session management, and streaming message routing.

**Files:**
- Modify: `d1-doctor-platform/orchestrator/src/api/ws_daemon.py`
- Test: `d1-doctor-platform/orchestrator/tests/test_ws_daemon.py`

**Step 1: Write the failing test**

Create `orchestrator/tests/test_ws_daemon.py`:

```python
"""Tests for ws_daemon WebSocket handler."""

import json
import uuid

import pytest
from unittest.mock import AsyncMock, MagicMock, patch

from orchestrator.src.api.ws_daemon import (
    handle_auth,
    handle_message,
    make_chat_message,
)


class TestChatMessageFactory:
    def test_make_chat_message(self):
        msg = make_chat_message("stream_chunk", "sess-1", "hello")
        assert msg["v"] == 1
        assert msg["type"] == "stream_chunk"
        assert msg["payload"]["session_id"] == "sess-1"
        assert msg["payload"]["content"] == "hello"
        assert "id" in msg
        assert "ts" in msg

    def test_make_error_message(self):
        msg = make_chat_message("error", "sess-1", "something broke")
        assert msg["type"] == "error"
        assert msg["payload"]["content"] == "something broke"


class TestAuth:
    @pytest.mark.asyncio
    async def test_valid_jwt_returns_true(self):
        ws = AsyncMock()
        auth_msg = {
            "v": 1,
            "id": str(uuid.uuid4()),
            "ts": 1234567890000,
            "type": "AUTH",
            "payload": {
                "jwt": "valid.jwt.token",
                "device_fingerprint": "fp-123",
            },
        }
        with patch(
            "orchestrator.src.api.ws_daemon.verify_jwt", return_value={"sub": "user-1"}
        ):
            result = await handle_auth(ws, auth_msg)
        assert result == {"sub": "user-1"}
        ws.send_text.assert_called_once()
        sent = json.loads(ws.send_text.call_args[0][0])
        assert sent["type"] == "AUTH_OK"

    @pytest.mark.asyncio
    async def test_invalid_jwt_returns_none(self):
        ws = AsyncMock()
        auth_msg = {
            "v": 1,
            "id": str(uuid.uuid4()),
            "ts": 1234567890000,
            "type": "AUTH",
            "payload": {"jwt": "bad.token", "device_fingerprint": "fp-123"},
        }
        with patch(
            "orchestrator.src.api.ws_daemon.verify_jwt", return_value=None
        ):
            result = await handle_auth(ws, auth_msg)
        assert result is None
        ws.send_text.assert_called_once()
        sent = json.loads(ws.send_text.call_args[0][0])
        assert sent["type"] == "AUTH_FAIL"


class TestMessageRouting:
    @pytest.mark.asyncio
    async def test_session_init_creates_session(self):
        ws = AsyncMock()
        registry = MagicMock()
        registry.get.return_value = None
        agent_service = MagicMock()
        msg = {
            "v": 1,
            "type": "session_init",
            "payload": {
                "session_id": "sess-1",
                "content": "",
                "metadata": {"locale": "zh-CN"},
            },
        }
        await handle_message(ws, msg, registry, agent_service)
        registry.create.assert_called_once()
        args = registry.create.call_args
        assert args[0][0] == "sess-1"  # session_id
        assert args[0][1].locale == "zh-CN"  # DrBobAgent locale

    @pytest.mark.asyncio
    async def test_user_message_without_session_returns_error(self):
        ws = AsyncMock()
        registry = MagicMock()
        registry.get.return_value = None
        agent_service = MagicMock()
        msg = {
            "v": 1,
            "type": "user_message",
            "payload": {"session_id": "sess-1", "content": "hello"},
        }
        await handle_message(ws, msg, registry, agent_service)
        ws.send_text.assert_called_once()
        sent = json.loads(ws.send_text.call_args[0][0])
        assert sent["type"] == "error"
        assert "session_init" in sent["payload"]["content"]

    @pytest.mark.asyncio
    async def test_user_message_streams_response(self):
        ws = AsyncMock()
        registry = MagicMock()
        from orchestrator.src.agents.dr_bob import DrBobAgent

        agent = DrBobAgent(locale="en")
        registry.get.return_value = {"agent": agent, "history": []}

        async def mock_stream(*args, **kwargs):
            yield {"type": "stream_chunk", "session_id": "sess-1", "content": "Hi"}
            yield {"type": "stream_end", "session_id": "sess-1", "content": ""}

        agent_service = MagicMock()
        agent_service.chat_stream = mock_stream

        msg = {
            "v": 1,
            "type": "user_message",
            "payload": {"session_id": "sess-1", "content": "hello"},
        }
        await handle_message(ws, msg, registry, agent_service)

        assert ws.send_text.call_count == 2
        chunk = json.loads(ws.send_text.call_args_list[0][0][0])
        assert chunk["type"] == "stream_chunk"
        end = json.loads(ws.send_text.call_args_list[1][0][0])
        assert end["type"] == "stream_end"

        # History should be updated
        registry.add_message.assert_any_call("sess-1", {"role": "user", "content": "hello"})
        registry.add_message.assert_any_call("sess-1", {"role": "assistant", "content": "Hi"})

    @pytest.mark.asyncio
    async def test_heartbeat_returns_ack(self):
        ws = AsyncMock()
        registry = MagicMock()
        agent_service = MagicMock()
        msg = {"v": 1, "type": "HEARTBEAT", "payload": {}}
        await handle_message(ws, msg, registry, agent_service)
        ws.send_text.assert_called_once()
        sent = json.loads(ws.send_text.call_args[0][0])
        assert sent["type"] == "HEARTBEAT_ACK"
```

**Step 2: Run test to verify it fails**

Run: `cd d1-doctor-platform && pytest orchestrator/tests/test_ws_daemon.py -v`
Expected: FAIL — ImportError (handle_auth, handle_message, make_chat_message don't exist).

**Step 3: Write the implementation**

Rewrite `orchestrator/src/api/ws_daemon.py`:

```python
"""WebSocket handler for daemon connections.

Full lifecycle: AUTH handshake → session management → message routing → streaming.
Replaces the previous ping/pong stub.
"""

from __future__ import annotations

import json
import logging
import time
import uuid

from fastapi import APIRouter, WebSocket, WebSocketDisconnect

from ..agents.dr_bob import DrBobAgent
from ..core.config import settings
from ..services.agent_service import AgentService
from ..services.redis import get_redis
from ..services.session_registry import SessionRegistry

logger = logging.getLogger(__name__)

router = APIRouter()

# Module-level singletons (initialized on first connection or at import time)
_registry: SessionRegistry | None = None
_agent_service: AgentService | None = None


def _get_registry() -> SessionRegistry:
    global _registry
    if _registry is None:
        _registry = SessionRegistry(redis=get_redis())
    return _registry


def _get_agent_service() -> AgentService:
    global _agent_service
    if _agent_service is None:
        _agent_service = AgentService(
            gateway_url=settings.litellm_api_base,
            gateway_key=settings.litellm_master_key,
        )
    return _agent_service


def make_chat_message(
    msg_type: str, session_id: str, content: str, metadata: dict | None = None
) -> dict:
    """Build a ChatMessage v1 dict."""
    msg = {
        "v": 1,
        "id": str(uuid.uuid4()),
        "ts": int(time.time() * 1000),
        "type": msg_type,
        "payload": {
            "session_id": session_id,
            "content": content,
        },
    }
    if metadata:
        msg["payload"]["metadata"] = metadata
    return msg


def verify_jwt(token: str) -> dict | None:
    """Verify a Supabase JWT and return claims, or None if invalid.

    Uses the HMAC secret from settings.supabase_jwt_secret.
    """
    import jwt as pyjwt

    try:
        claims = pyjwt.decode(
            token,
            settings.supabase_jwt_secret,
            algorithms=["HS256"],
            audience="authenticated",
        )
        return claims
    except Exception as e:
        logger.warning("JWT verification failed: %s", e)
        return None


async def handle_auth(ws: WebSocket, msg: dict) -> dict | None:
    """Process AUTH message. Returns JWT claims on success, None on failure."""
    payload = msg.get("payload", {})
    token = payload.get("jwt", "")
    fingerprint = payload.get("device_fingerprint", "")

    claims = verify_jwt(token)
    if claims:
        logger.info("AUTH_OK for user=%s device=%s", claims.get("sub"), fingerprint)
        resp = make_chat_message("AUTH_OK", "", "")
        await ws.send_text(json.dumps(resp))
        return claims
    else:
        logger.warning("AUTH_FAIL for device=%s", fingerprint)
        resp = make_chat_message("AUTH_FAIL", "", "Invalid or expired token")
        await ws.send_text(json.dumps(resp))
        return None


async def handle_message(
    ws: WebSocket,
    msg: dict,
    registry: SessionRegistry,
    agent_service: AgentService,
) -> None:
    """Route an authenticated message to the appropriate handler."""
    msg_type = msg.get("type", "")
    payload = msg.get("payload", {})
    session_id = payload.get("session_id", "")

    if msg_type == "HEARTBEAT":
        resp = make_chat_message("HEARTBEAT_ACK", "", "")
        await ws.send_text(json.dumps(resp))
        return

    if msg_type == "session_init":
        metadata = payload.get("metadata", {})
        locale = metadata.get("locale", "en")
        agent = DrBobAgent(locale=locale, model="kimi-k2-5")
        registry.create(session_id, agent)
        logger.info("Session initialized: %s (locale=%s)", session_id, locale)
        return

    if msg_type == "user_message":
        session = registry.get(session_id)
        if not session:
            err = make_chat_message(
                "error", session_id, "Send session_init first"
            )
            await ws.send_text(json.dumps(err))
            return

        user_content = payload.get("content", "")
        registry.add_message(session_id, {"role": "user", "content": user_content})

        # Stream LLM response
        full_response = []
        async for token_msg in agent_service.chat_stream(
            agent=session["agent"],
            session_id=session_id,
            user_message=user_content,
            history=session["history"][:-1],  # exclude the just-added user msg
        ):
            chat_msg = make_chat_message(
                token_msg["type"],
                token_msg["session_id"],
                token_msg["content"],
            )
            await ws.send_text(json.dumps(chat_msg))
            if token_msg["type"] == "stream_chunk":
                full_response.append(token_msg["content"])

        # Record assistant response in history
        if full_response:
            registry.add_message(
                session_id,
                {"role": "assistant", "content": "".join(full_response)},
            )
        return

    # Unknown message type
    err = make_chat_message("error", session_id, f"Unknown message type: {msg_type}")
    await ws.send_text(json.dumps(err))


@router.websocket("/daemon")
async def ws_daemon(websocket: WebSocket):
    """Handle authenticated WebSocket connections from daemons."""
    await websocket.accept()

    registry = _get_registry()
    agent_service = _get_agent_service()

    # Step 1: Wait for AUTH message
    try:
        raw = await websocket.receive_text()
        msg = json.loads(raw)
    except (WebSocketDisconnect, json.JSONDecodeError):
        return

    if msg.get("type") != "AUTH":
        err = make_chat_message("error", "", "Expected AUTH message")
        await websocket.send_text(json.dumps(err))
        await websocket.close(code=1008, reason="Expected AUTH")
        return

    claims = await handle_auth(websocket, msg)
    if not claims:
        await websocket.close(code=1008, reason="Authentication failed")
        return

    # Step 2: Message loop
    active_sessions: list[str] = []
    try:
        while True:
            raw = await websocket.receive_text()
            msg = json.loads(raw)
            await handle_message(websocket, msg, registry, agent_service)

            # Track sessions for cleanup
            sid = msg.get("payload", {}).get("session_id", "")
            if sid and msg.get("type") == "session_init" and sid not in active_sessions:
                active_sessions.append(sid)

    except WebSocketDisconnect:
        logger.info("Daemon disconnected (user=%s)", claims.get("sub"))
    except json.JSONDecodeError:
        logger.warning("Invalid JSON from daemon")
    finally:
        # Persist sessions to Redis on disconnect
        for sid in active_sessions:
            try:
                await registry.persist(sid)
            except Exception as e:
                logger.error("Failed to persist session %s: %s", sid, e)
            registry.remove(sid)
```

**Step 4: Run test to verify it passes**

Run: `cd d1-doctor-platform && pytest orchestrator/tests/test_ws_daemon.py -v`
Expected: All tests PASS.

**Step 5: Add PyJWT dependency**

The JWT verification uses `pyjwt`. Check if it's already available via `supabase` transitive dep. If not:

Run: `cd d1-doctor-platform && python -c "import jwt; print(jwt.__version__)"`
If ImportError: add `"PyJWT>=2.8"` to `orchestrator/pyproject.toml` dependencies.

**Step 6: Commit**

```bash
git add orchestrator/src/api/ws_daemon.py orchestrator/tests/test_ws_daemon.py
git commit -m "feat: rewrite ws_daemon with AUTH, sessions, and streaming"
```

---

## Task 5: Daemon main.rs — Wire All Modules

Replace the stub `main.rs` with full module wiring: config, redactor, SQLite, Axum (REST + WS /chat), CloudWsClient, and relay tasks.

**Files:**
- Modify: `d1-doctor-app/crates/daemon/src/main.rs`

**Step 1: Wire main.rs**

Replace the entire `main.rs` content:

```rust
//! Day 1 Doctor — Local Daemon
//!
//! Wires all modules: config → redactor → SQLite → Axum server (REST + WS)
//! → ChatRelay → CloudWsClient → cloud platform.

mod chat_relay;
mod cloud_ws;
mod command_relay;
mod connection_state;
mod executor;
mod filesystem;
mod fingerprint;
mod health;
mod local_db;
mod mcp_filesystem;
mod mcp_memory;
mod mcp_qmd;
mod mcp_registry;
mod mcp_shell;
mod mcp_system;
mod memory_store;
mod profile_detect;
pub mod redactor;
mod rest_api;
mod qmd;
mod security;
mod system_ops;
mod ws_client;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::ws::{Message as AxumWsMessage, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use chat_relay::{ChatMessage, ChatMessageType, ChatRelay};
use cloud_ws::{CloudWsClient, CloudWsConfig};
use d1_common::Config;
use redactor::Redactor;

/// Shared state available to all Axum handlers.
#[derive(Clone)]
struct DaemonState {
    relay: Arc<ChatRelay>,
    redactor: Arc<Redactor>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Day 1 Doctor daemon starting...");

    // 1. Load config
    let config = Config::load().unwrap_or_default();
    info!(
        port = config.daemon_port,
        orchestrator = %config.orchestrator_url,
        "Configuration loaded"
    );

    // 2. Create Redactor
    let redactor = Arc::new(Redactor::from_config(&config.redaction));
    info!("Redactor initialized");

    // 3. Open SQLite database
    let db_path = config.database.path.to_string_lossy().to_string();
    let _db = local_db::LocalDb::open(&db_path)?;
    info!(%db_path, "SQLite database opened");

    // 4. Create ChatRelay
    let (relay, mut cloud_rx) = ChatRelay::new();
    let relay = Arc::new(relay);
    info!("ChatRelay created");

    // 5. Build Axum router
    let state = DaemonState {
        relay: Arc::clone(&relay),
        redactor: Arc::clone(&redactor),
    };

    let app = Router::new()
        .route("/chat", get(ws_chat_handler))
        .route("/api/health", get(rest_api::health_check))
        .route("/api/memory/search", get(rest_api::memory_search))
        .with_state(state);

    let addr: SocketAddr = format!("0.0.0.0:{}", config.daemon_port).parse()?;
    info!(%addr, "Starting Axum server");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!("Axum server error: {}", e);
        }
    });

    // 6. Spawn CloudWsClient
    let cloud_client = CloudWsClient::new();
    let cloud_config = CloudWsConfig {
        url: config.orchestrator_url.clone(),
        jwt: config
            .supabase
            .as_ref()
            .map(|s| s.anon_key.clone())
            .unwrap_or_default(),
        device_fingerprint: fingerprint::get_or_create_fingerprint()
            .unwrap_or_else(|_| "unknown".to_string()),
    };

    let relay_for_state = Arc::clone(&relay);
    let _cloud_handle = cloud_client.spawn(cloud_config);

    // Watch cloud connection state to update relay
    let mut state_rx = cloud_client.subscribe_state();
    let relay_for_watcher = Arc::clone(&relay);
    tokio::spawn(async move {
        while state_rx.changed().await.is_ok() {
            let state = *state_rx.borrow();
            match state {
                cloud_ws::ConnectionState::Connected => {
                    if let Err(e) = relay_for_watcher.set_connected().await {
                        warn!("Failed to flush relay queue: {}", e);
                    }
                    info!("Relay: cloud connected, queue flushed");
                }
                cloud_ws::ConnectionState::Disconnected => {
                    relay_for_watcher.set_disconnected("").await;
                    info!("Relay: cloud disconnected");
                }
                _ => {}
            }
        }
    });

    // 7. Cloud writer task: relay cloud_rx → cloud WS (via redactor)
    // Note: CloudWsClient currently handles its own sending via the message_loop.
    // For now, we log cloud_rx messages. Full integration requires modifying
    // CloudWsClient to accept an mpsc::Sender for outbound messages.
    let redactor_for_writer = Arc::clone(&redactor);
    tokio::spawn(async move {
        while let Some(msg) = cloud_rx.recv().await {
            let json = match serde_json::to_string(&msg) {
                Ok(j) => j,
                Err(e) => {
                    error!("Failed to serialize cloud message: {}", e);
                    continue;
                }
            };
            let redacted = redactor_for_writer.redact(&json);
            info!(len = redacted.len(), "Cloud-bound message redacted and queued");
            // TODO: Send redacted message through CloudWsClient's sink
            // This requires CloudWsClient to expose a send() method
        }
    });

    // 8. Detect system profile (background)
    tokio::spawn(async {
        let facts = profile_detect::detect_system_profile();
        info!(count = facts.len(), "System profile detected");
    });

    info!("Daemon fully initialized, serving on {}", addr);

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received");
    cloud_client.shutdown();
    server.abort();

    Ok(())
}

/// Axum handler: upgrade HTTP to WebSocket for /chat
async fn ws_chat_handler(
    ws: WebSocketUpgrade,
    State(state): State<DaemonState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_chat_ws(socket, state))
}

/// Handle a local client WebSocket connection on /chat.
///
/// - Subscribe to broadcast for cloud→local messages
/// - Forward client messages through relay→cloud (after redaction)
async fn handle_chat_ws(ws: WebSocket, state: DaemonState) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut broadcast_rx = state.relay.subscribe_local();

    info!("Local client connected to /chat");

    // Task: broadcast → client WS
    let tx_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(json) => {
                    if ws_tx.send(AxumWsMessage::Text(json)).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    warn!("Failed to serialize broadcast message: {}", e);
                }
            }
        }
    });

    // Task: client WS → relay (redact → cloud)
    let relay = state.relay;
    let redactor = state.redactor;
    while let Some(Ok(msg)) = ws_rx.next().await {
        if let AxumWsMessage::Text(text) = msg {
            // Parse as ChatMessage
            match serde_json::from_str::<ChatMessage>(&text) {
                Ok(mut chat_msg) => {
                    // Redact content before sending to cloud
                    chat_msg.payload.content =
                        redactor.redact(&chat_msg.payload.content);
                    if let Err(e) = relay.send_to_cloud(chat_msg).await {
                        warn!("Failed to relay message to cloud: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Invalid ChatMessage from client: {}", e);
                }
            }
        }
    }

    tx_task.abort();
    info!("Local client disconnected from /chat");
}
```

**Step 2: Fix rest_api.rs State import**

Add `State` to the axum imports in `rest_api.rs` line 6:

```rust
use axum::{extract::Query, extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
```

**Step 3: Check compilation**

Run: `cd d1-doctor-app && cargo check -p d1-daemon 2>&1 | head -50`

Fix any compilation errors. Common issues:
- Missing `axum::extract::ws` — check Cargo.toml has `axum` with `ws` feature
- `fingerprint` module might not have `get_or_create_fingerprint()` — check and adjust
- `ChatRelay` fields are private — may need to wrap in `Arc` differently

**Step 4: Commit**

```bash
git add crates/daemon/src/main.rs crates/daemon/src/rest_api.rs
git commit -m "feat: wire daemon main.rs with all modules and /chat WS handler"
```

---

## Task 6: CLI Connection — Switch to ChatMessage v1

Replace Envelope-based protocol in CLI with ChatMessage v1 wire format, add `session_init` on connect, and streaming support.

**Files:**
- Modify: `d1-doctor-app/crates/cli/src/chat/connection.rs`
- Modify: `d1-doctor-app/crates/cli/src/chat/display.rs`
- Modify: `d1-doctor-app/crates/cli/src/chat/mod.rs`

**Step 1: Update connection.rs**

Replace the entire `connection.rs`:

```rust
//! Connection management for chat sessions.
//!
//! Uses ChatMessage v1 wire format: { v:1, id, ts, type, payload }.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use d1_daemon::chat_relay::{ChatMessage, ChatMessageType, ChatPayload};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

/// Where to connect for the chat session.
#[derive(Debug, Clone)]
pub enum ConnectionTarget {
    Local(u16),
    Cloud(String),
}

impl std::fmt::Display for ConnectionTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionTarget::Local(port) => write!(f, "localhost:{}", port),
            ConnectionTarget::Cloud(url) => write!(f, "{}", url),
        }
    }
}

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct ChatConnection {
    ws: Option<WsStream>,
    #[allow(dead_code)]
    target: ConnectionTarget,
}

impl ChatConnection {
    pub async fn connect(target: &ConnectionTarget) -> Result<Self> {
        let url = match target {
            ConnectionTarget::Local(port) => format!("ws://127.0.0.1:{}/chat", port),
            ConnectionTarget::Cloud(url) => url.clone(),
        };

        let (ws, _response) = tokio_tungstenite::connect_async(&url)
            .await
            .with_context(|| {
                crate::i18n::t_args("errors.connection_failed", &[("url", &url)])
            })?;

        Ok(Self {
            ws: Some(ws),
            target: target.clone(),
        })
    }

    /// Send session_init message with locale.
    pub async fn send_session_init(&mut self, session_id: &str, locale: &str) -> Result<()> {
        let ws = self.ws.as_mut().context("not connected")?;
        let msg = ChatMessage::session_init(session_id.to_string(), locale.to_string());
        let json = serde_json::to_string(&msg)?;
        ws.send(Message::Text(json)).await?;
        Ok(())
    }

    /// Send a user message and stream the response token-by-token.
    ///
    /// Calls `on_chunk` for each token as it arrives, returns the full response.
    pub async fn send_and_stream(
        &mut self,
        session_id: &str,
        message: &str,
        cancel: &Arc<AtomicBool>,
        on_chunk: impl Fn(&str),
    ) -> Result<String> {
        let ws = self
            .ws
            .as_mut()
            .context(crate::i18n::t("errors.not_connected"))?;

        // Send user_message
        let msg = ChatMessage::new(
            ChatMessageType::UserMessage,
            ChatPayload {
                session_id: session_id.to_string(),
                content: message.to_string(),
                metadata: None,
            },
        );
        let json = serde_json::to_string(&msg)?;
        ws.send(Message::Text(json)).await?;

        // Read streaming response
        let mut full_response = String::new();

        while let Some(msg) = ws.next().await {
            if cancel.load(Ordering::Relaxed) {
                return Err(anyhow::anyhow!(
                    "{}",
                    crate::i18n::t("errors.response_cancelled")
                ));
            }

            match msg? {
                Message::Text(text) => {
                    let chat_msg: ChatMessage = serde_json::from_str(&text)?;
                    match chat_msg.msg_type {
                        ChatMessageType::StreamChunk => {
                            on_chunk(&chat_msg.payload.content);
                            full_response.push_str(&chat_msg.payload.content);
                        }
                        ChatMessageType::StreamEnd => {
                            break;
                        }
                        ChatMessageType::AgentResponse => {
                            // Non-streaming full response
                            full_response = chat_msg.payload.content;
                            break;
                        }
                        ChatMessageType::Error => {
                            return Err(anyhow::anyhow!(
                                "{}",
                                crate::i18n::t_args(
                                    "errors.agent_error",
                                    &[("message", &chat_msg.payload.content)]
                                )
                            ));
                        }
                        _ => {} // Ignore other types
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }

        Ok(full_response)
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(mut ws) = self.ws.take() {
            let _ = ws.close(None).await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_target_display() {
        let local = ConnectionTarget::Local(9876);
        assert_eq!(local.to_string(), "localhost:9876");

        let cloud = ConnectionTarget::Cloud("wss://api.example.com/ws".to_string());
        assert_eq!(cloud.to_string(), "wss://api.example.com/ws");
    }
}
```

**Important:** The import `use d1_daemon::chat_relay::{...}` assumes `d1-daemon` exposes `chat_relay` as a library. Since daemon is a binary crate (no lib.rs), we need an alternative approach. Options:
1. Move `ChatMessage` types to `d1_common` (best for shared types)
2. Add a `lib.rs` to daemon that re-exports chat_relay
3. Duplicate the types in CLI (not great)

**Recommended: Move ChatMessage to d1_common.** This is the cleanest approach since ChatMessage is a shared wire format.

**Step 2: Move ChatMessage types to d1_common**

Create/modify `d1-doctor-app/crates/common/src/chat_message.rs` with the ChatMessage, ChatMessageType, ChatPayload types extracted from `chat_relay.rs`. Then re-export from `d1_common`.

Update `crates/common/src/lib.rs` to add `pub mod chat_message;`

Update `crates/daemon/src/chat_relay.rs` to `use d1_common::chat_message::*;` instead of defining the types inline.

Update `crates/cli/src/chat/connection.rs` to `use d1_common::chat_message::*;`

**Step 3: Update display.rs for incremental printing**

Add to `display.rs`:

```rust
/// Print a streaming token chunk (no newline, flush immediately).
pub fn print_chunk(token: &str) {
    print!("{}", token);
    let _ = io::stdout().flush();
}

/// Print the stream header (agent name prompt) before streaming starts.
pub fn print_stream_start() {
    println!();
    print!("\x1b[1;33mdr.bob>\x1b[0m ");
    let _ = io::stdout().flush();
}

/// Print newline after stream completes.
pub fn print_stream_end() {
    println!();
}
```

**Step 4: Update mod.rs to use session_init and streaming**

In `crates/cli/src/chat/mod.rs`, update `run_interactive`:

```rust
    // After connect, before loop:
    let locale = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
    conn.send_session_init(&session_id, &locale).await?;

    // In the message handling, replace send_and_stream call:
    display::print_stream_start();
    match conn
        .send_and_stream(&session_id, &text, &cancel_token, |chunk| {
            display::stop_typing_indicator_if_running(&cancel_token);
            display::print_chunk(chunk);
        })
        .await
    {
        Ok(response) => {
            display::print_stream_end();
            history.add_agent_response(&response)?;
        }
        // ... error handling unchanged
    }
```

**Step 5: Check compilation**

Run: `cd d1-doctor-app && cargo check -p d1-cli 2>&1 | head -50`
Fix any compilation errors.

**Step 6: Commit**

```bash
git add crates/common/src/chat_message.rs crates/common/src/lib.rs \
       crates/daemon/src/chat_relay.rs crates/cli/src/chat/connection.rs \
       crates/cli/src/chat/display.rs crates/cli/src/chat/mod.rs
git commit -m "feat: CLI uses ChatMessage v1 with session_init and streaming"
```

---

## Task 7: Daemon Cargo.toml — Verify Dependencies

Ensure all needed dependencies are present for the new main.rs.

**Files:**
- Check/Modify: `d1-doctor-app/crates/daemon/Cargo.toml`

**Step 1: Check dependencies**

Verify these are in `[dependencies]`:
- `axum` with features: `["ws"]` (for WebSocket upgrade support)
- `tokio` with features: `["full"]`
- `tokio-tungstenite`
- `futures-util`
- `serde_json`
- `tracing`, `tracing-subscriber`
- `d1-common` (workspace dependency)

**Step 2: Add missing deps if needed**

Run: `cd d1-doctor-app && cargo check -p d1-daemon 2>&1 | head -20`

If `axum::extract::ws` fails, add `features = ["ws"]` to axum dependency.

**Step 3: Commit if changed**

```bash
git add crates/daemon/Cargo.toml
git commit -m "fix: add missing daemon dependencies for WS handler"
```

---

## Task 8: Deploy & Integration Test

Apply Terraform changes and test the full chain.

**Step 1: Build and push updated platform image**

```bash
cd d1-doctor-platform
docker buildx build --platform linux/amd64 --provenance=false \
  -t us-central1-docker.pkg.dev/PROJECT_ID/d1d-registry/platform:dev-latest \
  -f orchestrator/Dockerfile --push .
```

**Step 2: Apply Terraform**

```bash
cd d1-doctor-platform/infra/terraform
terraform plan   # verify only env var changes
terraform apply
```

**Step 3: Force new revision (same tag)**

```bash
gcloud run services update dev-d1d-platform --region us-central1 --no-traffic
```

**Step 4: Verify platform health**

```bash
PLATFORM_URL=$(gcloud run services describe dev-d1d-platform --region us-central1 --format 'value(status.url)')
curl $PLATFORM_URL/health
curl $PLATFORM_URL/readyz
```

**Step 5: Test daemon locally**

```bash
cd d1-doctor-app
cargo run -p d1-daemon
# In another terminal:
websocat ws://127.0.0.1:9876/chat
# Type: {"v":1,"id":"test","ts":0,"type":"session_init","payload":{"session_id":"s1","content":"","metadata":{"locale":"en"}}}
# Then: {"v":1,"id":"test2","ts":0,"type":"user_message","payload":{"session_id":"s1","content":"hello"}}
```

**Step 6: Test CLI end-to-end**

```bash
cargo run -p d1-cli -- run
# Type a message, verify streaming tokens appear
```

**Step 7: Commit any fixes from integration testing**

---

## Non-Goals (follow-up)

Per design doc, these are explicitly out of scope:
- Command execution flow (daemon executor)
- Multi-agent delegation (Dr. Bob → specialist agents)
- MCP tool integration during agent execution
- Conversation history persistence to Supabase (Redis-only for now)
