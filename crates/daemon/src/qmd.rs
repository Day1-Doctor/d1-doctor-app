//! QMD Process Manager
//!
//! Manages the QMD (github.com/tobi/qmd) sidecar process lifecycle.
//! QMD is deployed as an MCP server using STDIO transport, providing
//! local semantic search capabilities for the Day1 Doctor daemon.

use std::path::PathBuf;
use std::process::Stdio;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};

/// Configuration for the QMD sidecar process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QmdConfig {
    /// Whether QMD integration is enabled.
    pub enabled: bool,
    /// Path to the QMD binary.
    pub binary_path: PathBuf,
    /// Path to the directory where QMD stores its models.
    pub model_path: PathBuf,
    /// Maximum number of search results to return.
    pub max_results: usize,
}

impl Default for QmdConfig {
    fn default() -> Self {
        let base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            enabled: true,
            binary_path: base.join(".d1doctor").join("bin").join("qmd"),
            model_path: base.join(".d1doctor").join("models"),
            max_results: 10,
        }
    }
}

/// Manages a QMD child process using STDIO transport.
pub struct QmdManager {
    config: QmdConfig,
    child: Option<Child>,
}

impl QmdManager {
    /// Create a new QmdManager with the given configuration.
    pub fn new(config: QmdConfig) -> Self {
        Self {
            config,
            child: None,
        }
    }

    /// Check whether the QMD binary exists at the configured path.
    pub fn is_available(&self) -> bool {
        self.config.binary_path.exists()
    }

    /// Spawn the QMD process with STDIO transport.
    ///
    /// Returns an error if the binary is not found or the process fails to start.
    pub async fn start(&mut self) -> anyhow::Result<()> {
        if !self.config.enabled {
            tracing::info!("QMD is disabled in configuration, skipping start");
            return Ok(());
        }

        if self.child.is_some() {
            tracing::warn!("QMD process is already running");
            return Ok(());
        }

        if !self.is_available() {
            return Err(anyhow::anyhow!(
                "QMD binary not found at {}",
                self.config.binary_path.display()
            ));
        }

        // Ensure model directory exists
        if !self.config.model_path.exists() {
            tokio::fs::create_dir_all(&self.config.model_path).await?;
        }

        tracing::info!(
            "Starting QMD process: {} (models: {})",
            self.config.binary_path.display(),
            self.config.model_path.display()
        );

        let child = Command::new(&self.config.binary_path)
            .arg("--stdio")
            .arg("--model-path")
            .arg(&self.config.model_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        tracing::info!("QMD process started with PID {}", child.id().unwrap_or(0));
        self.child = Some(child);
        Ok(())
    }

    /// Gracefully stop the QMD process.
    ///
    /// Sends a shutdown signal and waits for the process to exit.
    /// Falls back to kill if graceful shutdown fails.
    pub async fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(mut child) = self.child.take() {
            tracing::info!("Stopping QMD process...");

            // Try graceful shutdown by closing stdin
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.shutdown().await;
            }

            // Wait briefly for graceful exit, then kill
            match tokio::time::timeout(std::time::Duration::from_secs(5), child.wait()).await {
                Ok(Ok(status)) => {
                    tracing::info!("QMD process exited with status: {}", status);
                }
                Ok(Err(e)) => {
                    tracing::warn!("Error waiting for QMD process: {}", e);
                }
                Err(_) => {
                    tracing::warn!("QMD process did not exit gracefully, killing...");
                    let _ = child.kill().await;
                }
            }
        } else {
            tracing::debug!("QMD process is not running, nothing to stop");
        }
        Ok(())
    }

    /// Check whether the QMD process is still alive.
    pub fn health_check(&mut self) -> bool {
        match &mut self.child {
            Some(child) => {
                // try_wait returns Ok(None) if the process is still running
                match child.try_wait() {
                    Ok(None) => true,
                    Ok(Some(status)) => {
                        tracing::warn!("QMD process exited unexpectedly: {}", status);
                        false
                    }
                    Err(e) => {
                        tracing::error!("Failed to check QMD process status: {}", e);
                        false
                    }
                }
            }
            None => false,
        }
    }

    /// Send a JSON-RPC request to QMD via STDIO and read the response.
    ///
    /// This implements the MCP STDIO transport protocol: newline-delimited JSON.
    pub async fn send_request(
        &mut self,
        request: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let child = self
            .child
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("QMD process is not running"))?;

        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("QMD stdin is not available"))?;

        let stdout = child
            .stdout
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("QMD stdout is not available"))?;

        // Write JSON-RPC request followed by newline
        let mut payload = serde_json::to_vec(&request)?;
        payload.push(b'\n');
        stdin.write_all(&payload).await?;
        stdin.flush().await?;

        // Read response line
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let response: serde_json::Value = serde_json::from_str(line.trim())?;
        Ok(response)
    }

    /// Get a reference to the current configuration.
    pub fn config(&self) -> &QmdConfig {
        &self.config
    }
}

impl Drop for QmdManager {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // Best-effort synchronous kill on drop
            let _ = child.start_kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = QmdConfig::default();
        assert!(config.enabled);
        assert!(config.binary_path.ends_with("qmd"));
        assert!(config.model_path.ends_with("models"));
        assert_eq!(config.max_results, 10);
    }

    #[test]
    fn test_qmd_manager_not_available() {
        let config = QmdConfig {
            enabled: true,
            binary_path: PathBuf::from("/nonexistent/path/qmd"),
            model_path: PathBuf::from("/tmp/qmd-models"),
            max_results: 5,
        };
        let manager = QmdManager::new(config);
        assert!(!manager.is_available());
    }

    #[test]
    fn test_health_check_no_process() {
        let config = QmdConfig::default();
        let mut manager = QmdManager::new(config);
        assert!(!manager.health_check());
    }
}
