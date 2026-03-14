//! `d1 gateway` — gateway health, model listing, key management, balance, usage, and app setup.

use serde::{Deserialize, Serialize};

/// Health status of the AI gateway.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GatewayHealth {
    Healthy,
    Degraded,
    Offline,
}

/// A single LLM model exposed through the gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayModel {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub input_price_per_1k: f64,
    pub output_price_per_1k: f64,
    pub context_window: u32,
    pub available: bool,
}

/// Snapshot returned by [`collect_gateway_status`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayStatus {
    pub health: GatewayHealth,
    pub endpoint: String,
    pub model_count: usize,
}

/// Model info returned by the platform API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformModel {
    pub alias: String,
    pub provider: String,
    #[serde(default)]
    pub provider_model_id: Option<String>,
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub route_type: Option<String>,
    #[serde(default)]
    pub dd_input_per_1m: Option<f64>,
    #[serde(default)]
    pub dd_output_per_1m: Option<f64>,
    #[serde(default)]
    pub capabilities: Option<Vec<String>>,
    #[serde(default)]
    pub max_context_window: Option<i64>,
    #[serde(default)]
    pub is_public: Option<bool>,
}

/// Usage summary from the platform API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub dd_balance: f64,
    pub total_dd_consumed: f64,
    pub total_requests: i64,
    #[serde(default)]
    pub records: Vec<UsageRecord>,
}

/// A single usage record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    #[serde(default)]
    pub model_requested: Option<String>,
    #[serde(default)]
    pub tokens_input: Option<i64>,
    #[serde(default)]
    pub tokens_output: Option<i64>,
    #[serde(default)]
    pub dd_consumed: Option<f64>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Return the built-in model catalogue.
pub fn builtin_models() -> Vec<GatewayModel> {
    vec![
        GatewayModel {
            id: "gpt-4o".into(),
            name: "GPT-4o".into(),
            provider: "OpenAI".into(),
            input_price_per_1k: 0.005,
            output_price_per_1k: 0.015,
            context_window: 128_000,
            available: true,
        },
        GatewayModel {
            id: "gpt-4o-mini".into(),
            name: "GPT-4o Mini".into(),
            provider: "OpenAI".into(),
            input_price_per_1k: 0.00015,
            output_price_per_1k: 0.0006,
            context_window: 128_000,
            available: true,
        },
        GatewayModel {
            id: "claude-sonnet-4-20250514".into(),
            name: "Claude Sonnet 4".into(),
            provider: "Anthropic".into(),
            input_price_per_1k: 0.003,
            output_price_per_1k: 0.015,
            context_window: 200_000,
            available: true,
        },
        GatewayModel {
            id: "claude-haiku-3-5".into(),
            name: "Claude 3.5 Haiku".into(),
            provider: "Anthropic".into(),
            input_price_per_1k: 0.0008,
            output_price_per_1k: 0.004,
            context_window: 200_000,
            available: true,
        },
    ]
}

/// Collect gateway health (offline when daemon is not running).
pub fn collect_gateway_status() -> GatewayStatus {
    let config = d1_common::Config::load().unwrap_or_default();
    let models = builtin_models();

    GatewayStatus {
        health: GatewayHealth::Offline,
        endpoint: config.orchestrator_url.clone(),
        model_count: models.len(),
    }
}

/// Print gateway health to stdout.
pub fn print_gateway_status(status: &GatewayStatus) {
    let health_label = match status.health {
        GatewayHealth::Healthy => crate::i18n::t("gateway.health_healthy"),
        GatewayHealth::Degraded => crate::i18n::t("gateway.health_degraded"),
        GatewayHealth::Offline => crate::i18n::t("gateway.health_offline"),
    };

    println!("{}", crate::i18n::t("gateway.title"));
    println!();
    println!(
        "{}",
        crate::i18n::t_args("gateway.health_label", &[("health", &health_label)])
    );
    println!(
        "{}",
        crate::i18n::t_args("gateway.endpoint_label", &[("endpoint", &status.endpoint)])
    );
    println!(
        "{}",
        crate::i18n::t_args(
            "gateway.models_label",
            &[("count", &status.model_count.to_string())]
        )
    );
}

/// Print the model catalogue to stdout.
pub fn print_models(models: &[GatewayModel]) {
    println!(
        "{:<28} {:<12} {:>12} {:>12} {:>10}",
        crate::i18n::t("gateway.table.model"),
        crate::i18n::t("gateway.table.provider"),
        crate::i18n::t("gateway.table.input_price"),
        crate::i18n::t("gateway.table.output_price"),
        crate::i18n::t("gateway.table.context"),
    );
    println!("{}", "-".repeat(78));

    for m in models {
        let status = if m.available {
            String::new()
        } else {
            crate::i18n::t("gateway.table.offline_suffix")
        };
        println!(
            "{:<28} {:<12} {:>11.5} {:>11.5} {:>9}k",
            format!("{}{}", m.name, status),
            m.provider,
            m.input_price_per_1k,
            m.output_price_per_1k,
            m.context_window / 1000,
        );
    }
}

