//! Day 1 Doctor — Local Daemon (d1d)
//! Startup sequence: config → DB → WS server → orchestrator client → event loop
//! Spec: LocalStack_v2.4.1_Spec.md §2.1

mod config;
mod executor;
mod health;
mod local_db;
mod protocol;
mod router;
mod ws_client;
mod ws_server;

use config::DaemonConfig;
use local_db::LocalDb;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load config
    let cfg = DaemonConfig::load()?;
    setup_tracing(&cfg.daemon.log_level);
    info!(
        "Day 1 Doctor daemon v{} starting on port {}",
        env!("CARGO_PKG_VERSION"),
        cfg.daemon.port
    );

    // 2. Open SQLite DB
    let db_path = cfg.db_path();
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let db_path_str = db_path.to_str()
        .ok_or_else(|| anyhow::anyhow!("DB path contains non-UTF-8 characters: {:?}", db_path))?;
    let db = Arc::new(Mutex::new(LocalDb::open(db_path_str)?));
    info!("Database opened at {:?}", db_path);

    // 3. Create channels for orchestrator communication
    let (to_orchestrator_tx, to_orchestrator_rx) = mpsc::channel::<serde_json::Value>(256);
    let (from_orchestrator_tx, mut from_orchestrator_rx) = mpsc::channel::<serde_json::Value>(256);
    let (orch_status_tx, mut orch_status_rx) = mpsc::channel::<bool>(16);

    // 4. Start local WS server
    let mut server_state = ws_server::ServerState::new(db.clone(), to_orchestrator_tx);
    server_state.orchestrator_url = cfg.orchestrator.url.clone();
    server_state.device_id = cfg.auth.device_id.clone();
    let port = cfg.daemon.port;
    let server_state_clone = server_state.clone();
    let server_handle = tokio::spawn(async move {
        if let Err(e) = ws_server::run_ws_server(port, server_state_clone).await {
            tracing::error!("WS server error: {}", e);
        }
    });
    info!("Local WebSocket server listening on ws://localhost:{}/ws", port);

    // 5. Start orchestrator client (non-blocking)
    let orch_cfg = cfg.orchestrator.clone();
    tokio::spawn(async move {
        ws_client::run_orchestrator_loop(orch_cfg, orch_status_tx, to_orchestrator_rx, from_orchestrator_tx).await;
    });

    // 6. Route orchestrator status changes
    let server_state_for_status = server_state.clone();
    tokio::spawn(async move {
        while let Some(connected) = orch_status_rx.recv().await {
            server_state_for_status.set_orch_connected(connected).await;
        }
    });

    // 7. Route orchestrator messages to local clients
    let server_state_for_routing = server_state.clone();
    tokio::spawn(async move {
        while let Some(msg) = from_orchestrator_rx.recv().await {
            router::route_orchestrator_message(msg, &server_state_for_routing).await;
        }
    });

    // 8. Write PID file
    let pid_path = DaemonConfig::expand_path(&cfg.daemon.pid_file);
    if let Some(parent) = pid_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&pid_path, std::process::id().to_string()).ok();

    // 9. Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received. Cleaning up...");
    std::fs::remove_file(&pid_path).ok();
    server_handle.abort();

    Ok(())
}

fn setup_tracing(level: &str) {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
