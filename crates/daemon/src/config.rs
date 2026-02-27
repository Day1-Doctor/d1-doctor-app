//! Daemon configuration — reads ~/.d1doctor/config.toml with env override.
//! Spec: LocalStack_v2.4.1_Spec.md §2.5

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DaemonSection {
    pub port: u16,
    pub log_level: String,
    pub log_file: String,
    pub log_max_size_mb: u32,
    pub db_path: String,
    pub pid_file: String,
    pub shutdown_grace_secs: u32,
}

impl Default for DaemonSection {
    fn default() -> Self {
        Self {
            port: 9876,
            log_level: "info".into(),
            log_file: "~/.d1doctor/daemon.log".into(),
            log_max_size_mb: 50,
            db_path: "~/.d1doctor/d1doctor.db".into(),
            pid_file: "~/.d1doctor/d1d.pid".into(),
            shutdown_grace_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OrchestratorSection {
    pub url: String,
    pub allow_unauthenticated_local: bool,
    pub heartbeat_interval_secs: u64,
    pub reconnect_backoff_base_secs: u64,
    pub reconnect_backoff_max_secs: u64,
}

impl Default for OrchestratorSection {
    fn default() -> Self {
        Self {
            url: "wss://api.day1doctor.com/ws/connect".into(),
            allow_unauthenticated_local: false,
            heartbeat_interval_secs: 30,
            reconnect_backoff_base_secs: 1,
            reconnect_backoff_max_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AuthSection {
    pub device_id: String,
    pub supabase_url: String,
    pub supabase_anon_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PermissionsSection {
    pub approval_timeout_secs: u32,
}

impl Default for PermissionsSection {
    fn default() -> Self {
        Self { approval_timeout_secs: 300 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DaemonConfig {
    pub daemon: DaemonSection,
    pub orchestrator: OrchestratorSection,
    pub auth: AuthSection,
    pub permissions: PermissionsSection,
}

impl DaemonConfig {
    /// Load config from the given path. Returns default if file doesn't exist.
    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let mut cfg: Self = toml::from_str(&content)?;
        cfg.apply_auto_rules();
        Ok(cfg)
    }

    /// Load from default location (~/.d1doctor/config.toml), applying env overrides.
    pub fn load() -> Result<Self> {
        let path = default_config_path();
        let mut cfg = Self::load_from(&path)?;
        cfg.apply_env_overrides();
        Ok(cfg)
    }

    /// Auto-set allow_unauthenticated_local when URL is loopback.
    pub fn apply_auto_rules(&mut self) {
        let url = &self.orchestrator.url;
        if url.starts_with("ws://localhost") || url.starts_with("ws://127.0.0.1") {
            self.orchestrator.allow_unauthenticated_local = true;
        }
    }

    /// Apply D1_* environment variable overrides.
    fn apply_env_overrides(&mut self) {
        if let Ok(port) = std::env::var("D1_DAEMON_PORT") {
            if let Ok(p) = port.parse() {
                self.daemon.port = p;
            }
        }
        if let Ok(url) = std::env::var("D1_ORCHESTRATOR_URL") {
            self.orchestrator.url = url;
            self.apply_auto_rules();
        }
        if let Ok(level) = std::env::var("D1_LOG_LEVEL") {
            self.daemon.log_level = level;
        }
    }

    /// Expand ~ in a path string.
    pub fn expand_path(path: &str) -> PathBuf {
        if let Some(stripped) = path.strip_prefix("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(stripped);
            }
        }
        PathBuf::from(path)
    }

    pub fn db_path(&self) -> PathBuf {
        Self::expand_path(&self.daemon.db_path)
    }
}

pub fn default_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".d1doctor")
        .join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let cfg = DaemonConfig::default();
        assert_eq!(cfg.daemon.port, 9876);
        assert_eq!(cfg.orchestrator.heartbeat_interval_secs, 30);
        assert_eq!(cfg.permissions.approval_timeout_secs, 300);
    }

    #[test]
    fn test_load_from_toml() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "[daemon]\nport = 9877\nlog_level = \"debug\"").unwrap();
        let cfg = DaemonConfig::load_from(f.path()).unwrap();
        assert_eq!(cfg.daemon.port, 9877);
        assert_eq!(cfg.daemon.log_level, "debug");
    }

    #[test]
    fn test_missing_file_returns_default() {
        let cfg = DaemonConfig::load_from(std::path::Path::new("/nonexistent/path/config.toml")).unwrap();
        assert_eq!(cfg.daemon.port, 9876);
    }

    #[test]
    fn test_allow_unauthenticated_local_auto_set() {
        let mut cfg = DaemonConfig::default();
        cfg.orchestrator.url = "ws://localhost:8000/ws/connect".into();
        cfg.apply_auto_rules();
        assert!(cfg.orchestrator.allow_unauthenticated_local);
    }

    #[test]
    fn test_expand_path_tilde() {
        let expanded = DaemonConfig::expand_path("~/.d1doctor/db");
        let expanded_str = expanded.to_string_lossy();
        assert!(!expanded_str.starts_with("~"), "Should have expanded ~ to home dir");
    }
}
