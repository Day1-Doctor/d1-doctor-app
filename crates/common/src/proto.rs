//! Protocol types and serialization for the d1doctor protocol.
//!
//! NOTE: In Sprint 1, we use serde_json for serialization instead of prost/protobuf,
//! since the proto code-generation pipeline (A2) is wired but not yet exercised.
//! The Envelope struct mirrors the protobuf schema defined in proto/d1doctor/v1/.
//!
//! Sprint 3: Added prost-based message types for the install command and daemon
//! command execution loop.

use serde::{Deserialize, Serialize};

// ─── Prost message types (Sprint 3) ──────────────────────────────────────────

/// Message envelope — wraps every request/response on the wire (prost version).
#[derive(Clone, PartialEq, prost::Message)]
pub struct Envelope {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(string, tag = "2")]
    pub session_id: String,
    #[prost(int64, tag = "3")]
    pub timestamp_ms: i64,
    #[prost(int32, tag = "4")]
    pub r#type: i32,
    #[prost(bytes = "vec", tag = "5")]
    pub payload: Vec<u8>,
}

/// A request from the user / CLI to the orchestrator.
#[derive(Clone, PartialEq, prost::Message)]
pub struct UserRequest {
    #[prost(string, tag = "1")]
    pub text: String,
}

/// A single step in a plan.
#[derive(Clone, PartialEq, prost::Message)]
pub struct PlanStep {
    #[prost(int32, tag = "1")]
    pub step_number: i32,
    #[prost(string, tag = "2")]
    pub description: String,
    #[prost(string, tag = "3")]
    pub agent_name: String,
}

/// A proposed plan from the orchestrator to the CLI.
#[derive(Clone, PartialEq, prost::Message)]
pub struct PlanProposal {
    #[prost(string, tag = "1")]
    pub task_id: String,
    #[prost(string, tag = "2")]
    pub summary: String,
    #[prost(float, tag = "3")]
    pub estimated_credits: f32,
    #[prost(message, repeated, tag = "4")]
    pub steps: Vec<PlanStep>,
}

/// Approval or rejection of a proposed plan.
#[derive(Clone, PartialEq, prost::Message)]
pub struct PlanApproval {
    #[prost(string, tag = "1")]
    pub task_id: String,
    #[prost(int32, tag = "2")]
    pub action: i32,
}

/// Action values for PlanApproval.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApprovalAction {
    Unknown = 0,
    Approve = 1,
    Reject = 2,
}

/// A shell command to execute on the daemon.
#[derive(Clone, PartialEq, prost::Message)]
pub struct Command {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(string, tag = "2")]
    pub task_id: String,
    #[prost(int32, tag = "3")]
    pub step_number: i32,
    #[prost(string, tag = "4")]
    pub shell_command: String,
    #[prost(int64, tag = "5")]
    pub timeout_ms: i64,
}

/// Result of executing a shell command.
#[derive(Clone, PartialEq, prost::Message)]
pub struct CommandResult {
    #[prost(string, tag = "1")]
    pub command_id: String,
    #[prost(string, tag = "2")]
    pub task_id: String,
    #[prost(bool, tag = "3")]
    pub success: bool,
    #[prost(string, tag = "4")]
    pub stdout: String,
    #[prost(string, tag = "5")]
    pub stderr: String,
    #[prost(int32, tag = "6")]
    pub exit_code: i32,
    #[prost(int64, tag = "7")]
    pub duration_ms: i64,
}

/// Progress update for a running task.
#[derive(Clone, PartialEq, prost::Message)]
pub struct ProgressUpdate {
    #[prost(string, tag = "1")]
    pub task_id: String,
    #[prost(int32, tag = "2")]
    pub step_number: i32,
    #[prost(string, tag = "3")]
    pub message: String,
    #[prost(int32, tag = "4")]
    pub percent_complete: i32,
}

/// Placeholder MessageType enum — matches the proto enum values.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageType {
    Unknown = 0,
    Heartbeat = 1,
    UserRequest = 2,
    PlanProposal = 3,
    PlanApproval = 4,
    Command = 5,
    CommandResult = 6,
    ProgressUpdate = 7,
    CommandRequest = 8,
    CommandResponse = 9,
    HealthReport = 10,
    AuthResponse = 14,
}

// ─── JSON-based helpers (Sprint 1 compatibility) ─────────────────────────────
//
// The JSON Envelope below uses serde for backward compatibility with
// the existing daemon/ws_client code. New Sprint 3 code uses the prost
// Envelope above directly.

/// JSON-serializable Envelope (Sprint 1 compat). For new code use the prost Envelope.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JsonEnvelope {
    pub id: String,
    pub session_id: String,
    pub timestamp_ms: i64,
    #[serde(rename = "type")]
    pub r#type: i32,
    #[serde(default, with = "serde_base64")]
    pub payload: Vec<u8>,
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

