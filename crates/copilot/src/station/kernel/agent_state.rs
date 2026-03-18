use serde::{Deserialize, Serialize};

/// The finite set of agent states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Working,
    Thinking,
    Executing,
    Paused,
    Error,
}

/// Triggers that cause state transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trigger {
    TaskAssign,
    LlmCallStart,
    LlmCallEnd,
    ToolCallStart,
    ToolCallEnd,
    ApprovalNeeded,
    ApprovalGranted,
    ApprovalRejected,
    TaskComplete,
    TaskCancel,
    ErrorOccurred { message: String },
    Resume,
}

impl AgentStatus {
    /// Returns the valid transitions from this state as defined in PRD section 7.2.
    pub fn valid_transitions(&self) -> Vec<(TriggerKind, AgentStatus)> {
        match self {
            AgentStatus::Idle => vec![(TriggerKind::TaskAssign, AgentStatus::Working)],
            AgentStatus::Working => vec![
                (TriggerKind::LlmCallStart, AgentStatus::Thinking),
                (TriggerKind::ApprovalNeeded, AgentStatus::Paused),
                (TriggerKind::TaskComplete, AgentStatus::Idle),
                (TriggerKind::TaskCancel, AgentStatus::Idle),
                (TriggerKind::ErrorOccurred, AgentStatus::Error),
            ],
            AgentStatus::Thinking => vec![
                (TriggerKind::ToolCallStart, AgentStatus::Executing),
                (TriggerKind::LlmCallEnd, AgentStatus::Working),
                (TriggerKind::ErrorOccurred, AgentStatus::Error),
            ],
            AgentStatus::Executing => vec![
                (TriggerKind::ToolCallEnd, AgentStatus::Working),
                (TriggerKind::ErrorOccurred, AgentStatus::Error),
            ],
            AgentStatus::Paused => vec![
                (TriggerKind::ApprovalGranted, AgentStatus::Working),
                (TriggerKind::ApprovalRejected, AgentStatus::Error),
                (TriggerKind::Resume, AgentStatus::Idle),
                (TriggerKind::ErrorOccurred, AgentStatus::Error),
            ],
            AgentStatus::Error => vec![
                (TriggerKind::Resume, AgentStatus::Idle),
            ],
        }
    }

    /// Attempt a transition; returns the new state or an error describing why
    /// the transition is invalid.
    pub fn transition(&self, trigger: &Trigger) -> Result<AgentStatus, String> {
        let kind = TriggerKind::from(trigger);
        for (valid_kind, target) in self.valid_transitions() {
            if valid_kind == kind {
                return Ok(target);
            }
        }
        Err(format!(
            "invalid transition: {:?} + {:?} is not allowed",
            self, trigger
        ))
    }

    /// Human-readable name for the status.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Working => "working",
            Self::Thinking => "thinking",
            Self::Executing => "executing",
            Self::Paused => "paused",
            Self::Error => "error",
        }
    }
}

/// A discriminant-only mirror of [`Trigger`] used for matching transitions
/// without requiring the payload data (e.g. the error message).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerKind {
    TaskAssign,
    LlmCallStart,
    LlmCallEnd,
    ToolCallStart,
    ToolCallEnd,
    ApprovalNeeded,
    ApprovalGranted,
    ApprovalRejected,
    TaskComplete,
    TaskCancel,
    ErrorOccurred,
    Resume,
}

