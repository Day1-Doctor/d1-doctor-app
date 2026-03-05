//! MCP Wrapper for QMD
//!
//! Provides a high-level interface to QMD's capabilities as MCP tools.
//! Translates tool calls into JSON-RPC requests sent to the QMD process
//! via STDIO transport.

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::qmd::QmdManager;

/// Tool definitions exposed by the QMD MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// The QMD MCP server wrapper.
///
/// Wraps a `QmdManager` and exposes QMD's functionality as
/// discrete MCP tool calls (search, ingest, ingest_url).
pub struct QmdServer {
    manager: QmdManager,
}

impl QmdServer {
    /// Create a new QmdServer wrapping the given manager.
    pub fn new(manager: QmdManager) -> Self {
        Self { manager }
    }

    /// Return the list of tools this server provides.
    pub fn tool_definitions(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "search".to_string(),
                description: "Search indexed content using semantic similarity".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results to return",
                            "default": 10
                        }
                    },
                    "required": ["query"]
                }),
            },
            ToolDefinition {
                name: "ingest".to_string(),
                description: "Ingest text content into the QMD index".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "The text content to ingest"
                        },
                        "metadata": {
                            "type": "object",
                            "description": "Optional metadata to attach to the content",
                            "additionalProperties": { "type": "string" }
                        }
                    },
                    "required": ["content"]
                }),
            },
            ToolDefinition {
                name: "ingest_url".to_string(),
                description: "Fetch and ingest content from a URL".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to fetch and ingest"
                        }
                    },
                    "required": ["url"]
                }),
            },
        ]
    }

    /// Dispatch a tool call by name to the appropriate handler.
    ///
    /// Returns `Ok(value)` on success, or an error if the tool name
    /// is unknown or the underlying QMD call fails.
    pub async fn handle_tool_call(
        &mut self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        match tool_name {
            "search" => self.search(arguments).await,
            "ingest" => self.ingest(arguments).await,
            "ingest_url" => self.ingest_url(arguments).await,
            _ => Err(anyhow::anyhow!("Unknown QMD tool: {}", tool_name)),
        }
    }

    /// Search indexed content.
    ///
    /// Arguments:
    /// - `query` (string, required): The search query.
    /// - `limit` (integer, optional): Max results. Defaults to config max_results.
    async fn search(&mut self, arguments: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let query = arguments
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?;

        let default_limit = self.manager.config().max_results;
        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(default_limit);

        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "search",
                "arguments": {
                    "query": query,
                    "limit": limit
                }
            }
        });

        self.manager.send_request(request).await
    }

    /// Ingest text content with optional metadata.
    ///
    /// Arguments:
    /// - `content` (string, required): The text content to ingest.
    /// - `metadata` (object, optional): Key-value metadata pairs.
    async fn ingest(&mut self, arguments: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let content = arguments
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;

        let metadata = arguments
            .get("metadata")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "ingest",
                "arguments": {
                    "content": content,
                    "metadata": metadata
                }
            }
        });

        self.manager.send_request(request).await
    }

    /// Ingest content from a URL.
    ///
    /// Arguments:
    /// - `url` (string, required): The URL to fetch and ingest.
    async fn ingest_url(&mut self, arguments: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let url = arguments
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: url"))?;

        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "ingest_url",
                "arguments": {
                    "url": url
                }
            }
        });

        self.manager.send_request(request).await
    }

    /// Get a reference to the underlying QmdManager.
    pub fn manager(&self) -> &QmdManager {
        &self.manager
    }

    /// Get a mutable reference to the underlying QmdManager.
    pub fn manager_mut(&mut self) -> &mut QmdManager {
        &mut self.manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qmd::QmdConfig;
    use std::path::PathBuf;

    fn test_server() -> QmdServer {
        let config = QmdConfig {
            enabled: false,
            binary_path: PathBuf::from("/nonexistent/qmd"),
            model_path: PathBuf::from("/tmp/qmd-models"),
            max_results: 5,
        };
        QmdServer::new(QmdManager::new(config))
    }

    #[test]
    fn test_tool_definitions() {
        let server = test_server();
        let tools = server.tool_definitions();
        assert_eq!(tools.len(), 3);
        assert_eq!(tools[0].name, "search");
        assert_eq!(tools[1].name, "ingest");
        assert_eq!(tools[2].name, "ingest_url");
    }

    #[tokio::test]
    async fn test_unknown_tool() {
        let mut server = test_server();
        let result = server.handle_tool_call("nonexistent", json!({})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown QMD tool"));
    }

    #[tokio::test]
    async fn test_search_missing_query() {
        let mut server = test_server();
        let result = server.handle_tool_call("search", json!({})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("query"));
    }

    #[tokio::test]
    async fn test_ingest_missing_content() {
        let mut server = test_server();
        let result = server.handle_tool_call("ingest", json!({})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("content"));
    }

    #[tokio::test]
    async fn test_ingest_url_missing_url() {
        let mut server = test_server();
        let result = server.handle_tool_call("ingest_url", json!({})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("url"));
    }
}
