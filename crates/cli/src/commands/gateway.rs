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
///
/// In future this will be fetched from the gateway API; for now we keep a
/// static list so the CLI is usable offline.
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

    // Without a real HTTP call we report Offline; a future version will probe
    // the daemon's /gateway/health endpoint.
    GatewayStatus {
        health: GatewayHealth::Offline,
        endpoint: config.orchestrator_url.clone(),
        model_count: models.len(),
    }
}

/// Print gateway health to stdout.
pub fn print_gateway_status(status: &GatewayStatus) {
    let health_label = match status.health {
        GatewayHealth::Healthy => "Healthy",
        GatewayHealth::Degraded => "Degraded",
        GatewayHealth::Offline => "Offline",
    };

    println!("Gateway Status");
    println!();
    println!("  Health:   {}", health_label);
    println!("  Endpoint: {}", status.endpoint);
    println!("  Models:   {} available", status.model_count);
}

/// Print the model catalogue to stdout.
pub fn print_models(models: &[GatewayModel]) {
    println!(
        "{:<28} {:<12} {:>12} {:>12} {:>10}",
        "Model", "Provider", "Input/1K", "Output/1K", "Context"
    );
    println!("{}", "-".repeat(78));

    for m in models {
        let status = if m.available { "" } else { " (offline)" };
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

/// `d1 gateway status`
pub async fn run_status() -> anyhow::Result<()> {
    let status = collect_gateway_status();
    print_gateway_status(&status);
    Ok(())
}

/// `d1 gateway models`
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
        let models = builtin_models();
        assert!(!models.is_empty());
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
    fn test_context_windows_positive() {
        for m in builtin_models() {
            assert!(m.context_window > 0, "{} context window", m.id);
        }
    }

    #[test]
    fn test_collect_gateway_status_offline() {
        let status = collect_gateway_status();
        // Without a running daemon the gateway is offline.
        assert_eq!(status.health, GatewayHealth::Offline);
        assert_eq!(status.model_count, builtin_models().len());
    }

    #[test]
    fn test_print_gateway_status_does_not_panic() {
        let status = GatewayStatus {
            health: GatewayHealth::Healthy,
            endpoint: "https://example.com".into(),
            model_count: 4,
        };
        print_gateway_status(&status);
    }

    #[test]
    fn test_print_models_does_not_panic() {
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