impl From<&Trigger> for TriggerKind {
    fn from(trigger: &Trigger) -> Self {
        match trigger {
            Trigger::TaskAssign => TriggerKind::TaskAssign,
            Trigger::LlmCallStart => TriggerKind::LlmCallStart,
            Trigger::LlmCallEnd => TriggerKind::LlmCallEnd,
            Trigger::ToolCallStart => TriggerKind::ToolCallStart,
            Trigger::ToolCallEnd => TriggerKind::ToolCallEnd,
            Trigger::ApprovalNeeded => TriggerKind::ApprovalNeeded,
            Trigger::ApprovalGranted => TriggerKind::ApprovalGranted,
            Trigger::ApprovalRejected => TriggerKind::ApprovalRejected,
            Trigger::TaskComplete => TriggerKind::TaskComplete,
            Trigger::TaskCancel => TriggerKind::TaskCancel,
            Trigger::ErrorOccurred { .. } => TriggerKind::ErrorOccurred,
            Trigger::Resume => TriggerKind::Resume,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_to_working() {
        let result = AgentStatus::Idle.transition(&Trigger::TaskAssign);
        assert_eq!(result.unwrap(), AgentStatus::Working);
    }

    #[test]
    fn test_working_to_thinking() {
        let result = AgentStatus::Working.transition(&Trigger::LlmCallStart);
        assert_eq!(result.unwrap(), AgentStatus::Thinking);
    }

    #[test]
    fn test_thinking_to_executing() {
        let result = AgentStatus::Thinking.transition(&Trigger::ToolCallStart);
        assert_eq!(result.unwrap(), AgentStatus::Executing);
    }

    #[test]
    fn test_thinking_to_working_on_llm_end() {
        let result = AgentStatus::Thinking.transition(&Trigger::LlmCallEnd);
        assert_eq!(result.unwrap(), AgentStatus::Working);
    }

    #[test]
    fn test_executing_to_working() {
        let result = AgentStatus::Executing.transition(&Trigger::ToolCallEnd);
        assert_eq!(result.unwrap(), AgentStatus::Working);
    }

    #[test]
    fn test_working_to_paused() {
        let result = AgentStatus::Working.transition(&Trigger::ApprovalNeeded);
        assert_eq!(result.unwrap(), AgentStatus::Paused);
    }

    #[test]
    fn test_paused_to_working() {
        let result = AgentStatus::Paused.transition(&Trigger::ApprovalGranted);
        assert_eq!(result.unwrap(), AgentStatus::Working);
    }

    #[test]
    fn test_paused_approval_rejected_to_error() {
        let result = AgentStatus::Paused.transition(&Trigger::ApprovalRejected);
        assert_eq!(result.unwrap(), AgentStatus::Error);
    }

    #[test]
    fn test_working_to_idle_on_complete() {
        let result = AgentStatus::Working.transition(&Trigger::TaskComplete);
        assert_eq!(result.unwrap(), AgentStatus::Idle);
    }

    #[test]
    fn test_working_to_idle_on_cancel() {
        let result = AgentStatus::Working.transition(&Trigger::TaskCancel);
        assert_eq!(result.unwrap(), AgentStatus::Idle);
    }

    #[test]
    fn test_error_recovery() {
        let result = AgentStatus::Error.transition(&Trigger::Resume);
        assert_eq!(result.unwrap(), AgentStatus::Idle);
    }

    #[test]
    fn test_paused_resume_to_idle() {
        let result = AgentStatus::Paused.transition(&Trigger::Resume);
        assert_eq!(result.unwrap(), AgentStatus::Idle);
    }

    #[test]
    fn test_any_state_error_transition() {
        let trigger = Trigger::ErrorOccurred {
            message: "something broke".into(),
        };
        // Working, Thinking, Executing, Paused should all transition to Error.
        for status in [
            AgentStatus::Working,
            AgentStatus::Thinking,
            AgentStatus::Executing,
            AgentStatus::Paused,
        ] {
            let result = status.transition(&trigger);
            assert_eq!(result.unwrap(), AgentStatus::Error, "from {:?}", status);
        }
    }

    #[test]
    fn test_invalid_transition() {
        // IDLE + LlmCallStart should be invalid
        let result = AgentStatus::Idle.transition(&Trigger::LlmCallStart);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_transition_error_no_task_assign() {
        // ERROR + TaskAssign should be invalid (must Resume first)
        let result = AgentStatus::Error.transition(&Trigger::TaskAssign);
        assert!(result.is_err());
    }

    #[test]
    fn test_display_names() {
        assert_eq!(AgentStatus::Idle.display_name(), "idle");
        assert_eq!(AgentStatus::Working.display_name(), "working");
        assert_eq!(AgentStatus::Thinking.display_name(), "thinking");
        assert_eq!(AgentStatus::Executing.display_name(), "executing");
        assert_eq!(AgentStatus::Paused.display_name(), "paused");
        assert_eq!(AgentStatus::Error.display_name(), "error");
    }

    #[test]
    fn test_serde_roundtrip_status() {
        let status = AgentStatus::Thinking;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"thinking\"");
        let back: AgentStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, status);
    }

    #[test]
    fn test_serde_roundtrip_trigger() {
        let trigger = Trigger::ErrorOccurred {
            message: "oops".into(),
        };
        let json = serde_json::to_string(&trigger).unwrap();
        let back: Trigger = serde_json::from_str(&json).unwrap();
        match back {
            Trigger::ErrorOccurred { message } => assert_eq!(message, "oops"),
            _ => panic!("wrong variant"),
        }
    }
}
