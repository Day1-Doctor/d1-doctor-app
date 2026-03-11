//! Day 1 Doctor — Local Daemon
//!
//! The daemon runs on the user's machine and bridges local CLI/Mac App
//! connections to the cloud platform. It provides:
//! - A WebSocket endpoint (`/chat`) for real-time chat relay
//! - REST API endpoints for health and memory search
//! - A cloud WebSocket client for upstream connectivity
//! - Local SQLite storage for agent memory
//! - MCP tool servers (filesystem, shell, memory, system, QMD)

// ---------------------------------------------------------------------------
// Module declarations — every .rs file in this crate except main.rs
// ---------------------------------------------------------------------------
pub mod chat_relay;
pub mod cloud_ws;
pub mod command_relay;
pub mod connection_state;
pub mod executor;
pub mod filesystem;
pub mod fingerprint;
pub mod health;
pub mod local_db;
pub mod mcp_filesystem;
pub mod mcp_memory;
pub mod mcp_qmd;
pub mod mcp_registry;
pub mod mcp_shell;
pub mod mcp_system;
pub mod memory_store;
pub mod profile_detect;
pub mod qmd;
pub mod redactor;
pub mod rest_api;
pub mod security;
pub mod system_ops;
pub mod ws_app;
pub mod ws_client;

// ---------------------------------------------------------------------------
// Imports
// ---------------------------------------------------------------------------
use std::sync::Arc;

use axum::extract::ws::{Message as AxumWsMessage, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures::{SinkExt, StreamExt};
use tokio::signal;
use tracing::{debug, error, info, warn};

use chat_relay::{ChatMessage, ChatRelay};
use cloud_ws::{CloudWsClient, CloudWsConfig, ConnectionState};
use d1_common::Config;
use fingerprint::DeviceFingerprint;
use redactor::Redactor;

// ---------------------------------------------------------------------------
// Shared state for Axum handlers
// ---------------------------------------------------------------------------

/// State shared across all Axum handlers (WebSocket + REST).
#[derive(Clone)]
struct DaemonState {
    relay: Arc<ChatRelay>,
    redactor: Arc<Redactor>,
}

// ---------------------------------------------------------------------------
// CLI credentials
// ---------------------------------------------------------------------------

/// Try to read the CLI user's access token from ~/.d1-doctor/credentials.json.
fn read_cli_credentials() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let path = std::path::PathBuf::from(home)
        .join(".d1-doctor")
        .join("credentials.json");
    let data = std::fs::read_to_string(&path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&data).ok()?;
    parsed.get("access_token")?.as_str().map(|s| s.to_string())
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Day1 Doctor daemon starting...");

    // 1. Load config (defaults if missing)
    let config = Config::load().unwrap_or_default();
    info!(port = config.daemon_port, "Configuration loaded");

    // 2. Create Redactor
    let redactor = Arc::new(Redactor::from_config(&config.redaction));

    // 3. Open SQLite
    let db_path = config.database.path.to_string_lossy().to_string();
    let _db = local_db::LocalDb::open(&db_path)?;
    info!(%db_path, "SQLite database opened");

    // 4. Create ChatRelay
    let (relay, mut cloud_rx) = ChatRelay::new();
    let relay = Arc::new(relay);

    // 5. Build Axum router: /chat (WS) + /api/* (REST)
    let daemon_state = DaemonState {
        relay: Arc::clone(&relay),
        redactor: Arc::clone(&redactor),
    };

    let app = Router::new()
        .route("/ws", get(ws_app_handler))
        .route("/chat", get(ws_chat_handler))
        .route("/api/health", get(rest_api::health_check))
        .route("/api/memory/search", get(rest_api::memory_search))
        .with_state(daemon_state);

    let addr = format!("127.0.0.1:{}", config.daemon_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(%addr, "HTTP/WS server listening");

    // Spawn the server as a background task
    let server_handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!(%e, "Axum server error");
        }
    });

    // 6. Spawn CloudWsClient
    let jwt = read_cli_credentials()
        .or_else(|| config.supabase.as_ref().map(|s| s.anon_key.clone()))
        .unwrap_or_default();

    if jwt.is_empty() {
        warn!("No authentication token found — daemon will connect anonymously");
    } else {
        info!("Authentication token loaded");
    }

    let device_fp = DeviceFingerprint::generate()
        .map(|fp| fp.fingerprint)
        .unwrap_or_else(|e| {
            warn!(%e, "Failed to generate device fingerprint, using fallback");
            "unknown-device".to_string()
        });

    let cloud_client = CloudWsClient::new();
    let mut state_rx = cloud_client.subscribe_state();

    let cloud_config = CloudWsConfig {
        url: config.orchestrator_url.clone(),
        jwt,
        device_fingerprint: device_fp,
    };

    // Create channels for cloud WS communication
    let (cloud_outbound_tx, cloud_outbound_rx) = tokio::sync::mpsc::channel::<String>(256);
    let (cloud_inbound_tx, mut cloud_inbound_rx) = tokio::sync::mpsc::channel::<String>(256);

    let _cloud_handle = cloud_client.spawn(cloud_config, cloud_outbound_rx, cloud_inbound_tx);
    info!("Cloud WebSocket client spawned");

    // 7. Watch cloud state -> update relay connected/disconnected
    let relay_for_state = Arc::clone(&relay);
    let state_watcher = tokio::spawn(async move {
        loop {
            if state_rx.changed().await.is_err() {
                break;
            }
            let state = *state_rx.borrow();
            match state {
                ConnectionState::Connected => {
                    info!("Cloud connected — flushing relay queue");
                    if let Err(e) = relay_for_state.set_connected().await {
                        warn!(%e, "Failed to flush relay queue on connect");
                    }
                }
                ConnectionState::Disconnected => {
                    info!("Cloud disconnected");
                    relay_for_state.set_disconnected("").await;
                }
                _ => {
                    // Connecting / Authenticating — no relay action needed
                }
            }
        }
    });

    // 8. Cloud writer task: relay cloud_rx → redact → serialize → send to cloud WS
    let redactor_for_writer = Arc::clone(&redactor);
    let cloud_writer = tokio::spawn(async move {
        while let Some(msg) = cloud_rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(json) => {
                    let redacted = redactor_for_writer.redact(&json);
                    if let Err(e) = cloud_outbound_tx.send(redacted).await {
                        warn!("Failed to send to cloud: {}", e);
                    }
                }
                Err(e) => error!("Failed to serialize message: {}", e),
            }
        }
        info!("Cloud writer task ended (channel closed)");
    });

    // 8b. Cloud reader task: cloud WS → parse → relay to local clients
    let relay_for_reader = Arc::clone(&relay);
    let cloud_reader = tokio::spawn(async move {
        while let Some(text) = cloud_inbound_rx.recv().await {
            match serde_json::from_str::<ChatMessage>(&text) {
                Ok(msg) => {
                    if let Err(e) = relay_for_reader.send_to_local(msg) {
                        // NoLocalClients is normal if nobody is connected
                        debug!("No local clients for inbound message: {}", e);
                    }
                }
                Err(e) => warn!("Invalid ChatMessage from cloud: {}", e),
            }
        }
        info!("Cloud reader task ended (channel closed)");
    });

    // 9. Background system profile detection
    tokio::spawn(async move {
        let facts = profile_detect::detect_system_profile();
        info!(fact_count = facts.len(), "System profile detected");
        for fact in &facts {
            tracing::debug!(key = %fact.key, value = %fact.value, source = %fact.source, "profile fact");
        }
    });

    // 10. Wait for ctrl+c, then shutdown
    info!("Daemon ready — press Ctrl+C to stop");
    signal::ctrl_c().await?;
    info!("Shutdown signal received");

    cloud_client.shutdown();
    state_watcher.abort();
    cloud_writer.abort();
    cloud_reader.abort();
    server_handle.abort();

    info!("Day1 Doctor daemon stopped");
    Ok(())
}

