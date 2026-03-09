//! WebSocket client for communication with the cloud orchestrator.
//!
//! All outbound messages pass through the [`Redactor`] before being sent
//! to the cloud, ensuring sensitive data is stripped.

use crate::redactor::Redactor;

pub struct WsClient {
    redactor: Redactor,
    // TODO: WebSocket connection state
}

impl WsClient {
    /// Create a new client with the given redactor.
    pub fn new(redactor: Redactor) -> Self {
        Self { redactor }
    }

    pub async fn connect(_url: &str, redactor: Redactor) -> anyhow::Result<Self> {
        // TODO: Establish WebSocket connection to orchestrator
        Ok(Self { redactor })
    }

    /// Redact a text message before sending to the cloud.
    ///
    /// This is called automatically for every outbound message.
    pub fn redact_outbound(&self, text: &str) -> String {
        self.redactor.redact(text)
    }

    /// Redact a JSON payload before sending to the cloud.
    pub fn redact_outbound_json(&self, value: &serde_json::Value) -> serde_json::Value {
        self.redactor.redact_json(value)
    }

    /// Access the redactor for system info sanitisation, etc.
    pub fn redactor(&self) -> &Redactor {
        &self.redactor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use d1_common::config::RedactionConfig;

    #[test]
    fn ws_client_redacts_outbound_text() {
        let client = WsClient::new(Redactor::new());
        let input = "password=secret123";
        let out = client.redact_outbound(input);
        assert!(!out.contains("secret123"));
    }

    #[test]
    fn ws_client_redacts_outbound_json() {
        let client = WsClient::new(Redactor::new());
        let val = serde_json::json!({
            "content": "key is sk-abcdefghijklmnopqrstuvwxyz1234567890"
        });
        let out = client.redact_outbound_json(&val);
        assert!(!out["content"].as_str().unwrap().contains("sk-abcdef"));
    }

    #[test]
    fn ws_client_with_disabled_redaction() {
        let config = RedactionConfig {
            enabled: false,
            ..Default::default()
        };
        let client = WsClient::new(Redactor::from_config(&config));
        let input = "password=hunter2";
        let out = client.redact_outbound(input);
        assert_eq!(out, input);
    }
}
