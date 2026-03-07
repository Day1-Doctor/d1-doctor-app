//! Chat message relay between local app/CLI clients and cloud agent service.
//!
//! The relay bridges local WebSocket clients (on localhost:9876) with the cloud
//! agent WebSocket (wss://api.day1.doctor), forwarding user messages upstream
//! and streaming agent responses back to all connected local clients.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use uuid::Uuid;

/// Maximum number of messages to queue when cloud is disconnected.
const MAX_QUEUE_SIZE: usize = 256;

/// Broadcast channel capacity for local client fan-out.
const BROADCAST_CAPACITY: usize = 512;

// ---------------------------------------------------------------------------
// Wire protocol — v1: { v, id, ts, type, payload }
// ---------------------------------------------------------------------------

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
    /// User chat message (app → cloud).
    UserMessage,
    /// Full agent response (cloud → app).
    AgentResponse,
    /// Streaming token chunk (cloud → app).
    StreamChunk,
    /// Stream finished marker (cloud → app).
    StreamEnd,
    /// Session initialisation (app → cloud).
    SessionInit,
    /// Error notification (either direction).
    Error,
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

// ---------------------------------------------------------------------------
// Cloud connection state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudConnectionState {
    Connected,
    Disconnected,
}

// ---------------------------------------------------------------------------
// ChatRelay — the core relay engine
// ---------------------------------------------------------------------------

pub struct ChatRelay {
    /// Sender half: local clients push user messages here.
    to_cloud_tx: mpsc::Sender<ChatMessage>,
    /// Broadcast sender: cloud responses fan out to all local clients.
    to_local_tx: broadcast::Sender<ChatMessage>,
    /// Pending messages queued while cloud is disconnected.
    queue: Arc<RwLock<VecDeque<ChatMessage>>>,
    /// Current cloud connection state.
    cloud_state: Arc<RwLock<CloudConnectionState>>,
}

impl ChatRelay {
    /// Create a new relay instance.
    ///
    /// Returns `(relay, cloud_rx)` where `cloud_rx` is the receiving end that
    /// the cloud WebSocket writer task should consume.
    pub fn new() -> (Self, mpsc::Receiver<ChatMessage>) {
        let (to_cloud_tx, to_cloud_rx) = mpsc::channel(256);
        let (to_local_tx, _) = broadcast::channel(BROADCAST_CAPACITY);

        let relay = Self {
            to_cloud_tx,
            to_local_tx,
            queue: Arc::new(RwLock::new(VecDeque::new())),
            cloud_state: Arc::new(RwLock::new(CloudConnectionState::Disconnected)),
        };

        (relay, to_cloud_rx)
    }

    /// Subscribe a local client to receive cloud responses (broadcast).
    pub fn subscribe_local(&self) -> broadcast::Receiver<ChatMessage> {
        self.to_local_tx.subscribe()
    }

    /// Send a user message towards the cloud.
    ///
    /// If the cloud connection is down the message is queued (up to
    /// [`MAX_QUEUE_SIZE`]). Returns the number of currently queued messages
    /// when queuing occurs.
    pub async fn send_to_cloud(&self, msg: ChatMessage) -> Result<(), RelayError> {
        let state = *self.cloud_state.read().await;
        match state {
            CloudConnectionState::Connected => {
                self.to_cloud_tx
                    .send(msg)
                    .await
                    .map_err(|_| RelayError::ChannelClosed)?;
            }
            CloudConnectionState::Disconnected => {
                let mut q = self.queue.write().await;
                if q.len() >= MAX_QUEUE_SIZE {
                    return Err(RelayError::QueueFull);
                }
                q.push_back(msg);
            }
        }
        Ok(())
    }

    /// Deliver a message from the cloud to all local clients.
    ///
    /// Returns the number of receivers that got the message.
    pub fn send_to_local(&self, msg: ChatMessage) -> Result<usize, RelayError> {
        self.to_local_tx
            .send(msg)
            .map_err(|_| RelayError::NoLocalClients)
    }

    /// Mark the cloud connection as up and flush any queued messages.
    pub async fn set_connected(&self) -> Result<Vec<ChatMessage>, RelayError> {
        *self.cloud_state.write().await = CloudConnectionState::Connected;
        self.flush_queue().await
    }

