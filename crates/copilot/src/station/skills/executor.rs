use std::sync::Arc;

use super::skill_registry::SkillRegistry;
use super::skill_types::SkillDefinition;
use crate::station::tasks::decomposer::DecomposedPlan;

/// Standalone skill execution engine.
///
/// `SkillExecutor` is the canonical source for skill selection and prompt
/// composition. `AgentExecutor` delegates to this for skill-related logic
/// rather than duplicating the matching heuristics.
pub struct SkillExecutor {
    registry: Arc<SkillRegistry>,
}

impl SkillExecutor {
    /// Create a new executor backed by the given skill registry.
    pub fn new(registry: Arc<SkillRegistry>) -> Self {
        Self { registry }
    }

    /// Select the best skill for a task step based on agent role and step keywords.
    ///
    /// The algorithm:
    /// 1. Filter skills to those available for the agent's role.
    /// 2. Score each skill by counting keyword matches between the step title
    ///    and the skill's name (weight 2) and description (weight 1).
    /// 3. Return the highest-scoring skill, or `None` if no keyword matches.
    pub fn select_skill(&self, step_title: &str, agent_role: &str) -> Option<SkillDefinition> {
        let agent_skills = self.registry.skills_for_agent(agent_role);

        if agent_skills.is_empty() {
            return None;
        }

        let title_lower = step_title.to_lowercase();

        let mut best_skill: Option<&SkillDefinition> = None;
        let mut best_score: usize = 0;

        for skill in &agent_skills {
            let mut score: usize = 0;

            // Check skill name words against step title.
            for word in skill.name.to_lowercase().split_whitespace() {
                if word.len() >= 3 && title_lower.contains(&word) {
                    score += 2;
                }
            }

            // Check skill description words (less weight).
            for word in skill.description.to_lowercase().split_whitespace() {
                if word.len() >= 4 && title_lower.contains(&word) {
                    score += 1;
                }
            }

            if score > best_score {
                best_score = score;
                best_skill = Some(skill);
            }
        }

        // Only return a skill if we got at least one keyword match.
        if best_score > 0 {
            best_skill.cloned()
        } else {
            None
        }
    }

    /// Build a composite prompt from a skill's steps and runtime context.
    ///
    /// Concatenates the skill's step instructions into a single prompt,
    /// replacing `{{placeholder}}` template variables with values extracted
    /// from the provided context string.
    pub fn build_skill_prompt(&self, skill: &SkillDefinition, context: &str) -> String {
        let mut parts: Vec<String> = Vec::new();

        parts.push(format!(
            "Skill: {} - {}",
            skill.name, skill.description
        ));
        parts.push(String::new());

        for (i, step) in skill.steps.iter().enumerate() {
            let expanded = self.replace_template_vars(&step.prompt_template, context);
            parts.push(format!(
                "Step {} - {} (role: {}): {}",
                i + 1,
                step.title,
                step.agent_role,
                expanded
            ));
        }

        parts.push(String::new());
        parts.push(format!("Context: {}", context));

        parts.join("\n")
    }

    /// Match a decomposed plan's steps to skills and annotate each step.
    ///
    /// For each `PlannedStep`, this finds the best matching skill based on
    /// the step's title and suggested role, then stores the skill name on
    /// the step's `skill_name` field.
    pub fn annotate_plan(&self, plan: &mut DecomposedPlan) {
        for step in &mut plan.steps {
            if let Some(skill) = self.select_skill(&step.title, &step.suggested_role) {
                step.skill_name = Some(skill.name.clone());
            }
        }
    }

    /// Replace `{{key}}` placeholders in a template with the context string.
    ///
    /// Any placeholder found in the template is replaced with the full context
    /// value. This is a simple substitution suitable for the current rule-based
    /// approach; a future version may support structured context maps.
    fn replace_template_vars(&self, template: &str, context: &str) -> String {
        let mut result = template.to_string();
        // Find all {{...}} placeholders and replace with context.
        loop {
            let start = result.find("{{");
            let end = result.find("}}");
            match (start, end) {
                (Some(s), Some(e)) if e > s => {
                    result = format!("{}{}{}", &result[..s], context, &result[e + 2..]);
                }
                _ => break,
            }
        }
        result
    }

    /// Expose the underlying registry for callers that need direct access.
    pub fn registry(&self) -> &SkillRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::station::skills::skill_types::{SkillDefinition, SkillStep};
    use crate::station::tasks::decomposer::{DecomposedPlan, PlannedStep};

