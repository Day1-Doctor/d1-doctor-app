//! E2E Cost Tracking Tests — record usage, DD calculation, session totals, events.
//! All tests use in-memory runtime with no actual LLM calls.
//! Note: SQLite persistence tests live in the unit test suite (crate-internal)
//! because `db::init_memory()` is `#[cfg(test)]`-gated.

use std::sync::Arc;

use d1_copilot_lib::station::costs::cost_tracker::{CostTracker, ModelTier};
use d1_copilot_lib::station::events::{EventBus, EventType};

// ---------------------------------------------------------------------------
// Test: LLM call records cost in CostTracker
// ---------------------------------------------------------------------------

#[tokio::test]
async fn costs_llm_call_records_cost() {
    let tracker = CostTracker::new();

    // Simulate an LLM call with 100k prompt tokens + 50k completion tokens.
    tracker
        .record_usage("agent-1", "anthropic", "claude-sonnet-4", 100_000, 50_000)
        .await;

    let cost = tracker.get_agent_cost("agent-1").await.unwrap();
    assert_eq!(cost.tokens_in, 100_000);
    assert_eq!(cost.tokens_out, 50_000);
    assert_eq!(cost.request_count, 1);
    assert!(cost.cost_dd > 0.0, "DD cost should be positive");
    assert!(cost.cost_usd > 0.0, "USD cost should be positive");
}

// ---------------------------------------------------------------------------
// Test: DD cost calculated correctly for different model tiers
// ---------------------------------------------------------------------------

#[tokio::test]
async fn costs_dd_calculated_correctly_per_tier() {
    let tracker = CostTracker::new();

    // Light tier: 1 DD per 1M tokens.
    // gemini-flash -> Light
    tracker
        .record_usage("agent-light", "google", "gemini-flash", 500_000, 500_000)
        .await;
    let cost_light = tracker.get_agent_cost("agent-light").await.unwrap();
    // 1M tokens * 1 DD/1M = 1.0 DD
    assert!(
        (cost_light.cost_dd - 1.0).abs() < 0.001,
        "Light: expected 1.0 DD, got {}",
        cost_light.cost_dd
    );

    // Medium tier: 5 DD per 1M tokens.
    // claude-sonnet-4 -> Medium
    tracker
        .record_usage("agent-medium", "anthropic", "claude-sonnet-4", 500_000, 500_000)
        .await;
    let cost_medium = tracker.get_agent_cost("agent-medium").await.unwrap();
    // 1M tokens * 5 DD/1M = 5.0 DD
    assert!(
        (cost_medium.cost_dd - 5.0).abs() < 0.001,
        "Medium: expected 5.0 DD, got {}",
        cost_medium.cost_dd
    );

    // Heavy tier: 25 DD per 1M tokens.
    // claude-opus-4 -> Heavy
    tracker
        .record_usage("agent-heavy", "anthropic", "claude-opus-4", 500_000, 500_000)
        .await;
    let cost_heavy = tracker.get_agent_cost("agent-heavy").await.unwrap();
    // 1M tokens * 25 DD/1M = 25.0 DD
    assert!(
        (cost_heavy.cost_dd - 25.0).abs() < 0.001,
        "Heavy: expected 25.0 DD, got {}",
        cost_heavy.cost_dd
    );
}

// ---------------------------------------------------------------------------
// Test: session total accumulates across agents
// ---------------------------------------------------------------------------

#[tokio::test]
async fn costs_session_total_accumulates_across_agents() {
    let tracker = CostTracker::new();

    // Agent A: Heavy tier (opus) - 150k tokens
    tracker
        .record_usage("agent-a", "anthropic", "claude-opus-4", 100_000, 50_000)
        .await;

    // Agent B: Light tier (flash) - 300k tokens
    tracker
        .record_usage("agent-b", "google", "gemini-flash", 200_000, 100_000)
        .await;

    // Agent C: Medium tier (sonnet) - 200k tokens
    tracker
        .record_usage("agent-c", "anthropic", "claude-sonnet-4", 150_000, 50_000)
        .await;

    let total = tracker.get_session_total().await;
    assert_eq!(total.tokens_in, 450_000, "total tokens_in");
    assert_eq!(total.tokens_out, 200_000, "total tokens_out");
    assert_eq!(total.request_count, 3, "total request count");

    // Agent A: 150k * 25/1M = 3.75 DD
    // Agent B: 300k * 1/1M = 0.3 DD
    // Agent C: 200k * 5/1M = 1.0 DD
    // Total: 5.05 DD
    assert!(
        (total.cost_dd - 5.05).abs() < 0.001,
        "expected ~5.05 DD, got {}",
        total.cost_dd
    );
}

