use reqwest::Client;
use serde::{Deserialize, Serialize};

/// HTTP client for the Day1 Gateway dr-agent endpoints.
#[derive(Clone)]
pub struct LlmClient {
    http: Client,
    gateway_url: String,
    api_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    pub stream: bool,
    /// Tool definitions advertised to the model (OpenAI function-calling format).
    /// When present the model may return `tool_calls` in its response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Option<UsageInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub message: ChoiceMessage,
    pub finish_reason: Option<String>,
}

/// The message payload inside a ChatChoice.
///
/// Unlike the simple `ChatMessage` (used for request messages), this variant
/// carries optional `tool_calls` returned by the LLM when it wants to invoke
/// tools via the OpenAI function-calling protocol.
#[derive(Debug, Deserialize, Clone)]
pub struct ChoiceMessage {
    pub role: String,
    #[serde(default)]
    pub content: String,
    /// Tool calls requested by the model.  `None` when the model replies with
    /// plain text; `Some(vec![...])` when it wants to invoke tools.
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// A single tool invocation requested by the LLM (OpenAI function-calling format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: Option<String>,
    pub function: ToolCallFunction,
}

/// The function name + serialised JSON arguments inside a [`ToolCall`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    /// JSON-encoded arguments string, as returned by the LLM.
    pub arguments: String,
}

#[derive(Debug, Deserialize)]
pub struct UsageInfo {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct DecomposeResponse {
    pub plan: DecomposePlan,
    pub dd_cost: f64,
    pub model_used: String,
}

#[derive(Debug, Deserialize)]
pub struct DecomposePlan {
    pub steps: Vec<DecomposeStep>,
}

#[derive(Debug, Deserialize)]
pub struct DecomposeStep {
    pub title: String,
    pub description: Option<String>,
    pub suggested_role: String,
    pub step_index: u32,
    pub depends_on: Vec<u32>,
}

#[derive(Debug, Deserialize)]
pub struct BalanceResponse {
    pub dd_balance: f64,
    pub subscription_tier: Option<String>,
    pub tier_max_agents: Option<u32>,
}

impl LlmClient {
    pub fn new(gateway_url: &str) -> Self {
        Self {
            http: Client::new(),
            gateway_url: gateway_url.to_string(),
            api_key: None,
        }
    }

    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = Some(key.to_string());
        self
    }

    /// Return the configured gateway URL.
    pub fn gateway_url(&self) -> &str {
        &self.gateway_url
    }

