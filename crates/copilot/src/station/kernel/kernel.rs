use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::agent::{AgentDescriptor, AgentRole};
use super::agent_state::{AgentStatus, Trigger};

/// The Agent Kernel manages multiple agents and their state machines.
pub struct AgentKernel {
    agents: Arc<RwLock<HashMap<String, AgentDescriptor>>>,
}

impl AgentKernel {
    /// Create a new, empty kernel.
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new agent and return its ID.
    pub async fn register(&self, agent: AgentDescriptor) -> String {
        let id = agent.id.clone();
        let mut agents = self.agents.write().await;
        agents.insert(id.clone(), agent);
        id
    }

    /// Deregister an agent by ID, returning the removed descriptor.
    pub async fn deregister(&self, agent_id: &str) -> Result<AgentDescriptor, String> {
        let mut agents = self.agents.write().await;
        agents
            .remove(agent_id)
            .ok_or_else(|| format!("agent not found: {}", agent_id))
    }

    /// Apply a trigger to a specific agent's FSM.
    /// Returns `(old_status, new_status)` on success.
    pub async fn apply_trigger(
        &self,
        agent_id: &str,
        trigger: Trigger,
    ) -> Result<(AgentStatus, AgentStatus), String> {
        let mut agents = self.agents.write().await;
        let agent = agents
            .get_mut(agent_id)
            .ok_or_else(|| format!("agent not found: {}", agent_id))?;
        let old = agent.status;
        let new = agent.apply_trigger(&trigger)?;
        Ok((old, new))
    }

    /// Get a clone of an agent descriptor by ID.
    pub async fn get_agent(&self, agent_id: &str) -> Option<AgentDescriptor> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// List all registered agents.
    pub async fn list_agents(&self) -> Vec<AgentDescriptor> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }

    /// Get agents filtered by role.
    pub async fn agents_by_role(&self, role: AgentRole) -> Vec<AgentDescriptor> {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|a| a.role == role)
            .cloned()
            .collect()
    }

    /// Get agents filtered by status.
    pub async fn agents_by_status(&self, status: AgentStatus) -> Vec<AgentDescriptor> {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|a| a.status == status)
            .cloned()
            .collect()
    }

    /// Get the count of registered agents.
    pub async fn agent_count(&self) -> usize {
        let agents = self.agents.read().await;
        agents.len()
    }

    /// Assign a task to an agent by setting its `current_task_id`.
    pub async fn assign_task(&self, agent_id: &str, task_id: &str) -> Result<(), String> {
        let mut agents = self.agents.write().await;
        let agent = agents
            .get_mut(agent_id)
            .ok_or_else(|| format!("agent not found: {}", agent_id))?;
        agent.current_task_id = Some(task_id.to_string());
        agent.updated_at = chrono::Utc::now();
        Ok(())
    }

    /// Clear the agent's current task assignment.
    pub async fn clear_task(&self, agent_id: &str) -> Result<(), String> {
        let mut agents = self.agents.write().await;
        let agent = agents
            .get_mut(agent_id)
            .ok_or_else(|| format!("agent not found: {}", agent_id))?;
        agent.current_task_id = None;
        agent.updated_at = chrono::Utc::now();
        Ok(())
    }
}

