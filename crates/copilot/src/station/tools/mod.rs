// MCP Tool Servers — stub implementations defining tool interfaces.
//
// Each tool struct implements the `McpTool` trait and provides stub methods
// that return mock data. Real implementations will connect to MCP tool server
// processes in a future release.

pub mod browser;
pub mod clipboard;
pub mod data;
pub mod document;
pub mod web_fetch;
pub mod web_search;

/// Common trait for all MCP tool server interfaces.
pub trait McpTool: Send + Sync {
    /// The canonical tool name used in agent permission lists.
    fn name(&self) -> &str;

    /// Risk level: "low", "medium", or "high".
    /// High-risk tools require explicit user approval before execution.
    fn risk_level(&self) -> &str;

    /// Human-readable description of the tool's purpose.
    fn description(&self) -> &str;
}
