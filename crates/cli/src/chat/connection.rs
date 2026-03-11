//! Connection management for chat sessions.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use d1_common::chat_message::{ChatMessage, ChatMessageType, ChatPayload};
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
            .with_context(|| crate::i18n::t_args("errors.connection_failed", &[("url", &url)]))?;

        Ok(Self {
            ws: Some(ws),
            target: target.clone(),
        })
    }

    /// Send a session_init message so the server knows our session + locale.
    pub async fn send_session_init(&mut self, session_id: &str, locale: &str) -> Result<()> {
        let ws = self
            .ws
            .as_mut()
            .context(crate::i18n::t("errors.not_connected"))?;

        let msg = ChatMessage::session_init(session_id.to_string(), locale.to_string());
        let json = serde_json::to_string(&msg)?;
        ws.send(Message::Text(json)).await?;
        Ok(())
    }

    /// Send a user message and stream back the agent response.
    ///
    /// `on_chunk` is invoked for every `StreamChunk` token received. The full
    /// assembled response is returned when the stream finishes.
    pub async fn send_and_stream(
        &mut self,
        session_id: &str,
        message: &str,
        cancel: &Arc<AtomicBool>,
        on_chunk: impl Fn(&str),
    ) -> Result<String> {
        let ws = self
            .ws
            .as_mut()
            .context(crate::i18n::t("errors.not_connected"))?;

        // Build a ChatMessage v1 user message.
        let chat_msg = ChatMessage::new(
            ChatMessageType::UserMessage,
            ChatPayload {
                session_id: session_id.to_string(),
                content: message.to_string(),
                metadata: None,
            },
        );
        let json = serde_json::to_string(&chat_msg)?;
        ws.send(Message::Text(json)).await?;

        let mut full_response = String::new();

        while let Some(msg) = ws.next().await {
            if cancel.load(Ordering::Relaxed) {
                return Err(anyhow::anyhow!(
                    "{}",
                    crate::i18n::t("errors.response_cancelled")
                ));
            }

            match msg? {
                Message::Text(text) => {
                    let incoming: ChatMessage = serde_json::from_str(&text)?;
                    match incoming.msg_type {
                        ChatMessageType::StreamChunk => {
                            on_chunk(&incoming.payload.content);
                            full_response.push_str(&incoming.payload.content);
                        }
                        ChatMessageType::StreamEnd => {
                            break;
                        }
                        ChatMessageType::AgentResponse => {
                            full_response = incoming.payload.content;
                            break;
                        }
                        ChatMessageType::Error => {
                            return Err(anyhow::anyhow!(
                                "{}",
                                crate::i18n::t_args(
                                    "errors.agent_error",
                                    &[("message", &incoming.payload.content)]
                                )
                            ));
                        }
                        _ => {
                            // Ignore other message types (e.g. SessionInit echo).
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }

        Ok(full_response)
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
