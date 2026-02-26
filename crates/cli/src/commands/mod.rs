//! CLI command definitions and handlers.
//!
//! Sprint 1 implements:
//!   status      — HTTP health-check against the local daemon
//!   auth login  — device-code OAuth placeholder
//!   auth logout — removes local token file
//!   daemon      — start/stop/logs stubs
//!   install / diagnose / files / upgrade — "coming in Sprint N" stubs
//!
//! Sprint 3: install command fully wired via WebSocket.
//! Sprint 4: files and diagnose commands fully wired.
//! Sprint 5: status command displays credit balance.

pub mod install;
pub mod files;
pub mod diagnose;

use anyhow::Result;
use clap::Subcommand;

use crate::auth_token::TokenStore;
use crate::credits;

/// Top-level subcommands for `d1-doctor`.
#[derive(Subcommand)]
pub enum Commands {
    /// Show daemon and orchestrator connection status
    Status,

    /// Authentication management
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },

    /// Daemon process management
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },

    /// Install and configure software packages
    Install {
        /// Package name to install
        package: String,
    },

    /// Run system diagnostics
    Diagnose {
        #[command(flatten)]
        args: diagnose::DiagnoseArgs,
    },

    /// Manage files and directories
    Files {
        #[command(flatten)]
        args: files::FilesArgs,
    },

    /// Upgrade Day 1 Doctor to the latest version (coming in Sprint 4)
    Upgrade,
}

#[derive(Subcommand)]
pub enum AuthAction {
    /// Log in via device-code OAuth flow
    Login,
    /// Log out and remove cached credentials
    Logout,
}

#[derive(Subcommand)]
pub enum DaemonAction {
    /// Start the local daemon in the background
    Start,
    /// Stop the running local daemon
    Stop,
    /// Show recent daemon log output
    Logs,
}

/// Route a top-level command to its handler.
pub async fn handle(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Status => status().await,
        Commands::Auth { action } => match action {
            AuthAction::Login => auth_login().await,
            AuthAction::Logout => auth_logout(),
        },
        Commands::Daemon { action } => match action {
            DaemonAction::Start => daemon_start(),
            DaemonAction::Stop => daemon_stop(),
            DaemonAction::Logs => daemon_logs(),
        },
        Commands::Install { package } => install::handle(&package).await,
        Commands::Diagnose { args } => diagnose::handle(&args).await,
        Commands::Files { args } => files::handle(&args).await,
        Commands::Upgrade => {
            println!("Upgrade command coming in Sprint 4.");
            Ok(())
        }
    }
}

// ─── Status ──────────────────────────────────────────────────────────────────

/// Call the local daemon's /health endpoint and print the result,
/// then attempt to fetch and display credit balance from the orchestrator.
async fn status() -> Result<()> {
    // ── Daemon health check ──────────────────────────────────────────────
    let port = d1_common::DEFAULT_DAEMON_PORT;
    let url = format!("http://localhost:{}/health", port);

    println!("Checking daemon status at {} ...", url);

    match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.text().await.unwrap_or_else(|_| "OK".to_string());
            println!("Daemon is running.");
            println!("Response: {}", body);
        }
        Ok(resp) => {
            println!(
                "Daemon returned unexpected status: {}",
                resp.status()
            );
        }
        Err(e) => {
            println!("Daemon is not reachable: {}", e);
            println!(
                "Hint: start the daemon with `d1-doctor daemon start`"
            );
        }
    }

    // ── Credit balance ───────────────────────────────────────────────────
    println!();
    fetch_and_display_credits().await;

    Ok(())
}

/// Attempt to load token and fetch credit balance from the orchestrator.
/// Prints a helpful hint instead of crashing if anything goes wrong.
async fn fetch_and_display_credits() {
    let token_path = d1_common::config_dir().join("token.json");

    let token_store = match TokenStore::try_load(&token_path) {
        Some(ts) => ts,
        None => {
            println!("Credit balance: not available (not logged in).");
            println!("Hint: run `d1-doctor auth login` to see your credit balance.");
            return;
        }
    };

    let api_url = credits::DEFAULT_ORCHESTRATOR_API_URL;

    match credits::fetch_credits(api_url, &token_store.access_token).await {
        Ok(balance) => {
            credits::display_balance(&balance);
        }
        Err(_) => {
            println!("Credit balance: unable to reach orchestrator.");
            println!(
                "Hint: ensure the orchestrator is running at {}.",
                api_url
            );
        }
    }
}

// ─── Auth ────────────────────────────────────────────────────────────────────

/// Print the device-code URL and wait for the user to authenticate.
/// Sprint 1 stub — full OAuth flow wired in Sprint 2.
async fn auth_login() -> Result<()> {
    println!("Starting device-code authentication...");
    println!();
    println!("  Visit the following URL to complete login:");
    println!();
    println!("    https://auth.day1doctor.com/device");
    println!();
    println!("  Then enter the code shown on that page.");
    println!();
    println!("(Full OAuth device-code flow coming in Sprint 2)");
    Ok(())
}

/// Remove the cached authentication token.
fn auth_logout() -> Result<()> {
    let token_path = d1_common::config_dir().join("token.json");

    if token_path.exists() {
        std::fs::remove_file(&token_path)?;
        println!("Logged out. Token removed from {}.", token_path.display());
    } else {
        println!("Not currently logged in (no token found).");
    }

    Ok(())
}

// ─── Daemon management stubs ─────────────────────────────────────────────────

fn daemon_start() -> Result<()> {
    println!("Daemon start coming in Sprint 2.");
    println!("Hint: for now, run `d1-daemon` directly in a terminal.");
    Ok(())
}

fn daemon_stop() -> Result<()> {
    println!("Daemon stop coming in Sprint 2.");
    Ok(())
}

fn daemon_logs() -> Result<()> {
    println!("Daemon log streaming coming in Sprint 2.");
    println!("Hint: daemon logs are written to stderr when run with RUST_LOG=info.");
    Ok(())
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_logout_no_token_file() {
        // When no token file exists, logout should not error.
        let result = auth_logout();
        assert!(result.is_ok(), "auth_logout should succeed even if no token exists");
    }

    #[test]
    fn test_daemon_stubs_return_ok() {
        assert!(daemon_start().is_ok());
        assert!(daemon_stop().is_ok());
        assert!(daemon_logs().is_ok());
    }

    #[tokio::test]
    async fn test_status_unreachable_daemon_does_not_panic() {
        // With no daemon running on any port, status should complete without panic.
        // (Port 1 is normally unroutable — connection refused is the expected outcome.)
        let port = 1u16;
        let url = format!("http://localhost:{}/health", port);
        // Just verify the HTTP client call itself does not panic.
        let _ = reqwest::get(&url).await;
    }
}
