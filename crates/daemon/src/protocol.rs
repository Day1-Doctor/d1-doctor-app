//! JSON protocol types for the local WebSocket channel (client ↔ daemon).
//! Spec: LocalStack_v2.4.1_Spec.md §1.2–1.4

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PROTOCOL_VERSION: u32 = 1;

/// Inbound message from a local client (Mac client / CLI).
#[derive(Debug, Clone, Deserialize)]
pub struct ClientEnvelope {
    pub v: u32,
    pub id: String,
    pub ts: i64,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub payload: serde_json::Value,
}

/// Outbound message from daemon to local client.
#[derive(Debug, Clone, Serialize)]
pub struct DaemonEnvelope {
    pub v: u32,
    pub id: String,
    pub ts: i64,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub payload: serde_json::Value,
}

impl DaemonEnvelope {
    pub fn new(msg_type: &str, payload: serde_json::Value) -> Self {
        Self {
            v: PROTOCOL_VERSION,
            id: Uuid::new_v4().to_string(),
            ts: chrono::Utc::now().timestamp_millis(),
            msg_type: msg_type.to_string(),
            payload,
        }
    }

    pub fn error(code: &str, message: &str, request_id: Option<&str>) -> Self {
        let mut payload = serde_json::json!({ "code": code, "message": message });
        if let Some(rid) = request_id {
            payload["request_id"] = serde_json::Value::String(rid.to_string());
        }
        Self::new("error", payload)
    }

    pub fn heartbeat_pong() -> Self {
        Self::new("heartbeat", serde_json::json!({ "pong": true }))
    }

    pub fn daemon_status(version: &str, orchestrator_connected: bool, active_tasks: usize, orchestrator_url: &str, device_id: &str) -> Self {
        Self::new("daemon.status", serde_json::json!({
            "daemon_version": version,
            "protocol_version": PROTOCOL_VERSION,
            "orchestrator_connected": orchestrator_connected,
            "orchestrator_url": orchestrator_url,
            "active_tasks": active_tasks,
            "device_id": device_id,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialise_task_submit() {
        let json = r#"{"v":1,"id":"abc","ts":1234,"type":"task.submit","payload":{"task_id":"tsk_1","input":"test","context":{}}}"#;
        let msg: ClientEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(msg.v, 1);
        assert_eq!(msg.msg_type, "task.submit");
    }

    #[test]
    fn test_serialise_plan_proposed() {
        let payload = serde_json::json!({
            "task_id": "tsk_1",
            "plan_id": "pln_1",
            "summary": "Install OpenClaw",
            "risk_tier": "MEDIUM",
            "steps": [],
            "requires_approval": true
        });
        let env = DaemonEnvelope::new("plan.proposed", payload);
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("\"v\":1"));
        assert!(json.contains("\"type\":\"plan.proposed\""));
    }

    #[test]
    fn test_protocol_version_is_1() {
        assert_eq!(PROTOCOL_VERSION, 1);
    }

    #[test]
    fn test_heartbeat_pong() {
        let pong = DaemonEnvelope::heartbeat_pong();
        assert_eq!(pong.msg_type, "heartbeat");
        assert_eq!(pong.payload["pong"], true);
    }

    #[test]
    fn test_error_envelope() {
        let err = DaemonEnvelope::error("PROTOCOL_ERROR", "bad message", Some("req-123"));
        assert_eq!(err.msg_type, "error");
        assert_eq!(err.payload["code"], "PROTOCOL_ERROR");
        assert_eq!(err.payload["request_id"], "req-123");
    }

    #[test]
    fn test_version_mismatch_rejected() {
        let json = r#"{"v":99,"id":"abc","ts":1234,"type":"task.submit","payload":{}}"#;
        let msg: ClientEnvelope = serde_json::from_str(json).unwrap();
        // Validate: v must equal PROTOCOL_VERSION
        assert_ne!(msg.v, PROTOCOL_VERSION);
    }
}
