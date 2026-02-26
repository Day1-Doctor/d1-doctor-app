//! Protocol types and message definitions
//!
//! This module defines the protobuf message types used for communication
//! between CLI, daemon, and orchestrator. When the proto submodule is available,
//! this will be replaced with generated prost types.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use bytes::Bytes;

/// Protocol version
pub const PROTO_VERSION: u32 = 1;

/// Top-level envelope for all WebSocket messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub version: u32,
    pub message_id: String,
    pub timestamp: i64,
    pub sender: String,
    pub payload: EnvelopePayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EnvelopePayload {
    UserRequest(UserRequest),
    PlanProposal(PlanProposal),
    Command(Command),
    CommandResult(CommandResult),
    Heartbeat(Heartbeat),
    Error(ErrorMessage),
}

/// User request to Dr. Bob (AI assistant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRequest {
    pub request_id: String,
    pub session_id: String,
    pub content: String,
    pub context: Option<RequestContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub system_info: Option<SystemInfo>,
    pub recent_errors: Vec<String>,
    pub working_directory: Option<String>,
}

/// AI-generated plan proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanProposal {
    pub proposal_id: String,
    pub request_id: String,
    pub steps: Vec<ProposalStep>,
    pub estimated_duration: i32,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalStep {
    pub step_id: String,
    pub description: String,
    pub action_type: String,
    pub required_permissions: Vec<PermissionLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum PermissionLevel {
    Green,  // Always allowed
    Yellow, // Needs permission
    Red,    // Explicit approval required
}

/// Command to be executed locally
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub command_id: String,
    pub proposal_id: String,
    pub step_id: String,
    pub command_type: CommandType,
    pub target: String,
    pub arguments: Vec<String>,
    pub environment: std::collections::HashMap<String, String>,
    pub timeout: Option<u64>,
    pub sandbox_level: SandboxLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CommandType {
    Shell,
    PackageManager,
    FileOperation,
    SystemQuery,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SandboxLevel {
    NoSandbox,
    Light,
    Medium,
    Strict,
}

/// Result of command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub command_id: String,
    pub status: CommandStatus,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub metadata: Option<ResultMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CommandStatus {
    Success,
    Failure,
    Timeout,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultMetadata {
    pub executed_at: i64,
    pub executor: String,
    pub environment: std::collections::HashMap<String, String>,
}

/// System health check message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub daemon_id: String,
    pub uptime: u64,
    pub health_status: HealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub disk_percent: f32,
    pub last_check: i64,
}

/// Error message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub error_id: String,
    pub message: String,
    pub error_type: String,
    pub context: Option<String>,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub os_version: String,
    pub arch: String,
    pub hostname: String,
    pub cpu_count: u32,
    pub memory_bytes: u64,
    pub disk_bytes: u64,
}

// Helper implementations
impl Envelope {
    pub fn new(sender: String, payload: EnvelopePayload) -> Self {
        Self {
            version: PROTO_VERSION,
            message_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now().timestamp_millis(),
            sender,
            payload,
        }
    }
}

impl UserRequest {
    pub fn new(session_id: String, content: String) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            session_id,
            content,
            context: None,
        }
    }
}

impl Command {
    pub fn new(
        proposal_id: String,
        step_id: String,
        command_type: CommandType,
        target: String,
    ) -> Self {
        Self {
            command_id: Uuid::new_v4().to_string(),
            proposal_id,
            step_id,
            command_type,
            target,
            arguments: Vec::new(),
            environment: std::collections::HashMap::new(),
            timeout: None,
            sandbox_level: SandboxLevel::Medium,
        }
    }
}

impl CommandResult {
    pub fn success(
        command_id: String,
        stdout: String,
        duration_ms: u64,
    ) -> Self {
        Self {
            command_id,
            status: CommandStatus::Success,
            stdout,
            stderr: String::new(),
            exit_code: 0,
            duration_ms,
            metadata: None,
        }
    }

    pub fn failure(
        command_id: String,
        stderr: String,
        exit_code: i32,
        duration_ms: u64,
    ) -> Self {
        Self {
            command_id,
            status: CommandStatus::Failure,
            stdout: String::new(),
            stderr,
            exit_code,
            duration_ms,
            metadata: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_creation() {
        let payload = EnvelopePayload::Heartbeat(Heartbeat {
            daemon_id: "test".to_string(),
            uptime: 100,
            health_status: HealthStatus {
                is_healthy: true,
                cpu_percent: 10.0,
                memory_percent: 20.0,
                disk_percent: 30.0,
                last_check: 0,
            },
        });
        let envelope = Envelope::new("daemon".to_string(), payload);
        assert_eq!(envelope.version, PROTO_VERSION);
        assert_eq!(envelope.sender, "daemon");
    }

    #[test]
    fn test_user_request_creation() {
        let req = UserRequest::new("session1".to_string(), "Hello".to_string());
        assert_eq!(req.session_id, "session1");
        assert_eq!(req.content, "Hello");
    }

    #[test]
    fn test_command_result_success() {
        let result = CommandResult::success("cmd1".to_string(), "output".to_string(), 100);
        assert_eq!(result.exit_code, 0);
        assert!(matches!(result.status, CommandStatus::Success));
    }
}
