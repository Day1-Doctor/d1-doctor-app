use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::agent_state::{AgentStatus, Trigger};

/// Descriptor for an individual agent managed by the kernel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDescriptor {
    pub id: String,
    pub name: String,
    pub role: AgentRole,
    pub framework: Framework,
    pub default_model: String,
    pub status: AgentStatus,
    pub trust_score: f64,
    pub sprite_id: Option<String>,
    pub room: String,
    pub current_task_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// The role an agent plays within the multi-agent system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    Orchestrator,
    Researcher,
    Analyst,
    Writer,
    Coder,
    Operator,
}

/// The runtime framework backing the agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Framework {
    Builtin,
    ClaudeSdk,
    OpenClaw,
    IronClaw,
    Generic,
}

impl AgentDescriptor {
    /// Create a new agent with sensible defaults.
    pub fn new(name: &str, role: AgentRole, framework: Framework, default_model: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            role,
            framework,
            default_model: default_model.to_string(),
            status: AgentStatus::Idle,
            trust_score: 0.5,
            sprite_id: None,
            room: "main".to_string(),
            current_task_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Apply a trigger to this agent's FSM, returning the new status on success.
    pub fn apply_trigger(&mut self, trigger: &Trigger) -> Result<AgentStatus, String> {
        let new_status = self.status.transition(trigger)?;
        self.status = new_status;
        self.updated_at = Utc::now();
        Ok(new_status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_agent_defaults() {
        let agent = AgentDescriptor::new(
            "researcher-1",
            AgentRole::Researcher,
            Framework::Builtin,
            "claude-haiku-4-5",
        );
        assert_eq!(agent.name, "researcher-1");
        assert_eq!(agent.role, AgentRole::Researcher);
        assert_eq!(agent.framework, Framework::Builtin);
        assert_eq!(agent.default_model, "claude-haiku-4-5");
        assert_eq!(agent.status, AgentStatus::Idle);
        assert!((agent.trust_score - 0.5).abs() < f64::EPSILON);
        assert_eq!(agent.room, "main");
        assert!(agent.current_task_id.is_none());
        assert!(agent.sprite_id.is_none());
        assert!(!agent.id.is_empty());
    }

    #[test]
    fn test_apply_trigger_success() {
        let mut agent = AgentDescriptor::new("coder-1", AgentRole::Coder, Framework::ClaudeSdk, "claude-sonnet-4");
        assert_eq!(agent.status, AgentStatus::Idle);

        let new = agent.apply_trigger(&Trigger::TaskAssign).unwrap();
        assert_eq!(new, AgentStatus::Working);
        assert_eq!(agent.status, AgentStatus::Working);
    }

    #[test]
    fn test_apply_trigger_invalid() {
        let mut agent = AgentDescriptor::new("writer-1", AgentRole::Writer, Framework::Generic, "claude-sonnet-4");
        let result = agent.apply_trigger(&Trigger::LlmCallStart);
        assert!(result.is_err());
        // Status should remain unchanged on failure.
        assert_eq!(agent.status, AgentStatus::Idle);
    }

    #[test]
    fn test_full_lifecycle() {
        let mut agent = AgentDescriptor::new("analyst-1", AgentRole::Analyst, Framework::OpenClaw, "claude-sonnet-4");

        // idle -> working
        agent.apply_trigger(&Trigger::TaskAssign).unwrap();
        assert_eq!(agent.status, AgentStatus::Working);

        // working -> thinking
        agent.apply_trigger(&Trigger::LlmCallStart).unwrap();
        assert_eq!(agent.status, AgentStatus::Thinking);

        // thinking -> executing
        agent.apply_trigger(&Trigger::ToolCallStart).unwrap();
        assert_eq!(agent.status, AgentStatus::Executing);

        // executing -> working
        agent.apply_trigger(&Trigger::ToolCallEnd).unwrap();
        assert_eq!(agent.status, AgentStatus::Working);

        // working -> idle
        agent.apply_trigger(&Trigger::TaskComplete).unwrap();
        assert_eq!(agent.status, AgentStatus::Idle);
    }

    #[test]
    fn test_serde_roundtrip() {
        let agent = AgentDescriptor::new("operator-1", AgentRole::Operator, Framework::IronClaw, "claude-haiku-4-5");
        let json = serde_json::to_string(&agent).unwrap();
        let back: AgentDescriptor = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, agent.id);
        assert_eq!(back.name, agent.name);
        assert_eq!(back.role, agent.role);
        assert_eq!(back.framework, agent.framework);
        assert_eq!(back.status, agent.status);
    }

    #[test]
    fn test_agent_role_serde() {
        let role = AgentRole::Orchestrator;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"orchestrator\"");
    }

    #[test]
    fn test_framework_serde() {
        let fw = Framework::ClaudeSdk;
        let json = serde_json::to_string(&fw).unwrap();
        assert_eq!(json, "\"claude_sdk\"");
    }
}
