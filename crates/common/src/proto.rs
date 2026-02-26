//! Generated protobuf types for the d1doctor protocol.
//! Run `scripts/sync-proto.sh` then `cargo build` to regenerate.

include!(concat!(env!("OUT_DIR"), "/d1doctor.v1.rs"));

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

/// Serialize an Envelope to bytes for WebSocket transmission.
pub fn encode_envelope(env: &Envelope) -> Vec<u8> {
    use prost::Message;
    env.encode_to_vec()
}

/// Deserialize an Envelope from WebSocket bytes.
pub fn decode_envelope(bytes: &[u8]) -> Result<Envelope, prost::DecodeError> {
    use prost::Message;
    Envelope::decode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_envelope_roundtrip() {
        let env = heartbeat_envelope("test-session");
        assert_eq!(env.r#type, MessageType::Heartbeat as i32);
        assert!(!env.id.is_empty());
        let bytes = encode_envelope(&env);
        let decoded = decode_envelope(&bytes).unwrap();
        assert_eq!(decoded.id, env.id);
        assert_eq!(decoded.session_id, "test-session");
    }
}
