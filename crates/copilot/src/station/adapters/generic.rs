use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::adapter_trait::{AgentDescriptor, FrameworkAdapter};

/// Configuration for a generic OpenAI-compatible endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericAdapterConfig {
    /// The base URL of the OpenAI-compatible API (e.g. "https://api.openai.com/v1").
    pub endpoint_url: String,
    /// API key for authentication.
    pub api_key: String,
    /// Optional human-readable label for this endpoint.
    pub label: Option<String>,
}

/// Adapter for any OpenAI-compatible endpoint.
///
/// This is a stub implementation. The real version will perform HTTP requests
/// to the configured endpoint (chat completions, models listing, etc.).
pub struct GenericAdapter {
    config: GenericAdapterConfig,
}

impl GenericAdapter {
    /// Create a new adapter from configuration.
    pub fn new(config: GenericAdapterConfig) -> Self {
        Self { config }
    }

    /// Return the configured endpoint URL.
    pub fn endpoint_url(&self) -> &str {
        &self.config.endpoint_url
    }

    /// Return the configured label (falls back to the endpoint URL).
    pub fn label(&self) -> &str {
        self.config
            .label
            .as_deref()
            .unwrap_or(&self.config.endpoint_url)
    }
}

#[async_trait]
impl FrameworkAdapter for GenericAdapter {
    async fn discover(&self) -> Vec<AgentDescriptor> {
        // A generic endpoint always advertises one agent descriptor
        // representing the remote model.
        vec![AgentDescriptor {
            id: format!("generic-{}", sanitize_id(&self.config.endpoint_url)),
            name: self.label().to_string(),
            adapter_type: "generic".to_string(),
            version: None,
        }]
    }

    async fn health_check(&self) -> bool {
        // Stub: in production this would GET /v1/models to verify reachability.
        // For now, just check that the endpoint URL is non-empty and starts
        // with http.
        !self.config.endpoint_url.is_empty()
            && (self.config.endpoint_url.starts_with("http://")
                || self.config.endpoint_url.starts_with("https://"))
    }

    fn adapter_type(&self) -> &str {
        "generic"
    }
}

/// Produce a safe ID from a URL by keeping only alphanumeric chars and dashes.
fn sanitize_id(url: &str) -> String {
    url.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> GenericAdapterConfig {
        GenericAdapterConfig {
            endpoint_url: "https://api.openai.com/v1".to_string(),
            api_key: "sk-test-key".to_string(),
            label: Some("OpenAI".to_string()),
        }
    }

    #[tokio::test]
    async fn test_discover_returns_descriptor() {
        let adapter = GenericAdapter::new(make_config());
        let agents = adapter.discover().await;

        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].adapter_type, "generic");
        assert_eq!(agents[0].name, "OpenAI");
        assert!(agents[0].id.starts_with("generic-"));
    }

    #[tokio::test]
    async fn test_discover_without_label_uses_url() {
        let config = GenericAdapterConfig {
            endpoint_url: "https://api.example.com/v1".to_string(),
            api_key: "key".to_string(),
            label: None,
        };
        let adapter = GenericAdapter::new(config);
        let agents = adapter.discover().await;

        assert_eq!(agents[0].name, "https://api.example.com/v1");
    }

    #[tokio::test]
    async fn test_health_check_valid_url() {
        let adapter = GenericAdapter::new(make_config());
        assert!(adapter.health_check().await);
    }

    #[tokio::test]
    async fn test_health_check_empty_url() {
        let config = GenericAdapterConfig {
            endpoint_url: "".to_string(),
            api_key: "key".to_string(),
            label: None,
        };
        let adapter = GenericAdapter::new(config);
        assert!(!adapter.health_check().await);
    }

    #[tokio::test]
    async fn test_health_check_invalid_scheme() {
        let config = GenericAdapterConfig {
            endpoint_url: "ftp://example.com".to_string(),
            api_key: "key".to_string(),
            label: None,
        };
        let adapter = GenericAdapter::new(config);
        assert!(!adapter.health_check().await);
    }

    #[tokio::test]
    async fn test_adapter_type() {
        let adapter = GenericAdapter::new(make_config());
        assert_eq!(adapter.adapter_type(), "generic");
    }

    #[tokio::test]
    async fn test_endpoint_url_getter() {
        let adapter = GenericAdapter::new(make_config());
        assert_eq!(adapter.endpoint_url(), "https://api.openai.com/v1");
    }

    #[tokio::test]
    async fn test_label_getter_with_label() {
        let adapter = GenericAdapter::new(make_config());
        assert_eq!(adapter.label(), "OpenAI");
    }

    #[tokio::test]
    async fn test_label_getter_fallback() {
        let config = GenericAdapterConfig {
            endpoint_url: "https://api.example.com/v1".to_string(),
            api_key: "key".to_string(),
            label: None,
        };
        let adapter = GenericAdapter::new(config);
        assert_eq!(adapter.label(), "https://api.example.com/v1");
    }

    #[test]
    fn test_sanitize_id() {
        assert_eq!(
            sanitize_id("https://api.openai.com/v1"),
            "httpsapiopenaicomv1"
        );
        assert_eq!(sanitize_id("my-endpoint"), "my-endpoint");
    }
}
