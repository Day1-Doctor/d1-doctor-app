//! Day 1 Doctor — Local Daemon
//!
//! Startup sequence:
//! 1. Initialize tracing from config
//! 2. Load config from ~/.d1doctor/config.toml (or defaults)
//! 3. Open SQLite database
//! 4. Connect to orchestrator via WebSocket (with retry)
//! 5. Spawn heartbeat task (every 30s)
//! 6. Enter receive loop — dispatch by MessageType

mod executor;
mod health;
mod local_db;
mod ws_client;

use anyhow::Result;
use local_db::LocalDb;
use std::time::Duration;
use tokio::time;
use tracing::{error, info, warn};
use ws_client::WsClient;

const HEARTBEAT_INTERVAL_SECS: u64 = 30;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Init tracing — respect RUST_LOG env var, default to d1_daemon=info
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "d1_daemon=info".into()),
        )
        .init();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Day 1 Doctor daemon starting"
    );

    // 2. Load config — fall back to defaults if config file is missing
    let config = d1_common::Config::load().unwrap_or_else(|e| {
        warn!(error = %e, "Config load failed, using defaults");
        d1_common::Config::default()
    });
    info!(
        orchestrator_url = %config.orchestrator_url,
        daemon_port = config.daemon_port,
        "Config loaded"
    );

    // 3. Open local database
    let db_path = config.database.path.to_string_lossy().to_string();
    let db = LocalDb::open(&db_path).map_err(|e| {
        error!(error = %e, path = %db_path, "Failed to open local database");
        e
    })?;
    info!(path = %db_path, "Local database opened");

    // 4. Connect to orchestrator (blocks until first successful connect)
    let client = WsClient::new(&config.orchestrator_url);
    let mut conn = client.connect_with_retry().await?;
    info!("Connected to orchestrator");

    // Generate a session ID for this daemon run
    let session_id = uuid::Uuid::new_v4().to_string();

    // 5 + 6. Main event loop: heartbeat ticks and inbound message dispatch
    let mut heartbeat_interval = time::interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
    // Tick immediately fires on creation — skip the first instant tick so we
    // don't send a heartbeat before the connection is fully established.
    heartbeat_interval.tick().await;

    info!(session_id = %session_id, "Entering main event loop");

    loop {
        tokio::select! {
            _ = heartbeat_interval.tick() => {
                let envelope = build_heartbeat(&session_id);
                if let Err(e) = conn.send(&envelope).await {
                    error!(error = %e, "Failed to send heartbeat — reconnect needed");
                    break;
                }
                info!(session_id = %session_id, "Heartbeat sent");
                db.append_audit_log(&session_id, "HEARTBEAT_SENT", "ok", "LOW").ok();
            }

            result = conn.recv() => {
                match result {
                    Ok(Some(envelope)) => {
                        let msg_type = envelope.r#type;
                        info!(
                            msg_type = msg_type,
                            msg_id = %envelope.id,
                            session = %envelope.session_id,
                            "Received message"
                        );
                        db.append_audit_log(
                            &session_id,
                            "RECEIVED_MSG",
                            &format!("type={msg_type} id={}", envelope.id),
                            "LOW",
                        ).ok();
                        dispatch_message(&db, &session_id, envelope);
                    }
                    Ok(None) => {
                        warn!("WebSocket closed by orchestrator");
                        break;
                    }
                    Err(e) => {
                        error!(error = %e, "WebSocket receive error");
                        break;
                    }
                }
            }
        }
    }

    info!("Daemon shutting down");
    Ok(())
}

/// Build a HEARTBEAT Envelope for the given session.
fn build_heartbeat(session_id: &str) -> d1_common::proto::Envelope {
    d1_common::proto::heartbeat_envelope(session_id)
}

/// Dispatch an inbound Envelope by MessageType.
/// In Sprint 1 this is a routing stub — Sprint 2 will fill in execution.
fn dispatch_message(
    db: &local_db::LocalDb,
    session_id: &str,
    envelope: d1_common::proto::Envelope,
) {
    use d1_common::proto::MessageType;

    match envelope.r#type {
        t if t == MessageType::Heartbeat as i32 => {
            info!(msg_id = %envelope.id, "Received HEARTBEAT ack");
        }
        t if t == MessageType::CommandRequest as i32 => {
            warn!(
                msg_id = %envelope.id,
                "CommandRequest received — executor not yet wired (Sprint 2)"
            );
            db.append_audit_log(session_id, "CMD_STUB", &envelope.id, "MEDIUM").ok();
        }
        t => {
            warn!(msg_type = t, msg_id = %envelope.id, "Unknown message type — ignored");
        }
    }
}