// ─── Shared helpers ─────────────────────────────────────────────────────────

/// Resolve `~/.d1-doctor/credentials.json`.
fn credentials_path() -> anyhow::Result<std::path::PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| anyhow::anyhow!("{}", crate::i18n::t("errors.home_dir_error")))?;
    Ok(std::path::PathBuf::from(home)
        .join(".d1-doctor")
        .join("credentials.json"))
}

/// Read the JWT access token from local credentials.
fn load_access_token() -> anyhow::Result<String> {
    let path = credentials_path()?;
    let content = std::fs::read_to_string(&path)
        .map_err(|_| anyhow::anyhow!("{}", crate::i18n::t("auth.no_credentials")))?;
    let creds: serde_json::Value = serde_json::from_str(&content)?;
    creds
        .get("access_token")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| anyhow::anyhow!("Invalid credentials file: missing access_token"))
}

/// Derive the platform REST API base URL from the orchestrator WebSocket URL.
pub fn platform_api_base() -> String {
    let config = d1_common::Config::load().unwrap_or_default();
    let url = config
        .orchestrator_url
        .replace("wss://", "https://")
        .replace("ws://", "http://");
    // Strip trailing path like /ws
    if let Some(pos) = url.find("/ws") {
        url[..pos].to_string()
    } else {
        url
    }
}

/// Build a reqwest client, platform base URL, and JWT token for API calls.
pub fn api_client() -> anyhow::Result<(reqwest::Client, String, String)> {
    let token = load_access_token()?;
    let base_url = platform_api_base();
    let client = reqwest::Client::new();
    Ok((client, base_url, token))
}

// ─── Command runners ────────────────────────────────────────────────────────

pub async fn run_status() -> anyhow::Result<()> {
    let status = collect_gateway_status();
    print_gateway_status(&status);
    Ok(())
}

pub async fn run_models_list() -> anyhow::Result<()> {
    // Try to fetch from platform API first; fall back to built-in catalogue.
    match fetch_models_from_api().await {
        Ok(models) => print_platform_models(&models),
        Err(_) => {
            let models = builtin_models();
            print_models(&models);
        }
    }
    Ok(())
}

/// Fetch models from the platform API and print detailed info for a single model.
pub async fn run_models_info(alias: &str) -> anyhow::Result<()> {
    let models = fetch_models_from_api().await?;
    let model = models.iter().find(|m| m.alias == alias);
    match model {
        Some(m) => print_model_detail(m),
        None => {
            anyhow::bail!(
                "{}",
                crate::i18n::t_args("gateway.models_info.not_found", &[("alias", alias)])
            );
        }
    }
    Ok(())
}

/// Show DD credit balance.
pub async fn run_balance() -> anyhow::Result<()> {
    let (client, base_url, token) = api_client()?;

    let resp = client
        .get(format!("{}/api/v1/usage?days=1", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!(
            "{}",
            crate::i18n::t_args(
                "gateway.balance.fetch_failed",
                &[("status", &status.to_string()), ("body", &body)]
            )
        );
    }

    let summary: UsageSummary = resp.json().await?;

    println!("{}", crate::i18n::t("gateway.balance.title"));
    println!();
    println!(
        "{}",
        crate::i18n::t_args(
            "gateway.balance.dd_balance",
            &[("amount", &format!("{:.2}", summary.dd_balance))]
        )
    );
    Ok(())
}

/// Open the top-up page in the browser.
pub async fn run_topup() -> anyhow::Result<()> {
    let base_url = platform_api_base();
    let topup_url = format!("{}/dashboard/billing", base_url);

    println!(
        "{}",
        crate::i18n::t_args("gateway.topup.opening", &[("url", &topup_url)])
    );

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&topup_url).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&topup_url)
            .spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", &topup_url])
            .spawn();
    }

    println!("{}", crate::i18n::t("gateway.topup.hint"));
    Ok(())
}

