# Architecture Overview

Day 1 Doctor is a distributed system with cloud orchestration and local execution. This document explains the architecture and component interactions.

## System Design

```
┌─────────────────────────────────────────────────────────────┐
│                     Day 1 Doctor System                      │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐      ┌──────────────┐      ┌────────────┐ │
│  │ CLI Client   │      │ Desktop App  │      │  Web UI    │ │
│  │  (Rust)      │      │  (Tauri 2.0) │      │  (SPA)     │ │
│  └──────┬───────┘      └──────┬───────┘      └────┬───────┘ │
│         │                      │                    │          │
│         └──────────────────────┼────────────────────┘          │
│                                │                              │
│                         Device-Code OAuth                    │
│                                │                              │
│         ┌──────────────────────▼────────────────────┐        │
│         │    API Gateway & Auth Service            │        │
│         │    (cloud.day1doctor.com)                │        │
│         └──────────────────────┬────────────────────┘        │
│                                │                              │
│              ┌─────────────────┴─────────────────┐            │
│              │                                   │            │
│      ┌───────▼────────┐            ┌────────────▼────────┐  │
│      │   Orchestrator │            │   Credit Manager    │  │
│      │   (gRPC/Proto) │            │   (Cost tracking)   │  │
│      └───────┬────────┘            └─────────────────────┘  │
│              │                                               │
│              │ WebSocket + Protobuf                         │
│              │                                               │
│      ┌───────▼────────────────────────────────┐            │
│      │      Local Machine (User's Device)     │            │
│      │                                        │            │
│      │  ┌────────────────────────────────┐   │            │
│      │  │    D1 Doctor Daemon            │   │            │
│      │  │  (Tokio async runtime)         │   │            │
│      │  │                                │   │            │
│      │  │  ┌────────────────────────┐   │   │            │
│      │  │  │ WebSocket Client       │   │   │            │
│      │  │  │ (Listen for tasks)     │   │   │            │
│      │  │  └────────────────────────┘   │   │            │
│      │  │                                │   │            │
│      │  │  ┌────────────────────────┐   │   │            │
│      │  │  │ Command Executor       │   │   │            │
│      │  │  │ (Sandboxed shell)      │   │   │            │
│      │  │  └────────────────────────┘   │   │            │
│      │  │                                │   │            │
│      │  │  ┌────────────────────────┐   │   │            │
│      │  │  │ SQLite State DB        │   │   │            │
│      │  │  │ (Persistence)          │   │   │            │
│      │  │  └────────────────────────┘   │   │            │
│      │  │                                │   │            │
│      │  │  ┌────────────────────────┐   │   │            │
│      │  │  │ Health Monitor         │   │   │            │
│      │  │  │ (System diagnostics)   │   │   │            │
│      │  │  └────────────────────────┘   │   │            │
│      │  │                                │   │            │
│      │  │  ┌────────────────────────┐   │   │            │
│      │  │  │ Plugin Engine          │   │   │            │
│      │  │  │ (Custom automation)    │   │   │            │
│      │  │  └────────────────────────┘   │   │            │
│      │  └────────────────────────────────┘   │            │
│      │                                        │            │
│      └────────────────────────────────────────┘            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Components

### Cloud Services

**API Gateway**
- Device-code OAuth authentication
- User session management
- API endpoint routing

**Orchestrator**
- Receives setup requests (from CLI, web, or AI agents)
- Plans execution steps
- Tracks task progress
- Manages credits and permissions
- Communicates with local daemons via WebSocket

**Credit Manager**
- Tracks credit balance per user
- Bills for resource consumption
- Implements rate limiting

### Local Daemon

The daemon runs as a service on the user's machine with elevated permissions.

**WebSocket Client**
- Maintains persistent connection to orchestrator
- Receives task commands
- Sends task results and health telemetry
- Auto-reconnects on failure

**Command Executor**
- Executes shell commands with timeout limits
- Sandboxing via process isolation
- Captures stdout/stderr
- Logs command history in local DB

**SQLite State Database**
- Persists daemon configuration
- Stores command history
- Tracks completed tasks (for idempotency)
- Caches system information

**Health Monitor**
- Collects CPU, memory, disk metrics
- Monitors daemon health
- Detects system issues
- Reports to orchestrator periodically

**Plugin Engine**
- Loads and manages plugins
- Provides plugin lifecycle management
- Exposes MCP tool registration API

### Client Applications

**CLI Client** (d1-doctor command)
- Device-code login flow
- Command execution (install, diagnose, etc.)
- Status monitoring
- TUI progress display

**Desktop App** (Tauri 2.0)
- GUI for setup wizard
- Visual task progress tracking
- System information dashboard

**Web Dashboard**
- User account management
- Credit monitoring
- Setup history
- Plugin browsing

## Communication Protocols

### Client → Cloud (REST/gRPC)

```
POST /api/v1/auth/device-code
GET  /api/v1/user/balance
POST /api/v1/setup/create
```

### Daemon ↔ Orchestrator (WebSocket + Protobuf)

```protobuf
message TaskCommand {
  string task_id = 1;
  string command = 2;
  map<string, string> env = 3;
  uint32 timeout_seconds = 4;
  bool require_confirmation = 5;
}

