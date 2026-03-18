use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{oneshot, RwLock};

use super::risk::{classify_risk, RiskLevel};

/// A request asking the user to approve (or reject) a tool invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub tool_name: String,
    pub params: serde_json::Value,
    pub risk_level: RiskLevel,
    /// Human-readable context, e.g. "Task step 3/5: Installing dependency"
    pub context: String,
    pub created_at: DateTime<Utc>,
}

/// The decision a user makes on an approval request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalDecision {
    /// Approve this single invocation only.
    AllowOnce,
    /// Trust this agent at this risk level going forward (auto-approve).
    AllowAlways,
    /// Deny the tool invocation.
    Reject,
}

/// The user's response to an [`ApprovalRequest`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResponse {
    pub request_id: String,
    pub decision: ApprovalDecision,
}

// ---------------------------------------------------------------------------
// Internal bookkeeping for a request that is awaiting a response.
// ---------------------------------------------------------------------------

struct PendingApproval {
    request: ApprovalRequest,
    response_tx: oneshot::Sender<ApprovalResponse>,
}

// ---------------------------------------------------------------------------
// PermissionEngine
// ---------------------------------------------------------------------------

/// Central authority for tool-call permission checks.
///
/// Workflow:
/// 1. Agent calls [`check_permission`] before invoking a tool.
/// 2. If the method returns `Ok(())` the call is auto-approved.
/// 3. If it returns `Err(ApprovalRequest)` the caller must show an approval
///    modal and then forward the user's answer via [`respond`].
pub struct PermissionEngine {
    /// Pending approval requests keyed by request id.
    pending: Arc<RwLock<HashMap<String, PendingApproval>>>,
    /// Trust overrides: `(agent_id, risk_level)` -> auto-approve when `true`.
    trust_overrides: Arc<RwLock<HashMap<(String, RiskLevel), bool>>>,
    /// Agent trust scores (0.0 – 1.0). Higher → more autonomous.
    trust_scores: Arc<RwLock<HashMap<String, f64>>>,
}

