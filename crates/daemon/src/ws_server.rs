//! Local WebSocket server for client connections (Mac app, CLI).
//! Listens on localhost:9876/ws — JSON protocol per §1.2
//! Spec: LocalStack_v2.4.1_Spec.md §2.2

use crate::local_db::LocalDb;
use crate::protocol::{ClientEnvelope, DaemonEnvelope, PROTOCOL_VERSION};
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn};
use uuid::Uuid;

/// Shared state across all client connections.
#[derive(Clone)]
pub struct ServerState {
    /// Map from task_id → channel to send daemon events to subscribed client.
    pub task_subs: Arc<Mutex<HashMap<String, mpsc::Sender<DaemonEnvelope>>>>,
    /// Sender for forwarding client messages to the orchestrator.
    pub to_orchestrator: mpsc::Sender<serde_json::Value>,
    /// Database (shared).
    pub db: Arc<Mutex<LocalDb>>,
    /// Current daemon version string.
    pub version: String,
    /// Whether orchestrator is connected.
    pub orch_connected: Arc<Mutex<bool>>,
    /// Orchestrator URL (from config), included in daemon.status responses.
    pub orchestrator_url: String,
    /// Device ID (from config auth section), included in daemon.status responses.
    pub device_id: String,
}

impl ServerState {
    pub fn new(db: Arc<Mutex<LocalDb>>, to_orchestrator: mpsc::Sender<serde_json::Value>) -> Self {
        Self {
            task_subs: Arc::new(Mutex::new(HashMap::new())),
            to_orchestrator,
            db,
            version: env!("CARGO_PKG_VERSION").to_string(),
            orch_connected: Arc::new(Mutex::new(false)),
            orchestrator_url: String::new(),
            device_id: String::new(),
        }
    }

    pub async fn is_orch_connected(&self) -> bool {
        *self.orch_connected.lock().await
    }

    pub async fn set_orch_connected(&self, connected: bool) {
        *self.orch_connected.lock().await = connected;
    }
}

/// Run the local WebSocket server on the given port.
pub async fn run_ws_server(port: u16, state: ServerState) -> Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!("Local WS server listening on ws://{}/ws", addr);
    run_ws_server_on(listener, state).await
}

/// Run on a pre-bound listener (testable version).
pub async fn run_ws_server_on(listener: TcpListener, state: ServerState) -> Result<()> {
    loop {
        let (stream, addr) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, addr, state).await {
                warn!("Connection error from {}: {}", addr, e);
            }
        });
    }
}

