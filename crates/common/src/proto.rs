//! Protocol types and serialization for the d1doctor protocol.
//!
//! NOTE: In Sprint 1, we use serde_json for serialization instead of prost/protobuf,
//! since the proto code-generation pipeline (A2) is wired but not yet exercised.
//! The Envelope struct mirrors the protobuf schema defined in proto/d1doctor/v1/.

use serde::{Deserialize, Serialize};

/// Message envelope — wraps every request/response on the wire.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Envelope {
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

/// Placeholder MessageType enum — matches the proto enum values.
#[repr(i32)]
pub enum MessageType {
    Unknown = 0,
    Heartbeat = 1,
    CommandRequest = 2,
    CommandResponse = 3,
    HealthReport = 4,
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

/// Serialize an Envelope to bytes (JSON encoding for Sprint 1).
pub fn encode_envelope(env: &Envelope) -> Vec<u8> {
    serde_json::to_vec(env).unwrap_or_default()
}

/// Deserialize an Envelope from bytes (JSON decoding for Sprint 1).
pub fn decode_envelope(bytes: &[u8]) -> Result<Envelope, String> {
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
}
