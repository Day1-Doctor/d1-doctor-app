use std::path::PathBuf;
use std::time::Instant;

use serde_json::{json, Value};

use crate::station::tools::document::DocumentTool;
use crate::station::tools::filesystem::FilesystemTool;
use crate::station::tools::web_fetch::WebFetchTool;
use crate::station::tools::web_search::WebSearchTool;

/// Routes tool calls from the LLM to the correct tool implementation.
///
/// Supports multiple name aliases for each tool so the LLM can use either
/// the canonical or shorthand name (e.g. `"read_file"` or `"filesystem.read"`).
pub struct ToolDispatcher {
    filesystem: FilesystemTool,
    web_fetch: WebFetchTool,
    web_search: WebSearchTool,
    document: DocumentTool,
}

/// The outcome of executing a single tool call.
#[derive(Debug)]
pub struct ToolExecResult {
    /// JSON-serialisable output from the tool.
    pub output: Value,
    /// Whether the tool invocation succeeded.
    pub success: bool,
    /// Wall-clock execution time in milliseconds.
    pub duration_ms: u64,
}

impl ToolDispatcher {
    /// Create a new dispatcher with the given workspace root for filesystem ops.
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            filesystem: FilesystemTool::new(workspace_root),
            web_fetch: WebFetchTool::new(),
            web_search: WebSearchTool::new(),
            document: DocumentTool::new(),
        }
    }

    /// Create a dispatcher using the current working directory as workspace root.
    pub fn with_cwd() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new(cwd)
    }

    /// Execute a tool call by name with JSON parameters.
    ///
    /// Returns a [`ToolExecResult`] containing the output, success flag, and
    /// execution duration. Errors are captured as JSON strings rather than
    /// propagated, so the caller can always send a result back to the LLM.
    pub async fn execute(
        &self,
        tool_name: &str,
        params: Value,
    ) -> ToolExecResult {
        let start = Instant::now();
        let result = self.dispatch(tool_name, &params).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(output) => ToolExecResult {
                output,
                success: true,
                duration_ms,
            },
            Err(err) => ToolExecResult {
                output: json!({ "error": err }),
                success: false,
                duration_ms,
            },
        }
    }

    /// Internal dispatch table — routes the tool name to the concrete implementation.
    async fn dispatch(
        &self,
        tool_name: &str,
        params: &Value,
    ) -> Result<Value, String> {
        match tool_name {
            // ── Filesystem ────────────────────────────────────────────
            "read_file" | "filesystem.read" => {
                let path = param_str(params, "path")?;
                let content = self.filesystem.read(&path)?;
                Ok(json!({ "content": content }))
            }

            "write_file" | "filesystem.write" => {
                let path = param_str(params, "path")?;
                let content = param_str(params, "content")?;
                self.filesystem.write(&path, &content)?;
                Ok(json!({ "path": path, "written": true }))
            }

            "glob" | "filesystem.glob" => {
                let pattern = param_str(params, "pattern")?;
                let matches = self.filesystem.glob(&pattern)?;
                Ok(json!({ "matches": matches }))
            }

            "list_dir" | "filesystem.list_dir" => {
                let path = param_str(params, "path")?;
                let entries = self.filesystem.list_dir(&path)?;
                Ok(json!({ "entries": entries }))
            }

            // ── Web search ────────────────────────────────────────────
            "web_search" | "search" => {
                let query = param_str(params, "query")?;
                let limit = params
                    .get("limit")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5) as usize;
                let results = self.web_search.search(&query, limit).await?;
                Ok(serde_json::to_value(&results).unwrap_or(json!([])))
            }

            // ── Web fetch ─────────────────────────────────────────────
            "fetch_url" | "web_fetch" => {
                let url = param_str(params, "url")?;
                let result = self.web_fetch.fetch_url(&url).await?;
                Ok(serde_json::to_value(&result).unwrap_or(json!({})))
            }

            // ── Document ──────────────────────────────────────────────
            "create_markdown" => {
                let content = param_str(params, "content")?;
                let path = param_str(params, "path")?;
                let result_path = self.document.create_markdown(&content, &path)?;
                Ok(json!({ "path": result_path }))
            }

            // ── Unknown ───────────────────────────────────────────────
            _ => Err(format!("unknown tool: '{tool_name}'")),
        }
    }

    /// Return the list of tool names this dispatcher can handle.
    pub fn supported_tools(&self) -> Vec<&'static str> {
        vec![
            "read_file",
            "write_file",
            "glob",
            "list_dir",
            "web_search",
            "fetch_url",
            "create_markdown",
        ]
    }

    /// Return OpenAI function-calling tool definitions for all supported tools.
    ///
    /// These can be passed to `ChatRequest.tools` so the LLM knows which
    /// tools are available.
    pub fn tool_definitions() -> Vec<serde_json::Value> {
        vec![
            json!({
                "type": "function",
                "function": {
                    "name": "read_file",
                    "description": "Read the contents of a file at the given path",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "File path to read" }
                        },
                        "required": ["path"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "write_file",
                    "description": "Write content to a file, creating it if it does not exist",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "File path to write" },
                            "content": { "type": "string", "description": "Content to write" }
                        },
                        "required": ["path", "content"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "glob",
                    "description": "Search for files matching a glob pattern",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "pattern": { "type": "string", "description": "Glob pattern (e.g. '**/*.rs')" }
                        },
                        "required": ["pattern"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "list_dir",
                    "description": "List files and directories in a directory",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "Directory path" }
                        },
                        "required": ["path"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "web_search",
                    "description": "Search the web for information",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "query": { "type": "string", "description": "Search query" },
                            "limit": { "type": "integer", "description": "Max number of results (default 5)" }
                        },
                        "required": ["query"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "fetch_url",
                    "description": "Fetch the contents of a web page",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "url": { "type": "string", "description": "URL to fetch" }
                        },
                        "required": ["url"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "create_markdown",
                    "description": "Create a Markdown file with the given content",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "content": { "type": "string", "description": "Markdown content" },
                            "path": { "type": "string", "description": "Output file path (must end in .md)" }
                        },
                        "required": ["content", "path"]
                    }
                }
            }),
        ]
    }
}

