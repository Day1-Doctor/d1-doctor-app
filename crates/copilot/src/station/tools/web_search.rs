use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::McpTool;

/// HTTP request timeout for search API calls: 15 seconds.
const SEARCH_TIMEOUT: Duration = Duration::from_secs(15);

/// A single search result returned by the web search tool.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source: String,
}

/// MCP tool server interface for web search.
///
/// Real implementation that calls external search APIs.
///
/// Supported backends (checked in order):
/// 1. **Brave Search** — set `BRAVE_API_KEY` environment variable
/// 2. **Tavily Search** — set `TAVILY_API_KEY` environment variable
///
/// If no API key is configured, search returns a helpful error message.
pub struct WebSearchTool {
    client: reqwest::Client,
}

impl WebSearchTool {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(SEARCH_TIMEOUT)
            .user_agent("Day1Doctor-Copilot/3.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }

    /// Search the web for a query, returning up to `limit` results.
    ///
    /// Tries Brave Search first, then Tavily. Returns an error if no API key
    /// is configured.
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, String> {
        if let Ok(api_key) = std::env::var("BRAVE_API_KEY") {
            if !api_key.is_empty() {
                return self.search_brave(query, limit, &api_key).await;
            }
        }

        if let Ok(api_key) = std::env::var("TAVILY_API_KEY") {
            if !api_key.is_empty() {
                return self.search_tavily(query, limit, &api_key).await;
            }
        }

        Err(
            "No search API key configured. Set BRAVE_API_KEY or TAVILY_API_KEY \
             environment variable to enable web search."
                .to_string(),
        )
    }

    /// Search for recent news articles matching the query.
    ///
    /// Uses the same backend as `search` but requests news-specific results
    /// where the API supports it.
    pub async fn search_news(
        &self,
        query: &str,
    ) -> Result<Vec<SearchResult>, String> {
        if let Ok(api_key) = std::env::var("BRAVE_API_KEY") {
            if !api_key.is_empty() {
                return self.search_brave_news(query, &api_key).await;
            }
        }

        // Tavily doesn't have a dedicated news endpoint — use general search
        // with a news-biased query.
        if let Ok(api_key) = std::env::var("TAVILY_API_KEY") {
            if !api_key.is_empty() {
                let news_query = format!("{query} latest news");
                return self.search_tavily(&news_query, 5, &api_key).await;
            }
        }

        Err(
            "No search API key configured. Set BRAVE_API_KEY or TAVILY_API_KEY \
             environment variable to enable web search."
                .to_string(),
        )
    }

    /// Search for images matching the query.
    ///
    /// Currently only supported via Brave Search API.
    pub async fn search_images(
        &self,
        query: &str,
    ) -> Result<Vec<SearchResult>, String> {
        if let Ok(api_key) = std::env::var("BRAVE_API_KEY") {
            if !api_key.is_empty() {
                return self.search_brave_images(query, &api_key).await;
            }
        }

        Err(
            "Image search requires BRAVE_API_KEY environment variable."
                .to_string(),
        )
    }

    // ── Brave Search API ────────────────────────────────────────────

    async fn search_brave(
        &self,
        query: &str,
        limit: usize,
        api_key: &str,
    ) -> Result<Vec<SearchResult>, String> {
        let resp = self
            .client
            .get("https://api.search.brave.com/res/v1/web/search")
            .header("X-Subscription-Token", api_key)
            .header("Accept", "application/json")
            .query(&[
                ("q", query),
                ("count", &limit.min(20).to_string()),
            ])
            .send()
            .await
            .map_err(|e| format!("Brave search request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!(
                "Brave search returned status {}",
                resp.status()
            ));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("parse Brave response: {e}"))?;

        let results = body["web"]["results"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .take(limit)
                    .map(|item| SearchResult {
                        title: item["title"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        url: item["url"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        snippet: item["description"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        source: "brave".to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(results)
    }

    async fn search_brave_news(
        &self,
        query: &str,
        api_key: &str,
    ) -> Result<Vec<SearchResult>, String> {
        let resp = self
            .client
            .get("https://api.search.brave.com/res/v1/news/search")
            .header("X-Subscription-Token", api_key)
            .header("Accept", "application/json")
            .query(&[("q", query), ("count", "10")])
            .send()
            .await
            .map_err(|e| format!("Brave news search failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!(
                "Brave news search returned status {}",
                resp.status()
            ));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("parse Brave news response: {e}"))?;

        let results = body["results"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|item| SearchResult {
                        title: item["title"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        url: item["url"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        snippet: item["description"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        source: "brave-news".to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(results)
    }

    async fn search_brave_images(
        &self,
        query: &str,
        api_key: &str,
    ) -> Result<Vec<SearchResult>, String> {
        let resp = self
            .client
            .get("https://api.search.brave.com/res/v1/images/search")
            .header("X-Subscription-Token", api_key)
            .header("Accept", "application/json")
            .query(&[("q", query), ("count", "10")])
            .send()
            .await
            .map_err(|e| format!("Brave image search failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!(
                "Brave image search returned status {}",
                resp.status()
            ));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("parse Brave image response: {e}"))?;

        let results = body["results"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|item| SearchResult {
                        title: item["title"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        url: item["url"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        snippet: item["source"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        source: "brave-images".to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(results)
    }

    // ── Tavily Search API ───────────────────────────────────────────

    async fn search_tavily(
        &self,
        query: &str,
        limit: usize,
        api_key: &str,
    ) -> Result<Vec<SearchResult>, String> {
        let body = serde_json::json!({
            "api_key": api_key,
            "query": query,
            "max_results": limit.min(10),
            "include_answer": false,
        });

        let resp = self
            .client
            .post("https://api.tavily.com/search")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Tavily search request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!(
                "Tavily search returned status {}",
                resp.status()
            ));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("parse Tavily response: {e}"))?;

        let results = body["results"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .take(limit)
                    .map(|item| SearchResult {
                        title: item["title"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        url: item["url"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        snippet: item["content"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        source: "tavily".to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(results)
    }
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl McpTool for WebSearchTool {
    fn name(&self) -> &str {
        "web-search"
    }

    fn risk_level(&self) -> &str {
        "low"
    }

    fn description(&self) -> &str {
        "Search the web for information, news, and images"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_struct() {
        let result = SearchResult {
            title: "Test Title".to_string(),
            url: "https://example.com".to_string(),
            snippet: "A test snippet".to_string(),
            source: "test".to_string(),
        };

        assert_eq!(result.title, "Test Title");
        assert_eq!(result.url, "https://example.com");
        assert_eq!(result.snippet, "A test snippet");
        assert_eq!(result.source, "test");
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            title: "Test".to_string(),
            url: "https://example.com".to_string(),
            snippet: "Snippet".to_string(),
            source: "brave".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SearchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, deserialized);
    }

    #[tokio::test]
    async fn test_search_no_api_key_returns_error() {
        // Ensure no API keys are set for this test.
        std::env::remove_var("BRAVE_API_KEY");
        std::env::remove_var("TAVILY_API_KEY");

        let tool = WebSearchTool::new();
        let result = tool.search("test query", 5).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("No search API key configured"));
    }

    #[tokio::test]
    async fn test_search_news_no_api_key_returns_error() {
        std::env::remove_var("BRAVE_API_KEY");
        std::env::remove_var("TAVILY_API_KEY");

        let tool = WebSearchTool::new();
        let result = tool.search_news("test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_images_no_api_key_returns_error() {
        std::env::remove_var("BRAVE_API_KEY");

        let tool = WebSearchTool::new();
        let result = tool.search_images("test").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("BRAVE_API_KEY"));
    }

    #[test]
    fn test_mcp_trait() {
        let tool = WebSearchTool::new();
        assert_eq!(tool.name(), "web-search");
        assert_eq!(tool.risk_level(), "low");
        assert!(!tool.description().is_empty());
    }
}
