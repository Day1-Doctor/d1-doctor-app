//! Cloud WebSocket client for the daemon.
//!
//! Connects to `wss://api.day1.doctor/ws/daemon`, authenticates with a JWT token,
//! maintains a heartbeat, and auto-reconnects with exponential backoff.

use std::sync::Arc;
use std::time::Duration;

use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, watch, RwLock};
use tokio::time;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Default cloud WebSocket endpoint.
pub const DEFAULT_CLOUD_WS_URL: &str = "wss://api.day1.doctor/ws/daemon";

/// Heartbeat interval.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// Auth handshake timeout.
const AUTH_TIMEOUT: Duration = Duration::from_secs(10);

/// Backoff parameters.
const BACKOFF_INITIAL: Duration = Duration::from_secs(1);
const BACKOFF_MAX: Duration = Duration::from_secs(30);
const BACKOFF_MULTIPLIER: u32 = 2;

// ---------------------------------------------------------------------------
// Connection state
// ---------------------------------------------------------------------------

/// State of the cloud WebSocket connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Authenticating,
    Connected,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "disconnected"),
            Self::Connecting => write!(f, "connecting"),
            Self::Authenticating => write!(f, "authenticating"),
            Self::Connected => write!(f, "connected"),
        }
    }
}

// ---------------------------------------------------------------------------
// Protocol messages  (v1 wire format: { v, id, ts, type, payload })
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    pub v: u32,
    pub id: String,
    pub ts: i64,
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

impl WsMessage {
    pub fn new(msg_type: &str, payload: serde_json::Value) -> Self {
        Self {
            v: 1,
            id: Uuid::new_v4().to_string(),
            ts: chrono::Utc::now().timestamp_millis(),
            msg_type: msg_type.to_string(),
            payload,
        }
    }
}

/// Payload sent with the AUTH message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthPayload {
    pub jwt: String,
    pub device_fingerprint: String,
}

// ---------------------------------------------------------------------------
// CloudWsClient
// ---------------------------------------------------------------------------

/// Configuration for the cloud WebSocket client.
#[derive(Debug, Clone)]
pub struct CloudWsConfig {
    pub url: String,
    pub jwt: String,
    pub device_fingerprint: String,
}

/// Cloud WebSocket client that manages the connection lifecycle.
///
/// Call [`CloudWsClient::spawn`] to start the background connection loop.
/// Use [`CloudWsClient::state`] to query the current connection state and
/// [`CloudWsClient::subscribe_state`] to watch for state changes.
pub struct CloudWsClient {
    state: Arc<RwLock<ConnectionState>>,
    state_tx: watch::Sender<ConnectionState>,
    state_rx: watch::Receiver<ConnectionState>,
    shutdown_tx: watch::Sender<bool>,
}

impl CloudWsClient {
    /// Create a new client (does **not** connect yet).
    pub fn new() -> Self {
        let (state_tx, state_rx) = watch::channel(ConnectionState::Disconnected);
        let (shutdown_tx, _) = watch::channel(false);
        Self {
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            state_tx,
            state_rx,
            shutdown_tx,
        }
    }

    /// Current connection state.
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Subscribe to state changes.
    pub fn subscribe_state(&self) -> watch::Receiver<ConnectionState> {
        self.state_rx.clone()
    }