    fn make_executor() -> SkillExecutor {
        let registry = Arc::new(SkillRegistry::new());
        SkillExecutor::new(registry)
    }

    // -----------------------------------------------------------------------
    // select_skill tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_select_skill_matches_research_keywords() {
        let executor = make_executor();
        let skill = executor.select_skill("Deep research into market trends", "researcher");

        assert!(skill.is_some(), "should match a research skill");
        let skill = skill.unwrap();
        assert_eq!(skill.id, "deep-research");
    }

    #[test]
    fn test_select_skill_matches_code_review() {
        let executor = make_executor();
        let skill = executor.select_skill("Code review of the auth module", "coder");

        assert!(skill.is_some(), "should match the code review skill");
        let skill = skill.unwrap();
        assert_eq!(skill.id, "code-review");
    }

    #[test]
    fn test_select_skill_matches_report_writing() {
        let executor = make_executor();
        let skill = executor.select_skill("Write a report on Q4 earnings", "writer");

        assert!(skill.is_some(), "should match report writing skill");
        let skill = skill.unwrap();
        assert_eq!(skill.id, "report-writing");
    }

    #[test]
    fn test_select_skill_matches_email_drafting() {
        let executor = make_executor();
        let skill = executor.select_skill("Draft an email to the team", "writer");

        assert!(skill.is_some(), "should match email drafting skill");
        let skill = skill.unwrap();
        assert_eq!(skill.id, "email-drafting");
    }

    #[test]
    fn test_select_skill_matches_comparative_analysis() {
        let executor = make_executor();
        let skill =
            executor.select_skill("Comparative analysis of cloud providers", "analyst");

        assert!(skill.is_some(), "should match comparative analysis skill");
        let skill = skill.unwrap();
        assert_eq!(skill.id, "comparative-analysis");
    }

    #[test]
    fn test_select_skill_returns_none_for_unmatched_steps() {
        let executor = make_executor();
        let skill = executor.select_skill("Do something completely unrelated xyz", "coder");

        assert!(skill.is_none(), "should not match any skill");
    }

    #[test]
    fn test_select_skill_returns_none_for_unknown_role() {
        let executor = make_executor();
        let skill = executor.select_skill("Deep research topic", "unknown-role");

        assert!(skill.is_none(), "unknown role should have no skills");
    }

    #[test]
    fn test_select_skill_role_scoping() {
        let executor = make_executor();
        // "Deep research" keywords but wrong role — operator has no research skill.
        let skill = executor.select_skill("Deep research into market trends", "operator");

        // Operator only has web-automation; "deep research" shouldn't match it.
        assert!(
            skill.as_ref().map_or(true, |s| s.id != "deep-research"),
            "operator should not get deep-research skill"
        );
    }

    // -----------------------------------------------------------------------
    // build_skill_prompt tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_build_skill_prompt_replaces_template_variables() {
        let executor = make_executor();
        let skill = SkillDefinition {
            id: "test-skill".into(),
            name: "Test Skill".into(),
            description: "A test skill".into(),
            used_by: vec!["researcher".into()],
            steps: vec![
                SkillStep {
                    title: "Search".into(),
                    agent_role: "researcher".into(),
                    prompt_template: "Search for {{topic}} online".into(),
                    tools: vec!["web-search".into()],
                },
                SkillStep {
                    title: "Summarize".into(),
                    agent_role: "analyst".into(),
                    prompt_template: "Summarize {{findings}} concisely".into(),
                    tools: vec!["memory".into()],
                },
            ],
        };

        let prompt = executor.build_skill_prompt(&skill, "AI market trends");

        assert!(
            prompt.contains("Test Skill - A test skill"),
            "prompt should contain skill header"
        );
        assert!(
            prompt.contains("Search for AI market trends online"),
            "prompt should replace {{topic}}"
        );
        assert!(
            prompt.contains("Summarize AI market trends concisely"),
            "prompt should replace {{findings}}"
        );
        assert!(
            prompt.contains("Context: AI market trends"),
            "prompt should include context footer"
        );
    }

    #[test]
    fn test_build_skill_prompt_no_placeholders() {
        let executor = make_executor();
        let skill = SkillDefinition {
            id: "no-vars".into(),
            name: "Static".into(),
            description: "No variables".into(),
            used_by: vec!["coder".into()],
            steps: vec![SkillStep {
                title: "Do it".into(),
                agent_role: "coder".into(),
                prompt_template: "Just do the thing".into(),
                tools: vec!["memory".into()],
            }],
        };

        let prompt = executor.build_skill_prompt(&skill, "context text");

        assert!(prompt.contains("Just do the thing"));
        assert!(prompt.contains("Context: context text"));
    }

    #[test]
    fn test_build_skill_prompt_multiple_placeholders_in_step() {
        let executor = make_executor();
        let skill = SkillDefinition {
            id: "multi-var".into(),
            name: "Multi".into(),
            description: "Multiple vars".into(),
            used_by: vec!["writer".into()],
            steps: vec![SkillStep {
                title: "Compose".into(),
                agent_role: "writer".into(),
                prompt_template: "Write about {{topic}} for {{audience}} in {{tone}} tone"
                    .into(),
                tools: vec!["memory".into()],
            }],
        };

        let prompt = executor.build_skill_prompt(&skill, "quarterly results");

        // All placeholders replaced with the context string.
        assert!(prompt
            .contains("Write about quarterly results for quarterly results in quarterly results tone"));
    }

    // -----------------------------------------------------------------------
    // annotate_plan tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_annotate_plan_annotates_matching_steps() {
        let executor = make_executor();
        let mut plan = DecomposedPlan {
            original_description: "Research AI and write a report".into(),
            steps: vec![
                PlannedStep {
                    title: "Deep research into AI trends".into(),
                    description: "Search for info".into(),
                    suggested_role: "researcher".into(),
                    step_index: 0,
                    depends_on: vec![],
                    is_parallel: false,
                    skill_name: None,
                },
                PlannedStep {
                    title: "Write a report on findings".into(),
                    description: "Create report".into(),
                    suggested_role: "writer".into(),
                    step_index: 1,
                    depends_on: vec![0],
                    is_parallel: false,
                    skill_name: None,
                },
            ],
        };

        executor.annotate_plan(&mut plan);

        assert_eq!(
            plan.steps[0].skill_name,
            Some("Deep Research".to_string()),
            "research step should be annotated with Deep Research skill"
        );
        assert_eq!(
            plan.steps[1].skill_name,
            Some("Report Writing".to_string()),
            "write step should be annotated with Report Writing skill"
        );
    }

    #[test]
    fn test_annotate_plan_leaves_unmatched_steps_as_none() {
        let executor = make_executor();
        let mut plan = DecomposedPlan {
            original_description: "Do something weird".into(),
            steps: vec![PlannedStep {
                title: "Do something completely unrelated xyz".into(),
                description: "Mystery task".into(),
                suggested_role: "coder".into(),
                step_index: 0,
                depends_on: vec![],
                is_parallel: false,
                skill_name: None,
            }],
        };

        executor.annotate_plan(&mut plan);

        assert_eq!(
            plan.steps[0].skill_name, None,
            "unmatched step should remain None"
        );
    }

    #[test]
    fn test_annotate_plan_mixed_match_and_no_match() {
        let executor = make_executor();
        let mut plan = DecomposedPlan {
            original_description: "Research topic and do a random thing".into(),
            steps: vec![
                PlannedStep {
                    title: "Deep research into topic".into(),
                    description: "Search".into(),
                    suggested_role: "researcher".into(),
                    step_index: 0,
                    depends_on: vec![],
                    is_parallel: false,
                    skill_name: None,
                },
                PlannedStep {
                    title: "Miscellaneous xyz".into(),
                    description: "Unknown".into(),
                    suggested_role: "orchestrator".into(),
                    step_index: 1,
                    depends_on: vec![0],
                    is_parallel: false,
                    skill_name: None,
                },
            ],
        };

        executor.annotate_plan(&mut plan);

        assert!(
            plan.steps[0].skill_name.is_some(),
            "research step should be annotated"
        );
        assert_eq!(
            plan.steps[1].skill_name, None,
            "miscellaneous step should remain None"
        );
    }

    // -----------------------------------------------------------------------
    // replace_template_vars tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_replace_template_vars_basic() {
        let executor = make_executor();
        let result = executor.replace_template_vars("Hello {{name}}", "World");
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_replace_template_vars_no_placeholders() {
        let executor = make_executor();
        let result = executor.replace_template_vars("No placeholders here", "context");
        assert_eq!(result, "No placeholders here");
    }

    #[test]
    fn test_replace_template_vars_multiple() {
        let executor = make_executor();
        let result =
            executor.replace_template_vars("{{a}} and {{b}} and {{c}}", "val");
        assert_eq!(result, "val and val and val");
    }
}
