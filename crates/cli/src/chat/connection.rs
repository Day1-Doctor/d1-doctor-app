//! Connection management for chat sessions.

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
    Local(u16),
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

pub struct ChatConnection {
    ws: Option<WsStream>,
    #[allow(dead_code)]
    target: ConnectionTarget,
}

impl ChatConnection {
    pub async fn connect(target: &ConnectionTarget) -> Result<Self> {
        let url = match target {
            ConnectionTarget::Local(port) => format!("ws://127.0.0.1:{}/chat", port),
            ConnectionTarget::Cloud(url) => url.clone(),
        };

        let (ws, _response) = tokio_tungstenite::connect_async(&url)
            .await
            .with_context(|| {
                crate::i18n::t_args("errors.connection_failed", &[("url", &url)])
            })?;

        Ok(Self {
            ws: Some(ws),
            target: target.clone(),
        })
    }

    pub async fn send_and_stream(
        &mut self,
        session_id: &str,
        message: &str,
        cancel: &Arc<AtomicBool>,
    ) -> Result<String> {
        let ws = self
            .ws
            .as_mut()
            .context(crate::i18n::t("errors.not_connected"))?;

        let request = UserRequest::new(session_id.to_string(), message.to_string());
        let envelope = Envelope::new("cli".to_string(), EnvelopePayload::UserRequest(request));
        let json = serde_json::to_string(&envelope)?;
        ws.send(Message::Text(json)).await?;

        let mut response = String::new();

        while let Some(msg) = ws.next().await {
            if cancel.load(Ordering::Relaxed) {
                return Err(anyhow::anyhow!(
                    "{}",
                    crate::i18n::t("errors.response_cancelled")
                ));
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
                            return Err(anyhow::anyhow!(
                                "{}",
                                crate::i18n::t_args(
                                    "errors.agent_error",
                                    &[("message", &err.message)]
                                )
                            ));
                        }
                        _ => {
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
