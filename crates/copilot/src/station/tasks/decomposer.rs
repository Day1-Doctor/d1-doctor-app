use super::task_types::TaskSpec;

/// A plan produced by decomposing a natural-language task description
/// into a sequence of structured steps.
#[derive(Debug, Clone)]
pub struct DecomposedPlan {
    pub original_description: String,
    pub steps: Vec<PlannedStep>,
}

/// A single step within a decomposed plan.
#[derive(Debug, Clone)]
pub struct PlannedStep {
    pub title: String,
    pub description: String,
    /// Suggested agent role: "researcher", "analyst", "writer", "coder", "operator",
    /// or "orchestrator" (fallback).
    pub suggested_role: String,
    pub step_index: u32,
    /// Step indices this step depends on (must complete first).
    pub depends_on: Vec<u32>,
    /// Whether this step can run in parallel with the previous step.
    pub is_parallel: bool,
    /// Name of the matched skill, set by `SkillExecutor::annotate_plan`.
    pub skill_name: Option<String>,
}

/// Rule-based task decomposer.
///
/// Takes a natural language task description and produces structured sub-tasks
/// using keyword-matching heuristics. A future version will use LLM-powered
/// decomposition; this module provides the interface and baseline logic.
pub struct TaskDecomposer;

impl TaskDecomposer {
    pub fn new() -> Self {
        Self
    }

    /// Decompose a natural language task into structured steps.
    ///
    /// For now this uses keyword-based heuristics. Later this will call an LLM.
    pub fn decompose(&self, description: &str) -> DecomposedPlan {
        let lower = description.to_lowercase();
        let mut steps = Vec::new();
        let mut idx = 0u32;

        // Pattern: "research X and write Y" -> research step + write step
        if lower.contains("research") || lower.contains("find") || lower.contains("search") {
            steps.push(PlannedStep {
                title: format!("Research: {}", extract_research_topic(&lower)),
                description: "Search and gather information about the topic".into(),
                suggested_role: "researcher".into(),
                step_index: idx,
                depends_on: vec![],
                is_parallel: false,
                skill_name: None,
            });
            idx += 1;
        }

        if lower.contains("analy") || lower.contains("compar") || lower.contains("evaluat") {
            steps.push(PlannedStep {
                title: "Analyze findings".into(),
                description: "Analyze and synthesize the gathered information".into(),
                suggested_role: "analyst".into(),
                step_index: idx,
                depends_on: if idx > 0 { vec![idx - 1] } else { vec![] },
                is_parallel: false,
                skill_name: None,
            });
            idx += 1;
        }

        if lower.contains("write")
            || lower.contains("report")
            || lower.contains("document")
            || lower.contains("draft")
        {
            steps.push(PlannedStep {
                title: "Write output".into(),
                description: "Create the final written deliverable".into(),
                suggested_role: "writer".into(),
                step_index: idx,
                depends_on: if idx > 0 { vec![idx - 1] } else { vec![] },
                is_parallel: false,
                skill_name: None,
            });
            idx += 1;
        }

        if lower.contains("code")
            || lower.contains("implement")
            || lower.contains("build")
            || lower.contains("script")
        {
            steps.push(PlannedStep {
                title: "Implement code".into(),
                description: "Write and test the code".into(),
                suggested_role: "coder".into(),
                step_index: idx,
                depends_on: if idx > 0 { vec![idx - 1] } else { vec![] },
                is_parallel: false,
                skill_name: None,
            });
            idx += 1;
        }

        if lower.contains("save") || lower.contains("export") || lower.contains("deploy") {
            steps.push(PlannedStep {
                title: "Save/export output".into(),
                description: "Save the final output to the workspace".into(),
                suggested_role: "operator".into(),
                step_index: idx,
                depends_on: if idx > 0 { vec![idx - 1] } else { vec![] },
                is_parallel: false,
                skill_name: None,
            });
        }

        // Fallback: single step assigned to orchestrator
        if steps.is_empty() {
            steps.push(PlannedStep {
                title: description.to_string(),
                description: "Execute the task".into(),
                suggested_role: "orchestrator".into(),
                step_index: 0,
                depends_on: vec![],
                is_parallel: false,
                skill_name: None,
            });
        }

        DecomposedPlan {
            original_description: description.to_string(),
            steps,
        }
    }
}