    /// Return the configured API key, if any.
    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    /// List available models from the gateway.
    ///
    /// Calls GET /dr-agent/v1/models and returns the parsed JSON response.
    /// Returns an empty list on auth or network errors so the UI can degrade
    /// gracefully.
    pub async fn list_models(&self) -> Vec<serde_json::Value> {
        let api_key = match &self.api_key {
            Some(k) => k.clone(),
            None => return Vec::new(),
        };

        let resp = self
            .http
            .get(format!("{}/dr-agent/v1/models", self.gateway_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await;

        let resp = match resp {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        if !resp.status().is_success() {
            return Vec::new();
        }

        let body: serde_json::Value = match resp.json().await {
            Ok(b) => b,
            Err(_) => return Vec::new(),
        };

        // The gateway returns { "data": [...] } following OpenAI convention.
        body.get("data")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default()
    }

    /// Load auth token from ~/.day1copilot/auth.json.
    /// Reads the new `{ "token": "...", "token_type": "jwt" }` format.
    pub fn load_api_key(&mut self) -> Result<(), String> {
        let auth_file = dirs::home_dir()
            .ok_or("No home dir")?
            .join(".day1copilot/auth.json");
        if !auth_file.exists() {
            return Err("Not authenticated. Please sign in first.".into());
        }
        let content = std::fs::read_to_string(&auth_file).map_err(|e| e.to_string())?;
        let data: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        self.api_key = data
            .get("token")
            .and_then(|v| v.as_str())
            .map(String::from);
        Ok(())
    }

    /// Call /dr-agent/v1/chat/completions (non-streaming).
    pub async fn chat(
        &self,
        request: ChatRequest,
        agent_name: &str,
    ) -> Result<ChatResponse, String> {
        let api_key = self.api_key.as_ref().ok_or("No API key configured")?;

        let resp = self
            .http
            .post(format!("{}/dr-agent/v1/chat/completions", self.gateway_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("x-copilot-agent", agent_name)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Gateway request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Gateway error {}: {}", status, body));
        }

        resp.json::<ChatResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Call /dr-agent/v1/decompose
    pub async fn decompose(
        &self,
        description: &str,
        max_steps: u32,
    ) -> Result<DecomposeResponse, String> {
        let api_key = self.api_key.as_ref().ok_or("No API key configured")?;

        let body = serde_json::json!({
            "description": description,
            "max_steps": max_steps,
        });

        let resp = self
            .http
            .post(format!("{}/dr-agent/v1/decompose", self.gateway_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Decompose request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Decompose error {}: {}", status, body));
        }

        resp.json::<DecomposeResponse>()
            .await
            .map_err(|e| format!("Failed to parse decompose response: {}", e))
    }

    /// Check balance via /dr-agent/v1/balance
    pub async fn get_balance(&self) -> Result<BalanceResponse, String> {
        let api_key = self.api_key.as_ref().ok_or("No API key configured")?;

        let resp = self
            .http
            .get(format!("{}/dr-agent/v1/balance", self.gateway_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| format!("Balance request failed: {}", e))?;

        resp.json::<BalanceResponse>()
            .await
            .map_err(|e| format!("Failed to parse balance: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_client_new() {
        let client = LlmClient::new("https://gateway.day1.doctor");
        assert_eq!(client.gateway_url, "https://gateway.day1.doctor");
        assert!(client.api_key.is_none());
    }

    #[test]
    fn test_llm_client_with_api_key() {
        let client = LlmClient::new("https://gateway.day1.doctor").with_api_key("d1d_sk_test123");
        assert_eq!(client.api_key, Some("d1d_sk_test123".to_string()));
    }

    #[tokio::test]
    async fn test_llm_client_no_key_error() {
        let client = LlmClient::new("https://gateway.day1.doctor");
        let request = ChatRequest {
            model: "gpt-4o".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            max_tokens: None,
            temperature: None,
            stream: false,
            tools: None,
        };
        let result = client.chat(request, "dr-bob").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No API key configured"));
    }

    #[tokio::test]
    async fn test_decompose_no_key_error() {
        let client = LlmClient::new("https://gateway.day1.doctor");
        let result = client.decompose("test task", 6).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No API key configured"));
    }

    #[tokio::test]
    async fn test_balance_no_key_error() {
        let client = LlmClient::new("https://gateway.day1.doctor");
        let result = client.get_balance().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No API key configured"));
    }

    #[test]
    fn test_gateway_url_accessor() {
        let client = LlmClient::new("https://gateway.day1.doctor");
        assert_eq!(client.gateway_url(), "https://gateway.day1.doctor");
    }

    #[test]
    fn test_api_key_accessor() {
        let client = LlmClient::new("https://gateway.day1.doctor");
        assert!(client.api_key().is_none());

        let client2 = client.with_api_key("test_key");
        assert_eq!(client2.api_key(), Some("test_key"));
    }

    #[tokio::test]
    async fn test_list_models_no_key_returns_empty() {
        let client = LlmClient::new("https://gateway.day1.doctor");
        let models = client.list_models().await;
        assert!(models.is_empty());
    }

    #[tokio::test]
    async fn test_list_models_bad_url_returns_empty() {
        let client =
            LlmClient::new("http://127.0.0.1:1").with_api_key("test");
        let models = client.list_models().await;
        assert!(models.is_empty());
    }
}
