use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};

/// Upgrade HTTP to WebSocket.
pub async fn ws_upgrade(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_ws)
}

async fn handle_ws(mut socket: WebSocket) {
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

    // For now, echo back subscription acks and keep connection alive.
    // In the future, this will subscribe to EventBus and stream events.
    while let Some(Ok(msg)) = socket.next().await {
        match msg {
            Message::Text(text) => {
                // Parse subscription requests
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                    if parsed.get("type").and_then(|t| t.as_str()) == Some("subscribe") {
                        let ack = serde_json::json!({
                            "type": "subscribed",
                            "payload": { "filters": parsed.get("filters") }
                        });
                        if socket
                            .send(Message::Text(ack.to_string().into()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            }
            Message::Ping(data) => {
                if socket.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}
