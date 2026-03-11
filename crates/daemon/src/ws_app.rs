//! Mac App WebSocket handler — protocol translation layer.
//!
//! The Mac app connects to `/ws` and speaks the task-based protocol
//! (task.submit, agent.message, task.completed, etc.). This handler
//! translates between that protocol and ChatMessage v1 for the cloud bridge.

use std::sync::Arc;

use axum::extract::ws::{Message as AxumWsMessage, WebSocket};
use d1_common::chat_message::{ChatMessage, ChatMessageType, ChatPayload};
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::mpsc;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::chat_relay::ChatRelay;
use crate::redactor::Redactor;

/// Build a Mac-app-protocol envelope.
fn make_envelope(msg_type: &str, payload: Value) -> String {
    serde_json::json!({
        "v": 1,
        "id": Uuid::new_v4().to_string(),
        "ts": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        "type": msg_type,
        "payload": payload,
    })
    .to_string()
}

/// Handle a Mac app WebSocket connection on `/ws`.
pub async fn handle_app_ws(ws: WebSocket, relay: Arc<ChatRelay>, redactor: Arc<Redactor>) {
    let (mut ws_tx, mut ws_rx) = ws.split();

    // Channel for sending messages to the Mac app (both cloud responses and heartbeats)
    let (out_tx, mut out_rx) = mpsc::channel::<String>(256);

    // Writer task: drain the channel → WebSocket
    let writer_task = tokio::spawn(async move {
        while let Some(text) = out_rx.recv().await {
            if ws_tx.send(AxumWsMessage::Text(text.into())).await.is_err() {
                break;
            }
        }
    });

    // Send initial daemon.status
    let status = make_envelope(
        "daemon.status",
        serde_json::json!({
            "daemon_version": env!("CARGO_PKG_VERSION"),
            "protocol_version": 1,
            "orchestrator_connected": true,
            "orchestrator_url": "",
            "active_tasks": 0,
            "device_id": ""
        }),
    );
    let _ = out_tx.send(status).await;

    // Track the current task_id for mapping cloud responses
    let current_task_id = Arc::new(tokio::sync::Mutex::new(String::new()));
    let session_id = Uuid::new_v4().to_string();

    // Cloud response translator: ChatRelay broadcasts → Mac app protocol
    let mut broadcast_rx = relay.subscribe_local();
    let task_id_for_cloud = Arc::clone(&current_task_id);
    let out_tx_cloud = out_tx.clone();
    let cloud_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            let task_id = task_id_for_cloud.lock().await.clone();

            let envelope = match msg.msg_type {
                ChatMessageType::StreamChunk => make_envelope(
                    "agent.message",
                    serde_json::json!({
                        "task_id": task_id,
                        "step_id": "chat",
                        "agent": "planner",
                        "message": msg.payload.content,
                        "message_type": "INFO"
                    }),
                ),
                ChatMessageType::StreamEnd => make_envelope(
                    "task.completed",
                    serde_json::json!({
                        "task_id": task_id,
                        "summary": "",
                        "steps_completed": 1,
                        "steps_total": 1,
                        "duration_ms": 0,
                        "credits_used": 0
                    }),
                ),
                ChatMessageType::Error => make_envelope(
                    "task.failed",
                    serde_json::json!({
                        "task_id": task_id,
                        "error": {
                            "code": "INTERNAL_ERROR",
                            "message": msg.payload.content,
                            "steps_completed": 0,
                            "steps_total": 1
                        },
                        "credits_used": 0
                    }),
                ),
                ChatMessageType::SessionInit
                | ChatMessageType::SessionInitAck
                | ChatMessageType::Unknown => continue,
                _ => make_envelope(
                    "agent.message",
                    serde_json::json!({
                        "task_id": task_id,
                        "step_id": "chat",
                        "agent": "planner",
                        "message": msg.payload.content,
                        "message_type": "INFO"
                    }),
                ),
            };

            if out_tx_cloud.send(envelope).await.is_err() {
                break;
            }
        }
    });

    // Main loop: Mac app messages → ChatRelay
    let mut session_initialized = false;
    while let Some(Ok(msg)) = ws_rx.next().await {
        let text = match msg {
            AxumWsMessage::Text(t) => t,
            AxumWsMessage::Close(_) => break,
            _ => continue,
        };

        let parsed: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                warn!("Invalid JSON from Mac app: {}", e);
                continue;
            }
        };

        let msg_type = parsed.get("type").and_then(|t| t.as_str()).unwrap_or("");

        match msg_type {
            "task.submit" => {
                let payload = parsed.get("payload").cloned().unwrap_or(Value::Null);
                let task_id = payload
                    .get("task_id")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                let input = payload
                    .get("input")
                    .and_then(|i| i.as_str())
                    .unwrap_or("")
                    .to_string();

                *current_task_id.lock().await = task_id;

                // Send session_init on first task
                if !session_initialized {
                    let locale = std::env::var("LANG").unwrap_or_else(|_| "en".to_string());
                    let init_msg = ChatMessage::session_init(session_id.clone(), locale);
                    let _ = relay.send_to_cloud(init_msg).await;
                    session_initialized = true;
                }

                // Translate task.submit → user_message
                let redacted_input = redactor.redact(&input);
                let chat_msg = ChatMessage::new(
                    ChatMessageType::UserMessage,
                    ChatPayload {
                        session_id: session_id.clone(),
                        content: redacted_input,
                        metadata: None,
                    },
                );
                let _ = relay.send_to_cloud(chat_msg).await;
                debug!("task.submit translated to user_message");
            }
            "heartbeat" => {
                let pong = make_envelope("heartbeat", serde_json::json!({"pong": true}));
                let _ = out_tx.send(pong).await;
            }
            "plan.approve" | "task.cancel" | "permission.response" => {
                debug!("Received {} — forwarding not yet implemented", msg_type);
            }
            _ => {
                warn!("Unknown message type from Mac app: {}", msg_type);
            }
        }
    }

    cloud_task.abort();
    writer_task.abort();
}
