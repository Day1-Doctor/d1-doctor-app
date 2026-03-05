//! MCP server registry — discovery and lifecycle management of custom MCP tool servers.
//!
//! Reads `~/.d1doctor/mcp_servers.toml` to discover user-configured MCP servers,
//! spawns them as child processes, and manages their lifecycle.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use serde::Deserialize;
use tokio::process::{Child, Command};

// ---------------------------------------------------------------------------
// Configuration types
// ---------------------------------------------------------------------------

/// Configuration for a single MCP server, deserialised from TOML.
#[derive(Debug, Clone, Deserialize)]
pub struct McpServerConfig {
    /// Human-readable name for this server.
    pub name: String,

    /// Absolute path (or binary name on `$PATH`) to the server executable.
    pub command: String,

    /// Command-line arguments forwarded to the server process.
    #[serde(default)]
    pub args: Vec<String>,

    /// Extra environment variables injected into the server process.
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Whether the server should be started. Defaults to `true`.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// Top-level wrapper that maps the TOML file structure.
#[derive(Debug, Clone, Deserialize)]
pub struct McpServersConfig {
    #[serde(default)]
    pub servers: Vec<McpServerConfig>,
}

// ---------------------------------------------------------------------------
// Runtime types
// ---------------------------------------------------------------------------

/// Snapshot of a managed server's current state.
#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub name: String,
    pub pid: Option<u32>,
    pub status: ServerState,
    pub uptime_secs: Option<f64>,
}

/// Possible states for a managed server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerState {
    Running,
    Stopped,
    Error(String),
}

/// Internal bookkeeping for a running server.
struct ManagedServer {
    name: String,
    child: Child,
    started_at: Instant,
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Manages discovery and lifecycle of user-configured MCP servers.
pub struct McpRegistry {
    servers: Vec<ManagedServer>,
}

impl McpRegistry {
    /// Create an empty registry with no managed servers.
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
        }
    }

    /// Return the platform-appropriate default path for the config file:
    /// `~/.d1doctor/mcp_servers.toml`.
    pub fn default_config_path() -> PathBuf {
        d1_common::config_dir().join("mcp_servers.toml")
    }

    /// Read and parse an `McpServersConfig` from the given file path.
    pub fn load_config(path: &str) -> anyhow::Result<McpServersConfig> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("failed to read MCP config at {path}: {e}"))?;
        let config: McpServersConfig = toml::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("failed to parse MCP config at {path}: {e}"))?;
        Ok(config)
    }

    /// Spawn every *enabled* server defined in `config` as a child process.
    ///
    /// Servers whose `enabled` flag is `false` are silently skipped.
    pub async fn spawn_servers(&mut self, config: &McpServersConfig) -> anyhow::Result<()> {
        for server_cfg in &config.servers {
            if !server_cfg.enabled {
                tracing::info!(name = %server_cfg.name, "MCP server disabled, skipping");
                continue;
            }

            tracing::info!(
                name = %server_cfg.name,
                command = %server_cfg.command,
                "spawning MCP server"
            );

            let child = Command::new(&server_cfg.command)
                .args(&server_cfg.args)
                .envs(&server_cfg.env)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .kill_on_drop(true)
                .spawn()
                .map_err(|e| {
                    anyhow::anyhow!(
                        "failed to spawn MCP server '{}': {}",
                        server_cfg.name,
                        e
                    )
                })?;

            self.servers.push(ManagedServer {
                name: server_cfg.name.clone(),
                child,
                started_at: Instant::now(),
            });
        }

        Ok(())
    }

    /// Return a snapshot of every managed server's current status.
    pub fn list_servers(&self) -> Vec<ServerStatus> {
        self.servers
            .iter()
            .map(|s| {
                let pid = s.child.id();
                let (status, uptime) = if pid.is_some() {
                    (ServerState::Running, Some(s.started_at.elapsed().as_secs_f64()))
                } else {
                    (ServerState::Stopped, None)
                };

                ServerStatus {
                    name: s.name.clone(),
                    pid,
                    status,
                    uptime_secs: uptime,
                }
            })
            .collect()
    }

    /// Gracefully stop all managed servers.
    ///
    /// Sends a kill signal to each child process and waits for it to exit.
    pub async fn stop_all(&mut self) -> anyhow::Result<()> {
        for server in &mut self.servers {
            tracing::info!(name = %server.name, "stopping MCP server");
            if let Err(e) = server.child.kill().await {
                tracing::warn!(
                    name = %server.name,
                    error = %e,
                    "failed to kill MCP server (may have already exited)"
                );
            }
        }
        self.servers.clear();
        Ok(())
    }
}

impl Default for McpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_TOML: &str = r#"
[[servers]]
name = "my-custom-tool"
command = "/usr/local/bin/my-tool"
args = ["--stdio"]
enabled = true

[servers.env]
API_KEY = "your-key-here"

[[servers]]
name = "disabled-tool"
command = "/usr/bin/other-tool"
enabled = false
"#;

    #[test]
    fn parse_example_config() {
        let config: McpServersConfig = toml::from_str(EXAMPLE_TOML).unwrap();
        assert_eq!(config.servers.len(), 2);
    }

    #[test]
    fn verify_server_config_fields() {
        let config: McpServersConfig = toml::from_str(EXAMPLE_TOML).unwrap();

        let first = &config.servers[0];
        assert_eq!(first.name, "my-custom-tool");
        assert_eq!(first.command, "/usr/local/bin/my-tool");
        assert_eq!(first.args, vec!["--stdio"]);
        assert!(first.enabled);
        assert_eq!(first.env.get("API_KEY").unwrap(), "your-key-here");

        let second = &config.servers[1];
        assert_eq!(second.name, "disabled-tool");
        assert!(!second.enabled);
        assert!(second.args.is_empty());
        assert!(second.env.is_empty());
    }

    #[test]
    fn list_servers_on_empty_registry() {
        let registry = McpRegistry::new();
        let statuses = registry.list_servers();
        assert!(statuses.is_empty());
    }

    #[test]
    fn default_config_path_ends_correctly() {
        let path = McpRegistry::default_config_path();
        assert!(path.ends_with(".d1doctor/mcp_servers.toml"));
    }

    #[test]
    fn enabled_defaults_to_true() {
        let toml_str = r#"
[[servers]]
name = "minimal"
command = "echo"
"#;
        let config: McpServersConfig = toml::from_str(toml_str).unwrap();
        assert!(config.servers[0].enabled);
        assert!(config.servers[0].args.is_empty());
        assert!(config.servers[0].env.is_empty());
    }

    #[test]
    fn load_config_missing_file_returns_error() {
        let result = McpRegistry::load_config("/nonexistent/path/mcp_servers.toml");
        assert!(result.is_err());
    }
}
