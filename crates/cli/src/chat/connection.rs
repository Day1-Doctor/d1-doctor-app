//! Connection management for chat sessions.
//!
//! Supports connecting to the local daemon via WebSocket
//! or directly to the cloud orchestrator.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use d1_common::proto::{Envelope, EnvelopePayload, UserRequest};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

/// Where to connect for the chat session.
#[derive(Debug, Clone)]
pub enum ConnectionTarget {
    /// Connect to local daemon on the given port.
    Local(u16),
    /// Connect to a cloud WebSocket URL.
    Cloud(String),
}

impl std::fmt::Display for ConnectionTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionTarget::Local(port) => write!(f, "localhost:{}", port),
            ConnectionTarget::Cloud(url) => write!(f, "{}", url),
        }
    }
}

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// A WebSocket connection to the daemon or cloud.
pub struct ChatConnection {
    ws: Option<WsStream>,
    #[allow(dead_code)]
    target: ConnectionTarget,
}

impl ChatConnection {
    /// Establish a WebSocket connection to the target.
    pub async fn connect(target: &ConnectionTarget) -> Result<Self> {
        let url = match target {
            ConnectionTarget::Local(port) => format!("ws://127.0.0.1:{}/chat", port),
            ConnectionTarget::Cloud(url) => url.clone(),
        };

        let (ws, _response) = tokio_tungstenite::connect_async(&url)
            .await
            .with_context(|| format!("Failed to connect to {}", url))?;

        Ok(Self {
            ws: Some(ws),
            target: target.clone(),
        })
    }

    /// Send a user message and stream the response.
    ///
    /// Returns the full concatenated response text.
    /// Checks `cancel` flag between chunks; if set, returns a cancellation error.
    pub async fn send_and_stream(
        &mut self,
        session_id: &str,
        message: &str,
        cancel: &Arc<AtomicBool>,
    ) -> Result<String> {
        let ws = self.ws.as_mut().context("Not connected")?;

        let request = UserRequest::new(session_id.to_string(), message.to_string());
        let envelope = Envelope::new("cli".to_string(), EnvelopePayload::UserRequest(request));
        let json = serde_json::to_string(&envelope)?;
        ws.send(Message::Text(json)).await?;

        let mut response = String::new();

        while let Some(msg) = ws.next().await {
            if cancel.load(Ordering::Relaxed) {
                return Err(anyhow::anyhow!("Response cancelled"));
            }

            match msg? {
                Message::Text(text) => {
                    let env: Envelope = serde_json::from_str(&text)?;
                    match env.payload {
                        EnvelopePayload::PlanProposal(plan) => {
                            for step in &plan.steps {
                                response.push_str(&step.description);
                                response.push('\n');
                            }
                            break;
                        }
                        EnvelopePayload::Error(err) => {
                            return Err(anyhow::anyhow!("Agent error: {}", err.message));
                        }
                        _ => {
                            // Accumulate any text-based payload
                            response.push_str(&text);
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }

        Ok(response)
    }

    /// Gracefully disconnect.
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(mut ws) = self.ws.take() {
            let _ = ws.close(None).await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_target_display() {
        let local = ConnectionTarget::Local(9876);
        assert_eq!(local.to_string(), "localhost:9876");

        let cloud = ConnectionTarget::Cloud("wss://api.example.com/ws".to_string());
        assert_eq!(cloud.to_string(), "wss://api.example.com/ws");
    }
}
