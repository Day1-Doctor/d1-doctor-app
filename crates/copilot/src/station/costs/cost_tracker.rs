use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::station::db::DbHandle;
use crate::station::events::bus::EventBus;
use crate::station::events::event_types::{AgentEvent, EventType};

/// DD cost per 1 million tokens, grouped by model tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelTier {
    /// Budget / small models — 1 DD per 1M tokens.
    Light,
    /// Standard models — 5 DD per 1M tokens.
    Medium,
    /// Frontier / large models — 25 DD per 1M tokens.
    Heavy,
}

impl ModelTier {
    /// DD cost per 1 million tokens for this tier.
    pub fn dd_per_million_tokens(&self) -> f64 {
        match self {
            ModelTier::Light => 1.0,
            ModelTier::Medium => 5.0,
            ModelTier::Heavy => 25.0,
        }
    }

    /// Classify a model string into a tier.
    ///
    /// Heuristic: model names containing "opus" or "gpt-4" are heavy;
    /// names containing "sonnet", "gpt-3.5", or "haiku" are medium;
    /// everything else (mini, flash, etc.) is light.
    ///
    /// More specific patterns (e.g. "gpt-4o-mini") are checked before
    /// broader ones (e.g. "gpt-4o") to avoid false positives.
    pub fn classify(model: &str) -> Self {
        let lower = model.to_lowercase();
        // Check medium-tier patterns first (more specific substrings).
        if lower.contains("gpt-4o-mini")
            || lower.contains("sonnet")
            || lower.contains("gpt-3.5")
        {
            ModelTier::Medium
        } else if lower.contains("opus") || lower.contains("gpt-4o") || lower.contains("gpt-4-") {
            ModelTier::Heavy
        } else {
            // Light tier: haiku, flash, mini models, and anything unrecognized.
            ModelTier::Light
        }
    }
}

/// Per-agent cost summary for the current session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCostSummary {
    pub agent_id: String,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub cost_usd: f64,
    pub cost_dd: f64,
    pub request_count: u64,
    pub last_updated: DateTime<Utc>,
}

impl AgentCostSummary {
    fn new(agent_id: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            tokens_in: 0,
            tokens_out: 0,
            cost_usd: 0.0,
            cost_dd: 0.0,
            request_count: 0,
            last_updated: Utc::now(),
        }
    }
}

/// Tracks per-agent token usage and DD cost for the current session.
///
/// Thread-safe via `RwLock`. Emits `cost.updated` events on the EventBus
/// after each `record_usage` call. Optionally persists each usage record
/// to SQLite for historical analysis across sessions.
pub struct CostTracker {
    agents: Arc<RwLock<HashMap<String, AgentCostSummary>>>,
    event_bus: Option<Arc<EventBus>>,
    /// Optional SQLite handle for write-behind cost persistence.
    db: Option<DbHandle>,
}

