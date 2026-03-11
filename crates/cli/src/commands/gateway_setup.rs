//! `d1 gateway setup <app>` — auto-configure AI coding tools to use Day1 Doctor gateway.
//!
//! Supported apps: cursor, continue, openclaw, --generic
//!
//! This command:
//! 1. Ensures an API key exists (creates one if needed)
//! 2. Detects the app's config file path
//! 3. Backs up any existing config before modifying
//! 4. Writes OpenAI-compatible settings
//! 5. Prints confirmation

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::gateway::api_client;
use super::gateway_keys::{ApiKey, CreateKeyResponse};

/// Supported apps for auto-configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedApp {
    Cursor,
    Continue,
    OpenClaw,
    Generic,
}

impl SupportedApp {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cursor" => Some(Self::Cursor),
            "continue" => Some(Self::Continue),
            "openclaw" => Some(Self::OpenClaw),
            "generic" | "--generic" => Some(Self::Generic),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Cursor => "Cursor",
            Self::Continue => "Continue",
            Self::OpenClaw => "OpenClaw",
            Self::Generic => "Generic",
        }
    }
}

/// Gateway configuration to write into app config files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub api_base: String,
    pub api_key: String,
}

/// Run the setup flow for a given app.
pub async fn run_setup(app_name: &str) -> anyhow::Result<()> {
    let app = SupportedApp::from_str(app_name).ok_or_else(|| {
        anyhow::anyhow!(
            "{}",
            crate::i18n::t_args("gateway.setup.unsupported_app", &[("app", app_name)])
        )
    })?;

    println!(
        "{}",
        crate::i18n::t_args("gateway.setup.configuring", &[("app", app.display_name())])
    );

    // Step 1: Ensure we have an API key
    let api_key = ensure_api_key(app.display_name()).await?;

    // Step 2: Determine the gateway base URL
    let config = d1_common::Config::load().unwrap_or_default();
    let gateway_base = derive_gateway_url(&config.orchestrator_url);

    let gw_config = GatewayConfig {
        api_base: format!("{}/v1", gateway_base),
        api_key,
    };

    // Step 3: Write config for the app
    match app {
        SupportedApp::Cursor => setup_cursor(&gw_config)?,
        SupportedApp::Continue => setup_continue(&gw_config)?,
        SupportedApp::OpenClaw => setup_openclaw(&gw_config)?,
        SupportedApp::Generic => print_generic(&gw_config),
    }

    println!();
    println!("{}", crate::i18n::t("gateway.setup.done"));
    Ok(())
}

/// Ensure an API key exists for this app. If none exists, create one.
async fn ensure_api_key(app_name: &str) -> anyhow::Result<String> {
    let (client, base_url, token) = api_client()?;

    // List existing keys
    let resp = client
        .get(format!("{}/api/v1/api-keys", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await?;

    if resp.status().is_success() {
        let keys: Vec<ApiKey> = resp.json().await?;
        // Check if there is already an active key named after this app.
        // We cannot recover the plaintext key from an existing entry, so if one
        // exists we still need to create a new one. But we inform the user.
        if let Some(key) = keys.iter().find(|k| k.name == app_name && k.is_active) {
            println!(
                "{}",
                crate::i18n::t_args("gateway.setup.existing_key", &[("prefix", &key.key_prefix)])
            );
        }
    }

    // Create a new key for this app (full plaintext is only available at creation time)
    let resp = client
        .post(format!("{}/api/v1/api-keys", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "name": app_name }))
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!(
            "{}",
            crate::i18n::t_args(
                "gateway.keys.create_failed",
                &[("status", &status.to_string()), ("body", &body)]
            )
        );
    }

    let key: CreateKeyResponse = resp.json().await?;
    println!(
        "{}",
        crate::i18n::t_args(
            "gateway.setup.key_created",
            &[("name", &key.name), ("prefix", &key.key_prefix)]
        )
    );
    Ok(key.plaintext_key)
}

/// Derive the HTTPS gateway URL from the WebSocket orchestrator URL.
pub fn derive_gateway_url(orchestrator_url: &str) -> String {
    // wss://api.day1doctor.com/ws -> https://api.day1doctor.com
    let url = orchestrator_url
        .replace("wss://", "https://")
        .replace("ws://", "http://");
    // Strip trailing path segments like /ws
    if let Some(pos) = url.find("/ws") {
        url[..pos].to_string()
    } else {
        url
    }
}

// ─── Config path helpers ────────────────────────────────────────────────────

