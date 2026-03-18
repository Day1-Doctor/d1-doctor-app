use serde::{Deserialize, Serialize};

use super::McpTool;

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
/// Stub implementation — all methods return mock data.
/// The real implementation will proxy to an MCP web-search server process.
pub struct WebSearchTool;

impl WebSearchTool {
    pub fn new() -> Self {
        Self
    }

    /// Search the web for a query, returning up to `limit` results.
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let count = limit.min(3);
        (0..count)
            .map(|i| SearchResult {
                title: format!("Result {} for: {}", i + 1, query),
                url: format!("https://example.com/search?q={}&p={}", query, i),
                snippet: format!("This is a mock snippet for result {} matching '{}'.", i + 1, query),
                source: "mock-search-engine".to_string(),
            })
            .collect()
    }

    /// Search for recent news articles matching the query.
    pub fn search_news(&self, query: &str) -> Vec<SearchResult> {
        vec![SearchResult {
            title: format!("Breaking: {}", query),
            url: format!("https://news.example.com/{}", query.replace(' ', "-")),
            snippet: format!("Latest news coverage about {}.", query),
            source: "mock-news".to_string(),
        }]
    }

    /// Search for images matching the query.
    pub fn search_images(&self, query: &str) -> Vec<SearchResult> {
        vec![SearchResult {
            title: format!("Image: {}", query),
            url: format!("https://images.example.com/{}.png", query.replace(' ', "-")),
            snippet: format!("Mock image result for '{}'.", query),
            source: "mock-images".to_string(),
        }]
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
    fn test_search_returns_limited_results() {
        let tool = WebSearchTool::new();

        let results = tool.search("rust programming", 5);
        assert_eq!(results.len(), 3, "stub caps at 3 results");

        let results = tool.search("rust", 2);
        assert_eq!(results.len(), 2);

        for r in &results {
            assert!(r.title.contains("rust"));
            assert!(!r.url.is_empty());
            assert!(!r.snippet.is_empty());
            assert_eq!(r.source, "mock-search-engine");
        }
    }

    #[test]
    fn test_search_news_returns_results() {
        let tool = WebSearchTool::new();
        let results = tool.search_news("AI advances");
        assert_eq!(results.len(), 1);
        assert!(results[0].title.contains("AI advances"));
        assert_eq!(results[0].source, "mock-news");
    }

    #[test]
    fn test_search_images_returns_results() {
        let tool = WebSearchTool::new();
        let results = tool.search_images("sunset");
        assert_eq!(results.len(), 1);
        assert!(results[0].title.contains("sunset"));
        assert!(results[0].url.contains(".png"));
        assert_eq!(results[0].source, "mock-images");
    }
}
