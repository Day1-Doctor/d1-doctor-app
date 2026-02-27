//! Health check endpoint for the daemon.
//! GET /health → 200 OK {"status": "ok", "version": "0.1.0"}

use crate::ws_server::ServerState;
use serde_json::json;

/// Returns daemon health info as JSON.
pub async fn health_check(state: &ServerState) -> serde_json::Value {
    let orch_connected = state.is_orch_connected().await;
    json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "orchestrator_connected": orch_connected,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::LocalDb;
    use std::sync::Arc;
    use tokio::sync::{mpsc, Mutex};

    #[tokio::test]
    async fn test_health_check_returns_ok() {
        let db = Arc::new(Mutex::new(LocalDb::open(":memory:").unwrap()));
        let (tx, _) = mpsc::channel(1);
        let state = crate::ws_server::ServerState::new(db, tx);
        let health = health_check(&state).await;
        assert_eq!(health["status"], "ok");
        assert_eq!(health["orchestrator_connected"], false);
    }
}