impl Default for AgentKernel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::station::kernel::agent::Framework;

    #[tokio::test]
    async fn test_kernel_register_deregister() {
        let kernel = AgentKernel::new();
        let agent = AgentDescriptor::new("test-agent", AgentRole::Coder, Framework::Builtin, "claude-sonnet-4");
        let id = agent.id.clone();

        let returned_id = kernel.register(agent).await;
        assert_eq!(returned_id, id);

        // Verify it exists.
        let fetched = kernel.get_agent(&id).await;
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "test-agent");

        // Deregister.
        let removed = kernel.deregister(&id).await.unwrap();
        assert_eq!(removed.id, id);

        // Verify it's gone.
        assert!(kernel.get_agent(&id).await.is_none());
    }

    #[tokio::test]
    async fn test_kernel_deregister_not_found() {
        let kernel = AgentKernel::new();
        let result = kernel.deregister("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_kernel_apply_trigger() {
        let kernel = AgentKernel::new();
        let agent = AgentDescriptor::new("worker", AgentRole::Writer, Framework::Generic, "claude-sonnet-4");
        let id = kernel.register(agent).await;

        let (old, new) = kernel
            .apply_trigger(&id, Trigger::TaskAssign)
            .await
            .unwrap();
        assert_eq!(old, AgentStatus::Idle);
        assert_eq!(new, AgentStatus::Working);

        // Verify persisted.
        let a = kernel.get_agent(&id).await.unwrap();
        assert_eq!(a.status, AgentStatus::Working);
    }

    #[tokio::test]
    async fn test_kernel_apply_trigger_not_found() {
        let kernel = AgentKernel::new();
        let result = kernel.apply_trigger("nope", Trigger::TaskAssign).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_kernel_apply_trigger_invalid_transition() {
        let kernel = AgentKernel::new();
        let agent = AgentDescriptor::new("idle-agent", AgentRole::Analyst, Framework::Builtin, "claude-sonnet-4");
        let id = kernel.register(agent).await;

        let result = kernel.apply_trigger(&id, Trigger::LlmCallStart).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_kernel_multiple_agents() {
        let kernel = AgentKernel::new();
        let a1 = AgentDescriptor::new("agent-1", AgentRole::Coder, Framework::ClaudeSdk, "claude-sonnet-4");
        let a2 = AgentDescriptor::new("agent-2", AgentRole::Writer, Framework::Generic, "claude-sonnet-4");
        let a3 = AgentDescriptor::new("agent-3", AgentRole::Researcher, Framework::OpenClaw, "claude-haiku-4-5");

        let id1 = kernel.register(a1).await;
        let id2 = kernel.register(a2).await;
        let id3 = kernel.register(a3).await;

        assert_eq!(kernel.agent_count().await, 3);

        // Apply different triggers to different agents.
        kernel
            .apply_trigger(&id1, Trigger::TaskAssign)
            .await
            .unwrap();
        kernel
            .apply_trigger(&id2, Trigger::TaskAssign)
            .await
            .unwrap();
        // Agent 3 stays idle.

        // Advance agent 1 further.
        kernel
            .apply_trigger(&id1, Trigger::LlmCallStart)
            .await
            .unwrap();

        let a1 = kernel.get_agent(&id1).await.unwrap();
        let a2 = kernel.get_agent(&id2).await.unwrap();
        let a3 = kernel.get_agent(&id3).await.unwrap();

        assert_eq!(a1.status, AgentStatus::Thinking);
        assert_eq!(a2.status, AgentStatus::Working);
        assert_eq!(a3.status, AgentStatus::Idle);
    }

    #[tokio::test]
    async fn test_kernel_agents_by_role() {
        let kernel = AgentKernel::new();
        kernel
            .register(AgentDescriptor::new(
                "c1",
                AgentRole::Coder,
                Framework::Builtin,
                "claude-sonnet-4",
            ))
            .await;
        kernel
            .register(AgentDescriptor::new(
                "c2",
                AgentRole::Coder,
                Framework::Generic,
                "claude-sonnet-4",
            ))
            .await;
        kernel
            .register(AgentDescriptor::new(
                "w1",
                AgentRole::Writer,
                Framework::Builtin,
                "claude-sonnet-4",
            ))
            .await;

        let coders = kernel.agents_by_role(AgentRole::Coder).await;
        assert_eq!(coders.len(), 2);
        for c in &coders {
            assert_eq!(c.role, AgentRole::Coder);
        }

        let writers = kernel.agents_by_role(AgentRole::Writer).await;
        assert_eq!(writers.len(), 1);
    }

    #[tokio::test]
    async fn test_kernel_agents_by_status() {
        let kernel = AgentKernel::new();
        let a1 = AgentDescriptor::new("a1", AgentRole::Coder, Framework::Builtin, "claude-sonnet-4");
        let a2 = AgentDescriptor::new("a2", AgentRole::Writer, Framework::Generic, "claude-sonnet-4");
        let id1 = kernel.register(a1).await;
        kernel.register(a2).await;

        // Move a1 to Working.
        kernel
            .apply_trigger(&id1, Trigger::TaskAssign)
            .await
            .unwrap();

        let idle = kernel.agents_by_status(AgentStatus::Idle).await;
        assert_eq!(idle.len(), 1);

        let working = kernel.agents_by_status(AgentStatus::Working).await;
        assert_eq!(working.len(), 1);
        assert_eq!(working[0].id, id1);
    }

    #[tokio::test]
    async fn test_kernel_assign_and_clear_task() {
        let kernel = AgentKernel::new();
        let agent = AgentDescriptor::new("tasked", AgentRole::Operator, Framework::Builtin, "claude-haiku-4-5");
        let id = kernel.register(agent).await;

        kernel.assign_task(&id, "task-42").await.unwrap();
        let a = kernel.get_agent(&id).await.unwrap();
        assert_eq!(a.current_task_id.as_deref(), Some("task-42"));

        kernel.clear_task(&id).await.unwrap();
        let a = kernel.get_agent(&id).await.unwrap();
        assert!(a.current_task_id.is_none());
    }

    #[tokio::test]
    async fn test_kernel_assign_task_not_found() {
        let kernel = AgentKernel::new();
        let result = kernel.assign_task("nope", "task-1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_kernel_list_agents() {
        let kernel = AgentKernel::new();
        assert!(kernel.list_agents().await.is_empty());

        kernel
            .register(AgentDescriptor::new(
                "a",
                AgentRole::Coder,
                Framework::Builtin,
                "claude-sonnet-4",
            ))
            .await;
        kernel
            .register(AgentDescriptor::new(
                "b",
                AgentRole::Writer,
                Framework::Generic,
                "claude-sonnet-4",
            ))
            .await;

        let all = kernel.list_agents().await;
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_kernel_concurrent_access() {
        let kernel = Arc::new(AgentKernel::new());
        let a1 = AgentDescriptor::new("concurrent-1", AgentRole::Coder, Framework::Builtin, "claude-sonnet-4");
        let a2 = AgentDescriptor::new("concurrent-2", AgentRole::Writer, Framework::Generic, "claude-sonnet-4");
        let id1 = kernel.register(a1).await;
        let id2 = kernel.register(a2).await;

        let k1 = Arc::clone(&kernel);
        let k2 = Arc::clone(&kernel);
        let id1_clone = id1.clone();
        let id2_clone = id2.clone();

        let h1 = tokio::spawn(async move {
            k1.apply_trigger(&id1_clone, Trigger::TaskAssign)
                .await
                .unwrap()
        });
        let h2 = tokio::spawn(async move {
            k2.apply_trigger(&id2_clone, Trigger::TaskAssign)
                .await
                .unwrap()
        });

        let (r1, r2) = tokio::join!(h1, h2);
        let (old1, new1) = r1.unwrap();
        let (old2, new2) = r2.unwrap();

        assert_eq!(old1, AgentStatus::Idle);
        assert_eq!(new1, AgentStatus::Working);
        assert_eq!(old2, AgentStatus::Idle);
        assert_eq!(new2, AgentStatus::Working);

        // Verify both agents are in the expected state.
        let a1 = kernel.get_agent(&id1).await.unwrap();
        let a2 = kernel.get_agent(&id2).await.unwrap();
        assert_eq!(a1.status, AgentStatus::Working);
        assert_eq!(a2.status, AgentStatus::Working);
    }
}