    /// Spawn the background connection loop.
    ///
    /// Returns a `JoinHandle` for the task. The loop will keep reconnecting
    /// with exponential backoff until [`CloudWsClient::shutdown`] is called.
    ///
    /// - `outbound_rx`: messages from the daemon to send to the cloud WS.
    /// - `inbound_tx`: messages received from the cloud WS to forward into the daemon.
    pub fn spawn(
        &self,
        config: CloudWsConfig,
        outbound_rx: mpsc::Receiver<String>,
        inbound_tx: mpsc::Sender<String>,
    ) -> tokio::task::JoinHandle<()> {
        let state = Arc::clone(&self.state);
        let state_tx = self.state_tx.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        // mpsc::Receiver is not Clone — move it into the spawned task.
        let mut outbound_rx = outbound_rx;

        tokio::spawn(async move {
            let mut backoff = BACKOFF_INITIAL;

            loop {
                // Check shutdown before attempting connection.
                if *shutdown_rx.borrow() {
                    set_state(&state, &state_tx, ConnectionState::Disconnected).await;
                    info!("cloud_ws: shutdown requested, exiting connection loop");
                    return;
                }

                set_state(&state, &state_tx, ConnectionState::Connecting).await;
                info!(url = %config.url, "cloud_ws: connecting");

                match connect_and_auth(&config, &state, &state_tx).await {
                    Ok((mut sink, mut stream)) => {
                        info!("cloud_ws: authenticated, entering message loop");
                        backoff = BACKOFF_INITIAL; // reset on success

                        // Run the message loop (heartbeat + read + send).
                        let reason = message_loop(
                            &mut sink,
                            &mut stream,
                            &mut shutdown_rx,
                            &mut outbound_rx,
                            &inbound_tx,
                        )
                        .await;

                        info!(?reason, "cloud_ws: message loop ended");
                    }
                    Err(e) => {
                        warn!(%e, "cloud_ws: connection/auth failed");
                    }
                }

                set_state(&state, &state_tx, ConnectionState::Disconnected).await;

                // Check shutdown before sleeping.
                if *shutdown_rx.borrow() {
                    info!("cloud_ws: shutdown requested, exiting connection loop");
                    return;
                }

                info!(
                    backoff_secs = backoff.as_secs(),
                    "cloud_ws: reconnecting after backoff"
                );
                tokio::select! {
                    _ = time::sleep(backoff) => {}
                    _ = shutdown_rx.changed() => {
                        info!("cloud_ws: shutdown during backoff");
                        return;
                    }
                }

                // Exponential backoff.
                backoff = std::cmp::min(backoff * BACKOFF_MULTIPLIER, BACKOFF_MAX);
            }
        })
    }