impl CostTracker {
    /// Create a new cost tracker without an event bus.
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            event_bus: None,
            db: None,
        }
    }

    /// Create a new cost tracker wired to the given event bus.
    pub fn with_event_bus(event_bus: Arc<EventBus>) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            event_bus: Some(event_bus),
            db: None,
        }
    }

    /// Create a new cost tracker backed by an SQLite database.
    pub fn with_db(db: DbHandle) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            event_bus: None,
            db: Some(db),
        }
    }

    /// Create a new cost tracker with both an event bus and SQLite persistence.
    pub fn with_event_bus_and_db(event_bus: Arc<EventBus>, db: DbHandle) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            event_bus: Some(event_bus),
            db: Some(db),
        }
    }

    /// Persist a single usage record to the `session_costs` table (write-behind).
    fn persist_cost(
        &self,
        agent_id: &str,
        provider: &str,
        model: &str,
        tokens_in: u64,
        tokens_out: u64,
        cost_usd: f64,
        cost_dd: f64,
    ) {
        let db = match &self.db {
            Some(db) => db,
            None => return,
        };

        let conn = match db.lock() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("failed to acquire db lock for cost persist: {e}");
                return;
            }
        };

        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // Ensure the agent row exists (FK requirement).
        let _ = conn.execute(
            "INSERT INTO agents (id, name, role, framework, created_at, updated_at)
             VALUES (?1, ?1, 'unknown', 'unknown', ?2, ?2)
             ON CONFLICT(id) DO NOTHING",
            rusqlite::params![agent_id, now],
        );

        let result = conn.execute(
            "INSERT INTO session_costs (id, agent_id, provider, model, tokens_in, tokens_out, cost_usd, cost_dd, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![id, agent_id, provider, model, tokens_in as i64, tokens_out as i64, cost_usd, cost_dd, now],
        );

        if let Err(e) = result {
            tracing::warn!(agent_id = %agent_id, "failed to persist session cost: {e}");
        }
    }

    /// Load historical cost records from SQLite for summary display.
    ///
    /// Returns the total number of records loaded. Cost records are grouped
    /// into per-agent summaries and merged into the in-memory store.
    pub async fn load_from_db(&self) -> Result<usize, String> {
        let db = match &self.db {
            Some(db) => db,
            None => return Ok(0),
        };

        // Collect all rows while holding the db mutex, then release it.
        let summaries: Vec<AgentCostSummary> = {
            let conn = db.lock().map_err(|e| format!("db lock failed: {e}"))?;
            let mut stmt = conn
                .prepare(
                    "SELECT agent_id, SUM(tokens_in), SUM(tokens_out), SUM(cost_usd), SUM(cost_dd), COUNT(*)
                     FROM session_costs
                     GROUP BY agent_id",
                )
                .map_err(|e| format!("prepare failed: {e}"))?;

            let rows = stmt
                .query_map([], |row| {
                    Ok(AgentCostSummary {
                        agent_id: row.get(0)?,
                        tokens_in: row.get::<_, i64>(1)? as u64,
                        tokens_out: row.get::<_, i64>(2)? as u64,
                        cost_usd: row.get(3)?,
                        cost_dd: row.get(4)?,
                        request_count: row.get::<_, i64>(5)? as u64,
                        last_updated: Utc::now(),
                    })
                })
                .map_err(|e| format!("query failed: {e}"))?;

            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("row read failed: {e}"))?
        };

        let count = summaries.len();
        let mut agents = self.agents.write().await;
        for summary in summaries {
            agents.insert(summary.agent_id.clone(), summary);
        }
        Ok(count)
    }

    /// Record a usage event for an agent.
    ///
    /// Calculates the DD cost using the model's tier rate and updates the
    /// running totals. Emits a `cost.updated` event if an EventBus is
    /// configured. Persists the record to SQLite if a DbHandle is configured.
    pub async fn record_usage(
        &self,
        agent_id: &str,
        provider: &str,
        model: &str,
        tokens_in: u64,
        tokens_out: u64,
    ) {
        let tier = ModelTier::classify(model);
        let total_tokens = tokens_in + tokens_out;
        let dd_cost = (total_tokens as f64 / 1_000_000.0) * tier.dd_per_million_tokens();
        // 1 DD = $0.01 USD
        let usd_cost = dd_cost * 0.01;

        let (session_tokens, session_cost_dd) = {
            let mut agents = self.agents.write().await;
            let summary = agents
                .entry(agent_id.to_string())
                .or_insert_with(|| AgentCostSummary::new(agent_id));

            summary.tokens_in += tokens_in;
            summary.tokens_out += tokens_out;
            summary.cost_dd += dd_cost;
            summary.cost_usd += usd_cost;
            summary.request_count += 1;
            summary.last_updated = Utc::now();

            // Compute session totals for the event.
            let mut total_tokens_session: u64 = 0;
            let mut total_dd_session: f64 = 0.0;
            for s in agents.values() {
                total_tokens_session += s.tokens_in + s.tokens_out;
                total_dd_session += s.cost_dd;
            }
            (total_tokens_session, total_dd_session)
        };

        // Persist to SQLite.
        self.persist_cost(agent_id, provider, model, tokens_in, tokens_out, usd_cost, dd_cost);

        // Emit cost.updated event.
        if let Some(ref bus) = self.event_bus {
            let event = AgentEvent {
                id: Uuid::new_v4().to_string(),
                agent_id: agent_id.to_string(),
                timestamp: Utc::now(),
                event_type: EventType::CostUpdated {
                    session_tokens,
                    session_cost_dd,
                },
            };
            bus.publish(event).await;
        }
    }

    /// Get the cost summary for a specific agent.
    pub async fn get_agent_cost(&self, agent_id: &str) -> Option<AgentCostSummary> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// Get the session total across all agents.
    pub async fn get_session_total(&self) -> AgentCostSummary {
        let agents = self.agents.read().await;
        let mut total = AgentCostSummary::new("__session__");
        for s in agents.values() {
            total.tokens_in += s.tokens_in;
            total.tokens_out += s.tokens_out;
            total.cost_usd += s.cost_usd;
            total.cost_dd += s.cost_dd;
            total.request_count += s.request_count;
        }
        total
    }

    /// Get cost summaries for all agents.
    pub async fn get_all_costs(&self) -> Vec<AgentCostSummary> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }

    /// Reset all cost tracking data.
    pub async fn reset(&self) {
        let mut agents = self.agents.write().await;
        agents.clear();
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::station::db;

    #[tokio::test]
    async fn test_record_usage_calculates_correct_dd_cost() {
        let tracker = CostTracker::new();

        // 1M tokens with a Light model = 1 DD
        tracker
            .record_usage("agent-1", "openai", "gpt-4o-mini", 500_000, 500_000)
            .await;

        let cost = tracker.get_agent_cost("agent-1").await.unwrap();
        assert_eq!(cost.tokens_in, 500_000);
        assert_eq!(cost.tokens_out, 500_000);
        // gpt-4o-mini matches "gpt-4o-mini" -> Medium tier = 5 DD/1M
        assert!((cost.cost_dd - 5.0).abs() < 0.001);
        assert!((cost.cost_usd - 0.05).abs() < 0.001);
        assert_eq!(cost.request_count, 1);
    }

    #[tokio::test]
    async fn test_multiple_agents_tracked_independently() {
        let tracker = CostTracker::new();

        // Agent A: Heavy tier (opus)
        tracker
            .record_usage("agent-a", "anthropic", "claude-opus-4", 100_000, 50_000)
            .await;

        // Agent B: Light tier (flash)
        tracker
            .record_usage("agent-b", "google", "gemini-flash", 200_000, 100_000)
            .await;

        let cost_a = tracker.get_agent_cost("agent-a").await.unwrap();
        let cost_b = tracker.get_agent_cost("agent-b").await.unwrap();

        // Agent A: 150k tokens, Heavy = 25 DD/1M -> 0.15 * 25 = 3.75 DD
        assert_eq!(cost_a.tokens_in, 100_000);
        assert_eq!(cost_a.tokens_out, 50_000);
        assert!((cost_a.cost_dd - 3.75).abs() < 0.001);
        assert_eq!(cost_a.request_count, 1);

        // Agent B: 300k tokens, Light = 1 DD/1M -> 0.3 * 1 = 0.3 DD
        assert_eq!(cost_b.tokens_in, 200_000);
        assert_eq!(cost_b.tokens_out, 100_000);
        assert!((cost_b.cost_dd - 0.3).abs() < 0.001);
        assert_eq!(cost_b.request_count, 1);
    }

    #[tokio::test]
    async fn test_session_total_sums_correctly() {
        let tracker = CostTracker::new();

        tracker
            .record_usage("agent-a", "anthropic", "claude-opus-4", 100_000, 50_000)
            .await;
        tracker
            .record_usage("agent-b", "google", "gemini-flash", 200_000, 100_000)
            .await;

        let total = tracker.get_session_total().await;
        assert_eq!(total.tokens_in, 300_000);
        assert_eq!(total.tokens_out, 150_000);
        // 3.75 (agent-a) + 0.3 (agent-b) = 4.05 DD
        assert!((total.cost_dd - 4.05).abs() < 0.001);
        assert_eq!(total.request_count, 2);
    }

    #[tokio::test]
    async fn test_tier_rates_correct() {
        // Light: 1 DD per 1M tokens
        assert!((ModelTier::Light.dd_per_million_tokens() - 1.0).abs() < f64::EPSILON);
        // Medium: 5 DD per 1M tokens
        assert!((ModelTier::Medium.dd_per_million_tokens() - 5.0).abs() < f64::EPSILON);
        // Heavy: 25 DD per 1M tokens
        assert!((ModelTier::Heavy.dd_per_million_tokens() - 25.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_tier_classification() {
        assert_eq!(ModelTier::classify("claude-opus-4"), ModelTier::Heavy);
        assert_eq!(ModelTier::classify("gpt-4o"), ModelTier::Heavy);
        assert_eq!(ModelTier::classify("gpt-4-turbo"), ModelTier::Heavy);
        assert_eq!(ModelTier::classify("claude-sonnet-4"), ModelTier::Medium);
        assert_eq!(ModelTier::classify("gpt-3.5-turbo"), ModelTier::Medium);
        assert_eq!(ModelTier::classify("claude-haiku-3"), ModelTier::Light);
        assert_eq!(ModelTier::classify("gpt-4o-mini"), ModelTier::Medium);
        assert_eq!(ModelTier::classify("gemini-flash"), ModelTier::Light);
        assert_eq!(ModelTier::classify("llama-3-8b"), ModelTier::Light);
    }

    #[tokio::test]
    async fn test_accumulates_multiple_requests() {
        let tracker = CostTracker::new();

        // Two requests for the same agent.
        tracker
            .record_usage("agent-1", "anthropic", "claude-sonnet-4", 100_000, 50_000)
            .await;
        tracker
            .record_usage("agent-1", "anthropic", "claude-sonnet-4", 200_000, 100_000)
            .await;

        let cost = tracker.get_agent_cost("agent-1").await.unwrap();
        assert_eq!(cost.tokens_in, 300_000);
        assert_eq!(cost.tokens_out, 150_000);
        assert_eq!(cost.request_count, 2);
        // 150k * 5/1M + 300k * 5/1M = 0.75 + 1.5 = 2.25 DD
        assert!((cost.cost_dd - 2.25).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_reset_clears_all() {
        let tracker = CostTracker::new();

        tracker
            .record_usage("agent-1", "openai", "gpt-4o", 100_000, 50_000)
            .await;
        assert!(tracker.get_agent_cost("agent-1").await.is_some());

        tracker.reset().await;

        assert!(tracker.get_agent_cost("agent-1").await.is_none());
        let all = tracker.get_all_costs().await;
        assert!(all.is_empty());
        let total = tracker.get_session_total().await;
        assert_eq!(total.request_count, 0);
    }

    #[tokio::test]
    async fn test_get_all_costs() {
        let tracker = CostTracker::new();

        tracker
            .record_usage("agent-a", "anthropic", "claude-opus-4", 100_000, 50_000)
            .await;
        tracker
            .record_usage("agent-b", "openai", "gpt-4o", 200_000, 100_000)
            .await;

        let all = tracker.get_all_costs().await;
        assert_eq!(all.len(), 2);

        let ids: Vec<&str> = all.iter().map(|c| c.agent_id.as_str()).collect();
        assert!(ids.contains(&"agent-a"));
        assert!(ids.contains(&"agent-b"));
    }

    #[tokio::test]
    async fn test_event_bus_integration() {
        let bus = Arc::new(EventBus::new(64));
        let tracker = CostTracker::with_event_bus(bus.clone());
        let mut rx = bus.subscribe();

        tracker
            .record_usage("agent-1", "anthropic", "claude-opus-4", 100_000, 50_000)
            .await;

        let event = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
            .await
            .expect("timed out")
            .expect("recv failed");

        assert_eq!(event.agent_id, "agent-1");
        match event.event_type {
            EventType::CostUpdated {
                session_tokens,
                session_cost_dd,
            } => {
                assert_eq!(session_tokens, 150_000);
                assert!((session_cost_dd - 3.75).abs() < 0.001);
            }
            other => panic!("unexpected event type: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_nonexistent_agent() {
        let tracker = CostTracker::new();
        assert!(tracker.get_agent_cost("nonexistent").await.is_none());
    }

    // -----------------------------------------------------------------------
    // SQLite cost persistence tests (D1D-267)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_cost_persisted_to_sqlite() {
        let db_handle = db::init_memory().expect("init in-memory db");
        let tracker = CostTracker::with_db(db_handle.clone());

        tracker
            .record_usage("agent-1", "anthropic", "claude-opus-4", 100_000, 50_000)
            .await;

        // Verify written to session_costs table.
        let conn = db_handle.lock().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM session_costs WHERE agent_id = ?1",
                rusqlite::params!["agent-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        let model: String = conn
            .query_row(
                "SELECT model FROM session_costs WHERE agent_id = ?1",
                rusqlite::params!["agent-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(model, "claude-opus-4");
    }

    #[tokio::test]
    async fn test_cost_roundtrip_save_load() {
        let db_handle = db::init_memory().expect("init in-memory db");

        // Tracker 1: record some usage.
        let tracker1 = CostTracker::with_db(db_handle.clone());
        tracker1
            .record_usage("agent-a", "anthropic", "claude-opus-4", 100_000, 50_000)
            .await;
        tracker1
            .record_usage("agent-b", "openai", "gpt-4o-mini", 200_000, 100_000)
            .await;

        // Tracker 2: simulate restart — load from same DB.
        let tracker2 = CostTracker::with_db(db_handle);
        let loaded = tracker2.load_from_db().await.unwrap();
        assert_eq!(loaded, 2);

        let cost_a = tracker2.get_agent_cost("agent-a").await.unwrap();
        assert_eq!(cost_a.tokens_in, 100_000);
        assert_eq!(cost_a.tokens_out, 50_000);
        // Heavy tier: 150k tokens * 25/1M = 3.75 DD
        assert!((cost_a.cost_dd - 3.75).abs() < 0.001);
        assert_eq!(cost_a.request_count, 1);

        let cost_b = tracker2.get_agent_cost("agent-b").await.unwrap();
        assert_eq!(cost_b.tokens_in, 200_000);
        assert_eq!(cost_b.tokens_out, 100_000);
        // Medium tier: 300k tokens * 5/1M = 1.5 DD
        assert!((cost_b.cost_dd - 1.5).abs() < 0.001);
        assert_eq!(cost_b.request_count, 1);
    }

    #[tokio::test]
    async fn test_multiple_costs_same_agent_persisted() {
        let db_handle = db::init_memory().expect("init in-memory db");
        let tracker = CostTracker::with_db(db_handle.clone());

        tracker
            .record_usage("agent-1", "anthropic", "claude-opus-4", 100_000, 50_000)
            .await;
        tracker
            .record_usage("agent-1", "anthropic", "claude-opus-4", 200_000, 100_000)
            .await;

        // Should have 2 separate records in the DB.
        let conn = db_handle.lock().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM session_costs WHERE agent_id = ?1",
                rusqlite::params!["agent-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }
}