/// Resolve the Cursor config directory per OS.
pub fn cursor_config_path() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("{}", crate::i18n::t("errors.home_dir_error")))?;

    #[cfg(target_os = "macos")]
    let config_dir = home.join("Library/Application Support/Cursor/User");

    #[cfg(target_os = "linux")]
    let config_dir = home.join(".config/Cursor/User");

    #[cfg(target_os = "windows")]
    let config_dir = home.join("AppData/Roaming/Cursor/User");

    Ok(config_dir.join("settings.json"))
}

/// Resolve the Continue config file path.
pub fn continue_config_path() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("{}", crate::i18n::t("errors.home_dir_error")))?;
    Ok(home.join(".continue").join("config.json"))
}

/// Resolve the OpenClaw config file path.
pub fn openclaw_config_path() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("{}", crate::i18n::t("errors.home_dir_error")))?;
    Ok(home.join(".openclaw").join("config.yaml"))
}

// ─── Backup helper ──────────────────────────────────────────────────────────

/// Create a timestamped backup of a config file before modifying it.
fn backup_config(path: &std::path::Path) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S");
    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("config");
    let backup_name = format!("{}.{}.bak", filename, timestamp);
    let backup_path = path.with_file_name(backup_name);

    std::fs::copy(path, &backup_path)?;
    println!(
        "{}",
        crate::i18n::t_args(
            "gateway.setup.backup_created",
            &[("path", &backup_path.display().to_string())]
        )
    );
    Ok(())
}

// ─── App-specific config writers ────────────────────────────────────────────

/// Write Cursor settings with OpenAI-compatible API config.
///
/// Merges into the existing `settings.json` — only touches `openai.apiBase`
/// and `openai.apiKey` keys, leaving everything else intact.
pub fn write_cursor_config(
    settings_path: &std::path::Path,
    gw: &GatewayConfig,
) -> anyhow::Result<()> {
    // Read existing settings or start fresh
    let mut settings: serde_json::Value = if settings_path.exists() {
        let content = std::fs::read_to_string(settings_path)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        if let Some(parent) = settings_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        serde_json::json!({})
    };

    // Set OpenAI-compatible config for Cursor
    let obj = settings.as_object_mut().unwrap();
    obj.insert(
        "openai.apiBase".to_string(),
        serde_json::Value::String(gw.api_base.clone()),
    );
    obj.insert(
        "openai.apiKey".to_string(),
        serde_json::Value::String(gw.api_key.clone()),
    );

    let content = serde_json::to_string_pretty(&settings)?;
    std::fs::write(settings_path, &content)?;
    Ok(())
}

/// Write Continue config with a Day1 Doctor Gateway model provider entry.
///
/// Merges into the existing `config.json` — replaces any existing
/// "Day1 Doctor Gateway" model entry while preserving the rest.
pub fn write_continue_config(
    config_path: &std::path::Path,
    gw: &GatewayConfig,
) -> anyhow::Result<()> {
    let mut config: serde_json::Value = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        serde_json::json!({})
    };

    // Add Day1 Doctor as an OpenAI-compatible model provider
    let d1_model = serde_json::json!({
        "title": "Day1 Doctor Gateway",
        "provider": "openai",
        "model": "gpt-4o",
        "apiBase": gw.api_base,
        "apiKey": gw.api_key,
    });

    let obj = config.as_object_mut().unwrap();
    let models = obj.entry("models").or_insert_with(|| serde_json::json!([]));
    if let Some(arr) = models.as_array_mut() {
        // Remove any existing Day1 Doctor entries
        arr.retain(|m| {
            m.get("title")
                .and_then(|t| t.as_str())
                .map(|t| t != "Day1 Doctor Gateway")
                .unwrap_or(true)
        });
        arr.push(d1_model);
    }

    let content = serde_json::to_string_pretty(&config)?;
    std::fs::write(config_path, &content)?;
    Ok(())
}

/// Write OpenClaw config as YAML.
///
/// Creates or overwrites `~/.openclaw/config.yaml` with gateway settings.
pub fn write_openclaw_config(
    config_path: &std::path::Path,
    gw: &GatewayConfig,
) -> anyhow::Result<()> {
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Build YAML manually to avoid adding a serde_yaml dependency.
    // OpenClaw expects a simple key-value config.
    let content = format!(
        "# Day1 Doctor Gateway configuration\n\
         # Generated by: d1 gateway setup openclaw\n\
         \n\
         api_base: \"{}\"\n\
         api_key: \"{}\"\n",
        gw.api_base, gw.api_key,
    );

    std::fs::write(config_path, &content)?;
    Ok(())
}

