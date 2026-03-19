//! E2E Permission Engine Tests — risk classification, auto-approval, trust scores.
//! All tests use in-memory runtime with no actual LLM calls.

use std::sync::Arc;

use serde_json::json;

use d1_copilot_lib::station::permissions::approval::{
    ApprovalDecision, ApprovalResponse, PermissionEngine,
};
use d1_copilot_lib::station::permissions::risk::{classify_risk, RiskLevel};

// ---------------------------------------------------------------------------
// Test: LOW risk tool auto-approved
// ---------------------------------------------------------------------------

#[tokio::test]
async fn permissions_low_risk_auto_approved() {
    let engine = PermissionEngine::new();

    // All low-risk tools should auto-approve.
    let low_risk_tools = ["memory", "clipboard", "web-search", "web-fetch"];
    for tool in &low_risk_tools {
        let result = engine
            .check_permission("agent-1", "Agent One", tool, &json!({}), "test context")
            .await;
        assert!(
            result.is_ok(),
            "tool '{}' should be auto-approved (Low risk)",
            tool
        );
    }
}

// ---------------------------------------------------------------------------
// Test: LOW risk filesystem read operations auto-approved
// ---------------------------------------------------------------------------

#[tokio::test]
async fn permissions_filesystem_read_is_low_risk() {
    let engine = PermissionEngine::new();

    let read_ops = ["read", "glob", "grep"];
    for op in &read_ops {
        let params = json!({"operation": op});
        let result = engine
            .check_permission("agent-1", "Agent One", "filesystem", &params, "test")
            .await;
        assert!(
            result.is_ok(),
            "filesystem operation '{}' should be auto-approved",
            op
        );
    }
}

// ---------------------------------------------------------------------------
// Test: MEDIUM risk tool emits approval.requested event
// ---------------------------------------------------------------------------

#[tokio::test]
async fn permissions_medium_risk_requires_approval() {
    let engine = PermissionEngine::new();

    // filesystem write is MEDIUM risk.
    let params = json!({"operation": "write"});
    let result = engine
        .check_permission("agent-1", "Agent One", "filesystem", &params, "writing file")
        .await;

    assert!(result.is_err(), "MEDIUM risk should require approval");

    let req = result.unwrap_err();
    assert_eq!(req.risk_level, RiskLevel::Medium);
    assert_eq!(req.tool_name, "filesystem");
    assert_eq!(req.agent_id, "agent-1");
    assert_eq!(req.context, "writing file");
}

// ---------------------------------------------------------------------------
// Test: HIGH risk tool requires approval
// ---------------------------------------------------------------------------

#[tokio::test]
async fn permissions_high_risk_requires_approval() {
    let engine = PermissionEngine::new();

    let high_risk_tools = ["shell", "system", "browser"];
    for tool in &high_risk_tools {
        let result = engine
            .check_permission("agent-1", "Agent One", tool, &json!({}), "test")
            .await;
        assert!(
            result.is_err(),
            "tool '{}' should require approval (High risk)",
            tool
        );
        let req = result.unwrap_err();
        assert_eq!(req.risk_level, RiskLevel::High);
    }
}

// ---------------------------------------------------------------------------
// Test: permission engine risk classification for different tools
// ---------------------------------------------------------------------------

#[tokio::test]
async fn permissions_risk_classification_comprehensive() {
    let empty = json!({});

    // Low risk
    assert_eq!(classify_risk("memory", &empty), RiskLevel::Low);
    assert_eq!(classify_risk("clipboard", &empty), RiskLevel::Low);
    assert_eq!(classify_risk("web-search", &empty), RiskLevel::Low);
    assert_eq!(classify_risk("web-fetch", &empty), RiskLevel::Low);

    // Filesystem with read operation -> Low
    assert_eq!(
        classify_risk("filesystem", &json!({"operation": "read"})),
        RiskLevel::Low
    );
    assert_eq!(
        classify_risk("filesystem", &json!({"operation": "glob"})),
        RiskLevel::Low
    );
    assert_eq!(
        classify_risk("filesystem", &json!({"operation": "grep"})),
        RiskLevel::Low
    );

    // Filesystem with write operation -> Medium
    assert_eq!(
        classify_risk("filesystem", &json!({"operation": "write"})),
        RiskLevel::Medium
    );

    // Filesystem without operation -> Medium (default)
    assert_eq!(classify_risk("filesystem", &empty), RiskLevel::Medium);

    // Data and document -> Medium
    assert_eq!(classify_risk("data", &empty), RiskLevel::Medium);
    assert_eq!(classify_risk("document", &empty), RiskLevel::Medium);

    // Shell, system, browser -> High
    assert_eq!(classify_risk("shell", &empty), RiskLevel::High);
    assert_eq!(classify_risk("system", &empty), RiskLevel::High);
    assert_eq!(classify_risk("browser", &empty), RiskLevel::High);

    // Unknown -> Critical
    assert_eq!(classify_risk("unknown-tool", &empty), RiskLevel::Critical);
    assert_eq!(classify_risk("rm-rf-everything", &empty), RiskLevel::Critical);
}

// ---------------------------------------------------------------------------
// Test: trust score updates after approvals
// ---------------------------------------------------------------------------

