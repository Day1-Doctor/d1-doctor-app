//! Error types for d1-doctor
//!
//! Provides a unified error type for all d1-doctor components
//! with proper error context and conversion traits.

use std::io;
use thiserror::Error;

/// Result type for d1-doctor operations
pub type Result<T, E = D1Error> = std::result::Result<T, E>;

/// Unified error type for d1-doctor
#[derive(Error, Debug)]
pub enum D1Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerError(#[from] toml::ser::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Command execution error: {0}")]
    Execution(String),

    #[error("Health check failed: {0}")]
    HealthCheck(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Timeout")]
    Timeout,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl D1Error {
    /// Create a config error
    pub fn config(msg: impl Into<String>) -> Self {
        D1Error::Config(msg.into())
    }

    /// Create a database error
    pub fn database(msg: impl Into<String>) -> Self {
        D1Error::Database(msg.into())
    }

    /// Create a WebSocket error
    pub fn websocket(msg: impl Into<String>) -> Self {
        D1Error::WebSocket(msg.into())
    }

    /// Create a protocol error
    pub fn protocol(msg: impl Into<String>) -> Self {
        D1Error::Protocol(msg.into())
    }

    /// Create an auth error
    pub fn auth(msg: impl Into<String>) -> Self {
        D1Error::Auth(msg.into())
    }

    /// Create a permission denied error
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        D1Error::PermissionDenied(msg.into())
    }

    /// Create an execution error
    pub fn execution(msg: impl Into<String>) -> Self {
        D1Error::Execution(msg.into())
    }

    /// Create a health check error
    pub fn health_check(msg: impl Into<String>) -> Self {
        D1Error::HealthCheck(msg.into())
    }

    /// Create a session error
    pub fn session(msg: impl Into<String>) -> Self {
        D1Error::Session(msg.into())
    }

    /// Create a network error
    pub fn network(msg: impl Into<String>) -> Self {
        D1Error::Network(msg.into())
    }

    /// Create a not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        D1Error::NotFound(msg.into())
    }

    /// Create an invalid state error
    pub fn invalid_state(msg: impl Into<String>) -> Self {
        D1Error::InvalidState(msg.into())
    }

    /// Create an invalid argument error
    pub fn invalid_argument(msg: impl Into<String>) -> Self {
        D1Error::InvalidArgument(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        D1Error::Internal(msg.into())
    }

    /// Check if this is a timeout error
    pub fn is_timeout(&self) -> bool {
        matches!(self, D1Error::Timeout)
    }

    /// Check if this is a permission error
    pub fn is_permission_denied(&self) -> bool {
        matches!(self, D1Error::PermissionDenied(_))
    }

    /// Check if this is an auth error
    pub fn is_auth_error(&self) -> bool {
        matches!(self, D1Error::Auth(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = D1Error::config("test");
        assert!(matches!(err, D1Error::Config(_)));
    }

    #[test]
    fn test_error_is_timeout() {
        let err = D1Error::Timeout;
        assert!(err.is_timeout());
    }

    #[test]
    fn test_error_is_permission_denied() {
        let err = D1Error::permission_denied("denied");
        assert!(err.is_permission_denied());
    }
}
