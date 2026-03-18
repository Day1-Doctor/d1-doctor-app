use super::builtin_skills;
use super::skill_types::SkillDefinition;

/// Registry of available skill definitions.
///
/// The registry loads all built-in skills on construction and provides
/// lookup methods for querying skills by ID or by agent role.
pub struct SkillRegistry {
    skills: Vec<SkillDefinition>,
}

impl SkillRegistry {
    /// Create a new registry pre-loaded with all built-in skills.
    pub fn new() -> Self {
        Self {
            skills: builtin_skills::all_skills(),
        }
    }

    /// Look up a skill by its unique ID.
    pub fn get_skill(&self, id: &str) -> Option<&SkillDefinition> {
        self.skills.iter().find(|s| s.id == id)
    }

    /// Return all skills that the given agent role can use.
    pub fn skills_for_agent(&self, role: &str) -> Vec<&SkillDefinition> {
        self.skills
            .iter()
            .filter(|s| s.used_by.iter().any(|r| r == role))
            .collect()
    }

    /// Return a slice of all registered skills.
    pub fn list_all(&self) -> &[SkillDefinition] {
        &self.skills
    }

    /// Return the total number of registered skills.
    pub fn count(&self) -> usize {
        self.skills.len()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new_loads_all_skills() {
        let registry = SkillRegistry::new();
        assert_eq!(registry.count(), 16);
        assert_eq!(registry.list_all().len(), 16);
    }

    #[test]
    fn test_get_skill_by_id() {
        let registry = SkillRegistry::new();

        let skill = registry.get_skill("deep-research");
        assert!(skill.is_some());
        assert_eq!(skill.unwrap().name, "Deep Research");

        let missing = registry.get_skill("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_skills_for_researcher() {
        let registry = SkillRegistry::new();
        let researcher_skills = registry.skills_for_agent("researcher");

        // Researcher should have: deep-research, context-accumulation,
        // meeting-prep, fact-checking
        assert!(
            researcher_skills.len() >= 4,
            "researcher should have at least 4 skills, got {}",
            researcher_skills.len()
        );

        let ids: Vec<&str> = researcher_skills.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"deep-research"));
        assert!(ids.contains(&"context-accumulation"));
        assert!(ids.contains(&"meeting-prep"));
        assert!(ids.contains(&"fact-checking"));
    }

    #[test]
    fn test_skills_for_orchestrator() {
        let registry = SkillRegistry::new();
        let orchestrator_skills = registry.skills_for_agent("orchestrator");

        // Orchestrator uses the most skills: structured-planning, diagnostic-reasoning,
        // self-verification, parallel-dispatch, context-accumulation, task-handoff
        assert!(
            orchestrator_skills.len() >= 6,
            "orchestrator should have at least 6 skills, got {}",
            orchestrator_skills.len()
        );

        let ids: Vec<&str> = orchestrator_skills.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"structured-planning"));
        assert!(ids.contains(&"parallel-dispatch"));
        assert!(ids.contains(&"task-handoff"));
    }

    #[test]
    fn test_skills_for_coder() {
        let registry = SkillRegistry::new();
        let coder_skills = registry.skills_for_agent("coder");

        let ids: Vec<&str> = coder_skills.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"code-review"));
        assert!(ids.contains(&"self-verification"));
    }

    #[test]
    fn test_skills_for_writer() {
        let registry = SkillRegistry::new();
        let writer_skills = registry.skills_for_agent("writer");

        let ids: Vec<&str> = writer_skills.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"report-writing"));
        assert!(ids.contains(&"email-drafting"));
        assert!(ids.contains(&"content-repurpose"));
        assert!(ids.contains(&"meeting-prep"));
        assert!(ids.contains(&"self-verification"));
    }

    #[test]
    fn test_skills_for_analyst() {
        let registry = SkillRegistry::new();
        let analyst_skills = registry.skills_for_agent("analyst");

        let ids: Vec<&str> = analyst_skills.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"comparative-analysis"));
        assert!(ids.contains(&"data-summarization"));
        assert!(ids.contains(&"diagnostic-reasoning"));
        assert!(ids.contains(&"fact-checking"));
    }

    #[test]
    fn test_skills_for_operator() {
        let registry = SkillRegistry::new();
        let operator_skills = registry.skills_for_agent("operator");

        let ids: Vec<&str> = operator_skills.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"web-automation"));
    }

    #[test]
    fn test_skills_for_unknown_role() {
        let registry = SkillRegistry::new();
        let skills = registry.skills_for_agent("unknown-role");
        assert!(skills.is_empty());
    }

    #[test]
    fn test_default_impl() {
        let registry = SkillRegistry::default();
        assert_eq!(registry.count(), 16);
    }
}
