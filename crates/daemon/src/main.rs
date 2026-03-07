//! Day 1 Doctor — Local Daemon
//!
//! The daemon runs on the user's machine and executes commands from the
//! cloud orchestrator. It communicates via WebSocket using Protobuf messages.

mod executor;
mod health;
mod local_db;
mod mcp_memory;
mod memory_store;
mod profile_detect;
pub mod redactor;
mod ws_client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Day 1 Doctor daemon starting...");

    // TODO: Initialize components
    // 1. Load configuration (including RedactionConfig)
    // 2. Create Redactor from config for cloud-bound message sanitisation
    // 3. Open local SQLite database
    // 4. Connect to orchestrator via WebSocket
    // 5. Start health monitoring
    // 6. Enter main event loop

    Ok(())
}
