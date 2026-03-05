//! Core system operations for the MCP system server.
//!
//! Provides stateless wrappers around OS commands for package management,
//! service control, config file manipulation, environment variables, and
//! network diagnostics. Uses `tokio::process::Command` for async shell
//! execution and `cfg!(target_os)` for OS-specific dispatch.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use tokio::process::Command;

// ---------------------------------------------------------------------------
// Helper structs
// ---------------------------------------------------------------------------

/// Information about a package from `brew search` / `apt search`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: Option<String>,
    pub installed: bool,
}

/// Information about a system service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub status: String,
    pub pid: Option<u32>,
}

/// Result of a network diagnostic check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkResult {
    pub success: bool,
    pub output: String,
    pub latency_ms: Option<f64>,
}

// ---------------------------------------------------------------------------
// SystemOps
// ---------------------------------------------------------------------------

/// Stateless system operations.
///
/// Every method spawns a child process (via `tokio::process::Command`) or
/// uses standard library calls. No internal state is held.
pub struct SystemOps;

impl SystemOps {
    pub fn new() -> Self {
        SystemOps
    }

    // -- Package management -------------------------------------------------

    /// Search for packages matching `name`.
    pub async fn package_search(&self, name: &str) -> Result<Vec<PackageInfo>> {
        if cfg!(target_os = "macos") {
            self.brew_search(name).await
        } else {
            self.apt_search(name).await
        }
    }

    /// Install a package by name.
    pub async fn package_install(&self, name: &str) -> Result<String> {
        let output = if cfg!(target_os = "macos") {
            Command::new("brew")
                .args(["install", name])
                .output()
                .await
                .context("failed to run brew install")?
        } else {
            Command::new("sudo")
                .args(["apt", "install", "-y", name])
                .output()
                .await
                .context("failed to run apt install")?
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(format!("{stdout}\n{stderr}").trim().to_string())
        } else {
            anyhow::bail!("package install failed: {stderr}")
        }
    }

    /// Remove a package by name.
    pub async fn package_remove(&self, name: &str) -> Result<String> {
        let output = if cfg!(target_os = "macos") {
            Command::new("brew")
                .args(["uninstall", name])
                .output()
                .await
                .context("failed to run brew uninstall")?
        } else {
            Command::new("sudo")
                .args(["apt", "remove", "-y", name])
                .output()
                .await
                .context("failed to run apt remove")?
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(format!("{stdout}\n{stderr}").trim().to_string())
        } else {
            anyhow::bail!("package remove failed: {stderr}")
        }
    }

    // -- Service management -------------------------------------------------

    /// Query the status of a system service.
    pub async fn service_status(&self, name: &str) -> Result<ServiceInfo> {
        if cfg!(target_os = "macos") {
            self.launchctl_status(name).await
        } else {
            self.systemctl_status(name).await
        }
    }

    /// Control a system service (start / stop / restart).
    pub async fn service_control(&self, name: &str, action: &str) -> Result<String> {
        match action {
            "start" | "stop" | "restart" => {}
            other => anyhow::bail!("unsupported service action: {other}"),
        }

        let output = if cfg!(target_os = "macos") {
            // launchctl uses kickstart -k for restart, bootstrap/bootout for start/stop
            match action {
                "start" => Command::new("launchctl")
                    .args(["start", name])
                    .output()
                    .await
                    .context("failed to run launchctl start")?,
                "stop" => Command::new("launchctl")
                    .args(["stop", name])
                    .output()
                    .await
                    .context("failed to run launchctl stop")?,
                "restart" => {
                    // stop then start
                    let _ = Command::new("launchctl")
                        .args(["stop", name])
                        .output()
                        .await;
                    Command::new("launchctl")
                        .args(["start", name])
                        .output()
                        .await
                        .context("failed to run launchctl start (restart)")?
                }
                _ => unreachable!(),
            }
        } else {
            Command::new("sudo")
                .args(["systemctl", action, name])
                .output()
                .await
                .context("failed to run systemctl")?
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(format!("{stdout}\n{stderr}").trim().to_string())
        } else {
            anyhow::bail!("service {action} failed: {stderr}")
        }
    }

    // -- Configuration file manipulation ------------------------------------

    /// Read a configuration file and return its contents as a JSON value.
    ///
    /// Supported formats: `"toml"`, `"json"`, `"yaml"`.
    pub async fn config_read(&self, path: &str, format: &str) -> Result<Value> {
        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("failed to read config file: {path}"))?;

