use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A structured event emitted by an agent during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    pub id: String,
    pub agent_id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
}

/// The discriminated union of all event kinds, serialised with an
/// adjacent `"type"` / `"payload"` tag so the JSON is easy to
/// route on the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum EventType {
    #[serde(rename = "agent.state_changed")]
    AgentStateChanged { from: String, to: String },

    #[serde(rename = "token.stream")]
    TokenStream { delta: u32, total: u32 },

    #[serde(rename = "tool.started")]
    ToolStarted {
        tool_name: String,
        params: serde_json::Value,
    },

    #[serde(rename = "tool.finished")]
    ToolFinished {
        tool_name: String,
        result: serde_json::Value,
        duration_ms: u64,
    },

    #[serde(rename = "approval.requested")]
    ApprovalRequested {
        action: String,
        risk_level: String,
        context: String,
    },

    #[serde(rename = "artifact.created")]
    ArtifactCreated {
        task_id: String,
        artifact_type: String,
        path: String,
    },

    #[serde(rename = "task.step_completed")]
    TaskStepCompleted {
        task_id: String,
        step_index: u32,
        result: serde_json::Value,
    },

    #[serde(rename = "cost.updated")]
    CostUpdated {
        session_tokens: u64,
        session_cost_dd: f64,
    },
}
