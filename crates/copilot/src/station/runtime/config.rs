use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::presets::builtin_presets;

/// Top-level runtime configuration, persisted as TOML.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeConfig {
    pub version: String,
    pub gateway_url: String,
    pub default_provider: String,
    pub agents: Vec<AgentConfig>,
}

/// Per-agent configuration entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentConfig {
    pub name: String,
    pub code_name: String,
    pub model: String,
    pub enabled: bool,
}

impl RuntimeConfig {
    /// Create the default configuration with all 6 built-in agents enabled.
    pub fn default_config() -> Self {
        let presets = builtin_presets();
        let agents = presets
            .iter()
            .map(|p| AgentConfig {
                name: p.name.to_string(),
                code_name: p.code_name.to_string(),
                model: p.default_model.to_string(),
                enabled: true,
            })
            .collect();

        Self {
            version: "1.0.0".to_string(),
            gateway_url: "https://gateway.day1.doctor/v1".to_string(),
            default_provider: "anthropic".to_string(),
            agents,
        }
    }

    /// The user-local config directory: `~/.day1copilot/`.
    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .expect("failed to resolve home directory")
            .join(".day1copilot")
    }

    /// Full path to the runtime config file.
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("runtime.toml")
    }

    /// Load from `~/.day1copilot/runtime.toml`, or create the default config
    /// if the file does not exist.
    pub fn load_or_create() -> Result<Self, String> {
        let path = Self::config_path();

        if path.exists() {
            let content =
                std::fs::read_to_string(&path).map_err(|e| format!("read config: {e}"))?;
            let config: Self =
                toml::from_str(&content).map_err(|e| format!("parse config: {e}"))?;
            Ok(config)
        } else {
            let config = Self::default_config();
            config.save()?;
            Ok(config)
        }
    }

    /// Persist the current configuration to `~/.day1copilot/runtime.toml`.
    pub fn save(&self) -> Result<(), String> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir).map_err(|e| format!("create config dir: {e}"))?;

        let content = toml::to_string_pretty(self).map_err(|e| format!("serialize config: {e}"))?;
        let path = Self::config_path();
        std::fs::write(&path, content).map_err(|e| format!("write config: {e}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = RuntimeConfig::default_config();

        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.gateway_url, "https://gateway.day1.doctor/v1");
        assert_eq!(config.default_provider, "anthropic");
        assert_eq!(config.agents.len(), 6);

        // All agents enabled by default.
        for agent in &config.agents {
            assert!(agent.enabled, "{} should be enabled", agent.code_name);
        }

        // Verify orchestrator is first.
        assert_eq!(config.agents[0].code_name, "orchestrator");
        assert_eq!(config.agents[0].name, "Dr. Bob");
    }

    #[test]
    fn test_config_serialize_roundtrip() {
        let config = RuntimeConfig::default_config();
        let toml_str = toml::to_string_pretty(&config).expect("serialize to TOML");
        let parsed: RuntimeConfig = toml::from_str(&toml_str).expect("parse TOML back");

        assert_eq!(config, parsed);
    }

    #[test]
    fn test_config_agent_models() {
        let config = RuntimeConfig::default_config();

        let orchestrator = config
            .agents
            .iter()
            .find(|a| a.code_name == "orchestrator")
            .unwrap();
        assert_eq!(orchestrator.model, "claude-sonnet-4");

        let researcher = config
            .agents
            .iter()
            .find(|a| a.code_name == "researcher")
            .unwrap();
        assert_eq!(researcher.model, "claude-haiku-4-5");

        let operator = config
            .agents
            .iter()
            .find(|a| a.code_name == "operator")
            .unwrap();
        assert_eq!(operator.model, "claude-haiku-4-5");
    }

    #[test]
    fn test_config_dir_exists() {
        let dir = RuntimeConfig::config_dir();
        assert!(dir.ends_with(".day1copilot"));
    }

    #[test]
    fn test_config_path_exists() {
        let path = RuntimeConfig::config_path();
        assert!(path.ends_with("runtime.toml"));
    }

    #[test]
    fn test_config_custom_values() {
        let config = RuntimeConfig {
            version: "2.0.0".to_string(),
            gateway_url: "http://localhost:4000/v1".to_string(),
            default_provider: "openai".to_string(),
            agents: vec![AgentConfig {
                name: "TestBot".to_string(),
                code_name: "test".to_string(),
                model: "gpt-4o".to_string(),
                enabled: false,
            }],
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: RuntimeConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.version, "2.0.0");
        assert_eq!(parsed.agents.len(), 1);
        assert!(!parsed.agents[0].enabled);
    }
}
