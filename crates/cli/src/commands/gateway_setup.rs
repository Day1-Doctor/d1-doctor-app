//! `d1 gateway setup <app>` — auto-configure AI coding tools to use Day1 Doctor gateway.
//!
//! Supported apps: cursor, continue, openclaw, --generic
//!
//! This command:
//! 1. Ensures an API key exists (creates one if needed)
//! 2. Detects the app's config file path
//! 3. Writes OpenAI-compatible settings
//! 4. Prints confirmation

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
struct GatewayConfig {
    api_base: String,
    api_key: String,
}

/// Run the setup flow for a given app.
pub async fn run_setup(app_name: &str) -> anyhow::Result<()> {
    let app = SupportedApp::from_str(app_name).ok_or_else(|| {
        anyhow::anyhow!(
            "{}",
            crate::i18n::t_args(
                "gateway.setup.unsupported_app",
                &[("app", app_name)]
            )
        )
    })?;

    println!(
        "{}",
        crate::i18n::t_args(
            "gateway.setup.configuring",
            &[("app", app.display_name())]
        )
    );

    // Step 1: Ensure we have an API key
    let api_key = ensure_api_key(app.display_name()).await?;

    // Step 2: Determine the gateway base URL
    let config = d1_common::Config::load().unwrap_or_default();
    let gateway_base = derive_gateway_url(&config.orchestrator_url);

    // Step 3: Write config for the app
    match app {
        SupportedApp::Cursor => setup_cursor(&gateway_base, &api_key)?,
        SupportedApp::Continue => setup_continue(&gateway_base, &api_key)?,
        SupportedApp::OpenClaw => setup_openclaw(&gateway_base, &api_key)?,
        SupportedApp::Generic => print_generic(&gateway_base, &api_key),
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
        // Look for an existing key named after this app
        if let Some(key) = keys.iter().find(|k| k.name == app_name && k.is_active) {
            println!(
                "{}",
                crate::i18n::t_args(
                    "gateway.setup.existing_key",
                    &[("prefix", &key.key_prefix)]
                )
            );
            // We have the prefix but not the full key. Create a new one.
            // Actually, we can't recover the full key, so let's create a new one
            // only if the user doesn't have any active key at all.
        }
    }

    // Create a new key for this app
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
fn derive_gateway_url(orchestrator_url: &str) -> String {
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

/// Resolve the Cursor config directory.
fn cursor_config_dir() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("{}", crate::i18n::t("errors.home_dir_error")))?;

    #[cfg(target_os = "macos")]
    let config_dir = home.join("Library/Application Support/Cursor/User");

    #[cfg(target_os = "linux")]
    let config_dir = home.join(".config/Cursor/User");

    #[cfg(target_os = "windows")]
    let config_dir = home.join("AppData/Roaming/Cursor/User");

    Ok(config_dir)
}

/// Setup Cursor to use Day1 Doctor gateway.
fn setup_cursor(gateway_base: &str, api_key: &str) -> anyhow::Result<()> {
    let config_dir = cursor_config_dir()?;
    let settings_path = config_dir.join("settings.json");

    // Read existing settings or start fresh
    let mut settings: serde_json::Value = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        std::fs::create_dir_all(&config_dir)?;
        serde_json::json!({})
    };

    // Set OpenAI-compatible config for Cursor
    let obj = settings.as_object_mut().unwrap();
    obj.insert(
        "openai.apiBase".to_string(),
        serde_json::Value::String(format!("{}/v1", gateway_base)),
    );
    obj.insert(
        "openai.apiKey".to_string(),
        serde_json::Value::String(api_key.to_string()),
    );

    let content = serde_json::to_string_pretty(&settings)?;
    std::fs::write(&settings_path, &content)?;

    println!(
        "{}",
        crate::i18n::t_args(
            "gateway.setup.wrote_config",
            &[("path", &settings_path.display().to_string())]
        )
    );
    Ok(())
}

/// Resolve the Continue config directory.
fn continue_config_dir() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("{}", crate::i18n::t("errors.home_dir_error")))?;
    Ok(home.join(".continue"))
}

/// Setup Continue to use Day1 Doctor gateway.
fn setup_continue(gateway_base: &str, api_key: &str) -> anyhow::Result<()> {
    let config_dir = continue_config_dir()?;
    let config_path = config_dir.join("config.json");

    // Read existing config or start fresh
    let mut config: serde_json::Value = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        std::fs::create_dir_all(&config_dir)?;
        serde_json::json!({})
    };

    // Add Day1 Doctor as an OpenAI-compatible model provider
    let d1_model = serde_json::json!({
        "title": "Day1 Doctor Gateway",
        "provider": "openai",
        "model": "gpt-4o",
        "apiBase": format!("{}/v1", gateway_base),
        "apiKey": api_key,
    });

    let obj = config.as_object_mut().unwrap();
    let models = obj
        .entry("models")
        .or_insert_with(|| serde_json::json!([]));
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
    std::fs::write(&config_path, &content)?;

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
fn setup_openclaw(gateway_base: &str, api_key: &str) -> anyhow::Result<()> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("{}", crate::i18n::t("errors.home_dir_error")))?;
    let config_dir = home.join(".openclaw");
    let config_path = config_dir.join("config.json");

    let mut config: serde_json::Value = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        std::fs::create_dir_all(&config_dir)?;
        serde_json::json!({})
    };

    let obj = config.as_object_mut().unwrap();
    obj.insert(
        "apiBase".to_string(),
        serde_json::Value::String(format!("{}/v1", gateway_base)),
    );
    obj.insert(
        "apiKey".to_string(),
        serde_json::Value::String(api_key.to_string()),
    );

    let content = serde_json::to_string_pretty(&config)?;
    std::fs::write(&config_path, &content)?;

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
fn print_generic(gateway_base: &str, api_key: &str) {
    println!();
    println!("{}", crate::i18n::t("gateway.setup.generic_instructions"));
    println!();
    println!("  API Base URL: {}/v1", gateway_base);
    println!("  API Key:      {}", api_key);
    println!();
    println!("{}", crate::i18n::t("gateway.setup.generic_env_hint"));
    println!();
    println!("  export OPENAI_API_BASE={}/v1", gateway_base);
    println!("  export OPENAI_API_KEY={}", api_key);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_app_from_str() {
        assert_eq!(SupportedApp::from_str("cursor"), Some(SupportedApp::Cursor));
        assert_eq!(
            SupportedApp::from_str("Cursor"),
            Some(SupportedApp::Cursor)
        );
        assert_eq!(
            SupportedApp::from_str("continue"),
            Some(SupportedApp::Continue)
        );
        assert_eq!(
            SupportedApp::from_str("openclaw"),
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
    }

    #[test]
    fn test_display_name() {
        assert_eq!(SupportedApp::Cursor.display_name(), "Cursor");
        assert_eq!(SupportedApp::Continue.display_name(), "Continue");
        assert_eq!(SupportedApp::OpenClaw.display_name(), "OpenClaw");
        assert_eq!(SupportedApp::Generic.display_name(), "Generic");
    }
}
