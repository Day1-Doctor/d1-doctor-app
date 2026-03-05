//! REST API server for memory search and health endpoints.
//!
//! Provides HTTP endpoints for the client to query agent memory
//! and check daemon health status.

use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Query parameters for memory search
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    /// Search query string
    pub q: String,
    /// Scope filter (e.g., "system", "user", "session")
    pub scope: Option<String>,
    /// Maximum number of results to return
    pub limit: Option<usize>,
}

/// A single memory search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    /// Unique identifier for the memory entry
    pub id: String,
    /// The memory key or category
    pub key: String,
    /// The memory content
    pub value: String,
    /// Source of the memory (e.g., "profile", "session", "user")
    pub source: String,
    /// Relevance score (0.0 to 1.0)
    pub relevance: f64,
    /// Timestamp when the memory was created/updated
    pub timestamp: i64,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

/// Build the REST API router
pub fn build_router() -> Router {
    Router::new()
        .route("/api/memory/search", get(memory_search))
        .route("/api/health", get(health_check))
}

/// Start the REST API server on the given address
pub async fn start_rest_server(addr: SocketAddr) -> anyhow::Result<()> {
    let app = build_router();
    tracing::info!(%addr, "REST API server starting");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Handler for GET /api/memory/search?q=...&scope=...&limit=...
async fn memory_search(
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(10).min(100);
    let scope = params.scope.as_deref().unwrap_or("all");

    tracing::debug!(
        query = %params.q,
        scope = %scope,
        limit = %limit,
        "Memory search request"
    );

    // Perform a simple substring match against known memory entries.
    // In production, this would query the local SQLite database
    // with full-text search or vector similarity.
    let results = search_memory(&params.q, scope, limit);

    Json(results)
}

/// Handler for GET /api/health
async fn health_check() -> impl IntoResponse {
    let response = HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // TODO: track actual uptime
    };

    (StatusCode::OK, Json(response))
}

/// Search memory entries by query string.
///
/// This is a placeholder implementation that returns matches
/// based on simple substring matching. A real implementation
/// would use SQLite FTS5 or vector similarity search.
fn search_memory(query: &str, scope: &str, limit: usize) -> Vec<MemorySearchResult> {
    let query_lower = query.to_lowercase();

    // Seed entries from profile detection and system facts
    let all_entries = get_seed_entries();

    let mut results: Vec<MemorySearchResult> = all_entries
        .into_iter()
        .filter(|entry| {
            // Scope filter
            if scope != "all" && entry.source != scope {
                return false;
            }
            // Substring relevance
            entry.key.to_lowercase().contains(&query_lower)
                || entry.value.to_lowercase().contains(&query_lower)
        })
        .map(|mut entry| {
            // Compute a simple relevance score
            entry.relevance = compute_relevance(&entry, &query_lower);
            entry
        })
        .collect();

    // Sort by relevance descending
    results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(limit);
    results
}

/// Compute a simple relevance score based on match quality
fn compute_relevance(entry: &MemorySearchResult, query: &str) -> f64 {
    let key_lower = entry.key.to_lowercase();
    let value_lower = entry.value.to_lowercase();

    if key_lower == query {
        1.0
    } else if key_lower.contains(query) && value_lower.contains(query) {
        0.9
    } else if key_lower.contains(query) {
        0.7
    } else if value_lower.contains(query) {
        0.5
    } else {
        0.1
    }
}

/// Return seed memory entries (system profile facts).
/// In a full implementation, these would come from the local database.
fn get_seed_entries() -> Vec<MemorySearchResult> {
    use crate::profile_detect::detect_system_profile;

    let facts = detect_system_profile();
    let now = chrono::Utc::now().timestamp();

    facts
        .into_iter()
        .enumerate()
        .map(|(i, fact)| MemorySearchResult {
            id: format!("profile-{}", i),
            key: fact.key,
            value: fact.value,
            source: fact.source,
            relevance: 0.0,
            timestamp: now,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_search_returns_results() {
        let results = search_memory("os", "all", 10);
        assert!(!results.is_empty(), "Should find OS-related entries");
    }

    #[test]
    fn test_memory_search_respects_limit() {
        let results = search_memory("", "all", 2);
        // With empty query nothing matches substring, so could be empty
        // but limit should still be respected if there were matches
        assert!(results.len() <= 2);
    }

    #[test]
    fn test_memory_search_scope_filter() {
        let results = search_memory("os", "system", 10);
        for entry in &results {
            assert_eq!(entry.source, "system");
        }
    }

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "ok".to_string(),
            version: "0.1.0".to_string(),
            uptime_seconds: 42,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
    }

    #[test]
    fn test_compute_relevance_exact_match() {
        let entry = MemorySearchResult {
            id: "1".to_string(),
            key: "os".to_string(),
            value: "macOS".to_string(),
            source: "system".to_string(),
            relevance: 0.0,
            timestamp: 0,
        };
        let score = compute_relevance(&entry, "os");
        assert!((score - 1.0).abs() < f64::EPSILON);
    }
}