// ---------------------------------------------------------------------------
// /chat WebSocket handler
// ---------------------------------------------------------------------------

/// Axum handler that upgrades HTTP to WebSocket for the /ws endpoint (Mac App).
async fn ws_app_handler(
    ws: WebSocketUpgrade,
    State(state): State<DaemonState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_app::handle_app_ws(socket, state.relay, state.redactor))
}

/// Axum handler that upgrades HTTP to WebSocket for the /chat endpoint.
async fn ws_chat_handler(
    ws: WebSocketUpgrade,
    State(state): State<DaemonState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_chat_ws(socket, state))
}

/// Bidirectional bridge between a local WebSocket client and the ChatRelay.
///
/// - Broadcasts from cloud (via relay) are forwarded to the WS client.
/// - Messages from the WS client are redacted and sent to the cloud (via relay).
async fn handle_chat_ws(ws: WebSocket, state: DaemonState) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut broadcast_rx = state.relay.subscribe_local();

    // Task 1: broadcast (cloud responses) -> client WS
    let tx_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if ws_tx.send(AxumWsMessage::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Task 2: client WS -> relay (redact content, send to cloud)
    while let Some(Ok(msg)) = ws_rx.next().await {
        if let AxumWsMessage::Text(text) = msg {
            if let Ok(mut chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
                chat_msg.payload.content = state.redactor.redact(&chat_msg.payload.content);
                let _ = state.relay.send_to_cloud(chat_msg).await;
            }
        }
    }

    tx_task.abort();
}
