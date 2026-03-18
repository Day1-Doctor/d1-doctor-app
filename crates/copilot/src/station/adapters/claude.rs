use std::path::PathBuf;

use async_trait::async_trait;
use tokio::sync::mpsc;

use super::adapter_trait::{AdapterAgentEvent, AgentDescriptor, FrameworkAdapter};

/// Adapter for the Claude SDK (Anthropic's local CLI agent framework).
///
/// Discovery: checks whether `~/.claude/` exists.
/// Spawn: creates a mock agent process that emits periodic state changes.
pub struct ClaudeAdapter {
    /// Override for the Claude home directory (defaults to `~/.claude/`).
    claude_home: PathBuf,
}

impl ClaudeAdapter {
    /// Create a new adapter that probes the default `~/.claude/` directory.
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        Self {
            claude_home: home.join(".claude"),
        }
    }

    /// Create an adapter pointing at a custom directory (useful for tests).
    pub fn with_home(path: PathBuf) -> Self {
        Self { claude_home: path }
    }

    /// Spawn a mock agent that emits periodic state-change events.
    ///
    /// Returns a handle to the spawned task and a receiver for events.
    pub fn spawn(
        &self,
        agent_id: &str,
    ) -> (
        tokio::task::JoinHandle<()>,
        mpsc::Receiver<AdapterAgentEvent>,
    ) {
        let (tx, rx) = mpsc::channel(32);
        let id = agent_id.to_string();

        let handle = tokio::spawn(async move {
            let states = ["initializing", "ready", "thinking", "responding", "idle"];
            for state in &states {
                let event = AdapterAgentEvent {
                    agent_id: id.clone(),
                    event_type: "state_changed".to_string(),
                    payload: serde_json::json!({ "state": state }),
                };
                if tx.send(event).await.is_err() {
                    break;
                }
                tokio::task::yield_now().await;
            }
        });

        (handle, rx)
    }

    /// Subscribe to events from a previously spawned agent.
    ///
    /// In a real implementation this would return a channel connected to the
    /// subprocess's stdout/stderr stream. For now it returns a receiver that
    /// emits a single "subscribed" event.
    pub fn subscribe(&self, agent_id: &str) -> mpsc::Receiver<AdapterAgentEvent> {
        let (tx, rx) = mpsc::channel(8);
        let id = agent_id.to_string();

        tokio::spawn(async move {
            let _ = tx
                .send(AdapterAgentEvent {
                    agent_id: id,
                    event_type: "subscribed".to_string(),
                    payload: serde_json::json!({}),
                })
                .await;
        });

        rx
    }
}

impl Default for ClaudeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkAdapter for ClaudeAdapter {
    async fn discover(&self) -> Vec<AgentDescriptor> {
        if self.claude_home.exists() {
            vec![AgentDescriptor {
                id: "claude-sdk-default".to_string(),
                name: "Claude SDK".to_string(),
                adapter_type: "claude".to_string(),
                version: Some("1.0".to_string()),
            }]
        } else {
            vec![]
        }
    }

    async fn health_check(&self) -> bool {
        self.claude_home.exists()
    }

    fn adapter_type(&self) -> &str {
        "claude"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_discover_when_dir_exists() {
        let tmp = tempdir().unwrap();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir(&claude_dir).unwrap();

        let adapter = ClaudeAdapter::with_home(claude_dir);
        let agents = adapter.discover().await;

        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, "claude-sdk-default");
        assert_eq!(agents[0].adapter_type, "claude");
    }

    #[tokio::test]
    async fn test_discover_when_dir_missing() {
        let tmp = tempdir().unwrap();
        let claude_dir = tmp.path().join(".claude-nonexistent");

        let adapter = ClaudeAdapter::with_home(claude_dir);
        let agents = adapter.discover().await;

        assert!(agents.is_empty());
    }

    #[tokio::test]
    async fn test_spawn_returns_handle_and_events() {
        let tmp = tempdir().unwrap();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir(&claude_dir).unwrap();

        let adapter = ClaudeAdapter::with_home(claude_dir);
        let (handle, mut rx) = adapter.spawn("test-agent");

        // Collect all events.
        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        handle.await.unwrap();

        assert!(!events.is_empty());
        assert_eq!(events[0].agent_id, "test-agent");
        assert_eq!(events[0].event_type, "state_changed");

        // Should have 5 state transitions.
        assert_eq!(events.len(), 5);
    }

    #[tokio::test]
    async fn test_subscribe_receives_event() {
        let tmp = tempdir().unwrap();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir(&claude_dir).unwrap();

        let adapter = ClaudeAdapter::with_home(claude_dir);
        let mut rx = adapter.subscribe("sub-agent");

        let event = rx.recv().await.unwrap();
        assert_eq!(event.agent_id, "sub-agent");
        assert_eq!(event.event_type, "subscribed");
    }

    #[tokio::test]
    async fn test_health_check() {
        let tmp = tempdir().unwrap();
        let claude_dir = tmp.path().join(".claude");

        let adapter = ClaudeAdapter::with_home(claude_dir.clone());
        assert!(!adapter.health_check().await);

        fs::create_dir(&claude_dir).unwrap();
        assert!(adapter.health_check().await);
    }

    #[tokio::test]
    async fn test_adapter_type() {
        let adapter = ClaudeAdapter::with_home(PathBuf::from("/tmp/fake"));
        assert_eq!(adapter.adapter_type(), "claude");
    }
}
