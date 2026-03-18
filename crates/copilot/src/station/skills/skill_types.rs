use serde::{Deserialize, Serialize};

/// Definition of a reusable multi-step skill that agents can execute.
///
/// Skills are composable workflows that chain multiple agent actions together.
/// Each skill has a unique ID, descriptive metadata, and a sequence of steps
/// that are executed in order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    /// Unique kebab-case identifier, e.g. `"deep-research"`.
    pub id: String,
    /// Human-readable name for UI display.
    pub name: String,
    /// One-line description of what the skill does.
    pub description: String,
    /// Agent `code_name` values that can invoke this skill.
    pub used_by: Vec<String>,
    /// Ordered sequence of steps the skill executes.
    pub steps: Vec<SkillStep>,
}

/// A single step within a skill workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStep {
    /// Human-readable title for the step (shown in UI timeline).
    pub title: String,
    /// The agent role responsible for executing this step.
    pub agent_role: String,
    /// Prompt template with `{{placeholder}}` variables that are filled at runtime.
    pub prompt_template: String,
    /// Tool names the agent is allowed to use during this step.
    pub tools: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_definition_serialize_roundtrip() {
        let skill = SkillDefinition {
            id: "test-skill".into(),
            name: "Test Skill".into(),
            description: "A test skill".into(),
            used_by: vec!["orchestrator".into()],
            steps: vec![SkillStep {
                title: "Step 1".into(),
                agent_role: "orchestrator".into(),
                prompt_template: "Do {{thing}}".into(),
                tools: vec!["memory".into()],
            }],
        };

        let json = serde_json::to_string(&skill).expect("serialize");
        let parsed: SkillDefinition = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.id, "test-skill");
        assert_eq!(parsed.steps.len(), 1);
        assert_eq!(parsed.steps[0].tools, vec!["memory"]);
    }

    #[test]
    fn test_skill_step_template_placeholders() {
        let step = SkillStep {
            title: "Search".into(),
            agent_role: "researcher".into(),
            prompt_template: "Search for {{topic}} using {{method}}".into(),
            tools: vec!["web-search".into()],
        };

        assert!(step.prompt_template.contains("{{topic}}"));
        assert!(step.prompt_template.contains("{{method}}"));
    }
}