impl PermissionEngine {
    /// Create a new permission engine with empty state.
    pub fn new() -> Self {
        Self {
            pending: Arc::new(RwLock::new(HashMap::new())),
            trust_overrides: Arc::new(RwLock::new(HashMap::new())),
            trust_scores: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check whether a tool call is allowed without user intervention.
    ///
    /// Returns `Ok(())` when auto-approved, or `Err(ApprovalRequest)` when
    /// the user must make an explicit decision.
    pub async fn check_permission(
        &self,
        agent_id: &str,
        agent_name: &str,
        tool_name: &str,
        params: &serde_json::Value,
        context: &str,
    ) -> Result<(), ApprovalRequest> {
        let risk = classify_risk(tool_name, params);

        // LOW risk is always auto-approved.
        if risk == RiskLevel::Low {
            return Ok(());
        }

        // Check trust overrides (set by AllowAlways).
        if self.is_trusted(agent_id, risk).await {
            return Ok(());
        }

        // A high trust score auto-approves MEDIUM risk.
        if risk == RiskLevel::Medium {
            let score = self.get_trust_score(agent_id).await;
            if score >= 0.8 {
                return Ok(());
            }
        }

        // Anything else needs explicit approval.
        Err(ApprovalRequest {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            agent_name: agent_name.to_string(),
            tool_name: tool_name.to_string(),
            params: params.clone(),
            risk_level: risk,
            context: context.to_string(),
            created_at: Utc::now(),
        })
    }

    /// Submit an approval request and block until the user responds or the
    /// 5-minute timeout expires (auto-rejects on timeout).
    pub async fn request_approval(&self, request: ApprovalRequest) -> ApprovalResponse {
        let (tx, rx) = oneshot::channel();
        let req_id = request.id.clone();

        self.pending.write().await.insert(
            req_id.clone(),
            PendingApproval {
                request,
                response_tx: tx,
            },
        );

        match tokio::time::timeout(std::time::Duration::from_secs(300), rx).await {
            Ok(Ok(response)) => response,
            _ => {
                // Timeout or channel dropped — auto-reject.
                self.pending.write().await.remove(&req_id);
                ApprovalResponse {
                    request_id: req_id,
                    decision: ApprovalDecision::Reject,
                }
            }
        }
    }

    /// Respond to a pending approval request (called by the UI layer).
    ///
    /// If the user chose [`ApprovalDecision::AllowAlways`] the engine records a
    /// trust override so future calls at that risk level are auto-approved.
    pub async fn respond(&self, response: ApprovalResponse) -> Result<(), String> {
        let mut pending = self.pending.write().await;
        if let Some(approval) = pending.remove(&response.request_id) {
            // Persist the trust override when the user grants blanket trust.
            if let ApprovalDecision::AllowAlways = &response.decision {
                self.trust_overrides.write().await.insert(
                    (
                        approval.request.agent_id.clone(),
                        approval.request.risk_level,
                    ),
                    true,
                );
            }
            let _ = approval.response_tx.send(response);
            Ok(())
        } else {
            Err("No pending approval found".to_string())
        }
    }

    /// Return all currently pending approval requests (for UI rendering).
    pub async fn get_pending(&self) -> Vec<ApprovalRequest> {
        self.pending
            .read()
            .await
            .values()
            .map(|p| p.request.clone())
            .collect()
    }

    /// Adjust the trust score for an agent by `delta` (clamped to 0.0..=1.0).
    pub async fn update_trust_score(&self, agent_id: &str, delta: f64) {
        let mut scores = self.trust_scores.write().await;
        let current = scores.get(agent_id).copied().unwrap_or(0.5);
        let updated = (current + delta).clamp(0.0, 1.0);
        scores.insert(agent_id.to_string(), updated);
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    async fn get_trust_score(&self, agent_id: &str) -> f64 {
        self.trust_scores
            .read()
            .await
            .get(agent_id)
            .copied()
            .unwrap_or(0.5)
    }

    async fn is_trusted(&self, agent_id: &str, risk: RiskLevel) -> bool {
        self.trust_overrides
            .read()
            .await
            .get(&(agent_id.to_string(), risk))
            .copied()
            .unwrap_or(false)
    }
}

impl Default for PermissionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_auto_approve_low() {
        let engine = PermissionEngine::new();
        let result = engine
            .check_permission("a1", "Agent A", "memory", &json!({}), "test")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_needs_approval_high() {
        let engine = PermissionEngine::new();
        let result = engine
            .check_permission("a1", "Agent A", "shell", &json!({}), "running command")
            .await;
        assert!(result.is_err());
        let req = result.unwrap_err();
        assert_eq!(req.risk_level, RiskLevel::High);
        assert_eq!(req.tool_name, "shell");
    }

    #[tokio::test]
    async fn test_trust_override() {
        let engine = PermissionEngine::new();

        // Shell is HIGH → normally needs approval.
        let result = engine
            .check_permission("a1", "Agent A", "shell", &json!({}), "ctx")
            .await;
        assert!(result.is_err());

        // Set a trust override for HIGH.
        engine
            .trust_overrides
            .write()
            .await
            .insert(("a1".to_string(), RiskLevel::High), true);

        // Now it should auto-approve.
        let result = engine
            .check_permission("a1", "Agent A", "shell", &json!({}), "ctx")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_approval_flow() {
        let engine = Arc::new(PermissionEngine::new());

        // Build a request that needs approval.
        let req = engine
            .check_permission("a1", "Agent A", "shell", &json!({}), "ctx")
            .await
            .unwrap_err();

        let req_id = req.id.clone();
        let engine_clone = Arc::clone(&engine);

        // Spawn the approval wait in background.
        let handle = tokio::spawn(async move { engine_clone.request_approval(req).await });

        // Give the spawned task a moment to register.
        tokio::task::yield_now().await;

        // UI responds.
        engine
            .respond(ApprovalResponse {
                request_id: req_id.clone(),
                decision: ApprovalDecision::AllowOnce,
            })
            .await
            .unwrap();

        let response = handle.await.unwrap();
        assert_eq!(response.request_id, req_id);
        assert!(matches!(response.decision, ApprovalDecision::AllowOnce));
    }

    #[tokio::test]
    async fn test_pending_list() {
        let engine = PermissionEngine::new();

        let req1 = engine
            .check_permission("a1", "Agent A", "shell", &json!({}), "ctx1")
            .await
            .unwrap_err();
        let req2 = engine
            .check_permission("a2", "Agent B", "browser", &json!({}), "ctx2")
            .await
            .unwrap_err();

        // Insert both as pending (manually, since request_approval blocks).
        let (tx1, _rx1) = oneshot::channel();
        let (tx2, _rx2) = oneshot::channel();
        {
            let mut pending = engine.pending.write().await;
            pending.insert(
                req1.id.clone(),
                PendingApproval {
                    request: req1,
                    response_tx: tx1,
                },
            );
            pending.insert(
                req2.id.clone(),
                PendingApproval {
                    request: req2,
                    response_tx: tx2,
                },
            );
        }

        let list = engine.get_pending().await;
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_allow_always_sets_trust() {
        let engine = Arc::new(PermissionEngine::new());

        // Shell (HIGH) requires approval.
        let req = engine
            .check_permission("a1", "Agent A", "shell", &json!({}), "ctx")
            .await
            .unwrap_err();

        let req_id = req.id.clone();
        let engine_clone = Arc::clone(&engine);

        let handle = tokio::spawn(async move { engine_clone.request_approval(req).await });

        tokio::task::yield_now().await;

        // Respond with AllowAlways.
        engine
            .respond(ApprovalResponse {
                request_id: req_id,
                decision: ApprovalDecision::AllowAlways,
            })
            .await
            .unwrap();

        let _ = handle.await.unwrap();

        // Now the same agent + risk level should be auto-approved.
        let result = engine
            .check_permission("a1", "Agent A", "shell", &json!({}), "ctx")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_medium_auto_approve_with_high_trust() {
        let engine = PermissionEngine::new();

        // Default trust score is 0.5 → MEDIUM needs approval.
        let result = engine
            .check_permission("a1", "Agent A", "filesystem", &json!({"operation": "write"}), "ctx")
            .await;
        assert!(result.is_err());

        // Bump trust score above 0.8.
        engine.update_trust_score("a1", 0.4).await; // 0.5 + 0.4 = 0.9

        // Now MEDIUM should auto-approve.
        let result = engine
            .check_permission("a1", "Agent A", "filesystem", &json!({"operation": "write"}), "ctx")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_respond_unknown_request() {
        let engine = PermissionEngine::new();
        let result = engine
            .respond(ApprovalResponse {
                request_id: "nonexistent".to_string(),
                decision: ApprovalDecision::Reject,
            })
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No pending approval found");
    }

    #[tokio::test]
    async fn test_update_trust_score_clamping() {
        let engine = PermissionEngine::new();

        // Default is 0.5, add 0.6 → clamped to 1.0
        engine.update_trust_score("a1", 0.6).await;
        assert_eq!(engine.get_trust_score("a1").await, 1.0);

        // Subtract 2.0 → clamped to 0.0
        engine.update_trust_score("a1", -2.0).await;
        assert_eq!(engine.get_trust_score("a1").await, 0.0);
    }
}
