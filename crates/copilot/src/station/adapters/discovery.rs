use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Type of agent runtime detected on the system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdapterKind {
    Claude,
    OpenClaw,
    Ollama,
}

/// Information about a detected agent runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInfo {
    /// Human-readable name (e.g. "Claude SDK").
    pub name: String,
    /// Filesystem path where the runtime was found (if applicable).
    pub path: Option<PathBuf>,
    /// Which adapter type handles this runtime.
    pub adapter_type: AdapterKind,
}

/// Scans the local filesystem for installed agent runtimes.
///
/// Detected runtimes:
/// - `~/.claude/`     -> Claude SDK
/// - `~/.openclaw/`   -> OpenClaw
/// - localhost:11434   -> Ollama (network check skipped when `skip_network` is true)
///
/// The `home_dir` parameter allows tests to inject a fake home directory.
pub async fn discover_runtimes(home_dir: &PathBuf, skip_network: bool) -> Vec<RuntimeInfo> {
    let mut runtimes = Vec::new();

    // Claude SDK
    let claude_path = home_dir.join(".claude");
    if claude_path.exists() {
        runtimes.push(RuntimeInfo {
            name: "Claude SDK".to_string(),
            path: Some(claude_path),
            adapter_type: AdapterKind::Claude,
        });
    }

    // OpenClaw
    let openclaw_path = home_dir.join(".openclaw");
    if openclaw_path.exists() {
        runtimes.push(RuntimeInfo {
            name: "OpenClaw".to_string(),
            path: Some(openclaw_path),
            adapter_type: AdapterKind::OpenClaw,
        });
    }

    // Ollama (network probe)
    if !skip_network {
        if probe_ollama().await {
            runtimes.push(RuntimeInfo {
                name: "Ollama".to_string(),
                path: None,
                adapter_type: AdapterKind::Ollama,
            });
        }
    }

    runtimes
}

/// Probe localhost:11434 to see if Ollama is running.
async fn probe_ollama() -> bool {
    match tokio::time::timeout(
        std::time::Duration::from_millis(500),
        tokio::net::TcpStream::connect("127.0.0.1:11434"),
    )
    .await
    {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_discover_claude_only() {
        let tmp = tempdir().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir(home.join(".claude")).unwrap();

        let runtimes = discover_runtimes(&home, true).await;

        assert_eq!(runtimes.len(), 1);
        assert_eq!(runtimes[0].name, "Claude SDK");
        assert_eq!(runtimes[0].adapter_type, AdapterKind::Claude);
        assert!(runtimes[0].path.is_some());
    }

    #[tokio::test]
    async fn test_discover_openclaw_only() {
        let tmp = tempdir().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir(home.join(".openclaw")).unwrap();

        let runtimes = discover_runtimes(&home, true).await;

        assert_eq!(runtimes.len(), 1);
        assert_eq!(runtimes[0].name, "OpenClaw");
        assert_eq!(runtimes[0].adapter_type, AdapterKind::OpenClaw);
    }

    #[tokio::test]
    async fn test_discover_both() {
        let tmp = tempdir().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir(home.join(".claude")).unwrap();
        fs::create_dir(home.join(".openclaw")).unwrap();

        let runtimes = discover_runtimes(&home, true).await;

        assert_eq!(runtimes.len(), 2);
        let names: Vec<&str> = runtimes.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"Claude SDK"));
        assert!(names.contains(&"OpenClaw"));
    }

    #[tokio::test]
    async fn test_discover_nothing() {
        let tmp = tempdir().unwrap();
        let home = tmp.path().to_path_buf();

        let runtimes = discover_runtimes(&home, true).await;
        assert!(runtimes.is_empty());
    }

    #[tokio::test]
    async fn test_skip_network_skips_ollama() {
        let tmp = tempdir().unwrap();
        let home = tmp.path().to_path_buf();

        // Even if Ollama were running, skip_network=true should not probe.
        let runtimes = discover_runtimes(&home, true).await;
        let ollama = runtimes
            .iter()
            .find(|r| r.adapter_type == AdapterKind::Ollama);
        assert!(ollama.is_none());
    }
}