    /// Mark the cloud connection as down and notify local clients.
    pub async fn set_disconnected(&self, session_id: &str) {
        *self.cloud_state.write().await = CloudConnectionState::Disconnected;
        let _ = self.send_to_local(ChatMessage::error(
            session_id.to_string(),
            "Cloud connection lost".to_string(),
        ));
    }

    /// Drain queued messages, sending them through the cloud channel.
    ///
    /// Returns the messages that were flushed (for testing / logging).
    async fn flush_queue(&self) -> Result<Vec<ChatMessage>, RelayError> {
        let mut q = self.queue.write().await;
        let mut flushed = Vec::with_capacity(q.len());
        while let Some(msg) = q.pop_front() {
            self.to_cloud_tx
                .send(msg.clone())
                .await
                .map_err(|_| RelayError::ChannelClosed)?;
            flushed.push(msg);
        }
        Ok(flushed)
    }

    /// Number of messages currently queued.
    pub async fn queue_len(&self) -> usize {
        self.queue.read().await.len()
    }

    /// Current cloud connection state.
    pub async fn cloud_state(&self) -> CloudConnectionState {
        *self.cloud_state.read().await
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RelayError {
    #[error("cloud channel closed")]
    ChannelClosed,
    #[error("message queue is full")]
    QueueFull,
    #[error("no local clients connected")]
    NoLocalClients,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn user_msg(session: &str, content: &str) -> ChatMessage {
        ChatMessage::new(
            ChatMessageType::UserMessage,
            ChatPayload {
                session_id: session.to_string(),
                content: content.to_string(),
                metadata: None,
            },
        )
    }

    fn agent_msg(session: &str, content: &str) -> ChatMessage {
        ChatMessage::new(
            ChatMessageType::AgentResponse,
            ChatPayload {
                session_id: session.to_string(),
                content: content.to_string(),
                metadata: None,
            },
        )
    }

    fn stream_chunk(session: &str, token: &str) -> ChatMessage {
        ChatMessage::new(
            ChatMessageType::StreamChunk,
            ChatPayload {
                session_id: session.to_string(),
                content: token.to_string(),
                metadata: None,
            },
        )
    }

    fn stream_end(session: &str) -> ChatMessage {
        ChatMessage::new(
            ChatMessageType::StreamEnd,
            ChatPayload {
                session_id: session.to_string(),
                content: String::new(),
                metadata: None,
            },
        )
    }

    // -- wire format --------------------------------------------------------

    #[test]
    fn test_chat_message_serialization() {
        let msg = user_msg("s1", "hello");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"v\":1"));
        assert!(json.contains("\"type\":\"user_message\""));
        assert!(json.contains("\"session_id\":\"s1\""));

        let round: ChatMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(round.v, 1);
        assert_eq!(round.payload.content, "hello");
    }

    #[test]
    fn test_session_init_includes_locale() {
        let msg = ChatMessage::session_init("s1".into(), "en-US".into());
        assert_eq!(msg.msg_type, ChatMessageType::SessionInit);
        let meta = msg.payload.metadata.unwrap();
        assert_eq!(meta["locale"], "en-US");
    }

    #[test]
    fn test_error_message_construction() {
        let msg = ChatMessage::error("s1".into(), "boom".into());
        assert_eq!(msg.msg_type, ChatMessageType::Error);
        assert_eq!(msg.payload.content, "boom");
    }

    // -- relay: connected path ----------------------------------------------

    #[tokio::test]
    async fn test_send_to_cloud_when_connected() {
        let (relay, mut cloud_rx) = ChatRelay::new();
        relay.set_connected().await.unwrap();

        let msg = user_msg("s1", "hi");
        relay.send_to_cloud(msg.clone()).await.unwrap();

        let received = cloud_rx.recv().await.unwrap();
        assert_eq!(received.payload.content, "hi");
    }

    #[tokio::test]
    async fn test_send_to_local_broadcast() {
        let (relay, _cloud_rx) = ChatRelay::new();

        let mut rx1 = relay.subscribe_local();
        let mut rx2 = relay.subscribe_local();

        let msg = agent_msg("s1", "answer");
        let count = relay.send_to_local(msg).unwrap();
        assert_eq!(count, 2);

        let r1 = rx1.recv().await.unwrap();
        let r2 = rx2.recv().await.unwrap();
        assert_eq!(r1.payload.content, "answer");
        assert_eq!(r2.payload.content, "answer");
    }

