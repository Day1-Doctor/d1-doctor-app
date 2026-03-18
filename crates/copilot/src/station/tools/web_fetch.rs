use serde::{Deserialize, Serialize};

use super::McpTool;

/// The result of fetching a URL.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FetchResult {
    pub url: String,
    pub status_code: u16,
    pub content_type: String,
    pub body: String,
}

/// A table extracted from a web page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// MCP tool server interface for fetching web pages.
///
/// Stub implementation — returns mock data without making real HTTP requests.
pub struct WebFetchTool;

impl WebFetchTool {
    pub fn new() -> Self {
        Self
    }

    /// Fetch a URL and return its full response.
    pub fn fetch_url(&self, url: &str) -> FetchResult {
        FetchResult {
            url: url.to_string(),
            status_code: 200,
            content_type: "text/html".to_string(),
            body: format!("<html><body>Mock content for {}</body></html>", url),
        }
    }

    /// Fetch a URL and extract its text content (stripping HTML tags).
    pub fn extract_text(&self, url: &str) -> String {
        format!("Extracted text content from {}", url)
    }

    /// Fetch a URL and extract any HTML tables found on the page.
    pub fn extract_tables(&self, url: &str) -> Vec<Table> {
        vec![Table {
            headers: vec!["Column A".to_string(), "Column B".to_string()],
            rows: vec![vec![
                format!("mock-row-from-{}", url),
                "value".to_string(),
            ]],
        }]
    }
}

impl Default for WebFetchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl McpTool for WebFetchTool {
    fn name(&self) -> &str {
        "web-fetch"
    }

    fn risk_level(&self) -> &str {
        "low"
    }

    fn description(&self) -> &str {
        "Fetch web pages and extract text or tabular content"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_url_returns_mock() {
        let tool = WebFetchTool::new();
        let result = tool.fetch_url("https://example.com");
        assert_eq!(result.status_code, 200);
        assert_eq!(result.url, "https://example.com");
        assert!(result.body.contains("example.com"));
        assert_eq!(result.content_type, "text/html");
    }

    #[test]
    fn test_extract_text_and_tables() {
        let tool = WebFetchTool::new();

        let text = tool.extract_text("https://example.com/data");
        assert!(text.contains("example.com/data"));

        let tables = tool.extract_tables("https://example.com/table");
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].headers.len(), 2);
        assert_eq!(tables[0].rows.len(), 1);
    }
}
