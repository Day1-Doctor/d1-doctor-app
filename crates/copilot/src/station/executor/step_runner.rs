use std::sync::Arc;

use crate::station::kernel::agent::AgentDescriptor;
use crate::station::tasks::task_types::TaskSpec;

use super::agent_executor::{AgentExecutor, StepResult};

/// A retry wrapper around [`AgentExecutor`] that retries transient failures
/// with exponential backoff.
///
/// Transient errors (network timeouts, HTTP 429, HTTP 5xx) are retried up to
/// `max_retries` times with exponential backoff (1s, 2s, 4s, ...).
/// Non-transient errors are returned immediately.
pub struct StepRunner {
    executor: Arc<AgentExecutor>,
    max_retries: u32,
}

impl StepRunner {
    /// Create a new step runner wrapping the given executor.
    ///
    /// * `executor` -- the agent executor to delegate to.
    /// * `max_retries` -- maximum number of retry attempts for transient errors.
    pub fn new(executor: Arc<AgentExecutor>, max_retries: u32) -> Self {
        Self {
            executor,
            max_retries,
        }
    }

    /// Run a step with automatic retry on transient failures.
    ///
    /// Retries with exponential backoff: 1s, 2s, 4s, etc.
    /// Only retries on errors that look transient (network, 429, 5xx).
    pub async fn run_step(
        &self,
        step: &TaskSpec,
        agent: &AgentDescriptor,
    ) -> Result<StepResult, String> {
        let mut last_error = String::new();

        for attempt in 0..=self.max_retries {
            match self.executor.execute_step(step, agent).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if !is_transient_error(&e) || attempt == self.max_retries {
                        return Err(e);
                    }

                    last_error = e;

                    // Exponential backoff: 1s, 2s, 4s, ...
                    let delay_secs = 1u64 << attempt;
                    tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                }
            }
        }

        Err(last_error)
    }

    /// Get the maximum number of retries.
    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

/// Determine whether an error message indicates a transient failure that is
/// worth retrying.
fn is_transient_error(error: &str) -> bool {
    let lower = error.to_lowercase();
    lower.contains("timeout")
        || lower.contains("429")
        || lower.contains("500")
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
        || lower.contains("connection")
        || lower.contains("network")
        || lower.contains("gateway error 5")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_transient_timeout() {
        assert!(is_transient_error("Gateway request failed: timeout"));
        assert!(is_transient_error("Connection timeout after 30s"));
    }

    #[test]
    fn test_is_transient_rate_limit() {
        assert!(is_transient_error("Gateway error 429: rate limited"));
    }

    #[test]
    fn test_is_transient_server_error() {
        assert!(is_transient_error("Gateway error 500: internal server error"));
        assert!(is_transient_error("Gateway error 502: bad gateway"));
        assert!(is_transient_error("Gateway error 503: service unavailable"));
        assert!(is_transient_error("Gateway error 504: gateway timeout"));
    }

    #[test]
    fn test_is_transient_connection_error() {
        assert!(is_transient_error("connection refused"));
        assert!(is_transient_error("network unreachable"));
    }

    #[test]
    fn test_not_transient_auth_error() {
        assert!(!is_transient_error("No API key configured"));
        assert!(!is_transient_error("Gateway error 401: unauthorized"));
        assert!(!is_transient_error("Gateway error 403: forbidden"));
    }

    #[test]
    fn test_not_transient_bad_request() {
        assert!(!is_transient_error("Gateway error 400: bad request"));
        assert!(!is_transient_error("Failed to parse response: invalid JSON"));
    }

    #[test]
    fn test_not_transient_fsm_error() {
        assert!(!is_transient_error(
            "invalid transition: Idle + LlmCallStart is not allowed"
        ));
    }

    #[test]
    fn test_step_runner_max_retries() {
        // We can't easily test the full async retry loop without a mock,
        // but we can verify construction.
        let llm_client = Arc::new(tokio::sync::RwLock::new(
            crate::station::llm::client::LlmClient::new("https://gateway.day1.doctor"),
        ));
        let kernel = Arc::new(crate::station::kernel::kernel::AgentKernel::new());
        let event_bus = Arc::new(crate::station::events::bus::EventBus::new(64));
        let cost_tracker = Arc::new(crate::station::costs::cost_tracker::CostTracker::new());
        let permission_engine =
            Arc::new(crate::station::permissions::PermissionEngine::new());
        let skill_registry =
            Arc::new(crate::station::skills::skill_registry::SkillRegistry::new());

        let executor = Arc::new(AgentExecutor::new(
            llm_client,
            kernel,
            event_bus,
            cost_tracker,
            permission_engine,
            skill_registry,
        ));

        let runner = StepRunner::new(executor, 3);
        assert_eq!(runner.max_retries(), 3);
    }
}