    // -- relay: disconnected / queueing path --------------------------------

    #[tokio::test]
    async fn test_messages_queued_when_disconnected() {
        let (relay, _cloud_rx) = ChatRelay::new();
        // Default state is Disconnected.
        assert_eq!(
            relay.cloud_state().await,
            CloudConnectionState::Disconnected
        );

        relay.send_to_cloud(user_msg("s1", "a")).await.unwrap();
        relay.send_to_cloud(user_msg("s1", "b")).await.unwrap();
        assert_eq!(relay.queue_len().await, 2);
    }

    #[tokio::test]
    async fn test_queue_flushed_on_connect() {
        let (relay, mut cloud_rx) = ChatRelay::new();

        relay
            .send_to_cloud(user_msg("s1", "queued1"))
            .await
            .unwrap();
        relay
            .send_to_cloud(user_msg("s1", "queued2"))
            .await
            .unwrap();

        let flushed = relay.set_connected().await.unwrap();
        assert_eq!(flushed.len(), 2);
        assert_eq!(relay.queue_len().await, 0);

        let r1 = cloud_rx.recv().await.unwrap();
        let r2 = cloud_rx.recv().await.unwrap();
        assert_eq!(r1.payload.content, "queued1");
        assert_eq!(r2.payload.content, "queued2");
    }

    #[tokio::test]
    async fn test_queue_full_returns_error() {
        let (relay, _cloud_rx) = ChatRelay::new();

        for i in 0..MAX_QUEUE_SIZE {
            relay
                .send_to_cloud(user_msg("s1", &format!("m{i}")))
                .await
                .unwrap();
        }

        let result = relay.send_to_cloud(user_msg("s1", "overflow")).await;
        assert_eq!(result, Err(RelayError::QueueFull));
    }

    // -- relay: disconnect notification -------------------------------------

    #[tokio::test]
    async fn test_disconnect_notifies_local_clients() {
        let (relay, _cloud_rx) = ChatRelay::new();
        let mut local_rx = relay.subscribe_local();

        relay.set_disconnected("s1").await;

        let notification = local_rx.recv().await.unwrap();
        assert_eq!(notification.msg_type, ChatMessageType::Error);
        assert_eq!(notification.payload.content, "Cloud connection lost");
    }

    // -- streaming ----------------------------------------------------------

    #[tokio::test]
    async fn test_streaming_token_relay() {
        let (relay, _cloud_rx) = ChatRelay::new();
        let mut local_rx = relay.subscribe_local();

        // Simulate cloud sending streaming tokens.
        relay.send_to_local(stream_chunk("s1", "Hello")).unwrap();
        relay.send_to_local(stream_chunk("s1", " world")).unwrap();
        relay.send_to_local(stream_end("s1")).unwrap();

        let c1 = local_rx.recv().await.unwrap();
        let c2 = local_rx.recv().await.unwrap();
        let end = local_rx.recv().await.unwrap();

        assert_eq!(c1.msg_type, ChatMessageType::StreamChunk);
        assert_eq!(c1.payload.content, "Hello");
        assert_eq!(c2.payload.content, " world");
        assert_eq!(end.msg_type, ChatMessageType::StreamEnd);
    }

    // -- state transitions --------------------------------------------------

    #[tokio::test]
    async fn test_state_transitions() {
        let (relay, _cloud_rx) = ChatRelay::new();

        assert_eq!(
            relay.cloud_state().await,
            CloudConnectionState::Disconnected
        );

        relay.set_connected().await.unwrap();
        assert_eq!(relay.cloud_state().await, CloudConnectionState::Connected);

        relay.set_disconnected("s1").await;
        assert_eq!(
            relay.cloud_state().await,
            CloudConnectionState::Disconnected
        );
    }

    // -- multi-client broadcast ---------------------------------------------

    #[tokio::test]
    async fn test_multiple_local_clients_receive_same_events() {
        let (relay, _cloud_rx) = ChatRelay::new();

        let mut clients: Vec<_> = (0..5).map(|_| relay.subscribe_local()).collect();

        let msg = agent_msg("s1", "broadcast test");
        let count = relay.send_to_local(msg).unwrap();
        assert_eq!(count, 5);

        for client in &mut clients {
            let received = client.recv().await.unwrap();
            assert_eq!(received.payload.content, "broadcast test");
        }
    }
}
