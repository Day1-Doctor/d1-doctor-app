use crate::daemon_client::ping_daemon;
use anyhow::Result;
use colored::Colorize;
use std::time::Duration;

pub async fn execute() -> Result<()> {
    if ping_daemon().await {
        println!("{} Daemon is already running on port 9876", "✓".green());
        return Ok(());
    }

    println!("Starting Day 1 Doctor daemon...");

    // Try to find d1d binary
    let d1d_path = which_d1d();
    match d1d_path {
        Some(path) => {
            std::process::Command::new(&path)
                .arg("--config")
                .arg(expand_config_path())
                .spawn()
                .map_err(|e| anyhow::anyhow!("Failed to start daemon: {e}"))?;

            // Poll for daemon to become ready
            for i in 0..25 {
                tokio::time::sleep(Duration::from_millis(200)).await;
                if ping_daemon().await {
                    println!("{} Daemon started successfully", "✓".green());
                    return Ok(());
                }
                if i % 5 == 4 {
                    print!(".");
                }
            }
            anyhow::bail!("Daemon did not start within 5 seconds. Check logs: d1 logs")
        }
        None => {
            anyhow::bail!(
                "d1d binary not found. Install it with: cargo install --path crates/daemon"
            )
        }
    }
}

fn which_d1d() -> Option<String> {
    // Check common locations
    let candidates = ["d1d", "/usr/local/bin/d1d", "/opt/homebrew/bin/d1d"];
    for candidate in &candidates {
        if which::which(candidate).is_ok() {
            return Some(candidate.to_string());
        }
    }
    // Check cargo target directory
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let debug_path = format!("{manifest_dir}/../../target/debug/d1d");
        if std::path::Path::new(&debug_path).exists() {
            return Some(debug_path);
        }
    }
    None
}

fn expand_config_path() -> String {
    dirs::home_dir()
        .map(|h| {
            h.join(".d1doctor")
                .join("config.toml")
                .to_string_lossy()
                .into_owned()
        })
        .unwrap_or_else(|| "~/.d1doctor/config.toml".to_string())
}
