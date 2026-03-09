//! ChatMessage v1 wire format — shared between daemon and CLI.
//!
//! Wire protocol: `{ v, id, ts, type, payload }`

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    /// Protocol version (always 1).
    pub v: u32,
    /// Unique message id.
    pub id: String,
    /// Unix-millis timestamp.
    pub ts: i64,
    /// Message type discriminator.
    #[serde(rename = "type")]
    pub msg_type: ChatMessageType,
    /// Arbitrary JSON payload.
    pub payload: ChatPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChatMessageType {
    /// User chat message (app -> cloud).
    UserMessage,
    /// Full agent response (cloud -> app).
    AgentResponse,
    /// Streaming token chunk (cloud -> app).
    StreamChunk,
    /// Stream finished marker (cloud -> app).
    StreamEnd,
    /// Session initialisation (app -> cloud).
    SessionInit,
    /// Session init acknowledgement (cloud -> app).
    SessionInitAck,
    /// Error notification (either direction).
    Error,
    /// Catch-all for unrecognised message types (e.g. HEARTBEAT_ACK).
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatPayload {
    pub session_id: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl ChatMessage {
    pub fn new(msg_type: ChatMessageType, payload: ChatPayload) -> Self {
        Self {
            v: 1,
            id: Uuid::new_v4().to_string(),
            ts: Utc::now().timestamp_millis(),
            msg_type,
            payload,
        }
    }

    pub fn error(session_id: String, message: String) -> Self {
        Self::new(
            ChatMessageType::Error,
            ChatPayload {
                session_id,
                content: message,
                metadata: None,
            },
        )
    }

    pub fn session_init(session_id: String, locale: String) -> Self {
        Self::new(
            ChatMessageType::SessionInit,
            ChatPayload {
                session_id,
                content: String::new(),
                metadata: Some(serde_json::json!({ "locale": locale })),
            },
        )
    }
}
