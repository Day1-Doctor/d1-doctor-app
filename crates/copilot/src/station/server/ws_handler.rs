use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    Extension,
};
use futures::{SinkExt, StreamExt};

use super::state::ServerState;

/// Upgrade HTTP to WebSocket, injecting server state.
pub async fn ws_upgrade(
    ws: WebSocketUpgrade,
    Extension(state): Extension<ServerState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: ServerState) {
    // Send a welcome message
    let welcome = serde_json::json!({
        "type": "connected",
        "payload": { "version": "3.0.0-alpha", "server": "station-runtime" }
    });
    if socket
        .send(Message::Text(welcome.to_string().into()))
        .await
        .is_err()
    {
        return;
    }

    // Subscribe to the real EventBus
    let mut rx = state.event_bus.subscribe();

    // Split the socket so we can forward events and listen concurrently
    let (mut ws_tx, mut ws_rx) = socket.split();

    // Spawn a task to forward EventBus events to the WebSocket client
    let forward_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let json = serde_json::to_string(&event).unwrap_or_default();
                    if ws_tx.send(Message::Text(json.into())).await.is_err() {
                        break; // Client disconnected
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("WS client lagged {} events", n);
                }
                Err(_) => break, // Channel closed
            }
        }
    });

    // Handle incoming messages from the client (subscriptions, pings)
    while let Some(Ok(msg)) = ws_rx.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                    if parsed.get("type").and_then(|t| t.as_str()) == Some("subscribe") {
                        // Acknowledged -- we're already forwarding all events.
                        // Future: apply filters based on parsed["filters"].
                    }
                }
            }
            Message::Ping(_data) => {
                // Pong is handled automatically by axum
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    forward_task.abort();
}
