//! WebSocket client for the orchestrator channel.
//! Spec: LocalStack_v2.4.1_Spec.md §2.1, §2.1a
//! For v2.4.1 local dev: connects to ws://localhost:8000/ws/connect
//! Binary Protobuf framing deferred to v2.5.0 — uses JSON passthrough for now.

use crate::config::OrchestratorSection;
use tokio::sync::mpsc;
use tracing::{info, warn};

/// Exponential backoff with max cap for reconnect attempts.
pub struct ReconnectBackoff {
    base_secs: u64,
    current_secs: u64,
    max_secs: u64,
}

impl ReconnectBackoff {
    pub fn new(base: u64, max: u64) -> Self {
        Self { base_secs: base, current_secs: base, max_secs: max }
    }

    pub fn next_delay_secs(&mut self) -> u64 {
        let delay = self.current_secs;
        self.current_secs = (self.current_secs * 2).min(self.max_secs);
        delay
    }

    pub fn reset(&mut self) {
        self.current_secs = self.base_secs;
    }
}

/// Sender channel: messages to send TO the orchestrator.
pub type OrchestratorSender = mpsc::Sender<serde_json::Value>;
/// Receiver channel: messages received FROM the orchestrator (forwarded to local clients).
pub type OrchestratorReceiver = mpsc::Receiver<serde_json::Value>;

/// Run the orchestrator connection loop. Never returns (runs forever with reconnect).
/// Emits connection status changes via `status_tx`.
pub async fn run_orchestrator_loop(
    config: OrchestratorSection,
    status_tx: mpsc::Sender<bool>, // true = connected, false = disconnected
    mut outbound_rx: mpsc::Receiver<serde_json::Value>,
    inbound_tx: mpsc::Sender<serde_json::Value>,
) {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;

    let mut backoff = ReconnectBackoff::new(
        config.reconnect_backoff_base_secs,
        config.reconnect_backoff_max_secs,
    );

    loop {
        info!("Connecting to orchestrator at {}", config.url);
        match connect_async(&config.url).await {
            Ok((ws_stream, _)) => {
                backoff.reset();
                let _ = status_tx.send(true).await;
                info!("Connected to orchestrator");

                let (mut write, mut read) = ws_stream.split();

                // Forward loop: receive from orchestrator → send to local clients
                let mut recv_task = tokio::spawn({
                    let inbound_tx = inbound_tx.clone();
                    async move {
                        while let Some(msg) = read.next().await {
                            match msg {
                                Ok(Message::Text(text)) => {
                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                                        let _ = inbound_tx.send(json).await;
                                    }
                                }
                                Ok(Message::Binary(data)) => {
                                    // For v2.4.1: treat binary as UTF-8 JSON (proto deferred)
                                    if let Ok(text) = std::str::from_utf8(&data) {
                                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
                                            let _ = inbound_tx.send(json).await;
                                        }
                                    }
                                }
                                Ok(Message::Close(_)) => break,
                                Err(e) => {
                                    warn!("Orchestrator WS error: {e}");
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                });

                // Send loop: outbound queue → orchestrator
                loop {
                    tokio::select! {
                        msg = outbound_rx.recv() => {
                            match msg {
                                Some(json) => {
                                    let text = serde_json::to_string(&json).unwrap_or_default();
                                    if let Err(e) = write.send(Message::Text(text)).await {
                                        warn!("Failed to send to orchestrator: {e}");
                                        break;
                                    }
                                }
                                None => break, // channel closed
                            }
                        }
                        _ = &mut recv_task => break, // recv loop ended
                    }
                }
            }
            Err(e) => {
                warn!("Failed to connect to orchestrator: {e}");
            }
        }

        let _ = status_tx.send(false).await;
        warn!("Orchestrator disconnected. Reconnecting...");

        let delay = backoff.next_delay_secs();
        tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_sequence() {
        let mut backoff = ReconnectBackoff::new(1, 30);
        let delays: Vec<u64> = (0..6).map(|_| backoff.next_delay_secs()).collect();
        // Should be: 1, 2, 4, 8, 16, 30 (capped)
        assert_eq!(delays[0], 1);
        assert_eq!(delays[1], 2);
        assert_eq!(delays[2], 4);
        assert_eq!(delays[3], 8);
        assert_eq!(delays[4], 16);
        assert_eq!(delays[5], 30); // capped at max
    }

    #[test]
    fn test_backoff_reset() {
        let mut backoff = ReconnectBackoff::new(1, 30);
        backoff.next_delay_secs(); // 1
        backoff.next_delay_secs(); // 2
        backoff.next_delay_secs(); // 4
        backoff.reset();
        assert_eq!(backoff.next_delay_secs(), 1); // back to base
    }

    #[test]
    fn test_backoff_capped_at_max() {
        let mut backoff = ReconnectBackoff::new(10, 30);
        backoff.next_delay_secs(); // 10
        backoff.next_delay_secs(); // 20
        let third = backoff.next_delay_secs(); // 30 (capped)
        let fourth = backoff.next_delay_secs(); // still 30
        assert_eq!(third, 30);
        assert_eq!(fourth, 30);
    }
}