async fn handle_connection(stream: TcpStream, addr: SocketAddr, state: ServerState) -> Result<()> {
    // Enforce /ws path per spec §2.2. The server binds on 127.0.0.1 (loopback-only)
    // so the security risk of path-mismatch is minimal, but we reject non-/ws paths for
    // spec compliance. ErrorResponse = tungstenite::http::Response<Option<String>>.
    use tokio_tungstenite::tungstenite::http;
    let ws_stream: tokio_tungstenite::WebSocketStream<TcpStream> =
        tokio_tungstenite::accept_hdr_async(stream, |req: &Request, response: Response| {
            let path = req.uri().path();
            if path != "/ws" {
                let error_response = http::Response::builder()
                    .status(http::StatusCode::NOT_FOUND)
                    .body(Some(format!("Not found: {}", path)))
                    .expect("failed to build error response");
                Err(error_response)
            } else {
                Ok(response)
            }
        }).await?;
    info!("New client connected: {}", addr);

    let (mut write, mut read) = ws_stream.split();

    // Create per-client channel for server-push messages
    let (push_tx, mut push_rx) = mpsc::channel::<DaemonEnvelope>(64);
    let _client_id = Uuid::new_v4().to_string();

    // Send initial daemon.status
    let orch_connected = state.is_orch_connected().await;
    let status = DaemonEnvelope::daemon_status(&state.version, orch_connected, 0, &state.orchestrator_url, &state.device_id);
    let status_json = serde_json::to_string(&status)?;
    write.send(Message::Text(status_json)).await?;

    loop {
        tokio::select! {
            // Receive from client
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientEnvelope>(&text) {
                            Ok(envelope) => {
                                if envelope.v != PROTOCOL_VERSION {
                                    let err = DaemonEnvelope::error(
                                        "PROTOCOL_VERSION_MISMATCH",
                                        &format!("Expected protocol version {}, got {}", PROTOCOL_VERSION, envelope.v),
                                        Some(&envelope.id),
                                    );
                                    write.send(Message::Text(serde_json::to_string(&err)?)).await?;
                                    continue;
                                }

                                match envelope.msg_type.as_str() {
                                    "heartbeat" => {
                                        let pong = DaemonEnvelope::heartbeat_pong();
                                        write.send(Message::Text(serde_json::to_string(&pong)?)).await?;
                                    }
                                    "task.submit" => {
                                        if let Some(task_id) = envelope.payload.get("task_id").and_then(|v| v.as_str()) {
                                            // Register subscriber
                                            state.task_subs.lock().await.insert(task_id.to_string(), push_tx.clone());
                                            // Insert task into DB — use async lock to avoid silently
                                            // dropping writes when the mutex is contended (Fix 1).
                                            let input = envelope.payload.get("input").and_then(|v| v.as_str()).unwrap_or("");
                                            {
                                                let db = state.db.lock().await;
                                                let _ = db.insert_task(task_id, input);
                                            }
                                            // Forward to orchestrator (DB lock released above)
                                            let _ = state.to_orchestrator.send(envelope.payload.clone()).await;
                                        } else {
                                            let err = DaemonEnvelope::error("PROTOCOL_ERROR", "task.submit missing task_id", Some(&envelope.id));
                                            write.send(Message::Text(serde_json::to_string(&err)?)).await?;
                                        }
                                    }
                                    "plan.approve" | "permission.response" | "task.cancel" => {
                                        let _ = state.to_orchestrator.send(envelope.payload.clone()).await;
                                    }
                                    unknown => {
                                        let err = DaemonEnvelope::error(
                                            "PROTOCOL_ERROR",
                                            &format!("Unknown message type: {}", unknown),
                                            Some(&envelope.id),
                                        );
                                        write.send(Message::Text(serde_json::to_string(&err)?)).await?;
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse message from {}: {}", addr, e);
                                let err = DaemonEnvelope::error("PROTOCOL_ERROR", &format!("Invalid JSON: {}", e), None);
                                write.send(Message::Text(serde_json::to_string(&err)?)).await?;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("Client {} disconnected", addr);
                        break;
                    }
                    Some(Err(e)) => {
                        warn!("WebSocket error from {}: {}", addr, e);
                        break;
                    }
                    _ => {}
                }
            }

            // Push messages from server → client (e.g. plan.proposed, step.started)
            push_msg = push_rx.recv() => {
                match push_msg {
                    Some(msg) => {
                        let json = serde_json::to_string(&msg)?;
                        if let Err(e) = write.send(Message::Text(json)).await {
                            warn!("Failed to push to client {}: {}", addr, e);
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }

    // Clean up task subscriptions for this client
    // (In a real implementation, we'd track which tasks this client owns)

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    async fn make_test_state() -> (ServerState, mpsc::Receiver<serde_json::Value>) {
        let db = Arc::new(Mutex::new(LocalDb::open(":memory:").unwrap()));
        let (orch_tx, orch_rx) = mpsc::channel(64);
        let state = ServerState::new(db, orch_tx);
        (state, orch_rx)
    }

    #[tokio::test]
    async fn test_server_starts_and_accepts_connection() {
        let (state, _) = make_test_state().await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(run_ws_server_on(listener, state));

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let url = format!("ws://127.0.0.1:{}/ws", port);
        let result = tokio_tungstenite::connect_async(&url).await;
        assert!(result.is_ok(), "Should connect to server");
    }

    #[tokio::test]
    async fn test_heartbeat_roundtrip() {
        let (state, _) = make_test_state().await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(run_ws_server_on(listener, state));

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let url = format!("ws://127.0.0.1:{}/ws", port);
        let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();

        // First message is daemon.status
        let first = ws.next().await.unwrap().unwrap();
        if let Message::Text(text) = first {
            let json: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(json["type"], "daemon.status");
        }

        // Send heartbeat
        let ping = serde_json::json!({"v": 1, "id": "test-1", "ts": 0, "type": "heartbeat", "payload": {"ping": true}});
        ws.send(Message::Text(serde_json::to_string(&ping).unwrap())).await.unwrap();

        // Expect pong
        let response = ws.next().await.unwrap().unwrap();
        if let Message::Text(text) = response {
            let json: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(json["type"], "heartbeat");
            assert_eq!(json["payload"]["pong"], true);
        } else {
            panic!("Expected text response");
        }
    }

    #[tokio::test]
    async fn test_unknown_protocol_version_returns_error() {
        let (state, _) = make_test_state().await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(run_ws_server_on(listener, state));

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let url = format!("ws://127.0.0.1:{}/ws", port);
        let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();

        // Skip daemon.status
        let _ = ws.next().await;

        // Send wrong version
        let msg = serde_json::json!({"v": 99, "id": "test-1", "ts": 0, "type": "task.submit", "payload": {}});
        ws.send(Message::Text(serde_json::to_string(&msg).unwrap())).await.unwrap();

        // Expect error
        let response = ws.next().await.unwrap().unwrap();
        if let Message::Text(text) = response {
            let json: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(json["type"], "error");
            assert_eq!(json["payload"]["code"], "PROTOCOL_VERSION_MISMATCH");
        }
    }

    #[tokio::test]
    async fn test_unknown_message_type_returns_error() {
        let (state, _) = make_test_state().await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(run_ws_server_on(listener, state));

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let url = format!("ws://127.0.0.1:{}/ws", port);
        let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();

        // Skip daemon.status
        let _ = ws.next().await;

        let msg = serde_json::json!({"v": 1, "id": "test-1", "ts": 0, "type": "unknown.type", "payload": {}});
        ws.send(Message::Text(serde_json::to_string(&msg).unwrap())).await.unwrap();

        let response = ws.next().await.unwrap().unwrap();
        if let Message::Text(text) = response {
            let json: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(json["type"], "error");
        }
    }
}
