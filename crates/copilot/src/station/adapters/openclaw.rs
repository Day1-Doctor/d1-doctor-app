use std::path::PathBuf;

use async_trait::async_trait;

use super::adapter_trait::{AgentDescriptor, FrameworkAdapter};

/// Default port where the OpenClaw gateway listens locally.
pub const OPENCLAW_DEFAULT_PORT: u16 = 18789;

/// Adapter for the OpenClaw agent framework.
///
/// Discovery: checks for `~/.openclaw/` directory and whether the local
/// gateway is reachable on `localhost:18789`.
///
/// Currently a stub — real implementation will open a WebSocket connection
/// to the gateway and proxy agent events.
pub struct OpenClawAdapter {
    /// Override path for the OpenClaw home directory.
    openclaw_home: PathBuf,
    /// Gateway endpoint (host:port).
    gateway_addr: String,
}

impl OpenClawAdapter {
    /// Create an adapter probing the default locations.
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        Self {
            openclaw_home: home.join(".openclaw"),
            gateway_addr: format!("127.0.0.1:{}", OPENCLAW_DEFAULT_PORT),
        }
    }

    /// Create an adapter with custom home directory and gateway address.
    pub fn with_config(home: PathBuf, gateway_addr: String) -> Self {
        Self {
            openclaw_home: home,
            gateway_addr,
        }
    }

    /// Attempt a TCP connection to the gateway to see if it is running.
    /// Returns `true` if the connection succeeds within 500 ms.
    async fn probe_gateway(&self) -> bool {
        match tokio::time::timeout(
            std::time::Duration::from_millis(500),
            tokio::net::TcpStream::connect(&self.gateway_addr),
        )
        .await
        {
            Ok(Ok(_)) => true,
            _ => false,
        }
    }
}

impl Default for OpenClawAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkAdapter for OpenClawAdapter {
    async fn discover(&self) -> Vec<AgentDescriptor> {
        // Must have the home directory present.
        if !self.openclaw_home.exists() {
            return vec![];
        }

        vec![AgentDescriptor {
            id: "openclaw-gateway".to_string(),
            name: "OpenClaw Gateway".to_string(),
            adapter_type: "openclaw".to_string(),
            version: None,
        }]
    }

    async fn health_check(&self) -> bool {
        self.openclaw_home.exists() && self.probe_gateway().await
    }

    fn adapter_type(&self) -> &str {
        "openclaw"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_discover_with_home_dir() {
        let tmp = tempdir().unwrap();
        let oc_dir = tmp.path().join(".openclaw");
        fs::create_dir(&oc_dir).unwrap();

        let adapter = OpenClawAdapter::with_config(oc_dir, "127.0.0.1:0".to_string());
        let agents = adapter.discover().await;

        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, "openclaw-gateway");
        assert_eq!(agents[0].adapter_type, "openclaw");
    }

    #[tokio::test]
    async fn test_discover_without_home_dir() {
        let tmp = tempdir().unwrap();
        let oc_dir = tmp.path().join(".openclaw-missing");

        let adapter = OpenClawAdapter::with_config(oc_dir, "127.0.0.1:0".to_string());
        let agents = adapter.discover().await;
        assert!(agents.is_empty());
    }

    #[tokio::test]
    async fn test_health_check_no_gateway() {
        // Point at a port that is almost certainly not listening.
        let tmp = tempdir().unwrap();
        let oc_dir = tmp.path().join(".openclaw");
        fs::create_dir(&oc_dir).unwrap();

        let adapter =
            OpenClawAdapter::with_config(oc_dir, "127.0.0.1:19999".to_string());
        // health_check should fail because the gateway isn't running.
        assert!(!adapter.health_check().await);
    }

    #[tokio::test]
    async fn test_health_check_no_home_dir() {
        let tmp = tempdir().unwrap();
        let oc_dir = tmp.path().join(".openclaw-missing");

        let adapter = OpenClawAdapter::with_config(oc_dir, "127.0.0.1:0".to_string());
        assert!(!adapter.health_check().await);
    }

    #[tokio::test]
    async fn test_adapter_type() {
        let adapter = OpenClawAdapter::with_config(
            PathBuf::from("/tmp/fake"),
            "127.0.0.1:0".to_string(),
        );
        assert_eq!(adapter.adapter_type(), "openclaw");
    }
}
