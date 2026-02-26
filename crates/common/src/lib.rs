//! Common types and utilities for Day 1 Doctor
//!
//! This crate provides shared types, configuration, error handling,
//! and protocol definitions used across all d1-doctor components.

pub mod config;
pub mod errors;
pub mod proto;

pub use config::Config;
pub use errors::{D1Error, Result};
pub use proto::*;

/// Current version of the d1-doctor protocol
pub const PROTOCOL_VERSION: u32 = 1;

/// Maximum size of a protobuf message (100 MB)
pub const MAX_MESSAGE_SIZE: usize = 100 * 1024 * 1024;

/// Default port for the local daemon
pub const DEFAULT_DAEMON_PORT: u16 = 9876;

/// Default config directory
pub fn config_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .map(|home| home.join(".d1doctor"))
        .unwrap_or_else(|| std::path::PathBuf::from(".d1doctor"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir() {
        let dir = config_dir();
        assert!(dir.ends_with(".d1doctor"));
    }
}
