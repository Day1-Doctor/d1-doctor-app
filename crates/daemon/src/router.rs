//! Message router: routes inbound orchestrator messages to the correct local client subscriber.
//! Spec: LocalStack_v2.4.1_Spec.md §2.3

use crate::protocol::DaemonEnvelope;
use crate::ws_server::ServerState;
use tracing::warn;

/// Route an inbound orchestrator message to the appropriate local client.
/// Messages are routed by task_id field in the payload.
pub async fn route_orchestrator_message(msg: serde_json::Value, state: &ServerState) {
    // Extract task_id from payload
    let task_id = msg.get("task_id")
        .or_else(|| msg.get("payload").and_then(|p| p.get("task_id")))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let msg_type = msg.get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let payload = msg.get("payload").cloned().unwrap_or(msg.clone());

    let envelope = DaemonEnvelope::new(&msg_type, payload);

    if let Some(task_id) = task_id {
        let subs = state.task_subs.lock().await;
        if let Some(sender) = subs.get(&task_id) {
            if let Err(e) = sender.send(envelope).await {
                warn!("Failed to route message for task {}: {}", task_id, e);
            }
        } else {
            warn!("No subscriber for task_id: {}", task_id);
        }
    } else {
        // Broadcast to all (e.g. daemon.status changes)
        let subs = state.task_subs.lock().await;
        for sender in subs.values() {
            let _ = sender.send(envelope.clone()).await;
        }
    }
}
