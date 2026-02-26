//! Generated protobuf types for the d1doctor protocol.
//! 
//! NOTE: Proto bindings are not compiled in this sprint (Sprint 1 foundation).
//! These stubs allow the crate to compile without proto code generation.

/// Placeholder Envelope type - will be replaced by generated proto code
#[derive(Debug, Clone, Default)]
pub struct Envelope {
    pub id: String,
    pub session_id: String,
    pub timestamp_ms: i64,
    pub r#type: i32,
    pub payload: Vec<u8>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Placeholder MessageType enum
#[repr(i32)]
pub enum MessageType {
    Unknown = 0,
    Heartbeat = 1,
}

/// Build a HEARTBEAT Envelope.
pub fn heartbeat_envelope(session_id: impl Into<String>) -> Envelope {
    use std::time::{SystemTime, UNIX_EPOCH};
    Envelope {
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

/// Serialize an Envelope to bytes (stub - returns empty vec without prost).
pub fn encode_envelope(_env: &Envelope) -> Vec<u8> {
    vec![]
}

/// Deserialize an Envelope from bytes (stub).
pub fn decode_envelope(_bytes: &[u8]) -> Result<Envelope, String> {
    Ok(Envelope::default())
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
}