impl Default for TaskDecomposer {
    fn default() -> Self {
        Self::new()
    }
}

impl PlannedStep {
    /// Convert this planned step into a `TaskSpec` sub-task for the given parent.
    pub fn to_subtask(&self, parent_id: &str) -> TaskSpec {
        TaskSpec::new_subtask(&self.title, parent_id, self.step_index)
    }
}

/// Simple extraction: return everything after "research", "find", or "search".
/// Truncated to a reasonable length.
fn extract_research_topic(desc: &str) -> String {
    desc.split("research")
        .nth(1)
        .or_else(|| desc.split("find").nth(1))
        .or_else(|| desc.split("search").nth(1))
        .map(|s| s.trim().chars().take(80).collect())
        .unwrap_or_else(|| "the topic".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decomposer() -> TaskDecomposer {
        TaskDecomposer::new()
    }

    #[test]
    fn test_research_and_write() {
        let plan = decomposer().decompose("Research AI agents and write a report");
        assert!(
            plan.steps.len() >= 2,
            "expected at least 2 steps, got {}",
            plan.steps.len()
        );

        let roles: Vec<&str> = plan
            .steps
            .iter()
            .map(|s| s.suggested_role.as_str())
            .collect();
        assert!(
            roles.contains(&"researcher"),
            "should include researcher role"
        );
        assert!(roles.contains(&"writer"), "should include writer role");
    }

    #[test]
    fn test_research_analyze_write() {
        let plan =
            decomposer().decompose("Research competitors, analyze them, and write a comparison");
        assert_eq!(plan.steps.len(), 3, "expected 3 steps");

        assert_eq!(plan.steps[0].suggested_role, "researcher");
        assert_eq!(plan.steps[1].suggested_role, "analyst");
        assert_eq!(plan.steps[2].suggested_role, "writer");
    }

    #[test]
    fn test_code_task() {
        let plan = decomposer().decompose("Build a web server");
        assert!(!plan.steps.is_empty());

        let has_coder = plan.steps.iter().any(|s| s.suggested_role == "coder");
        assert!(has_coder, "should include a coder step");
    }

    #[test]
    fn test_simple_task() {
        let plan = decomposer().decompose("Hello world");
        assert_eq!(plan.steps.len(), 1, "fallback should produce 1 step");
        assert_eq!(plan.steps[0].suggested_role, "orchestrator");
        assert_eq!(plan.steps[0].title, "Hello world");
    }

    #[test]
    fn test_dependencies() {
        let plan =
            decomposer().decompose("Research competitors, analyze them, and write a comparison");
        // Step 0 (research) depends on nothing
        assert!(plan.steps[0].depends_on.is_empty());
        // Step 1 (analyze) depends on step 0
        assert_eq!(plan.steps[1].depends_on, vec![0]);
        // Step 2 (write) depends on step 1
        assert_eq!(plan.steps[2].depends_on, vec![1]);
    }

    #[test]
    fn test_original_description_preserved() {
        let desc = "Research AI agents and write a report";
        let plan = decomposer().decompose(desc);
        assert_eq!(plan.original_description, desc);
    }

    #[test]
    fn test_step_indices_sequential() {
        let plan = decomposer().decompose("Research, analyze, write, and deploy");
        for (i, step) in plan.steps.iter().enumerate() {
            assert_eq!(step.step_index, i as u32, "step_index should be sequential");
        }
    }

    #[test]
    fn test_export_deploy_step() {
        let plan = decomposer().decompose("Save the final results and deploy");
        let has_operator = plan.steps.iter().any(|s| s.suggested_role == "operator");
        assert!(has_operator, "should include an operator step");
    }

    #[test]
    fn test_planned_step_to_subtask() {
        let step = PlannedStep {
            title: "Research: AI agents".into(),
            description: "Search and gather information".into(),
            suggested_role: "researcher".into(),
            step_index: 0,
            depends_on: vec![],
            is_parallel: false,
            skill_name: None,
        };

        let subtask = step.to_subtask("parent-123");
        assert_eq!(subtask.title, "Research: AI agents");
        assert_eq!(subtask.parent_id, Some("parent-123".to_string()));
        assert_eq!(subtask.step_index, Some(0));
        assert_eq!(
            subtask.status,
            super::super::task_types::TaskStatus::Pending
        );
    }
}