/// Show usage summary for the last N days.
pub async fn run_usage(days: u32) -> anyhow::Result<()> {
    let (client, base_url, token) = api_client()?;

    let resp = client
        .get(format!("{}/api/v1/usage?days={}", base_url, days))
        .header("Authorization", format!("Bearer {}", token))
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!(
            "{}",
            crate::i18n::t_args(
                "gateway.usage.fetch_failed",
                &[("status", &status.to_string()), ("body", &body)]
            )
        );
    }

    let summary: UsageSummary = resp.json().await?;

    println!(
        "{}",
        crate::i18n::t_args("gateway.usage.title", &[("days", &days.to_string())])
    );
    println!();
    println!(
        "{}",
        crate::i18n::t_args(
            "gateway.usage.total_requests",
            &[("count", &summary.total_requests.to_string())]
        )
    );
    println!(
        "{}",
        crate::i18n::t_args(
            "gateway.usage.total_dd",
            &[("amount", &format!("{:.2}", summary.total_dd_consumed))]
        )
    );
    println!(
        "{}",
        crate::i18n::t_args(
            "gateway.balance.dd_balance",
            &[("amount", &format!("{:.2}", summary.dd_balance))]
        )
    );

    if !summary.records.is_empty() {
        println!();
        println!(
            "  {:<20} {:<20} {:>10} {:>10} {:>10}",
            crate::i18n::t("gateway.usage.col_date"),
            crate::i18n::t("gateway.usage.col_model"),
            crate::i18n::t("gateway.usage.col_input"),
            crate::i18n::t("gateway.usage.col_output"),
            crate::i18n::t("gateway.usage.col_dd"),
        );
        println!("  {}", "-".repeat(74));

        for r in &summary.records {
            let date = r
                .created_at
                .as_deref()
                .unwrap_or("-")
                .split('T')
                .next()
                .unwrap_or("-");
            let model = r.model_requested.as_deref().unwrap_or("-");
            let input = r.tokens_input.unwrap_or(0);
            let output = r.tokens_output.unwrap_or(0);
            let dd = r.dd_consumed.unwrap_or(0.0);
            println!(
                "  {:<20} {:<20} {:>10} {:>10} {:>9.2}",
                date, model, input, output, dd
            );
        }
    }

    Ok(())
}

// ─── Platform API helpers ───────────────────────────────────────────────────

/// Fetch the model list from the platform API.
async fn fetch_models_from_api() -> anyhow::Result<Vec<PlatformModel>> {
    let (client, base_url, token) = api_client()?;

    let resp = client
        .get(format!("{}/v1/models", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("Failed to fetch models from API");
    }

    // The OpenAI-compatible endpoint returns { "data": [...] }
    let body: serde_json::Value = resp.json().await?;

    // Try OpenAI format first, then fall back to plain array
    if let Some(data) = body.get("data") {
        let models: Vec<PlatformModel> = serde_json::from_value(data.clone())?;
        Ok(models)
    } else if body.is_array() {
        let models: Vec<PlatformModel> = serde_json::from_value(body)?;
        Ok(models)
    } else {
        anyhow::bail!("Unexpected models response format");
    }
}

/// Print platform models in a table.
fn print_platform_models(models: &[PlatformModel]) {
    println!(
        "{:<20} {:<12} {:<10} {:>14} {:>14} {:>10}",
        crate::i18n::t("gateway.table.model"),
        crate::i18n::t("gateway.table.provider"),
        "Tier",
        "DD In/1M",
        "DD Out/1M",
        crate::i18n::t("gateway.table.context"),
    );
    println!("{}", "-".repeat(84));

    for m in models {
        let tier = m.tier.as_deref().unwrap_or("-");
        let dd_in = m
            .dd_input_per_1m
            .map(|v| format!("{:.0}", v))
            .unwrap_or_else(|| "-".to_string());
        let dd_out = m
            .dd_output_per_1m
            .map(|v| format!("{:.0}", v))
            .unwrap_or_else(|| "-".to_string());
        let ctx = m
            .max_context_window
            .map(|v| format!("{}k", v / 1000))
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<20} {:<12} {:<10} {:>14} {:>14} {:>10}",
            m.alias, m.provider, tier, dd_in, dd_out, ctx
        );
    }
}

