use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Descriptor returned by adapter discovery, describing a detected agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDescriptor {
    /// Unique identifier for the agent (e.g. "claude-sdk-default").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// The adapter type that manages this agent (e.g. "claude", "openclaw", "generic").
    pub adapter_type: String,
    /// Optional version string.
    pub version: Option<String>,
}

/// Event emitted by a running agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterAgentEvent {
    pub agent_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
}

/// Common trait implemented by every framework adapter.
///
/// Each adapter knows how to discover, spawn, and communicate with agents
/// provided by a particular framework (Claude SDK, OpenClaw, generic
/// OpenAI-compatible endpoints, etc.).
#[async_trait]
pub trait FrameworkAdapter: Send + Sync {
    /// Probe the local environment and return descriptors for any agents
    /// that this adapter can manage.
    async fn discover(&self) -> Vec<AgentDescriptor>;

    /// Check whether the adapter's backing service is reachable.
    async fn health_check(&self) -> bool;

    /// Return the adapter type identifier (e.g. "claude", "openclaw", "generic").
    fn adapter_type(&self) -> &str;
}
