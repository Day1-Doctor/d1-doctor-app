//! Application configuration
//!
//! Reads from ~/.d1doctor/config.toml

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

use crate::errors::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Orchestrator WebSocket URL
    #[serde(default = "default_orchestrator_url")]
    pub orchestrator_url: String,

    /// Local daemon listen port
    #[serde(default = "default_daemon_port")]
    pub daemon_port: u16,

    /// Supabase configuration
    pub supabase: Option<SupabaseConfig>,

    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Permission zones
    #[serde(default)]
    pub permissions: PermissionsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseConfig {
    pub project_url: String,
    pub anon_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Path to SQLite database
    #[serde(default = "default_db_path")]
    pub path: PathBuf,

    /// Enable WAL mode
    #[serde(default = "default_true")]
    pub wal_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level: trace, debug, info, warn, error
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Enable JSON formatting
    #[serde(default)]
    pub json: bool,

    /// Log file path (optional)
    pub file: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsConfig {
    /// Timeout for permission approval (seconds)
    #[serde(default = "default_permission_timeout")]
    pub approval_timeout: u64,

    /// Cache TTL (seconds)
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,
}

fn default_orchestrator_url() -> String {
    "wss://api.day1doctor.com/ws".to_string()
}

fn default_daemon_port() -> u16 {
    crate::DEFAULT_DAEMON_PORT
}

fn default_db_path() -> PathBuf {
    crate::config_dir().join("d1doctor.db")
}

fn default_true() -> bool {
    true
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_permission_timeout() -> u64 {
    300 // 5 minutes
}

fn default_cache_ttl() -> u64 {
    3600 // 1 hour
}

impl Default for Config {
    fn default() -> Self {
        Self {
            orchestrator_url: default_orchestrator_url(),
            daemon_port: default_daemon_port(),
            supabase: None,
            database: DatabaseConfig::default(),
            logging: LoggingConfig::default(),
            permissions: PermissionsConfig::default(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: default_db_path(),
            wal_enabled: true,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            json: false,
            file: None,
        }
    }
}

impl Default for PermissionsConfig {
    fn default() -> Self {
        Self {
            approval_timeout: default_permission_timeout(),
            cache_ttl: default_cache_ttl(),
        }
    }
}

impl Config {
    /// Load configuration from ~/.d1doctor/config.toml
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        debug!(?config_path, "Loading configuration");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config = toml::from_str(&content)?;
            debug!("Configuration loaded successfully");
            Ok(config)
        } else {
            warn!("Configuration file not found, using defaults");
            Ok(Self::default())
        }
    }

    /// Get the configuration file path
    pub fn config_path() -> PathBuf {
        crate::config_dir().join("config.toml")
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<()> {
        let config_dir = crate::config_dir();
        std::fs::create_dir_all(&config_dir)?;

        let config_path = Self::config_path();
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        debug!(?config_path, "Configuration saved");
        Ok(())
    }

    /// Merge another config into this one (other takes precedence)
    pub fn merge(&mut self, other: Config) {
        if other.orchestrator_url != default_orchestrator_url() {
            self.orchestrator_url = other.orchestrator_url;
        }
        if other.daemon_port != default_daemon_port() {
            self.daemon_port = other.daemon_port;
        }
        if other.supabase.is_some() {
            self.supabase = other.supabase;
        }
        self.database = other.database;
        self.logging = other.logging;
        self.permissions = other.permissions;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.orchestrator_url.is_empty());
        assert!(config.daemon_port > 0);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml = toml::to_string_pretty(&config).unwrap();
        assert!(toml.contains("orchestrator_url"));
    }
}