#[tokio::test]
async fn permissions_trust_score_updates_after_approvals() {
    let engine = PermissionEngine::new();

    // Default trust score is 0.5.
    // After positive delta, trust increases.
    engine.update_trust_score("agent-1", 0.2).await;

    // With trust score 0.7, MEDIUM risk still requires approval (threshold is 0.8).
    let params = json!({"operation": "write"});
    let result = engine
        .check_permission("agent-1", "Agent One", "filesystem", &params, "ctx")
        .await;
    assert!(result.is_err(), "trust 0.7 should still require approval for MEDIUM");

    // Increase trust score to 0.9 (0.7 + 0.2 = 0.9).
    engine.update_trust_score("agent-1", 0.2).await;

    // Now MEDIUM risk should auto-approve.
    let result = engine
        .check_permission("agent-1", "Agent One", "filesystem", &params, "ctx")
        .await;
    assert!(result.is_ok(), "trust 0.9 should auto-approve MEDIUM risk");
}

// ---------------------------------------------------------------------------
// Test: trust override via AllowAlways
// ---------------------------------------------------------------------------

#[tokio::test]
async fn permissions_allow_always_sets_trust_override() {
    let engine = Arc::new(PermissionEngine::new());

    // Shell (HIGH) requires approval initially.
    let result = engine
        .check_permission("agent-1", "Agent One", "shell", &json!({}), "ctx")
        .await;
    assert!(result.is_err());

    let req = result.unwrap_err();
    let req_id = req.id.clone();
    let engine_clone = Arc::clone(&engine);

    // Spawn approval request in background.
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

    // Same agent + HIGH risk should now auto-approve.
    let result = engine
        .check_permission("agent-1", "Agent One", "shell", &json!({}), "ctx")
        .await;
    assert!(result.is_ok(), "AllowAlways should set permanent trust override");
}

// ---------------------------------------------------------------------------
// Test: approval queue FIFO ordering
// ---------------------------------------------------------------------------

#[tokio::test]
async fn permissions_approval_queue_fifo_ordering() {
    let engine = PermissionEngine::new();

    let req1 = engine
        .check_permission("a1", "Agent A", "shell", &json!({}), "ctx1")
        .await
        .unwrap_err();
    let req2 = engine
        .check_permission("a2", "Agent B", "browser", &json!({}), "ctx2")
        .await
        .unwrap_err();
    let req3 = engine
        .check_permission("a3", "Agent C", "system", &json!({}), "ctx3")
        .await
        .unwrap_err();

    let id1 = req1.id.clone();
    let id2 = req2.id.clone();
    let id3 = req3.id.clone();

    engine.enqueue_approval(req1).await;
    engine.enqueue_approval(req2).await;
    engine.enqueue_approval(req3).await;

    assert_eq!(engine.queue_len().await, 3);

    let ordered = engine.get_pending_ordered().await;
    assert_eq!(ordered[0].id, id1, "first enqueued should be first");
    assert_eq!(ordered[1].id, id2, "second enqueued should be second");
    assert_eq!(ordered[2].id, id3, "third enqueued should be third");
}

// ---------------------------------------------------------------------------
// Test: trust score clamping at boundaries
// ---------------------------------------------------------------------------

#[tokio::test]
async fn permissions_trust_score_clamped_at_boundaries() {
    let engine = PermissionEngine::new();

    // Exceed upper bound: 0.5 + 0.8 = 1.0 (clamped)
    engine.update_trust_score("agent-1", 0.8).await;

    // Another positive delta should stay at 1.0.
    engine.update_trust_score("agent-1", 0.5).await;

    // Subtract a lot: 1.0 - 3.0 = 0.0 (clamped)
    engine.update_trust_score("agent-1", -3.0).await;

    // MEDIUM risk should require approval again (score = 0.0).
    let result = engine
        .check_permission(
            "agent-1",
            "Agent One",
            "filesystem",
            &json!({"operation": "write"}),
            "ctx",
        )
        .await;
    assert!(result.is_err(), "trust 0.0 should require approval for MEDIUM risk");
}

// ---------------------------------------------------------------------------
// Test: unknown tools are classified as Critical
// ---------------------------------------------------------------------------

#[tokio::test]
async fn permissions_unknown_tool_critical_risk() {
    let engine = PermissionEngine::new();

    let result = engine
        .check_permission("agent-1", "Agent One", "never-seen-tool", &json!({}), "ctx")
        .await;
    assert!(result.is_err());

    let req = result.unwrap_err();
    assert_eq!(req.risk_level, RiskLevel::Critical);
}

// ---------------------------------------------------------------------------
// Test: respond to queued approval removes from queue
// ---------------------------------------------------------------------------

#[tokio::test]
async fn permissions_respond_queued_removes_from_queue() {
    let engine = PermissionEngine::new();

    let req = engine
        .check_permission("a1", "Agent A", "shell", &json!({}), "ctx")
        .await
        .unwrap_err();
    let req_id = req.id.clone();

    engine.enqueue_approval(req).await;
    assert_eq!(engine.queue_len().await, 1);

    engine
        .respond_queued(ApprovalResponse {
            request_id: req_id,
            decision: ApprovalDecision::AllowOnce,
        })
        .await
        .unwrap();

    assert_eq!(engine.queue_len().await, 0);
}