/// Print detailed info for a single model.
fn print_model_detail(model: &PlatformModel) {
    println!();
    println!(
        "{}",
        crate::i18n::t_args("gateway.models_info.title", &[("alias", &model.alias)])
    );
    println!();
    println!("  Provider:        {}", model.provider);
    if let Some(ref id) = model.provider_model_id {
        println!("  Model ID:        {}", id);
    }
    if let Some(ref tier) = model.tier {
        println!("  Tier:            {}", tier);
    }
    if let Some(ref route) = model.route_type {
        println!("  Route Type:      {}", route);
    }
    if let Some(dd_in) = model.dd_input_per_1m {
        println!("  DD Input/1M:     {:.0}", dd_in);
    }
    if let Some(dd_out) = model.dd_output_per_1m {
        println!("  DD Output/1M:    {:.0}", dd_out);
    }
    if let Some(ctx) = model.max_context_window {
        println!("  Context Window:  {}k", ctx / 1000);
    }
    if let Some(ref caps) = model.capabilities {
        println!("  Capabilities:    {}", caps.join(", "));
    }
    if let Some(ref status) = model.status {
        println!("  Status:          {}", status);
    }
    if let Some(public) = model.is_public {
        println!("  Public:          {}", if public { "yes" } else { "no" });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_models_not_empty() {
        assert!(!builtin_models().is_empty());
    }

    #[test]
    fn test_builtin_models_all_available() {
        for m in builtin_models() {
            assert!(m.available, "model {} should be available", m.id);
        }
    }

    #[test]
    fn test_model_prices_positive() {
        for m in builtin_models() {
            assert!(m.input_price_per_1k > 0.0, "{} input price", m.id);
            assert!(m.output_price_per_1k > 0.0, "{} output price", m.id);
        }
    }

    #[test]
    fn test_collect_gateway_status_offline() {
        let status = collect_gateway_status();
        assert_eq!(status.health, GatewayHealth::Offline);
        assert_eq!(status.model_count, builtin_models().len());
    }

    #[test]
    fn test_print_gateway_status_does_not_panic() {
        crate::i18n::init("en");
        let status = GatewayStatus {
            health: GatewayHealth::Healthy,
            endpoint: "https://example.com".into(),
            model_count: 4,
        };
        print_gateway_status(&status);
    }

    #[test]
    fn test_print_models_does_not_panic() {
        crate::i18n::init("en");
        print_models(&builtin_models());
    }

    #[test]
    fn test_gateway_model_serialization() {
        let models = builtin_models();
        let json = serde_json::to_string(&models).unwrap();
        let parsed: Vec<GatewayModel> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), models.len());
    }

    #[test]
    fn test_platform_api_base_wss() {
        // This test verifies the URL derivation logic indirectly.
        // The actual function reads from config, so we test the logic pattern.
        let url = "wss://gateway.day1.doctor/ws"
            .replace("wss://", "https://")
            .replace("ws://", "http://");
        let result = if let Some(pos) = url.find("/ws") {
            url[..pos].to_string()
        } else {
            url
        };
        assert_eq!(result, "https://gateway.day1.doctor");
    }

    #[test]
    fn test_platform_model_deserialization() {
        let json = r#"{
            "alias": "gpt-4o",
            "provider": "openai",
            "provider_model_id": "gpt-4o",
            "tier": "standard",
            "status": "active",
            "route_type": "external_llm",
            "dd_input_per_1m": 325.0,
            "dd_output_per_1m": 1300.0,
            "capabilities": ["chat", "vision"],
            "max_context_window": 128000,
            "is_public": true
        }"#;
        let model: PlatformModel = serde_json::from_str(json).unwrap();
        assert_eq!(model.alias, "gpt-4o");
        assert_eq!(model.dd_input_per_1m, Some(325.0));
    }

    #[test]
    fn test_usage_summary_deserialization() {
        let json = r#"{
            "dd_balance": 842.5,
            "total_dd_consumed": 57.5,
            "total_requests": 42,
            "records": []
        }"#;
        let summary: UsageSummary = serde_json::from_str(json).unwrap();
        assert_eq!(summary.dd_balance, 842.5);
        assert_eq!(summary.total_requests, 42);
    }

    #[test]
    fn test_print_platform_models_does_not_panic() {
        crate::i18n::init("en");
        let models = vec![PlatformModel {
            alias: "gpt-4o".into(),
            provider: "openai".into(),
            provider_model_id: Some("gpt-4o".into()),
            tier: Some("standard".into()),
            status: Some("active".into()),
            route_type: Some("external_llm".into()),
            dd_input_per_1m: Some(325.0),
            dd_output_per_1m: Some(1300.0),
            capabilities: Some(vec!["chat".into()]),
            max_context_window: Some(128000),
            is_public: Some(true),
        }];
        print_platform_models(&models);
    }

    #[test]
    fn test_print_model_detail_does_not_panic() {
        crate::i18n::init("en");
        let model = PlatformModel {
            alias: "gpt-4o".into(),
            provider: "openai".into(),
            provider_model_id: Some("gpt-4o".into()),
            tier: Some("standard".into()),
            status: Some("active".into()),
            route_type: Some("external_llm".into()),
            dd_input_per_1m: Some(325.0),
            dd_output_per_1m: Some(1300.0),
            capabilities: Some(vec!["chat".into(), "vision".into()]),
            max_context_window: Some(128000),
            is_public: Some(true),
        };
        print_model_detail(&model);
    }

    #[test]
    fn test_credentials_path_not_empty() {
        let path = credentials_path();
        assert!(path.is_ok());
        let p = path.unwrap();
        assert!(p.to_str().unwrap().contains(".d1-doctor"));
        assert!(p.to_str().unwrap().contains("credentials.json"));
    }
}
