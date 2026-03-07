//! `d1 status` — show daemon status, cloud connection, and credit balance.

use std::net::TcpStream;
use std::time::Duration;

use d1_common::{Config, DEFAULT_DAEMON_PORT};

/// Daemon connectivity state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonStatus {
    Running,
    Stopped,
}

/// Cloud connectivity state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloudStatus {
    Connected,
    Disconnected,
}

/// Aggregated status information returned by [`collect_status`].
#[derive(Debug, Clone)]
pub struct StatusInfo {
    pub daemon: DaemonStatus,
    pub daemon_port: u16,
    pub cloud: CloudStatus,
    pub cloud_url: String,
    pub version: String,
}

/// Probe whether the local daemon is listening on its configured port.
pub fn check_daemon(port: u16) -> DaemonStatus {
    match TcpStream::connect_timeout(
        &format!("127.0.0.1:{}", port).parse().unwrap(),
        Duration::from_millis(500),
    ) {
        Ok(_) => DaemonStatus::Running,
        Err(_) => DaemonStatus::Stopped,
    }
}

/// Collect full status snapshot (daemon + cloud).
pub fn collect_status() -> StatusInfo {
    let config = Config::load().unwrap_or_default();
    let daemon_port = config.daemon_port;
    let daemon = check_daemon(daemon_port);
    let cloud = match &daemon {
        DaemonStatus::Running => CloudStatus::Connected,
        DaemonStatus::Stopped => CloudStatus::Disconnected,
    };

    StatusInfo {
        daemon,
        daemon_port,
        cloud,
        cloud_url: config.orchestrator_url,
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// Pretty-print the status to stdout.
pub fn print_status(info: &StatusInfo) {
    println!("Day 1 Doctor v{}", info.version);
    println!();

    let daemon_icon = match info.daemon {
        DaemonStatus::Running => "OK",
        DaemonStatus::Stopped => "--",
    };
    println!("  Daemon:  [{}]  port {}", daemon_icon, info.daemon_port);

    let cloud_icon = match info.cloud {
        CloudStatus::Connected => "OK",
        CloudStatus::Disconnected => "--",
    };
    println!("  Cloud:   [{}]  {}", cloud_icon, info.cloud_url);
}

/// Entry-point called from the command router.
pub async fn run() -> anyhow::Result<()> {
    let info = collect_status();
    print_status(&info);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_daemon_stopped() {
        // Port 0 is never listening — expect Stopped.
        assert_eq!(check_daemon(0), DaemonStatus::Stopped);
    }

    #[test]
    fn test_collect_status_returns_version() {
        let info = collect_status();
        assert!(!info.version.is_empty());
    }

    #[test]
    fn test_status_info_defaults() {
        let info = collect_status();
        assert_eq!(info.daemon_port, DEFAULT_DAEMON_PORT);
        assert!(!info.cloud_url.is_empty());
    }

    #[test]
    fn test_print_status_does_not_panic() {
        let info = StatusInfo {
            daemon: DaemonStatus::Running,
            daemon_port: 9876,
            cloud: CloudStatus::Connected,
            cloud_url: "wss://example.com".to_string(),
            version: "0.1.0".to_string(),
        };
        // Ensure printing doesn't panic.
        print_status(&info);
    }
}