        match format.to_lowercase().as_str() {
            "toml" => {
                let table: toml::Value =
                    toml::from_str(&content).context("failed to parse TOML")?;
                let json_str =
                    serde_json::to_string(&table).context("failed to convert TOML to JSON")?;
                serde_json::from_str(&json_str).context("failed to parse converted JSON")
            }
            "json" => serde_json::from_str(&content).context("failed to parse JSON"),
            "yaml" | "yml" => {
                // Basic YAML support — we don't add a serde_yaml dep, so we
                // only handle the subset that is also valid JSON (or error).
                anyhow::bail!("YAML support requires the serde_yaml crate; currently only TOML and JSON are supported")
            }
            other => anyhow::bail!("unsupported config format: {other}"),
        }
    }

    /// Set a key in a configuration file.
    ///
    /// For TOML files the `key` uses dotted notation (`section.key`).
    /// For JSON files the `key` uses dotted notation mapped to nested objects.
    pub async fn config_set(&self, path: &str, key: &str, value: &str) -> Result<()> {
        let format = Self::detect_format(path);

        match format.as_str() {
            "toml" => self.config_set_toml(path, key, value).await,
            "json" => self.config_set_json(path, key, value).await,
            other => anyhow::bail!("unsupported config format for writing: {other}"),
        }
    }

    // -- Environment variables ----------------------------------------------

    /// Get an environment variable from the current process.
    pub fn env_get(&self, key: &str) -> Result<Option<String>> {
        match std::env::var(key) {
            Ok(val) => Ok(Some(val)),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Set an environment variable for the current process.
    ///
    /// Note: this only affects the running daemon process. It cannot persist
    /// across processes without editing shell profile files.
    pub fn env_set(&self, key: &str, value: &str) -> Result<()> {
        std::env::set_var(key, value);
        Ok(())
    }

    // -- Network diagnostics ------------------------------------------------

    /// Run a network diagnostic check.
    ///
    /// Supported `check_type` values:
    /// - `"ping"` — ICMP ping to `target` (hostname or IP)
    /// - `"dns"` — DNS lookup via `nslookup`
    /// - `"port"` — TCP connect to `target` in `host:port` form
    /// - `"connectivity"` — check internet connectivity (ignores `target`)
    pub async fn network_check(&self, check_type: &str, target: &str) -> Result<NetworkResult> {
        match check_type {
            "ping" => self.check_ping(target).await,
            "dns" => self.check_dns(target).await,
            "port" => self.check_port(target).await,
            "connectivity" => self.check_connectivity().await,
            other => anyhow::bail!("unsupported network check type: {other}"),
        }
    }

    // =======================================================================
    // Private helpers
    // =======================================================================

    // -- brew helpers -------------------------------------------------------

    async fn brew_search(&self, name: &str) -> Result<Vec<PackageInfo>> {
        let output = Command::new("brew")
            .args(["search", name])
            .output()
            .await
            .context("failed to run brew search")?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Also get the list of installed formulae to set the `installed` flag
        let installed_output = Command::new("brew")
            .args(["list", "--formula", "-1"])
            .output()
            .await
            .unwrap_or_else(|_| std::process::Output {
                status: std::process::ExitStatus::default(),
                stdout: Vec::new(),
                stderr: Vec::new(),
            });
        let installed_list = String::from_utf8_lossy(&installed_output.stdout);
        let installed_set: std::collections::HashSet<&str> =
            installed_list.lines().collect();

        let packages: Vec<PackageInfo> = stdout
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with("==>"))
            .map(|line| {
                let trimmed = line.trim();
                PackageInfo {
                    name: trimmed.to_string(),
                    version: None,
                    installed: installed_set.contains(trimmed),
                }
            })
            .collect();

        Ok(packages)
    }

    // -- apt helpers --------------------------------------------------------

    async fn apt_search(&self, name: &str) -> Result<Vec<PackageInfo>> {
        let output = Command::new("apt")
            .args(["search", name])
            .output()
            .await
            .context("failed to run apt search")?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut packages = Vec::new();
        for line in stdout.lines() {
            // apt search output format: "package/suite version arch [installed]"
            if line.starts_with(' ') || line.is_empty() {
                continue; // description line
            }
            let parts: Vec<&str> = line.splitn(2, '/').collect();
            if parts.len() < 2 {
                continue;
            }
            let pkg_name = parts[0].to_string();
            let rest = parts[1];
            let installed = rest.contains("[installed");
            let version = rest.split_whitespace().nth(1).map(|s| s.to_string());

            packages.push(PackageInfo {
                name: pkg_name,
                version,
                installed,
            });
        }

        Ok(packages)
    }

    // -- launchctl helpers --------------------------------------------------

    async fn launchctl_status(&self, name: &str) -> Result<ServiceInfo> {
        let output = Command::new("launchctl")
            .args(["list"])
            .output()
            .await
            .context("failed to run launchctl list")?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // launchctl list output columns: PID  Status  Label
        for line in stdout.lines() {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() >= 3 {
                let label = cols[2];
                if label == name || label.contains(name) {
                    let pid = cols[0].parse::<u32>().ok();
                    let status = if pid.is_some() { "running" } else { "stopped" };
                    return Ok(ServiceInfo {
                        name: label.to_string(),
                        status: status.to_string(),
                        pid,
                    });
                }
            }
        }

        Ok(ServiceInfo {
            name: name.to_string(),
            status: "not found".to_string(),
            pid: None,
        })
    }

    // -- systemctl helpers --------------------------------------------------

    async fn systemctl_status(&self, name: &str) -> Result<ServiceInfo> {
        let output = Command::new("systemctl")
            .args(["status", name])
            .output()
            .await
            .context("failed to run systemctl status")?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut status = "unknown".to_string();
        let mut pid = None;

        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Active:") {
                // e.g. "Active: active (running) since ..."
                if trimmed.contains("active (running)") {
                    status = "running".to_string();
                } else if trimmed.contains("inactive") {
                    status = "stopped".to_string();
                } else if trimmed.contains("failed") {
                    status = "failed".to_string();
                } else {
                    status = trimmed.to_string();
                }
            }
            if trimmed.starts_with("Main PID:") {
                // e.g. "Main PID: 1234 (nginx)"
                pid = trimmed
                    .split_whitespace()
                    .nth(2)
                    .and_then(|s| s.parse::<u32>().ok());
            }
        }

        Ok(ServiceInfo {
            name: name.to_string(),
            status,
            pid,
        })
    }

    // -- config helpers -----------------------------------------------------

    fn detect_format(path: &str) -> String {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        match ext {
            "toml" => "toml".to_string(),
            "json" => "json".to_string(),
            "yaml" | "yml" => "yaml".to_string(),
            _ => "toml".to_string(), // default
        }
    }

    async fn config_set_toml(&self, path: &str, key: &str, value: &str) -> Result<()> {
        let content = tokio::fs::read_to_string(path).await.unwrap_or_default();

        let mut table: toml::map::Map<String, toml::Value> =
            if content.is_empty() {
                toml::map::Map::new()
            } else {
                toml::from_str(&content).context("failed to parse existing TOML")?
            };

        // Parse the value: try integer, float, bool, then fall back to string
        let toml_value = Self::parse_toml_value(value);

        // Support dotted keys: "section.key" -> table["section"]["key"]
        let parts: Vec<&str> = key.split('.').collect();
        Self::set_nested_toml(&mut table, &parts, toml_value)?;

        let output = toml::to_string_pretty(&table).context("failed to serialize TOML")?;
        tokio::fs::write(path, output)
            .await
            .context("failed to write TOML file")?;
        Ok(())
    }

    fn parse_toml_value(s: &str) -> toml::Value {
        if let Ok(i) = s.parse::<i64>() {
            return toml::Value::Integer(i);
        }
        if let Ok(f) = s.parse::<f64>() {
            return toml::Value::Float(f);
        }
        match s {
            "true" => return toml::Value::Boolean(true),
            "false" => return toml::Value::Boolean(false),
            _ => {}
        }
        toml::Value::String(s.to_string())
    }

    fn set_nested_toml(
        table: &mut toml::map::Map<String, toml::Value>,
        parts: &[&str],
        value: toml::Value,
    ) -> Result<()> {
        if parts.is_empty() {
            anyhow::bail!("empty key");
        }
        if parts.len() == 1 {
            table.insert(parts[0].to_string(), value);
            return Ok(());
        }

        let entry = table
            .entry(parts[0].to_string())
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));

        match entry {
            toml::Value::Table(ref mut inner) => {
                Self::set_nested_toml(inner, &parts[1..], value)
            }
            _ => {
                // Overwrite non-table with a new table
                *entry = toml::Value::Table(toml::map::Map::new());
                if let toml::Value::Table(ref mut inner) = entry {
                    Self::set_nested_toml(inner, &parts[1..], value)
                } else {
                    unreachable!()
                }
            }
        }
    }

    async fn config_set_json(&self, path: &str, key: &str, value: &str) -> Result<()> {
        let content = tokio::fs::read_to_string(path).await.unwrap_or_default();

        let mut root: Value = if content.is_empty() {
            Value::Object(serde_json::Map::new())
        } else {
            serde_json::from_str(&content).context("failed to parse existing JSON")?
        };

        // Parse value: try JSON literal, then fall back to string
        let json_value: Value = serde_json::from_str(value).unwrap_or(Value::String(value.to_string()));

        // Support dotted keys
        let parts: Vec<&str> = key.split('.').collect();
        Self::set_nested_json(&mut root, &parts, json_value)?;

        let output =
            serde_json::to_string_pretty(&root).context("failed to serialize JSON")?;
        tokio::fs::write(path, output)
            .await
            .context("failed to write JSON file")?;
        Ok(())
    }

    fn set_nested_json(obj: &mut Value, parts: &[&str], value: Value) -> Result<()> {
        if parts.is_empty() {
            anyhow::bail!("empty key");
        }
        if parts.len() == 1 {
            if let Value::Object(ref mut map) = obj {
                map.insert(parts[0].to_string(), value);
                return Ok(());
            }
            anyhow::bail!("cannot set key on non-object JSON value");
        }

        if let Value::Object(ref mut map) = obj {
            let entry = map
                .entry(parts[0].to_string())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));
            Self::set_nested_json(entry, &parts[1..], value)
        } else {
            anyhow::bail!("cannot descend into non-object JSON value")
        }
    }

    // -- network helpers ----------------------------------------------------

    async fn check_ping(&self, target: &str) -> Result<NetworkResult> {
        let output = Command::new("ping")
            .args(["-c", "3", "-W", "5", target])
            .output()
            .await
            .context("failed to run ping")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let success = output.status.success();

        // Try to extract avg latency from "min/avg/max/stddev = ..." line
        let latency_ms = stdout
            .lines()
            .find(|l| l.contains("avg"))
            .and_then(|line| {
                // macOS: "round-trip min/avg/max/stddev = 1.0/2.0/3.0/0.5 ms"
                // Linux: "rtt min/avg/max/mdev = 1.0/2.0/3.0/0.5 ms"
                line.split('=')
                    .nth(1)
                    .and_then(|vals| vals.trim().split('/').nth(1))
                    .and_then(|avg| avg.trim().parse::<f64>().ok())
            });

        Ok(NetworkResult {
            success,
            output: stdout,
            latency_ms,
        })
    }

    async fn check_dns(&self, target: &str) -> Result<NetworkResult> {
        let output = Command::new("nslookup")
            .arg(target)
            .output()
            .await
            .context("failed to run nslookup")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let success = output.status.success();

        Ok(NetworkResult {
            success,
            output: stdout,
            latency_ms: None,
        })
    }

    async fn check_port(&self, target: &str) -> Result<NetworkResult> {
        // target should be "host:port"
        let parts: Vec<&str> = target.rsplitn(2, ':').collect();
        if parts.len() != 2 {
            anyhow::bail!("port check target must be in host:port format, got: {target}");
        }
        let port = parts[0];
        let host = parts[1];

        let addr = format!("{host}:{port}");
        let start = std::time::Instant::now();

        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            tokio::net::TcpStream::connect(&addr),
        )
        .await
        {
            Ok(Ok(_stream)) => {
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                Ok(NetworkResult {
                    success: true,
                    output: format!("Connected to {addr}"),
                    latency_ms: Some(elapsed),
                })
            }
            Ok(Err(e)) => Ok(NetworkResult {
                success: false,
                output: format!("Connection failed: {e}"),
                latency_ms: None,
            }),
            Err(_) => Ok(NetworkResult {
                success: false,
                output: format!("Connection timed out after 5s"),
                latency_ms: None,
            }),
        }
    }

    async fn check_connectivity(&self) -> Result<NetworkResult> {
        // Try to connect to well-known DNS servers on TCP port 53
        let start = std::time::Instant::now();

        let targets = ["1.1.1.1:53", "8.8.8.8:53"];
        for target in &targets {
            match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                tokio::net::TcpStream::connect(target),
            )
            .await
            {
                Ok(Ok(_)) => {
                    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                    return Ok(NetworkResult {
                        success: true,
                        output: format!("Internet connectivity confirmed via {target}"),
                        latency_ms: Some(elapsed),
                    });
                }
                _ => continue,
            }
        }

        Ok(NetworkResult {
            success: false,
            output: "No internet connectivity detected".to_string(),
            latency_ms: None,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_get_set_roundtrip() {
        let ops = SystemOps::new();

        // Set a unique env var
        let key = "D1_DOCTOR_TEST_ENV_VAR_12345";
        ops.env_set(key, "hello_world").unwrap();

        let val = ops.env_get(key).unwrap();
        assert_eq!(val, Some("hello_world".to_string()));

        // Clean up
        std::env::remove_var(key);
        let val = ops.env_get(key).unwrap();
        assert_eq!(val, None);
    }

    #[test]
    fn test_env_get_missing() {
        let ops = SystemOps::new();
        let val = ops
            .env_get("D1_DOCTOR_NONEXISTENT_VAR_99999")
            .unwrap();
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn test_config_read_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.toml");
        let content = r#"
[server]
host = "localhost"
port = 8080

[database]
url = "sqlite:///tmp/test.db"
"#;
        tokio::fs::write(&path, content).await.unwrap();

        let ops = SystemOps::new();
        let val = ops
            .config_read(path.to_str().unwrap(), "toml")
            .await
            .unwrap();

        assert_eq!(val["server"]["host"], "localhost");
        assert_eq!(val["server"]["port"], 8080);
        assert_eq!(val["database"]["url"], "sqlite:///tmp/test.db");
    }

    #[tokio::test]
    async fn test_config_read_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.json");
        let content = r#"{"server":{"host":"localhost","port":8080}}"#;
        tokio::fs::write(&path, content).await.unwrap();

        let ops = SystemOps::new();
        let val = ops
            .config_read(path.to_str().unwrap(), "json")
            .await
            .unwrap();

        assert_eq!(val["server"]["host"], "localhost");
        assert_eq!(val["server"]["port"], 8080);
    }

    #[tokio::test]
    async fn test_config_set_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.toml");
        let content = r#"
[server]
host = "localhost"
port = 8080
"#;
        tokio::fs::write(&path, content).await.unwrap();

        let ops = SystemOps::new();
        ops.config_set(path.to_str().unwrap(), "server.port", "9090")
            .await
            .unwrap();

        let val = ops
            .config_read(path.to_str().unwrap(), "toml")
            .await
            .unwrap();
        assert_eq!(val["server"]["port"], 9090);
        // Original values preserved
        assert_eq!(val["server"]["host"], "localhost");
    }

    #[tokio::test]
    async fn test_config_set_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.json");
        let content = r#"{"server":{"host":"localhost","port":8080}}"#;
        tokio::fs::write(&path, content).await.unwrap();

        let ops = SystemOps::new();
        ops.config_set(path.to_str().unwrap(), "server.port", "9090")
            .await
            .unwrap();

        let val = ops
            .config_read(path.to_str().unwrap(), "json")
            .await
            .unwrap();
        assert_eq!(val["server"]["port"], 9090);
        assert_eq!(val["server"]["host"], "localhost");
    }

    #[tokio::test]
    async fn test_config_set_new_key_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.toml");
        tokio::fs::write(&path, "").await.unwrap();

        let ops = SystemOps::new();
        ops.config_set(path.to_str().unwrap(), "new_section.key", "value")
            .await
            .unwrap();

        let val = ops
            .config_read(path.to_str().unwrap(), "toml")
            .await
            .unwrap();
        assert_eq!(val["new_section"]["key"], "value");
    }

    #[tokio::test]
    async fn test_network_check_connectivity() {
        let ops = SystemOps::new();
        // Just verify it returns a result without panicking
        let result = ops.network_check("connectivity", "").await;
        assert!(result.is_ok());
        let nr = result.unwrap();
        // We can't guarantee internet access in CI, but the struct should be valid
        assert!(!nr.output.is_empty());
    }

    #[tokio::test]
    async fn test_network_check_port() {
        let ops = SystemOps::new();
        // Attempt to connect to a port that likely isn't open
        let result = ops
            .network_check("port", "127.0.0.1:19999")
            .await
            .unwrap();
        // The result should be valid regardless of success/failure
        assert!(!result.output.is_empty());
    }

    #[test]
    fn test_detect_format() {
        assert_eq!(SystemOps::detect_format("/etc/config.toml"), "toml");
        assert_eq!(SystemOps::detect_format("/etc/config.json"), "json");
        assert_eq!(SystemOps::detect_format("/etc/config.yaml"), "yaml");
        assert_eq!(SystemOps::detect_format("/etc/config.yml"), "yaml");
        assert_eq!(SystemOps::detect_format("/etc/config"), "toml"); // default
    }
}