message TaskResult {
  string task_id = 1;
  int32 exit_code = 2;
  string stdout = 3;
  string stderr = 4;
  int64 duration_ms = 5;
}
```

## Data Flow: Installing a Package

1. **User initiates**: `d1-doctor install docker`
2. **CLI authenticates** with device-code OAuth
3. **CLI sends request** to Orchestrator API
4. **Orchestrator plans** installation steps
5. **Orchestrator sends** TaskCommand to daemon via WebSocket
6. **Daemon executes** command in sandbox
7. **Daemon sends** TaskResult back to orchestrator
8. **Orchestrator tracks** progress and credits
9. **CLI receives** status update and displays progress
10. **User gets** installation confirmation

## Security Model

### Permission Boundaries

- Daemon runs with elevated privileges but in isolated environment
- Each command is validated before execution
- File access restricted to user's home directory
- Network access limited to approved domains
- Environment variables sanitized

### Authentication & Authorization

- Device-code OAuth flow (no passwords)
- JWT tokens for API access
- Daemon-orchestrator connection authenticated with mTLS
- Audit logging of all executed commands

### Credit-based Billing

- Each operation costs credits
- Premium services cost more
- Rate limiting per user
- Prevents abuse and runaway costs

## Extensibility

### Plugin System

Plugins extend Day 1 Doctor with custom functionality:

- Implement `Plugin` trait from SDK
- Register MCP tools for AI agents
- Access local system state safely
- Persistent storage in local DB

### Third-party Integrations

Future integrations for:
- Configuration management (Terraform, Ansible)
- Monitoring (Datadog, New Relic)
- CI/CD pipelines (GitHub Actions, GitLab CI)

## Deployment

### Cloud Components

- Hosted on AWS/GCP with multi-region redundancy
- Auto-scaling orchestrator fleet
- PostgreSQL for user/credit data
- Redis for session management

### Local Components

- Single daemon process per machine
- Systemd unit for Linux
- LaunchAgent for macOS
- Windows Service for Windows

## Performance Characteristics

- **Daemon startup**: < 500ms
- **Command execution**: 100ms - 60s (depends on task)
- **WebSocket latency**: < 50ms (LAN), < 500ms (WAN)
- **Health reporting**: Every 30 seconds
- **Concurrent tasks**: Up to 10 per daemon

## High Availability

### Cloud

- Multi-region orchestrator deployment
- Automatic failover
- Session persistence in Redis
- Graceful degradation on partial outage

### Local

- Daemon persists unfinished tasks
- Resumes on reconnect
- Client-side timeout with retry logic

## Future Enhancements

- P2P mesh networking for offline execution
- GPU acceleration for large tasks
- Hardware security module integration
- Kubernetes integration