    /// Signal the background loop to shut down.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

type WsSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type WsStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

async fn set_state(
    state: &Arc<RwLock<ConnectionState>>,
    tx: &watch::Sender<ConnectionState>,
    new: ConnectionState,
) {
    *state.write().await = new;
    let _ = tx.send(new);
    debug!(%new, "cloud_ws: state changed");
}

/// Connect to the server and perform the AUTH handshake.
async fn connect_and_auth(
    config: &CloudWsConfig,
    state: &Arc<RwLock<ConnectionState>>,
    state_tx: &watch::Sender<ConnectionState>,
) -> anyhow::Result<(WsSink, WsStream)> {
    let (ws, _resp) = connect_async(&config.url).await?;
    let (mut sink, mut stream) = ws.split();

    // Transition to Authenticating.
    set_state(state, state_tx, ConnectionState::Authenticating).await;

    // Build and send AUTH message (ChatMessage v1 format: payload.content is JSON string).
    let auth_payload = AuthPayload {
        jwt: config.jwt.clone(),
        device_fingerprint: config.device_fingerprint.clone(),
    };
    let content_json = serde_json::to_string(&auth_payload)?;
    let v1_payload = serde_json::json!({
        "session_id": "",
        "content": content_json,
    });
    let auth_msg = WsMessage::new("AUTH", v1_payload);
    let text = serde_json::to_string(&auth_msg)?;
    sink.send(Message::Text(text)).await?;
    debug!("cloud_ws: AUTH sent, waiting for response");

    // Wait for AUTH_OK or AUTH_FAIL.
    let response = time::timeout(AUTH_TIMEOUT, stream.next()).await;

    match response {
        Ok(Some(Ok(Message::Text(text)))) => {
            let msg: WsMessage = serde_json::from_str(&text)?;
            match msg.msg_type.as_str() {
                "AUTH_OK" => {
                    info!("cloud_ws: AUTH_OK received");
                    set_state(state, state_tx, ConnectionState::Connected).await;
                    Ok((sink, stream))
                }
                "AUTH_FAIL" => {
                    let reason = msg
                        .payload
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    Err(anyhow::anyhow!("AUTH_FAIL: {}", reason))
                }
                other => Err(anyhow::anyhow!(
                    "unexpected message type during auth: {}",
                    other
                )),
            }
        }
        Ok(Some(Ok(Message::Close(_)))) => Err(anyhow::anyhow!("server closed during auth")),
        Ok(Some(Err(e))) => Err(anyhow::anyhow!("ws error during auth: {}", e)),
        Ok(None) => Err(anyhow::anyhow!("stream ended during auth")),
        Err(_) => Err(anyhow::anyhow!("auth handshake timed out")),
        _ => Err(anyhow::anyhow!("unexpected frame during auth")),
    }
}

/// Reason the message loop exited.
#[derive(Debug)]
enum LoopExit {
    ServerClosed,
    Error(String),
    Shutdown,
}

/// Main message loop: sends heartbeats, reads incoming messages, and sends
/// outbound messages from the daemon to the cloud.
async fn message_loop(
    sink: &mut WsSink,
    stream: &mut WsStream,
    shutdown_rx: &mut watch::Receiver<bool>,
    outbound_rx: &mut mpsc::Receiver<String>,
    inbound_tx: &mpsc::Sender<String>,
) -> LoopExit {
    let mut heartbeat_interval = time::interval(HEARTBEAT_INTERVAL);
    // Skip the first immediate tick.
    heartbeat_interval.tick().await;

    loop {
        tokio::select! {
            _ = heartbeat_interval.tick() => {
                let ping = WsMessage::new("HEARTBEAT", serde_json::json!({}));
                match serde_json::to_string(&ping) {
                    Ok(text) => {
                        if let Err(e) = sink.send(Message::Text(text)).await {
                            return LoopExit::Error(format!("heartbeat send failed: {e}"));
                        }
                        debug!("cloud_ws: heartbeat sent");
                    }
                    Err(e) => {
                        return LoopExit::Error(format!("heartbeat serialize failed: {e}"));
                    }
                }
            }
            msg = stream.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        debug!(len = text.len(), "cloud_ws: received text message");
                        // Forward inbound cloud message to the daemon relay.
                        if let Err(e) = inbound_tx.send(text).await {
                            warn!("cloud_ws: failed to forward inbound message: {}", e);
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if let Err(e) = sink.send(Message::Pong(data)).await {
                            return LoopExit::Error(format!("pong send failed: {e}"));
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("cloud_ws: server sent close frame");
                        return LoopExit::ServerClosed;
                    }
                    Some(Err(e)) => {
                        error!(%e, "cloud_ws: read error");
                        return LoopExit::Error(e.to_string());
                    }
                    None => {
                        info!("cloud_ws: stream ended");
                        return LoopExit::ServerClosed;
                    }
                    _ => {} // Binary, Pong, Frame — ignore.
                }
            }
            // Send outbound messages from the daemon to the cloud WS.
            Some(text) = outbound_rx.recv() => {
                if let Err(e) = sink.send(Message::Text(text)).await {
                    return LoopExit::Error(format!("outbound send failed: {e}"));
                }
                debug!("cloud_ws: sent outbound message");
            }
            _ = shutdown_rx.changed() => {
                info!("cloud_ws: shutdown signal received");
                let _ = sink.send(Message::Close(None)).await;
                return LoopExit::Shutdown;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_display() {
        assert_eq!(ConnectionState::Disconnected.to_string(), "disconnected");
        assert_eq!(ConnectionState::Connecting.to_string(), "connecting");
        assert_eq!(
            ConnectionState::Authenticating.to_string(),
            "authenticating"
        );
        assert_eq!(ConnectionState::Connected.to_string(), "connected");
    }

    #[test]
    fn test_ws_message_new() {
        let msg = WsMessage::new("AUTH", serde_json::json!({"jwt": "tok"}));
        assert_eq!(msg.v, 1);
        assert_eq!(msg.msg_type, "AUTH");
        assert!(!msg.id.is_empty());
        assert!(msg.ts > 0);
    }

    #[test]
    fn test_ws_message_roundtrip() {
        let msg = WsMessage::new("HEARTBEAT", serde_json::json!({}));
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.v, msg.v);
        assert_eq!(parsed.msg_type, "HEARTBEAT");
        assert_eq!(parsed.id, msg.id);
    }

    #[test]
    fn test_auth_payload_serialization() {
        let payload = AuthPayload {
            jwt: "my.jwt.token".to_string(),
            device_fingerprint: "fp-123".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("my.jwt.token"));
        assert!(json.contains("fp-123"));

        let parsed: AuthPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.jwt, "my.jwt.token");
        assert_eq!(parsed.device_fingerprint, "fp-123");
    }

    #[test]
    fn test_auth_message_wire_format() {
        let auth_payload = AuthPayload {
            jwt: "tok".to_string(),
            device_fingerprint: "dev1".to_string(),
        };
        let msg = WsMessage::new("AUTH", serde_json::to_value(&auth_payload).unwrap());
        let wire: serde_json::Value = serde_json::to_value(&msg).unwrap();

        assert_eq!(wire["v"], 1);
        assert_eq!(wire["type"], "AUTH");
        assert_eq!(wire["payload"]["jwt"], "tok");
        assert_eq!(wire["payload"]["device_fingerprint"], "dev1");
    }

    #[tokio::test]
    async fn test_client_initial_state() {
        let client = CloudWsClient::new();
        assert_eq!(client.state().await, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_client_state_subscribe() {
        let client = CloudWsClient::new();
        let rx = client.subscribe_state();
        assert_eq!(*rx.borrow(), ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_client_shutdown_idempotent() {
        let client = CloudWsClient::new();
        client.shutdown();
        client.shutdown(); // should not panic
        assert_eq!(client.state().await, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_set_state_updates_both() {
        let state = Arc::new(RwLock::new(ConnectionState::Disconnected));
        let (tx, rx) = watch::channel(ConnectionState::Disconnected);

        set_state(&state, &tx, ConnectionState::Connecting).await;
        assert_eq!(*state.read().await, ConnectionState::Connecting);
        assert_eq!(*rx.borrow(), ConnectionState::Connecting);

        set_state(&state, &tx, ConnectionState::Connected).await;
        assert_eq!(*state.read().await, ConnectionState::Connected);
        assert_eq!(*rx.borrow(), ConnectionState::Connected);
    }

    #[test]
    fn test_backoff_progression() {
        let mut backoff = BACKOFF_INITIAL;
        let expected = [1, 2, 4, 8, 16, 30, 30];
        for &exp in &expected {
            assert_eq!(backoff.as_secs(), exp);
            backoff = std::cmp::min(backoff * BACKOFF_MULTIPLIER, BACKOFF_MAX);
        }
    }

    #[tokio::test]
    async fn test_spawn_shutdown_immediate() {
        let client = CloudWsClient::new();
        let config = CloudWsConfig {
            url: "ws://127.0.0.1:1".to_string(), // will fail to connect
            jwt: "test".to_string(),
            device_fingerprint: "fp".to_string(),
        };

        let (_, outbound_rx) = mpsc::channel(1);
        let (inbound_tx, _) = mpsc::channel(1);
        let handle = client.spawn(config, outbound_rx, inbound_tx);
        // Give it a moment to start, then shut down.
        tokio::time::sleep(Duration::from_millis(50)).await;
        client.shutdown();
        // Should exit gracefully.
        let result = tokio::time::timeout(Duration::from_secs(5), handle).await;
        assert!(result.is_ok(), "spawn task should exit after shutdown");
    }

    #[test]
    fn test_connection_state_equality() {
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
        assert_ne!(ConnectionState::Disconnected, ConnectionState::Connected);
    }

    #[test]
    fn test_cloud_ws_config_clone() {
        let config = CloudWsConfig {
            url: "wss://example.com/ws".to_string(),
            jwt: "token".to_string(),
            device_fingerprint: "fp-1".to_string(),
        };
        let cloned = config.clone();
        assert_eq!(cloned.url, config.url);
        assert_eq!(cloned.jwt, config.jwt);
        assert_eq!(cloned.device_fingerprint, config.device_fingerprint);
    }

    #[test]
    fn test_default_constants() {
        assert_eq!(HEARTBEAT_INTERVAL, Duration::from_secs(30));
        assert_eq!(AUTH_TIMEOUT, Duration::from_secs(10));
        assert_eq!(BACKOFF_INITIAL, Duration::from_secs(1));
        assert_eq!(BACKOFF_MAX, Duration::from_secs(30));
    }
}
