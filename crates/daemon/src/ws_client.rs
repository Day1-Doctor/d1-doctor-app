//! WebSocket client for communicating with the cloud orchestrator.
//!
//! Sends and receives binary Protobuf Envelope frames.
//! Reconnects automatically with exponential backoff on disconnect.

use anyhow::{Context, Result};
use d1_common::proto::{decode_envelope, encode_envelope, Envelope};
use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
};
use tracing::{error, info, warn};

pub struct WsClient {
    url: String,
}

impl WsClient {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }

    /// Connect to the orchestrator WebSocket endpoint.
    /// Retries with exponential backoff (1s → 2s → 4s → … → 60s cap).
    pub async fn connect_with_retry(&self) -> Result<WsConnection> {
        let mut delay = Duration::from_secs(1);
        let max_delay = Duration::from_secs(60);

        loop {
            match connect_async(&self.url).await {
                Ok((stream, _)) => {
                    info!(url = %self.url, "WebSocket connected");
                    return Ok(WsConnection::new(stream));
                }
                Err(e) => {
                    warn!(error = %e, retry_in = ?delay, "WebSocket connection failed, retrying");
                    sleep(delay).await;
                    delay = (delay * 2).min(max_delay);
                }
            }
        }
    }
}

type WsStream = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
>;

pub struct WsConnection {
    inner: WsStream,
}

impl WsConnection {
    fn new(stream: WsStream) -> Self {
        Self { inner: stream }
    }

    /// Send an Envelope as a binary WebSocket frame.
    pub async fn send(&mut self, envelope: &Envelope) -> Result<()> {
        let bytes = encode_envelope(envelope);
        self.inner
            .send(Message::Binary(bytes))
            .await
            .context("Failed to send envelope")?;
        Ok(())
    }

    /// Receive the next Envelope from the WebSocket.
    /// Returns None if the connection was closed cleanly.
    pub async fn recv(&mut self) -> Result<Option<Envelope>> {
        loop {
            match self.inner.next().await {
                Some(Ok(Message::Binary(bytes))) => {
                    let env = decode_envelope(&bytes)
                        .map_err(|e| anyhow::anyhow!("Failed to decode envelope: {}", e))
                        .context("Failed to decode envelope from binary frame")?;
                    return Ok(Some(env));
                }
                Some(Ok(Message::Ping(data))) => {
                    self.inner.send(Message::Pong(data)).await.ok();
                    continue;
                }
                Some(Ok(Message::Close(_))) | None => {
                    return Ok(None);
                }
                Some(Ok(_)) => continue,
                Some(Err(e)) => {
                    error!(error = %e, "WebSocket receive error");
                    return Err(e.into());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use d1_common::proto::{decode_envelope, encode_envelope, heartbeat_envelope, MessageType};

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = heartbeat_envelope("session-abc");
        let bytes = encode_envelope(&original);
        let decoded = decode_envelope(&bytes).expect("decode should succeed");

        assert_eq!(decoded.id, original.id);
        assert_eq!(decoded.session_id, "session-abc");
        assert_eq!(decoded.r#type, MessageType::Heartbeat as i32);
    }

    #[test]
    fn test_heartbeat_envelope_fields() {
        let env = heartbeat_envelope("test-session-123");
        assert!(!env.id.is_empty(), "id should be a non-empty UUID");
        assert_eq!(env.session_id, "test-session-123");
        assert_eq!(env.r#type, MessageType::Heartbeat as i32);
        assert!(env.timestamp_ms > 0, "timestamp should be set");
    }
}
