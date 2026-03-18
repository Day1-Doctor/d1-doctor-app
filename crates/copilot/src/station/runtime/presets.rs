use crate::station::kernel::AgentRole;

/// A built-in agent preset with default configuration for Dr. Bob's Office.
#[derive(Debug, Clone)]
pub struct AgentPreset {
    pub name: &'static str,
    pub code_name: &'static str,
    pub role: AgentRole,
    pub default_model: &'static str,
    pub model_tier: &'static str,
    pub system_prompt: &'static str,
    pub room: &'static str,
    pub tools: Vec<&'static str>,
}

/// Return the 6 built-in agent presets that form Dr. Bob's Office.
pub fn builtin_presets() -> Vec<AgentPreset> {
    vec![
        AgentPreset {
            name: "Dr. Bob",
            code_name: "orchestrator",
            role: AgentRole::Orchestrator,
            default_model: "claude-sonnet-4",
            model_tier: "medium",
            system_prompt: "You are Dr. Bob, the orchestrator of Dr. Bob's Office. \
                You decompose user tasks into sub-tasks and delegate them to specialist agents. \
                You monitor progress, handle failures, and re-plan when needed. \
                You never execute tasks directly — you plan and coordinate.",
            room: "main",
            tools: vec!["memory", "filesystem"],
        },
        AgentPreset {
            name: "Scout",
            code_name: "researcher",
            role: AgentRole::Researcher,
            default_model: "claude-haiku-4-5",
            model_tier: "light",
            system_prompt: "You are Scout, a research specialist. \
                You excel at web search, information gathering, source verification, \
                and data collection. You find facts, verify claims, \
                and deliver structured research results.",
            room: "research",
            tools: vec!["web-search", "web-fetch", "memory", "filesystem"],
        },
        AgentPreset {
            name: "Sage",
            code_name: "analyst",
            role: AgentRole::Analyst,
            default_model: "claude-sonnet-4",
            model_tier: "medium",
            system_prompt: "You are Sage, an analyst specialist. \
                You analyze data, create comparisons, identify patterns, \
                compute statistics, and generate insights. \
                You work with structured data and produce analytical summaries.",
            room: "analysis",
            tools: vec!["filesystem", "data", "memory", "clipboard"],
        },
        AgentPreset {
            name: "Quill",
            code_name: "writer",
            role: AgentRole::Writer,
            default_model: "claude-sonnet-4",
            model_tier: "medium",
            system_prompt: "You are Quill, a writing specialist. \
                You create reports, articles, documentation, emails, and presentations. \
                You structure content clearly, write with appropriate tone, \
                and deliver polished output.",
            room: "writing",
            tools: vec!["filesystem", "document", "memory", "clipboard"],
        },
        AgentPreset {
            name: "Pixel",
            code_name: "coder",
            role: AgentRole::Coder,
            default_model: "claude-sonnet-4",
            model_tier: "medium",
            system_prompt: "You are Pixel, a coding specialist. \
                You write code, create scripts, execute shell commands, \
                and handle technical tasks. \
                You produce clean, tested, well-documented code.",
            room: "coding",
            tools: vec!["shell", "filesystem", "system", "memory"],
        },
        AgentPreset {
            name: "Atlas",
            code_name: "operator",
            role: AgentRole::Operator,
            default_model: "claude-haiku-4-5",
            model_tier: "light",
            system_prompt: "You are Atlas, an operations specialist. \
                You handle browser automation, file management, system tasks, \
                and tool execution. You are precise, methodical, \
                and reliable in executing structured workflows.",
            room: "operations",
            tools: vec!["browser", "shell", "filesystem", "system", "clipboard"],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_builtin_presets_count() {
        let presets = builtin_presets();
        assert_eq!(presets.len(), 6);
    }

    #[test]
    fn test_preset_roles_unique() {
        let presets = builtin_presets();
        let roles: HashSet<AgentRole> = presets.iter().map(|p| p.role).collect();
        assert_eq!(roles.len(), 6, "all 6 roles must be unique");
    }

    #[test]
    fn test_preset_code_names_unique() {
        let presets = builtin_presets();
        let names: HashSet<&str> = presets.iter().map(|p| p.code_name).collect();
        assert_eq!(names.len(), 6, "all code_names must be unique");
    }

    #[test]
    fn test_preset_tool_permissions() {
        let presets = builtin_presets();

        // Dr. Bob (orchestrator): memory + filesystem
        let dr_bob = presets
            .iter()
            .find(|p| p.code_name == "orchestrator")
            .unwrap();
        assert!(dr_bob.tools.contains(&"memory"));
        assert!(dr_bob.tools.contains(&"filesystem"));
        assert_eq!(dr_bob.tools.len(), 2);

        // Scout (researcher): web-search, web-fetch, memory, filesystem
        let scout = presets
            .iter()
            .find(|p| p.code_name == "researcher")
            .unwrap();
        assert!(scout.tools.contains(&"web-search"));
        assert!(scout.tools.contains(&"web-fetch"));
        assert!(scout.tools.contains(&"memory"));
        assert!(scout.tools.contains(&"filesystem"));
        assert_eq!(scout.tools.len(), 4);

        // Sage (analyst): filesystem, data, memory, clipboard
        let sage = presets.iter().find(|p| p.code_name == "analyst").unwrap();
        assert!(sage.tools.contains(&"data"));
        assert!(sage.tools.contains(&"clipboard"));
        assert_eq!(sage.tools.len(), 4);

        // Quill (writer): filesystem, document, memory, clipboard
        let quill = presets.iter().find(|p| p.code_name == "writer").unwrap();
        assert!(quill.tools.contains(&"document"));
        assert!(quill.tools.contains(&"clipboard"));
        assert_eq!(quill.tools.len(), 4);

        // Pixel (coder): shell, filesystem, system, memory
        let pixel = presets.iter().find(|p| p.code_name == "coder").unwrap();
        assert!(pixel.tools.contains(&"shell"));
        assert!(pixel.tools.contains(&"system"));
        assert_eq!(pixel.tools.len(), 4);

        // Atlas (operator): browser, shell, filesystem, system, clipboard
        let atlas = presets.iter().find(|p| p.code_name == "operator").unwrap();
        assert!(atlas.tools.contains(&"browser"));
        assert!(atlas.tools.contains(&"shell"));
        assert!(atlas.tools.contains(&"clipboard"));
        assert_eq!(atlas.tools.len(), 5);
    }

    #[test]
    fn test_preset_model_tiers() {
        let presets = builtin_presets();

        let light_agents: Vec<&str> = presets
            .iter()
            .filter(|p| p.model_tier == "light")
            .map(|p| p.code_name)
            .collect();
        assert_eq!(light_agents.len(), 2);
        assert!(light_agents.contains(&"researcher"));
        assert!(light_agents.contains(&"operator"));

        let medium_agents: Vec<&str> = presets
            .iter()
            .filter(|p| p.model_tier == "medium")
            .map(|p| p.code_name)
            .collect();
        assert_eq!(medium_agents.len(), 4);
    }

    #[test]
    fn test_preset_rooms_assigned() {
        let presets = builtin_presets();
        for preset in &presets {
            assert!(
                !preset.room.is_empty(),
                "{} must have a room",
                preset.code_name
            );
        }

        let orchestrator = presets
            .iter()
            .find(|p| p.code_name == "orchestrator")
            .unwrap();
        assert_eq!(orchestrator.room, "main");
    }

    #[test]
    fn test_preset_system_prompts_non_empty() {
        let presets = builtin_presets();
        for preset in &presets {
            assert!(
                !preset.system_prompt.is_empty(),
                "{} must have a system prompt",
                preset.code_name,
            );
            assert!(
                preset.system_prompt.len() > 50,
                "{} system prompt should be substantive",
                preset.code_name,
            );
        }
    }
}