/// Extract a required string parameter from a JSON value.
fn param_str(params: &Value, key: &str) -> Result<String, String> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("missing required parameter: '{key}'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dispatcher() -> ToolDispatcher {
        let dir = std::env::temp_dir().join("d1d-tool-dispatch-test");
        std::fs::create_dir_all(&dir).ok();
        ToolDispatcher::new(dir)
    }

    #[tokio::test]
    async fn test_read_file_not_found() {
        let dispatcher = make_dispatcher();
        let result = dispatcher
            .execute("read_file", json!({ "path": "/nonexistent/file.txt" }))
            .await;
        assert!(!result.success);
        assert!(result.output["error"].as_str().unwrap().len() > 0);
    }

    #[tokio::test]
    async fn test_write_and_read_roundtrip() {
        let dir = std::env::temp_dir().join("d1d-tool-dispatch-rw");
        std::fs::create_dir_all(&dir).ok();
        let dispatcher = ToolDispatcher::new(dir.clone());

        let file_path = dir.join("test_dispatch.txt");
        let path_str = file_path.to_str().unwrap();

        // Write
        let write_result = dispatcher
            .execute(
                "write_file",
                json!({ "path": path_str, "content": "hello dispatch" }),
            )
            .await;
        assert!(write_result.success);

        // Read
        let read_result = dispatcher
            .execute("read_file", json!({ "path": path_str }))
            .await;
        assert!(read_result.success);
        assert_eq!(read_result.output["content"], "hello dispatch");

        // Cleanup
        std::fs::remove_file(&file_path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[tokio::test]
    async fn test_glob_finds_files() {
        let dir = std::env::temp_dir().join("d1d-tool-dispatch-glob");
        std::fs::create_dir_all(&dir).ok();
        std::fs::write(dir.join("a.txt"), "a").ok();
        std::fs::write(dir.join("b.txt"), "b").ok();

        let dispatcher = ToolDispatcher::new(dir.clone());
        let result = dispatcher
            .execute("glob", json!({ "pattern": "*.txt" }))
            .await;
        assert!(result.success);
        let matches = result.output["matches"].as_array().unwrap();
        assert!(matches.len() >= 2);

        // Cleanup
        std::fs::remove_file(dir.join("a.txt")).ok();
        std::fs::remove_file(dir.join("b.txt")).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[tokio::test]
    async fn test_list_dir() {
        let dir = std::env::temp_dir().join("d1d-tool-dispatch-listdir");
        std::fs::create_dir_all(&dir).ok();
        std::fs::write(dir.join("file1.txt"), "").ok();

        let dispatcher = ToolDispatcher::new(dir.clone());
        let result = dispatcher
            .execute("list_dir", json!({ "path": dir.to_str().unwrap() }))
            .await;
        assert!(result.success);
        let entries = result.output["entries"].as_array().unwrap();
        assert!(entries.iter().any(|e| e.as_str() == Some("file1.txt")));

        // Cleanup
        std::fs::remove_file(dir.join("file1.txt")).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[tokio::test]
    async fn test_unknown_tool_returns_error() {
        let dispatcher = make_dispatcher();
        let result = dispatcher
            .execute("nonexistent_tool", json!({}))
            .await;
        assert!(!result.success);
        assert!(
            result.output["error"]
                .as_str()
                .unwrap()
                .contains("unknown tool")
        );
    }

    #[tokio::test]
    async fn test_missing_required_param() {
        let dispatcher = make_dispatcher();
        let result = dispatcher.execute("read_file", json!({})).await;
        assert!(!result.success);
        assert!(
            result.output["error"]
                .as_str()
                .unwrap()
                .contains("missing required parameter")
        );
    }

    #[tokio::test]
    async fn test_alias_filesystem_read() {
        let dispatcher = make_dispatcher();
        // filesystem.read should work the same as read_file
        let result = dispatcher
            .execute("filesystem.read", json!({ "path": "/nonexistent" }))
            .await;
        assert!(!result.success); // file doesn't exist, but tool was found
        assert!(result.output["error"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_alias_filesystem_write() {
        let dir = std::env::temp_dir().join("d1d-tool-dispatch-alias-w");
        std::fs::create_dir_all(&dir).ok();
        let dispatcher = ToolDispatcher::new(dir.clone());

        let file_path = dir.join("alias_test.txt");
        let path_str = file_path.to_str().unwrap();

        let result = dispatcher
            .execute(
                "filesystem.write",
                json!({ "path": path_str, "content": "via alias" }),
            )
            .await;
        assert!(result.success);

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "via alias");

        // Cleanup
        std::fs::remove_file(&file_path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[tokio::test]
    async fn test_create_markdown() {
        let dir = std::env::temp_dir().join("d1d-tool-dispatch-md");
        std::fs::create_dir_all(&dir).ok();
        let dispatcher = ToolDispatcher::new(dir.clone());

        let file_path = dir.join("test.md");
        let path_str = file_path.to_str().unwrap();

        let result = dispatcher
            .execute(
                "create_markdown",
                json!({ "content": "# Hello", "path": path_str }),
            )
            .await;
        assert!(result.success);
        assert_eq!(result.output["path"].as_str().unwrap(), path_str);

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "# Hello");

        // Cleanup
        std::fs::remove_file(&file_path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_supported_tools_list() {
        let dispatcher = make_dispatcher();
        let tools = dispatcher.supported_tools();
        assert!(tools.contains(&"read_file"));
        assert!(tools.contains(&"write_file"));
        assert!(tools.contains(&"glob"));
        assert!(tools.contains(&"list_dir"));
        assert!(tools.contains(&"web_search"));
        assert!(tools.contains(&"fetch_url"));
        assert!(tools.contains(&"create_markdown"));
    }

    #[test]
    fn test_tool_definitions_valid_json() {
        let defs = ToolDispatcher::tool_definitions();
        assert_eq!(defs.len(), 7);
        for def in &defs {
            assert_eq!(def["type"], "function");
            assert!(def["function"]["name"].is_string());
            assert!(def["function"]["parameters"].is_object());
        }
    }

    #[tokio::test]
    async fn test_duration_tracked() {
        let dispatcher = make_dispatcher();
        let result = dispatcher
            .execute("read_file", json!({ "path": "/nonexistent" }))
            .await;
        // Duration should be non-negative (it could be 0 on fast machines)
        assert!(result.duration_ms < 10_000);
    }

    #[test]
    fn test_param_str_extracts_value() {
        let params = json!({ "path": "/tmp/test.txt" });
        assert_eq!(param_str(&params, "path").unwrap(), "/tmp/test.txt");
    }

    #[test]
    fn test_param_str_missing_key() {
        let params = json!({});
        let err = param_str(&params, "path").unwrap_err();
        assert!(err.contains("missing required parameter"));
    }

    #[test]
    fn test_param_str_non_string_value() {
        let params = json!({ "path": 42 });
        let err = param_str(&params, "path").unwrap_err();
        assert!(err.contains("missing required parameter"));
    }
}
