use super::client::LlmClient;
use crate::station::tasks::decomposer::{DecomposedPlan, PlannedStep, TaskDecomposer};

/// Enhanced decomposer that tries the gateway first, falls back to keyword-based.
pub struct LlmDecomposer {
    client: LlmClient,
    fallback: TaskDecomposer,
}

impl LlmDecomposer {
    pub fn new(client: LlmClient) -> Self {
        Self {
            client,
            fallback: TaskDecomposer::new(),
        }
    }

    /// Decompose using LLM via gateway. Falls back to keyword-based if gateway unavailable.
    pub async fn decompose(&self, description: &str) -> DecomposedPlan {
        // Try LLM decomposition first
        match self.client.decompose(description, 6).await {
            Ok(response) => {
                let steps = response
                    .plan
                    .steps
                    .into_iter()
                    .map(|s| PlannedStep {
                        title: s.title,
                        description: s.description.unwrap_or_default(),
                        suggested_role: s.suggested_role,
                        step_index: s.step_index,
                        depends_on: s.depends_on,
                        is_parallel: false,
                    })
                    .collect();

                DecomposedPlan {
                    original_description: description.to_string(),
                    steps,
                }
            }
            Err(e) => {
                tracing::warn!("LLM decompose failed, using keyword fallback: {}", e);
                self.fallback.decompose(description)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_llm_decomposer_fallback() {
        // Client without an API key will fail, triggering the keyword fallback.
        let client = LlmClient::new("https://gateway.day1.doctor");
        let decomposer = LlmDecomposer::new(client);
        let plan = decomposer.decompose("Research AI and write a report").await;

        // Should fall back to keyword-based decomposition
        assert!(
            !plan.steps.is_empty(),
            "fallback should produce at least one step"
        );
        assert_eq!(plan.original_description, "Research AI and write a report");
        // Keyword decomposer should detect "research" and "write"
        let roles: Vec<&str> = plan
            .steps
            .iter()
            .map(|s| s.suggested_role.as_str())
            .collect();
        assert!(
            roles.contains(&"researcher"),
            "keyword fallback should detect researcher role"
        );
        assert!(
            roles.contains(&"writer"),
            "keyword fallback should detect writer role"
        );
    }

    #[tokio::test]
    async fn test_llm_decomposer_fallback_simple() {
        let client = LlmClient::new("https://gateway.day1.doctor");
        let decomposer = LlmDecomposer::new(client);
        let plan = decomposer.decompose("Hello world").await;

        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].suggested_role, "orchestrator");
    }
}