// ---------------------------------------------------------------------------
// Test: same agent accumulates multiple requests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn costs_accumulates_multiple_requests_same_agent() {
    let tracker = CostTracker::new();

    tracker
        .record_usage("agent-1", "anthropic", "claude-sonnet-4", 100_000, 50_000)
        .await;
    tracker
        .record_usage("agent-1", "anthropic", "claude-sonnet-4", 200_000, 100_000)
        .await;
    tracker
        .record_usage("agent-1", "anthropic", "claude-sonnet-4", 50_000, 25_000)
        .await;

    let cost = tracker.get_agent_cost("agent-1").await.unwrap();
    assert_eq!(cost.tokens_in, 350_000);
    assert_eq!(cost.tokens_out, 175_000);
    assert_eq!(cost.request_count, 3);
    // Total: 525k tokens, Medium tier (5 DD/1M) = 2.625 DD
    assert!((cost.cost_dd - 2.625).abs() < 0.001);
}

// ---------------------------------------------------------------------------
// Test: cost.updated events emitted to EventBus
// ---------------------------------------------------------------------------

#[tokio::test]
async fn costs_events_emitted_to_event_bus() {
    let bus = Arc::new(EventBus::new(64));
    let tracker = CostTracker::with_event_bus(bus.clone());
    let mut rx = bus.subscribe();

    // First usage.
    tracker
        .record_usage("agent-1", "anthropic", "claude-opus-4", 100_000, 50_000)
        .await;

    let event1 = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .expect("timed out")
        .expect("recv failed");

    assert_eq!(event1.agent_id, "agent-1");
    match event1.event_type {
        EventType::CostUpdated {
            session_tokens,
            session_cost_dd,
        } => {
            assert_eq!(session_tokens, 150_000, "first event: session tokens");
            // 150k * 25/1M = 3.75 DD
            assert!(
                (session_cost_dd - 3.75).abs() < 0.001,
                "first event: session cost DD"
            );
        }
        other => panic!("expected CostUpdated, got {:?}", other),
    }

    // Second usage from different agent.
    tracker
        .record_usage("agent-2", "google", "gemini-flash", 200_000, 100_000)
        .await;

    let event2 = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .expect("timed out")
        .expect("recv failed");

    assert_eq!(event2.agent_id, "agent-2");
    match event2.event_type {
        EventType::CostUpdated {
            session_tokens,
            session_cost_dd,
        } => {
            // Session total: 150k + 300k = 450k tokens
            assert_eq!(session_tokens, 450_000, "second event: session tokens");
            // 3.75 (opus) + 0.3 (flash) = 4.05 DD
            assert!(
                (session_cost_dd - 4.05).abs() < 0.001,
                "second event: session cost DD, got {}",
                session_cost_dd
            );
        }
        other => panic!("expected CostUpdated, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Test: no event emitted without event bus
// ---------------------------------------------------------------------------

#[tokio::test]
async fn costs_no_event_without_bus() {
    // CostTracker without event bus should still work, just no events.
    let tracker = CostTracker::new();

    tracker
        .record_usage("agent-1", "anthropic", "claude-opus-4", 100_000, 50_000)
        .await;

    let cost = tracker.get_agent_cost("agent-1").await.unwrap();
    assert_eq!(cost.tokens_in, 100_000);
    assert!(cost.cost_dd > 0.0);
}

// ---------------------------------------------------------------------------
// Test: model tier classification
// ---------------------------------------------------------------------------

#[tokio::test]
async fn costs_model_tier_classification() {
    // Heavy
    assert_eq!(ModelTier::classify("claude-opus-4"), ModelTier::Heavy);
    assert_eq!(ModelTier::classify("gpt-4o"), ModelTier::Heavy);
    assert_eq!(ModelTier::classify("gpt-4-turbo"), ModelTier::Heavy);

    // Medium
    assert_eq!(ModelTier::classify("claude-sonnet-4"), ModelTier::Medium);
    assert_eq!(ModelTier::classify("gpt-3.5-turbo"), ModelTier::Medium);
    assert_eq!(ModelTier::classify("gpt-4o-mini"), ModelTier::Medium);

    // Light
    assert_eq!(ModelTier::classify("claude-haiku-3"), ModelTier::Light);
    assert_eq!(ModelTier::classify("gemini-flash"), ModelTier::Light);
    assert_eq!(ModelTier::classify("llama-3-8b"), ModelTier::Light);
}

// ---------------------------------------------------------------------------
// Test: reset clears all cost data
// ---------------------------------------------------------------------------

#[tokio::test]
async fn costs_reset_clears_all() {
    let tracker = CostTracker::new();

    tracker
        .record_usage("agent-1", "anthropic", "claude-opus-4", 100_000, 50_000)
        .await;
    tracker
        .record_usage("agent-2", "openai", "gpt-4o", 200_000, 100_000)
        .await;

    assert!(tracker.get_agent_cost("agent-1").await.is_some());

    tracker.reset().await;

    assert!(tracker.get_agent_cost("agent-1").await.is_none());
    assert!(tracker.get_agent_cost("agent-2").await.is_none());
    let total = tracker.get_session_total().await;
    assert_eq!(total.request_count, 0);
    assert_eq!(total.tokens_in, 0);
}

// ---------------------------------------------------------------------------
// Test: DD-to-USD conversion is correct (1 DD = $0.01)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn costs_dd_to_usd_conversion() {
    let tracker = CostTracker::new();

    // 1M tokens, Heavy tier = 25 DD = $0.25 USD
    tracker
        .record_usage("agent-1", "anthropic", "claude-opus-4", 500_000, 500_000)
        .await;

    let cost = tracker.get_agent_cost("agent-1").await.unwrap();
    assert!((cost.cost_dd - 25.0).abs() < 0.001);
    assert!(
        (cost.cost_usd - 0.25).abs() < 0.001,
        "1 DD = $0.01, so 25 DD = $0.25"
    );
}

// ---------------------------------------------------------------------------
// Test: get_all_costs returns all tracked agents
// ---------------------------------------------------------------------------

#[tokio::test]
async fn costs_get_all_costs_returns_all() {
    let tracker = CostTracker::new();

    tracker
        .record_usage("agent-a", "anthropic", "claude-opus-4", 100_000, 50_000)
        .await;
    tracker
        .record_usage("agent-b", "openai", "gpt-4o", 200_000, 100_000)
        .await;
    tracker
        .record_usage("agent-c", "google", "gemini-flash", 50_000, 25_000)
        .await;

    let all = tracker.get_all_costs().await;
    assert_eq!(all.len(), 3);

    let ids: Vec<&str> = all.iter().map(|c| c.agent_id.as_str()).collect();
    assert!(ids.contains(&"agent-a"));
    assert!(ids.contains(&"agent-b"));
    assert!(ids.contains(&"agent-c"));
}

// ---------------------------------------------------------------------------
// Test: nonexistent agent returns None
// ---------------------------------------------------------------------------

#[tokio::test]
async fn costs_nonexistent_agent_returns_none() {
    let tracker = CostTracker::new();
    assert!(tracker.get_agent_cost("nonexistent").await.is_none());
}

// ---------------------------------------------------------------------------
// Test: tier rates are correct constants
// ---------------------------------------------------------------------------

#[test]
fn costs_tier_rates_correct() {
    assert!((ModelTier::Light.dd_per_million_tokens() - 1.0).abs() < f64::EPSILON);
    assert!((ModelTier::Medium.dd_per_million_tokens() - 5.0).abs() < f64::EPSILON);
    assert!((ModelTier::Heavy.dd_per_million_tokens() - 25.0).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// Test: event bus cost events have correct session accumulation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn costs_event_bus_session_accumulation() {
    let bus = Arc::new(EventBus::new(64));
    let tracker = CostTracker::with_event_bus(bus.clone());
    let mut rx = bus.subscribe();

    // Three sequential requests to same agent.
    tracker
        .record_usage("agent-1", "anthropic", "claude-sonnet-4", 100_000, 50_000)
        .await;
    tracker
        .record_usage("agent-1", "anthropic", "claude-sonnet-4", 200_000, 100_000)
        .await;
    tracker
        .record_usage("agent-1", "anthropic", "claude-sonnet-4", 50_000, 25_000)
        .await;

    // Collect three events and verify session tokens accumulate.
    let mut session_tokens_sequence = Vec::new();
    for _ in 0..3 {
        let event = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
            .await
            .expect("timed out")
            .expect("recv failed");
        if let EventType::CostUpdated { session_tokens, .. } = event.event_type {
            session_tokens_sequence.push(session_tokens);
        }
    }

    // Session tokens should increase with each request.
    assert_eq!(session_tokens_sequence.len(), 3);
    assert!(
        session_tokens_sequence[0] < session_tokens_sequence[1],
        "session tokens should increase"
    );
    assert!(
        session_tokens_sequence[1] < session_tokens_sequence[2],
        "session tokens should increase"
    );

    // Final session total: (100k+50k) + (200k+100k) + (50k+25k) = 525k
    assert_eq!(session_tokens_sequence[2], 525_000);
}
