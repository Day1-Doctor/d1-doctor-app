//! `d1 gateway` — gateway health and model listing.

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

pub async fn run_status() -> anyhow::Result<()> {
    let status = collect_gateway_status();
    print_gateway_status(&status);
    Ok(())
}

pub async fn run_models() -> anyhow::Result<()> {
    let models = builtin_models();
    print_models(&models);
    Ok(())
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
}
