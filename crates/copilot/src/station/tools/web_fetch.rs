use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::McpTool;

/// Maximum response body size: 1 MB.
const MAX_BODY_SIZE: usize = 1_024 * 1_024;

/// HTTP request timeout: 30 seconds.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

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
/// Real implementation using `reqwest` with timeout and size limits.
pub struct WebFetchTool {
    client: reqwest::Client,
}

impl WebFetchTool {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent("Day1Doctor-Copilot/3.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }

    /// Fetch a URL and return its full response.
    ///
    /// Enforces a 30-second timeout and 1 MB body size limit.
    pub async fn fetch_url(&self, url: &str) -> Result<FetchResult, String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("fetch '{}': {e}", url))?;

        let status_code = response.status().as_u16();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        // Check content-length header if available.
        if let Some(len) = response.content_length() {
            if len as usize > MAX_BODY_SIZE {
                return Err(format!(
                    "response too large: {len} bytes (limit: {MAX_BODY_SIZE})"
                ));
            }
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("read response body: {e}"))?;

        if bytes.len() > MAX_BODY_SIZE {
            return Err(format!(
                "response too large: {} bytes (limit: {MAX_BODY_SIZE})",
                bytes.len()
            ));
        }

        let body = String::from_utf8_lossy(&bytes).to_string();

