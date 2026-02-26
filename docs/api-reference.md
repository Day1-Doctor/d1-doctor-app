# API Reference

## REST API (Cloud Services)

### Authentication

#### Device Code Flow

Request a device code for user authentication.

```http
POST /api/v1/auth/device-code
```

Response:
```json
{
  "device_code": "ABC123",
  "user_code": "WXYZ-1234",
  "verification_uri": "https://day1doctor.com/authorize",
  "expires_in": 900,
  "interval": 5
}
```

### User API

#### Get User Profile

```http
GET /api/v1/user
Authorization: Bearer {token}
```

Response:
```json
{
  "id": "user_123",
  "email": "user@example.com",
  "username": "myusername",
  "created_at": "2024-01-15T10:30:00Z",
  "subscription": "pro"
}
```

#### Get Credit Balance

```http
GET /api/v1/user/balance
Authorization: Bearer {token}
```

Response:
```json
{
  "available": 1000.5,
  "used": 249.5,
  "monthly_limit": 2000,
  "currency": "USD"
}
```

### Setup API

#### Create Setup Task

```http
POST /api/v1/setup/create
Authorization: Bearer {token}
Content-Type: application/json

{
  "title": "Install Docker",
  "description": "Set up Docker for local development",
  "packages": ["docker", "docker-compose"],
  "plugins": ["setup-docker-completion"],
  "options": {
    "version": "latest",
    "include_compose": true
  }
}
```

Response:
```json
{
  "task_id": "task_456",
  "status": "queued",
  "estimated_cost": 10.5,
  "estimated_duration_seconds": 120,
  "created_at": "2024-01-15T11:45:00Z"
}
```

#### Get Task Status

```http
GET /api/v1/setup/tasks/{task_id}
Authorization: Bearer {token}
```

Response:
```json
{
  "task_id": "task_456",
  "status": "in_progress",
  "progress": {
    "current_step": 2,
    "total_steps": 5,
    "percent": 40
  },
  "cost_so_far": 2.5,
  "started_at": "2024-01-15T11:46:00Z",
  "estimated_completion": "2024-01-15T11:48:00Z"
}
```

#### List User Tasks

```http
GET /api/v1/setup/tasks?limit=20&offset=0
Authorization: Bearer {token}
```

Response:
```json
{
  "tasks": [
    {
      "task_id": "task_456",
      "title": "Install Docker",
      "status": "completed",
      "cost": 10.5,
      "created_at": "2024-01-15T11:45:00Z",
      "completed_at": "2024-01-15T11:48:00Z"
    }
  ],
  "total": 42,
  "limit": 20,
  "offset": 0
}
```

## WebSocket API (Daemon ↔ Orchestrator)

### Connection

```
wss://api.day1doctor.com/ws?token={jwt_token}
```

### Message Types

#### TaskCommand

Orchestrator → Daemon

```protobuf
message TaskCommand {
  string task_id = 1;           // Unique task identifier
  string command = 2;           // Shell command to execute
  map<string, string> env = 3;  // Environment variables
  uint32 timeout_seconds = 4;   // Execution timeout
  bool require_confirmation = 5; // Needs user approval
  string working_dir = 6;       // Execution directory
}
```

#### TaskResult

Daemon → Orchestrator

```protobuf
message TaskResult {
  string task_id = 1;           // Matching task ID
  int32 exit_code = 2;          // Process exit code
  string stdout = 3;            // Standard output
  string stderr = 4;            // Standard error
  int64 duration_ms = 5;        // Execution time
  repeated string warnings = 6; // Non-fatal issues
  string system_info = 7;       // JSON system diagnostics
}
```

#### HealthReport

Daemon → Orchestrator (periodic)

```protobuf
message HealthReport {
  string daemon_version = 1;
  SystemMetrics metrics = 2;
  repeated string active_tasks = 3;
  int64 uptime_seconds = 4;
  string last_command = 5;
  int64 timestamp = 6;
}

message SystemMetrics {
  float cpu_percent = 1;
  float memory_percent = 2;
  float disk_percent = 3;
  string os = 4;
  string os_version = 5;
}
```

### Example WebSocket Message Flow

```
CLIENT → SERVER: TaskCommand
  { task_id: "t123", command: "brew install node" }

SERVER → CLIENT: Acknowledgment
  { status: "accepted" }

CLIENT → SERVER: TaskResult (after execution)
  { 
    task_id: "t123",
    exit_code: 0,
    stdout: "node@20.5.0 installed",
    duration_ms: 15000
  }

SERVER → CLIENT: Billing Event
  { 
    task_id: "t123",
    cost: 2.5,
    status: "charged"
  }
```

## Protobuf Definitions

All message definitions are in `proto/d1doctor/v1/`:

- `service.proto` - Core RPC services
- `task.proto` - Task and command messages
- `system.proto` - System information structures
- `user.proto` - User and auth structures

Generate Rust bindings:

```bash
cargo build -p d1-doctor-common
```

## Error Codes

### HTTP API

- `200 OK` - Successful request
- `400 Bad Request` - Invalid input
- `401 Unauthorized` - Missing or invalid token
- `403 Forbidden` - Permission denied
- `404 Not Found` - Resource not found
- `429 Too Many Requests` - Rate limit exceeded
- `500 Internal Server Error` - Server error

### WebSocket

Error messages are sent as:

```json
{
  "type": "error",
  "code": "INSUFFICIENT_CREDITS",
  "message": "Not enough credits for this operation",
  "details": {
    "required": 10.5,
    "available": 5.0
  }
}
```

Common error codes:
- `INSUFFICIENT_CREDITS` - User balance too low
- `UNSUPPORTED_PLATFORM` - Task not supported on OS
- `PERMISSION_DENIED` - Command requires elevation
- `NETWORK_ERROR` - Connection issue
- `TIMEOUT` - Command exceeded timeout
- `INVALID_COMMAND` - Unrecognized command

## Rate Limiting

API requests are rate-limited:

- Authenticated users: 100 requests/minute
- Anonymous: 10 requests/minute
- Batch operations: 10 operations/minute

Rate limit headers:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 87
X-RateLimit-Reset: 1705325400
```

## Pagination

List endpoints support pagination:

```
GET /api/v1/setup/tasks?limit=20&offset=40
```

Parameters:
- `limit`: Items per page (max 100)
- `offset`: Number of items to skip
- `sort`: Sort field (default: created_at)
- `order`: asc or desc (default: desc)

Response includes:

```json
{
  "data": [...],
  "total": 500,
  "limit": 20,
  "offset": 40
}
```

## Webhooks (Future)

Webhooks for task completion events:

```http
POST {webhook_url}
X-Signature: sha256={signature}

{
  "event": "task.completed",
  "task_id": "task_456",
  "status": "success",
  "cost": 10.5,
  "timestamp": "2024-01-15T11:48:00Z"
}
```

## SDK Examples

### Rust CLI

```rust
use d1_doctor_cli::commands::Commands;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd = Commands::Install {
        package: "docker".to_string(),
    };
    
    commands::handle(cmd).await?;
    Ok(())
}
```

### Plugins

```rust
use d1_doctor_sdk::Plugin;

#[async_trait]
impl Plugin for MyPlugin {
    async fn execute(&self, input: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        // Your plugin logic
        Ok(serde_json::json!({"status": "success"}))
    }
}
```