/// Build a HEARTBEAT Envelope (JSON/Sprint-1 compat).
pub fn heartbeat_envelope(session_id: impl Into<String>) -> JsonEnvelope {
    use std::time::{SystemTime, UNIX_EPOCH};
    JsonEnvelope {
        id: uuid::Uuid::new_v4().to_string(),
        session_id: session_id.into(),
        timestamp_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64,
        r#type: MessageType::Heartbeat as i32,
        payload: vec![],
        metadata: Default::default(),
    }
}

/// Build an AUTH_RESPONSE Envelope with a JWT token and device_id.
pub fn auth_response_envelope(
    session_id: impl Into<String>,
    jwt_token: impl Into<String>,
    device_id: impl Into<String>,
) -> JsonEnvelope {
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("jwt_token".to_string(), jwt_token.into());
    metadata.insert("device_id".to_string(), device_id.into());
    JsonEnvelope {
        id: uuid::Uuid::new_v4().to_string(),
        session_id: session_id.into(),
        timestamp_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64,
        r#type: MessageType::AuthResponse as i32,
        payload: vec![],
        metadata,
    }
}

/// Serialize a JsonEnvelope to bytes (JSON encoding for Sprint 1).
pub fn encode_envelope(env: &JsonEnvelope) -> Vec<u8> {
    serde_json::to_vec(env).unwrap_or_default()
}

/// Deserialize a JsonEnvelope from bytes (JSON decoding for Sprint 1).
pub fn decode_envelope(bytes: &[u8]) -> Result<JsonEnvelope, String> {
    serde_json::from_slice(bytes).map_err(|e| e.to_string())
}

/// serde helper: encode Vec<u8> as base64 string in JSON.
mod serde_base64 {
    use base64::Engine as _;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&base64::engine::general_purpose::STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(d)?;
        base64::engine::general_purpose::STANDARD
            .decode(&s)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_envelope() {
        let env = heartbeat_envelope("test-session");
        assert_eq!(env.r#type, MessageType::Heartbeat as i32);
        assert!(!env.id.is_empty());
        assert_eq!(env.session_id, "test-session");
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = heartbeat_envelope("roundtrip-session");
        let bytes = encode_envelope(&original);
        assert!(!bytes.is_empty(), "encoded bytes should not be empty");
        let decoded = decode_envelope(&bytes).expect("decode should succeed");
        assert_eq!(decoded.id, original.id);
        assert_eq!(decoded.session_id, original.session_id);
        assert_eq!(decoded.r#type, original.r#type);
        assert_eq!(decoded.timestamp_ms, original.timestamp_ms);
    }

    #[test]
    fn test_decode_invalid_bytes_returns_error() {
        let result = decode_envelope(b"not valid json at all !!!");
        assert!(result.is_err(), "invalid bytes should return Err");
    }

    #[test]
    fn test_auth_response_envelope_can_be_serialized() {
        let env = auth_response_envelope("session-1", "jwt-token-xyz", "device-001");
        assert_eq!(env.r#type, MessageType::AuthResponse as i32);
        assert!(!env.id.is_empty());
        assert_eq!(env.session_id, "session-1");
        assert_eq!(env.metadata.get("jwt_token").map(|s| s.as_str()), Some("jwt-token-xyz"));
        assert_eq!(env.metadata.get("device_id").map(|s| s.as_str()), Some("device-001"));

        // Verify it round-trips through JSON serialization
        let bytes = encode_envelope(&env);
        assert!(!bytes.is_empty());
        let decoded = decode_envelope(&bytes).expect("decode should succeed");
        assert_eq!(decoded.r#type, MessageType::AuthResponse as i32);
        assert_eq!(decoded.metadata.get("jwt_token").map(|s| s.as_str()), Some("jwt-token-xyz"));
    }

    #[test]
    fn test_prost_envelope_roundtrip() {
        use prost::Message as _;
        let mut env = Envelope::default();
        env.id = "test-id".to_string();
        env.session_id = "sess-1".to_string();
        env.r#type = MessageType::UserRequest as i32;
        env.payload = b"hello".to_vec();

        let bytes = env.encode_to_vec();
        let decoded = Envelope::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded.id, "test-id");
        assert_eq!(decoded.session_id, "sess-1");
        assert_eq!(decoded.r#type, MessageType::UserRequest as i32);
        assert_eq!(decoded.payload, b"hello");
    }

    #[test]
    fn test_plan_proposal_roundtrip() {
        use prost::Message as _;
        let mut proposal = PlanProposal::default();
        proposal.task_id = "task-1".to_string();
        proposal.summary = "Install node".to_string();
        proposal.estimated_credits = 1.5;
        let mut step = PlanStep::default();
        step.step_number = 1;
        step.description = "brew install node".to_string();
        step.agent_name = "executor".to_string();
        proposal.steps.push(step);

        let bytes = proposal.encode_to_vec();
        let decoded = PlanProposal::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded.task_id, "task-1");
        assert_eq!(decoded.steps.len(), 1);
        assert_eq!(decoded.steps[0].description, "brew install node");
    }
}
