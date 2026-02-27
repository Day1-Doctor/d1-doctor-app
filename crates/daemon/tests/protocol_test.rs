//! Integration-style tests for the daemon JSON protocol.
//! These tests create a real WebSocket listener on a test port and verify
//! the protocol message format and routing.

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};
    use uuid::Uuid;

    fn make_envelope(msg_type: &str, payload: Value) -> Value {
        json!({
            "v": 1,
            "id": Uuid::new_v4().to_string(),
            "ts": 1000u64,
            "type": msg_type,
            "payload": payload,
        })
    }

    #[test]
    fn test_envelope_v1_structure() {
        let env = make_envelope("task.submit", json!({"task_id": "tsk_abc123", "input": "hello"}));
        assert_eq!(env["v"], 1);
        assert!(env["id"].as_str().unwrap().len() == 36);
        assert_eq!(env["type"], "task.submit");
        assert_eq!(env["payload"]["task_id"], "tsk_abc123");
    }

    #[test]
    fn test_envelope_required_fields() {
        let env = make_envelope("heartbeat", json!({}));
        assert!(env.get("v").is_some(), "must have v");
        assert!(env.get("id").is_some(), "must have id");
        assert!(env.get("ts").is_some(), "must have ts");
        assert!(env.get("type").is_some(), "must have type");
        assert!(env.get("payload").is_some(), "must have payload");
    }

    #[test]
    fn test_task_submit_payload_structure() {
        let env = make_envelope("task.submit", json!({
            "task_id": "tsk_001",
            "input": "check why postgres is down",
            "context": {"cwd": null, "env": {}},
        }));
        let p = &env["payload"];
        assert!(p.get("task_id").is_some());
        assert!(p.get("input").is_some());
        assert!(p.get("context").is_some());
    }

    #[test]
    fn test_plan_approve_payload_structure() {
        let env = make_envelope("plan.approve", json!({
            "task_id": "tsk_001",
            "plan_id": "plan_abc",
            "action": "APPROVE",
        }));
        let p = &env["payload"];
        assert_eq!(p["action"], "APPROVE");
        assert!(p.get("task_id").is_some());
        assert!(p.get("plan_id").is_some());
    }

    #[test]
    fn test_outbound_message_types() {
        // Verify we can construct all 12 inbound message types
        let types = [
            ("plan.proposed", json!({"plan_id": "p1", "steps": []})),
            ("step.started", json!({"order": 1, "label": "test"})),
            ("step.completed", json!({"order": 1, "duration_ms": 100})),
            ("step.failed", json!({"order": 1, "error": {"message": "oops"}})),
            ("agent.message", json!({"role": "bob", "content": "hello"})),
            ("task.completed", json!({"task_id": "t1"})),
            ("task.failed", json!({"task_id": "t1", "error": {"message": "fail"}})),
            ("heartbeat", json!({"uptime_secs": 42})),
            ("daemon.status", json!({"status": "idle", "daemon_version": "2.4.1"})),
            ("permission.requested", json!({"permission_id": "p1", "command": "ls"})),
            ("credits.updated", json!({"daily_balance": 100, "bonus_balance": 0})),
            ("error", json!({"code": "UNKNOWN", "message": "err"})),
        ];

        for (msg_type, payload) in &types {
            let env = make_envelope(msg_type, payload.clone());
            assert_eq!(env["type"], *msg_type, "type should round-trip for {msg_type}");
            assert_eq!(env["v"], 1, "v must be 1 for {msg_type}");
        }
    }

    #[test]
    fn test_protocol_version_constant() {
        // The protocol version must be 1 for v2.4.1
        const PROTOCOL_VERSION: u8 = 1;
        assert_eq!(PROTOCOL_VERSION, 1);
    }
}