        Ok(FetchResult {
            url: url.to_string(),
            status_code,
            content_type,
            body,
        })
    }

    /// Strip HTML tags from a string and return plain text.
    ///
    /// Uses a simple state-machine parser — no additional dependencies needed.
    pub fn extract_text(html: &str) -> String {
        let mut result = String::with_capacity(html.len());
        let mut in_tag = false;
        let mut in_script = false;
        let mut in_style = false;
        let mut tag_name = String::new();
        let mut collecting_tag_name = false;
        let mut is_closing_tag = false;

        for ch in html.chars() {
            match ch {
                '<' => {
                    in_tag = true;
                    collecting_tag_name = true;
                    is_closing_tag = false;
                    tag_name.clear();
                }
                '>' => {
                    in_tag = false;
                    collecting_tag_name = false;
                    let tag_lower = tag_name.to_lowercase();
                    if is_closing_tag {
                        if tag_lower == "script" {
                            in_script = false;
                        } else if tag_lower == "style" {
                            in_style = false;
                        } else if tag_lower == "p" || tag_lower == "div" {
                            result.push('\n');
                        }
                    } else {
                        if tag_lower == "script" {
                            in_script = true;
                        } else if tag_lower == "style" {
                            in_style = true;
                        } else if tag_lower == "br"
                            || tag_lower == "p"
                            || tag_lower == "div"
                            || tag_lower == "li"
                        {
                            result.push('\n');
                        }
                    }
                }
                _ if in_tag => {
                    if collecting_tag_name {
                        if ch == '/' && tag_name.is_empty() {
                            // This is a closing tag: </tagname>
                            is_closing_tag = true;
                        } else if ch.is_whitespace() || ch == '/' {
                            collecting_tag_name = false;
                        } else {
                            tag_name.push(ch);
                        }
                    }
                }
                _ if in_script || in_style => {}
                _ => {
                    result.push(ch);
                }
            }
        }

        // Decode common HTML entities.
        let result = result
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&nbsp;", " ");

        // Collapse excessive whitespace while preserving line structure.
        let mut cleaned = String::with_capacity(result.len());
        let mut prev_was_newline = false;
        for line in result.lines() {
            let trimmed = line.split_whitespace().collect::<Vec<_>>().join(" ");
            if trimmed.is_empty() {
                if !prev_was_newline {
                    cleaned.push('\n');
                    prev_was_newline = true;
                }
            } else {
                if prev_was_newline && !cleaned.is_empty() {
                    // Already added a newline.
                }
                cleaned.push_str(&trimmed);
                cleaned.push('\n');
                prev_was_newline = false;
            }
        }

        cleaned.trim().to_string()
    }

    /// Fetch a URL and extract any HTML tables found on the page.
    ///
    /// Uses a basic regex-free parser for simple HTML tables.
    /// Complex or nested tables may not be fully captured.
    pub async fn extract_tables(&self, url: &str) -> Result<Vec<Table>, String> {
        let result = self.fetch_url(url).await?;
        Ok(Self::parse_tables_from_html(&result.body))
    }

    /// Parse HTML tables from a string.
    fn parse_tables_from_html(html: &str) -> Vec<Table> {
        let mut tables = Vec::new();
        let lower = html.to_lowercase();
        let mut search_from = 0;

        while let Some(table_start) = lower[search_from..].find("<table") {
            let abs_start = search_from + table_start;
            let Some(table_end) = lower[abs_start..].find("</table>") else {
                break;
            };
            let abs_end = abs_start + table_end + 8;
            let table_html = &html[abs_start..abs_end];

            let mut headers = Vec::new();
            let mut rows = Vec::new();
            let table_lower = table_html.to_lowercase();

            // Extract <th> headers.
            let mut th_from = 0;
            while let Some(th_start) = table_lower[th_from..].find("<th") {
                let abs_th = th_from + th_start;
                if let Some(close) = table_lower[abs_th..].find('>') {
                    let content_start = abs_th + close + 1;
                    if let Some(th_end) = table_lower[content_start..].find("</th>") {
                        let text = Self::extract_text(&table_html[content_start..content_start + th_end]);
                        headers.push(text.trim().to_string());
                        th_from = content_start + th_end + 5;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            // Extract <tr> rows with <td> cells.
            let mut tr_from = 0;
            while let Some(tr_start) = table_lower[tr_from..].find("<tr") {
                let abs_tr = tr_from + tr_start;
                if let Some(tr_end) = table_lower[abs_tr..].find("</tr>") {
                    let row_html = &table_html[abs_tr..abs_tr + tr_end + 5];
                    let row_lower = row_html.to_lowercase();

                    let mut cells = Vec::new();
                    let mut td_from = 0;
                    while let Some(td_start) = row_lower[td_from..].find("<td") {
                        let abs_td = td_from + td_start;
                        if let Some(close) = row_lower[abs_td..].find('>') {
                            let content_start = abs_td + close + 1;
                            if let Some(td_end) = row_lower[content_start..].find("</td>") {
                                let text = Self::extract_text(
                                    &row_html[content_start..content_start + td_end],
                                );
                                cells.push(text.trim().to_string());
                                td_from = content_start + td_end + 5;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    if !cells.is_empty() {
                        rows.push(cells);
                    }

                    tr_from = abs_tr + tr_end + 5;
                } else {
                    break;
                }
            }

            if !headers.is_empty() || !rows.is_empty() {
                tables.push(Table { headers, rows });
            }

            search_from = abs_end;
        }

        tables
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
    fn test_extract_text_strips_tags() {
        let html = "<html><body><h1>Hello</h1><p>World</p></body></html>";
        let text = WebFetchTool::extract_text(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert!(!text.contains("<h1>"));
        assert!(!text.contains("<p>"));
    }

    #[test]
    fn test_extract_text_strips_script_and_style() {
        let html = r#"
            <html>
            <head><style>body { color: red; }</style></head>
            <body>
                <script>var x = 1;</script>
                <p>Visible content</p>
            </body>
            </html>
        "#;
        let text = WebFetchTool::extract_text(html);
        assert!(text.contains("Visible content"));
        assert!(!text.contains("color: red"));
        assert!(!text.contains("var x"));
    }

    #[test]
    fn test_extract_text_decodes_entities() {
        let html = "<p>A &amp; B &lt; C &gt; D &quot;E&quot; &#39;F&#39;</p>";
        let text = WebFetchTool::extract_text(html);
        assert!(text.contains("A & B < C > D \"E\" 'F'"));
    }

    #[test]
    fn test_extract_text_empty_input() {
        assert_eq!(WebFetchTool::extract_text(""), "");
    }

    #[test]
    fn test_extract_text_plain_text_passthrough() {
        let text = WebFetchTool::extract_text("Just plain text");
        assert_eq!(text, "Just plain text");
    }

    #[test]
    fn test_parse_tables_from_html() {
        let html = r#"
            <table>
                <tr><th>Name</th><th>Age</th></tr>
                <tr><td>Alice</td><td>30</td></tr>
                <tr><td>Bob</td><td>25</td></tr>
            </table>
        "#;
        let tables = WebFetchTool::parse_tables_from_html(html);
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].headers, vec!["Name", "Age"]);
        assert_eq!(tables[0].rows.len(), 2);
        assert_eq!(tables[0].rows[0], vec!["Alice", "30"]);
        assert_eq!(tables[0].rows[1], vec!["Bob", "25"]);
    }

    #[test]
    fn test_parse_tables_no_tables() {
        let html = "<html><body><p>No tables here</p></body></html>";
        let tables = WebFetchTool::parse_tables_from_html(html);
        assert!(tables.is_empty());
    }

    #[test]
    fn test_mcp_trait() {
        let tool = WebFetchTool::new();
        assert_eq!(tool.name(), "web-fetch");
        assert_eq!(tool.risk_level(), "low");
        assert!(!tool.description().is_empty());
    }
}
