//! CLI → Daemon WebSocket client.
//! Connects to ws://localhost:9876/ws, sends/receives JSON protocol v1 messages.

use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

const DAEMON_WS_URL: &str = "ws://localhost:9876/ws";

// ── Outbound (CLI → Daemon) ───────────────────────────────────────────────────

#[derive(Serialize)]
pub struct Envelope<P: Serialize> {
    pub v: u8,
    pub id: String,
    pub ts: u64,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub payload: P,
}

impl<P: Serialize> Envelope<P> {
    pub fn new(msg_type: &str, payload: P) -> Self {
        Self {
            v: 1,
            id: Uuid::new_v4().to_string(),
            ts: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            msg_type: msg_type.to_string(),
            payload,
        }
    }
}

#[derive(Serialize)]
pub struct TaskSubmitPayload {
    pub task_id: String,
    pub input: String,
    pub context: serde_json::Value,
}

#[derive(Serialize)]
pub struct PlanApprovePayload {
    pub task_id: String,
    pub plan_id: String,
    pub action: String, // "APPROVE" or "REJECT"
    pub modifications: Option<serde_json::Value>,
}

// ── Inbound (Daemon → CLI) ────────────────────────────────────────────────────

#[derive(Deserialize, Debug, Clone)]
pub struct InboundEnvelope {
    pub v: u8,
    pub id: String,
    pub ts: u64,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub payload: serde_json::Value,
}

// ── Connection ────────────────────────────────────────────────────────────────

pub struct DaemonClient {
    sender: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    receiver: futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
}

impl DaemonClient {
    /// Connect to daemon WS. Fails if daemon not running.
    pub async fn connect() -> Result<Self> {
        let (ws, _) = connect_async(DAEMON_WS_URL)
            .await
            .map_err(|e| anyhow!("Cannot connect to daemon at {DAEMON_WS_URL}: {e}\nIs the daemon running? Try: d1 start"))?;
        let (sender, receiver) = ws.split();
        Ok(Self { sender, receiver })
    }

    /// Send a JSON-serializable message.
    pub async fn send<P: Serialize>(&mut self, msg_type: &str, payload: P) -> Result<()> {
        let envelope = Envelope::new(msg_type, payload);
        let json = serde_json::to_string(&envelope)?;
        self.sender.send(Message::Text(json.into())).await?;
        Ok(())
    }

    /// Wait for the next message from daemon (with timeout).
    pub async fn recv_timeout(&mut self, timeout_secs: u64) -> Result<Option<InboundEnvelope>> {
        let result = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            self.receiver.next(),
        )
        .await;

        match result {
            Ok(Some(Ok(Message::Text(text)))) => {
                let env: InboundEnvelope = serde_json::from_str(&text)
                    .map_err(|e| anyhow!("Failed to parse daemon message: {e}\nRaw: {text}"))?;
                Ok(Some(env))
            }
            Ok(Some(Ok(_))) => Ok(None), // binary/ping/pong, skip
            Ok(Some(Err(e))) => Err(anyhow!("WebSocket error: {e}")),
            Ok(None) => Ok(None), // stream closed
            Err(_) => Err(anyhow!("Timeout waiting for daemon response")),
        }
    }
}

/// Check if daemon is reachable on port 9876.
pub async fn ping_daemon() -> bool {
    tokio::net::TcpStream::connect("127.0.0.1:9876")
        .await
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_serialization() {
        let env = Envelope::new("heartbeat", serde_json::json!({"ping": true}));
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("\"type\":\"heartbeat\""));
        assert!(json.contains("\"v\":1"));
        assert!(json.contains("\"ping\":true"));
    }

    #[test]
    fn test_envelope_has_uuid_id() {
        let env = Envelope::new("test", serde_json::json!({}));
        assert_eq!(env.id.len(), 36); // UUID4 format
        assert!(env.id.contains('-'));
    }

    #[test]
    fn test_envelope_has_timestamp() {
        let env = Envelope::new("test", serde_json::json!({}));
        assert!(env.ts > 0);
    }

    #[tokio::test]
    async fn test_ping_daemon_returns_false_when_not_running() {
        // If no daemon running, should return false (not panic)
        // (could return true if daemon is running, that's fine too)
        let result = ping_daemon().await;
        // Just verify it doesn't panic
        let _ = result;
    }
}