/// Setup Cursor to use Day1 Doctor gateway.
fn setup_cursor(gw: &GatewayConfig) -> anyhow::Result<()> {
    let settings_path = cursor_config_path()?;
    backup_config(&settings_path)?;
    write_cursor_config(&settings_path, gw)?;
    println!(
        "{}",
        crate::i18n::t_args(
            "gateway.setup.wrote_config",
            &[("path", &settings_path.display().to_string())]
        )
    );
    Ok(())
}

/// Setup Continue to use Day1 Doctor gateway.
fn setup_continue(gw: &GatewayConfig) -> anyhow::Result<()> {
    let config_path = continue_config_path()?;
    backup_config(&config_path)?;
    write_continue_config(&config_path, gw)?;
    println!(
        "{}",
        crate::i18n::t_args(
            "gateway.setup.wrote_config",
            &[("path", &config_path.display().to_string())]
        )
    );
    Ok(())
}

/// Setup OpenClaw to use Day1 Doctor gateway.
fn setup_openclaw(gw: &GatewayConfig) -> anyhow::Result<()> {
    let config_path = openclaw_config_path()?;
    backup_config(&config_path)?;
    write_openclaw_config(&config_path, gw)?;
    println!(
        "{}",
        crate::i18n::t_args(
            "gateway.setup.wrote_config",
            &[("path", &config_path.display().to_string())]
        )
    );
    Ok(())
}

