//! Day 1 Doctor — Local Daemon
//!
//! Startup sequence:
//! 1. Initialize tracing from config
//! 2. Load config from ~/.d1doctor/config.toml (or defaults)
//! 3. Open SQLite database
//! 4. Connect to orchestrator via WebSocket (with retry)
//! 5. Send AUTH_RESPONSE envelope immediately after connecting
//! 6. Spawn heartbeat task (every 30s)
//! 7. Enter receive loop — dispatch by MessageType

pub mod executor;
pub mod health;
pub mod local_db;
pub mod message_handler;
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

    // 5. Send AUTH_RESPONSE immediately after connecting
    let auth_envelope = build_auth_response(&session_id);
    if let Err(e) = conn.send(&auth_envelope).await {
        error!(error = %e, "Failed to send AUTH_RESPONSE — continuing anyway");
    } else {
        info!(session_id = %session_id, "AUTH_RESPONSE sent");
        db.append_audit_log(&session_id, "AUTH_RESPONSE_SENT", "ok", "LOW").ok();
    }

    // 6 + 7. Main event loop: heartbeat ticks and inbound message dispatch
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
fn build_heartbeat(session_id: &str) -> d1_common::proto::JsonEnvelope {
    d1_common::proto::heartbeat_envelope(session_id)
}

/// Build an AUTH_RESPONSE Envelope, loading JWT from the token file if present.
fn build_auth_response(session_id: &str) -> d1_common::proto::JsonEnvelope {
    let token_path = d1_common::config_dir().join("token.json");
    let (jwt_token, device_id) = if token_path.exists() {
        match std::fs::read_to_string(&token_path) {
            Ok(json) => {
                let parsed: serde_json::Value = serde_json::from_str(&json).unwrap_or_default();
                let token = parsed
                    .get("access_token")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let user_id = parsed
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                (token, user_id)
            }
            Err(_) => (String::new(), "unknown".to_string()),
        }
    } else {
        (String::new(), "unknown".to_string())
    };

    d1_common::proto::auth_response_envelope(session_id, jwt_token, device_id)
}

/// Dispatch an inbound Envelope by MessageType.
/// In Sprint 1 this is a routing stub — Sprint 2 will fill in execution.
fn dispatch_message(
    db: &local_db::LocalDb,
    session_id: &str,
    envelope: d1_common::proto::JsonEnvelope,
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

#[cfg(test)]
mod tests {
    use super::*;
    use d1_common::proto::MessageType;

    #[test]
    fn test_build_auth_response_type() {
        let env = build_auth_response("test-session");
        assert_eq!(env.r#type, MessageType::AuthResponse as i32);
        assert!(!env.id.is_empty());
        assert_eq!(env.session_id, "test-session");
    }

    #[test]
    fn test_build_auth_response_encodes() {
        let env = build_auth_response("session-xyz");
        let bytes = d1_common::proto::encode_envelope(&env);
        assert!(!bytes.is_empty(), "AUTH_RESPONSE envelope should encode to non-empty bytes");
        let decoded = d1_common::proto::decode_envelope(&bytes)
            .expect("AUTH_RESPONSE envelope should decode");
        assert_eq!(decoded.r#type, MessageType::AuthResponse as i32);
    }
}
