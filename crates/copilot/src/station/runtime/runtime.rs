use std::sync::Arc;

use crate::station::events::EventBus;
use crate::station::kernel::{AgentDescriptor, AgentKernel, Framework};

use super::presets::{self, AgentPreset};

/// Manages the built-in Dr. Bob's Office runtime: 6 preset agents registered
/// with the kernel and connected to the Day1 Doctor gateway for LLM calls.
pub struct BuiltinRuntime {
    kernel: Arc<AgentKernel>,
    event_bus: Arc<EventBus>,
    gateway_url: String,
    presets: Vec<AgentPreset>,
}

impl BuiltinRuntime {
    /// Create a new runtime bound to an existing kernel and event bus.
    pub fn new(kernel: Arc<AgentKernel>, event_bus: Arc<EventBus>, gateway_url: &str) -> Self {
        Self {
            kernel,
            event_bus,
            gateway_url: gateway_url.to_string(),
            presets: presets::builtin_presets(),
        }
    }

    /// Initialize Dr. Bob's Office -- register all 6 preset agents with the kernel.
    ///
    /// Returns a list of the newly-assigned agent IDs.
    pub async fn initialize(&self) -> Result<Vec<String>, String> {
        let mut agent_ids = Vec::new();

        for preset in &self.presets {
            let mut agent =
                AgentDescriptor::new(preset.name, preset.role, Framework::Builtin, preset.default_model);
            agent.room = preset.room.to_string();

            let id = self.kernel.register(agent).await;
            agent_ids.push(id);
        }

        Ok(agent_ids)
    }

    /// Get the gateway URL for LLM calls.
    pub fn gateway_url(&self) -> &str {
        &self.gateway_url
    }

    /// Get the shared event bus.
    pub fn event_bus(&self) -> &Arc<EventBus> {
        &self.event_bus
    }

    /// Look up an agent preset by its `code_name`.
    pub fn get_preset(&self, code_name: &str) -> Option<&AgentPreset> {
        self.presets.iter().find(|p| p.code_name == code_name)
    }

    /// Check whether a given MCP tool server name is allowed for the agent
    /// identified by `code_name`.
    pub fn is_tool_allowed(&self, code_name: &str, tool_name: &str) -> bool {
        self.presets
            .iter()
            .find(|p| p.code_name == code_name)
            .map(|p| p.tools.contains(&tool_name))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_runtime() -> BuiltinRuntime {
        let kernel = Arc::new(AgentKernel::new());
        let event_bus = Arc::new(EventBus::new(128));
        BuiltinRuntime::new(kernel, event_bus, "https://gateway.day1.doctor/v1")
    }

    #[tokio::test]
    async fn test_runtime_initialize() {
        let rt = make_runtime();
        let ids = rt.initialize().await.unwrap();

        assert_eq!(ids.len(), 6, "should register 6 agents");
        assert_eq!(rt.kernel.agent_count().await, 6);

        // Each ID should be unique.
        let unique: std::collections::HashSet<&String> = ids.iter().collect();
        assert_eq!(unique.len(), 6);
    }

    #[tokio::test]
    async fn test_runtime_agent_rooms() {
        let rt = make_runtime();
        let _ids = rt.initialize().await.unwrap();

        // Find Dr. Bob and verify room.
        let agents = rt.kernel.list_agents().await;
        let dr_bob = agents.iter().find(|a| a.name == "Dr. Bob").unwrap();
        assert_eq!(dr_bob.room, "main");

        let scout = agents.iter().find(|a| a.name == "Scout").unwrap();
        assert_eq!(scout.room, "research");

        let atlas = agents.iter().find(|a| a.name == "Atlas").unwrap();
        assert_eq!(atlas.room, "operations");
    }

    #[test]
    fn test_gateway_url() {
        let rt = make_runtime();
        assert_eq!(rt.gateway_url(), "https://gateway.day1.doctor/v1");
    }

    #[test]
    fn test_get_preset() {
        let rt = make_runtime();

        let orchestrator = rt.get_preset("orchestrator").unwrap();
        assert_eq!(orchestrator.name, "Dr. Bob");

        let researcher = rt.get_preset("researcher").unwrap();
        assert_eq!(researcher.name, "Scout");

        assert!(rt.get_preset("nonexistent").is_none());
    }

    #[test]
    fn test_tool_allowed() {
        let rt = make_runtime();

        // Dr. Bob: memory + filesystem only
        assert!(rt.is_tool_allowed("orchestrator", "memory"));
        assert!(rt.is_tool_allowed("orchestrator", "filesystem"));
        assert!(!rt.is_tool_allowed("orchestrator", "shell"));
        assert!(!rt.is_tool_allowed("orchestrator", "web-search"));

        // Scout: web-search, web-fetch, memory, filesystem
        assert!(rt.is_tool_allowed("researcher", "web-search"));
        assert!(rt.is_tool_allowed("researcher", "web-fetch"));
        assert!(!rt.is_tool_allowed("researcher", "shell"));

        // Pixel (coder): shell, filesystem, system, memory
        assert!(rt.is_tool_allowed("coder", "shell"));
        assert!(rt.is_tool_allowed("coder", "system"));
        assert!(!rt.is_tool_allowed("coder", "browser"));

        // Atlas (operator): browser, shell, filesystem, system, clipboard
        assert!(rt.is_tool_allowed("operator", "browser"));
        assert!(rt.is_tool_allowed("operator", "clipboard"));

        // Unknown agent
        assert!(!rt.is_tool_allowed("unknown", "memory"));
    }

    #[tokio::test]
    async fn test_runtime_idempotent_initialize() {
        let rt = make_runtime();

        let ids1 = rt.initialize().await.unwrap();
        let ids2 = rt.initialize().await.unwrap();

        // Each call registers new agents (IDs differ).
        assert_eq!(ids1.len(), 6);
        assert_eq!(ids2.len(), 6);
        assert_eq!(rt.kernel.agent_count().await, 12);

        // All IDs unique across both calls.
        let all: std::collections::HashSet<String> = ids1.into_iter().chain(ids2).collect();
        assert_eq!(all.len(), 12);
    }
}