/// Print generic configuration instructions.
fn print_generic(gw: &GatewayConfig) {
    println!();
    println!("{}", crate::i18n::t("gateway.setup.generic_instructions"));
    println!();
    println!("  API Base URL: {}", gw.api_base);
    println!("  API Key:      {}", gw.api_key);
    println!();
    println!("{}", crate::i18n::t("gateway.setup.generic_env_hint"));
    println!();
    println!("  export OPENAI_API_BASE={}", gw.api_base);
    println!("  export OPENAI_API_KEY={}", gw.api_key);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_app_from_str() {
        assert_eq!(SupportedApp::from_str("cursor"), Some(SupportedApp::Cursor));
        assert_eq!(SupportedApp::from_str("Cursor"), Some(SupportedApp::Cursor));
        assert_eq!(SupportedApp::from_str("CURSOR"), Some(SupportedApp::Cursor));
        assert_eq!(
            SupportedApp::from_str("continue"),
            Some(SupportedApp::Continue)
        );
        assert_eq!(
            SupportedApp::from_str("Continue"),
            Some(SupportedApp::Continue)
        );
        assert_eq!(
            SupportedApp::from_str("openclaw"),
            Some(SupportedApp::OpenClaw)
        );
        assert_eq!(
            SupportedApp::from_str("OpenClaw"),
            Some(SupportedApp::OpenClaw)
        );
        assert_eq!(
            SupportedApp::from_str("generic"),
            Some(SupportedApp::Generic)
        );
        assert_eq!(
            SupportedApp::from_str("--generic"),
            Some(SupportedApp::Generic)
        );
        assert_eq!(SupportedApp::from_str("unknown"), None);
        assert_eq!(SupportedApp::from_str(""), None);
    }

    #[test]
    fn test_derive_gateway_url() {
        assert_eq!(
            derive_gateway_url("wss://api.day1doctor.com/ws"),
            "https://api.day1doctor.com"
        );
        assert_eq!(
            derive_gateway_url("ws://localhost:8000/ws"),
            "http://localhost:8000"
        );
        assert_eq!(
            derive_gateway_url("wss://api.example.com"),
            "https://api.example.com"
        );
        assert_eq!(
            derive_gateway_url("wss://api.day1doctor.com/ws/v2"),
            "https://api.day1doctor.com"
        );
        // Edge case: URL with no scheme change
        assert_eq!(
            derive_gateway_url("https://api.day1doctor.com"),
            "https://api.day1doctor.com"
        );
    }

    #[test]
    fn test_display_name() {
        assert_eq!(SupportedApp::Cursor.display_name(), "Cursor");
        assert_eq!(SupportedApp::Continue.display_name(), "Continue");
        assert_eq!(SupportedApp::OpenClaw.display_name(), "OpenClaw");
        assert_eq!(SupportedApp::Generic.display_name(), "Generic");
    }

    // ─── Config writing tests ───────────────────────────────────────────────

    fn test_gw_config() -> GatewayConfig {
        GatewayConfig {
            api_base: "https://api.day1doctor.com/v1".to_string(),
            api_key: "d1d_sk_test123".to_string(),
        }
    }

    #[test]
    fn test_write_cursor_config_fresh() {
        let dir = tempfile::tempdir().unwrap();
        let settings_path = dir.path().join("settings.json");
        let gw = test_gw_config();

        write_cursor_config(&settings_path, &gw).unwrap();

        let content = std::fs::read_to_string(&settings_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(
            parsed["openai.apiBase"].as_str().unwrap(),
            "https://api.day1doctor.com/v1"
        );
        assert_eq!(parsed["openai.apiKey"].as_str().unwrap(), "d1d_sk_test123");
    }

    #[test]
    fn test_write_cursor_config_preserves_existing() {
        let dir = tempfile::tempdir().unwrap();
        let settings_path = dir.path().join("settings.json");

        // Write existing settings first
        let existing = serde_json::json!({
            "editor.fontSize": 14,
            "theme": "dark"
        });
        std::fs::write(
            &settings_path,
            serde_json::to_string_pretty(&existing).unwrap(),
        )
        .unwrap();

        let gw = test_gw_config();
        write_cursor_config(&settings_path, &gw).unwrap();

        let content = std::fs::read_to_string(&settings_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        // Gateway config added
        assert_eq!(
            parsed["openai.apiBase"].as_str().unwrap(),
            "https://api.day1doctor.com/v1"
        );
        // Existing settings preserved
        assert_eq!(parsed["editor.fontSize"].as_i64().unwrap(), 14);
        assert_eq!(parsed["theme"].as_str().unwrap(), "dark");
    }

    #[test]
    fn test_write_cursor_config_overwrites_old_gateway() {
        let dir = tempfile::tempdir().unwrap();
        let settings_path = dir.path().join("settings.json");

        // Write settings with old gateway config
        let existing = serde_json::json!({
            "openai.apiBase": "https://old-gateway.example.com/v1",
            "openai.apiKey": "old_key",
            "editor.fontSize": 14,
        });
        std::fs::write(
            &settings_path,
            serde_json::to_string_pretty(&existing).unwrap(),
        )
        .unwrap();

        let gw = test_gw_config();
        write_cursor_config(&settings_path, &gw).unwrap();

        let content = std::fs::read_to_string(&settings_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(
            parsed["openai.apiBase"].as_str().unwrap(),
            "https://api.day1doctor.com/v1"
        );
        assert_eq!(parsed["openai.apiKey"].as_str().unwrap(), "d1d_sk_test123");
    }

    #[test]
    fn test_write_cursor_config_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let settings_path = dir.path().join("nested").join("dir").join("settings.json");
        let gw = test_gw_config();

        write_cursor_config(&settings_path, &gw).unwrap();

        assert!(settings_path.exists());
    }

    #[test]
    fn test_write_cursor_config_handles_malformed_json() {
        let dir = tempfile::tempdir().unwrap();
        let settings_path = dir.path().join("settings.json");

        // Write malformed JSON
        std::fs::write(&settings_path, "not valid json {{{").unwrap();

        let gw = test_gw_config();
        // Should not error — falls back to empty object
        write_cursor_config(&settings_path, &gw).unwrap();

        let content = std::fs::read_to_string(&settings_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(
            parsed["openai.apiBase"].as_str().unwrap(),
            "https://api.day1doctor.com/v1"
        );
    }

    #[test]
    fn test_write_continue_config_fresh() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        let gw = test_gw_config();

        write_continue_config(&config_path, &gw).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        let models = parsed["models"].as_array().unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0]["title"].as_str().unwrap(), "Day1 Doctor Gateway");
        assert_eq!(models[0]["provider"].as_str().unwrap(), "openai");
        assert_eq!(
            models[0]["apiBase"].as_str().unwrap(),
            "https://api.day1doctor.com/v1"
        );
        assert_eq!(models[0]["apiKey"].as_str().unwrap(), "d1d_sk_test123");
    }

    #[test]
    fn test_write_continue_config_preserves_other_models() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");

        // Write existing config with another model
        let existing = serde_json::json!({
            "models": [
                {
                    "title": "My Custom Model",
                    "provider": "openai",
                    "model": "gpt-3.5-turbo",
                    "apiKey": "sk-custom"
                }
            ],
            "systemMessage": "You are helpful."
        });
        std::fs::write(
            &config_path,
            serde_json::to_string_pretty(&existing).unwrap(),
        )
        .unwrap();

        let gw = test_gw_config();
        write_continue_config(&config_path, &gw).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        let models = parsed["models"].as_array().unwrap();
        assert_eq!(models.len(), 2);
        // Original model preserved
        assert_eq!(models[0]["title"].as_str().unwrap(), "My Custom Model");
        // Day1 Doctor added
        assert_eq!(models[1]["title"].as_str().unwrap(), "Day1 Doctor Gateway");
        // Other config preserved
        assert_eq!(
            parsed["systemMessage"].as_str().unwrap(),
            "You are helpful."
        );
    }

    #[test]
    fn test_write_continue_config_replaces_existing_d1d_entry() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");

        // Write config with an old Day1 Doctor entry
        let existing = serde_json::json!({
            "models": [
                {
                    "title": "Day1 Doctor Gateway",
                    "provider": "openai",
                    "model": "gpt-4o",
                    "apiBase": "https://old.example.com/v1",
                    "apiKey": "old_key"
                },
                {
                    "title": "Another Model",
                    "provider": "anthropic",
                    "model": "claude-sonnet"
                }
            ]
        });
        std::fs::write(
            &config_path,
            serde_json::to_string_pretty(&existing).unwrap(),
        )
        .unwrap();

        let gw = test_gw_config();
        write_continue_config(&config_path, &gw).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        let models = parsed["models"].as_array().unwrap();
        // Old D1D entry removed, new one added; other model kept
        assert_eq!(models.len(), 2);
        assert_eq!(models[0]["title"].as_str().unwrap(), "Another Model");
        assert_eq!(models[1]["title"].as_str().unwrap(), "Day1 Doctor Gateway");
        assert_eq!(
            models[1]["apiBase"].as_str().unwrap(),
            "https://api.day1doctor.com/v1"
        );
    }

    #[test]
    fn test_write_openclaw_config_fresh() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        let gw = test_gw_config();

        write_openclaw_config(&config_path, &gw).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("api_base: \"https://api.day1doctor.com/v1\""));
        assert!(content.contains("api_key: \"d1d_sk_test123\""));
        assert!(content.contains("# Day1 Doctor Gateway configuration"));
    }

    #[test]
    fn test_write_openclaw_config_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir
            .path()
            .join("nested")
            .join("openclaw")
            .join("config.yaml");
        let gw = test_gw_config();

        write_openclaw_config(&config_path, &gw).unwrap();
        assert!(config_path.exists());
    }

    #[test]
    fn test_write_openclaw_config_is_valid_yaml_content() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        let gw = test_gw_config();

        write_openclaw_config(&config_path, &gw).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        // Verify it's valid YAML-like key:value structure
        let mut found_api_base = false;
        let mut found_api_key = false;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            assert!(
                line.contains(": "),
                "Non-comment line should be key: value format: {}",
                line
            );
            if line.starts_with("api_base:") {
                found_api_base = true;
            }
            if line.starts_with("api_key:") {
                found_api_key = true;
            }
        }
        assert!(found_api_base, "Should contain api_base key");
        assert!(found_api_key, "Should contain api_key key");
    }

    #[test]
    fn test_backup_config_creates_backup() {
        crate::i18n::init("en");
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("settings.json");

        // Write original
        std::fs::write(&config_path, r#"{"original": true}"#).unwrap();

        backup_config(&config_path).unwrap();

        // Check that a .bak file was created
        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 2, "Should have original + backup");

        let bak_entry = entries
            .iter()
            .find(|e| e.file_name().to_str().unwrap().ends_with(".bak"));
        assert!(bak_entry.is_some(), "Should have a .bak file");

        // Backup content should match original
        let bak_content = std::fs::read_to_string(bak_entry.unwrap().path()).unwrap();
        assert_eq!(bak_content, r#"{"original": true}"#);
    }

    #[test]
    fn test_backup_config_no_file_noop() {
        crate::i18n::init("en");
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("nonexistent.json");

        // Should not error when file doesn't exist
        backup_config(&config_path).unwrap();
    }

    #[test]
    fn test_gateway_config_serialization() {
        let gw = GatewayConfig {
            api_base: "https://api.example.com/v1".to_string(),
            api_key: "d1d_sk_abc123".to_string(),
        };
        let json = serde_json::to_string(&gw).unwrap();
        let parsed: GatewayConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.api_base, gw.api_base);
        assert_eq!(parsed.api_key, gw.api_key);
    }
}
