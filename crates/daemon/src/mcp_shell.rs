//! MCP-style shell tool server wrapping the local [`Executor`].
//!
//! Exposes three tools — `execute`, `execute_script`, and `dry_run` — with
//! JSON Schema definitions and a unified `handle_tool_call` dispatcher.
//! This module does **not** implement MCP transport; it provides the tool
//! logic that a future MCP transport layer will wrap.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::executor::Executor;

/// Metadata describing a tool that the shell server exposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Shell tool server backed by an [`Executor`].
pub struct ShellServer {
    executor: Executor,
}

impl ShellServer {
    /// Create a new `ShellServer` with a default `Executor`.
    pub fn new() -> Self {
        Self {
            executor: Executor::default(),
        }
    }

    /// Return the JSON Schema definitions for every tool this server exposes.
    pub fn tool_definitions() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "execute".to_string(),
                description: "Execute a shell command and return its output.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The shell command to execute."
                        },
                        "timeout_ms": {
                            "type": "integer",
                            "description": "Optional timeout in milliseconds."
                        },
                        "cwd": {
                            "type": "string",
                            "description": "Optional working directory for the command."
                        }
                    },
                    "required": ["command"]
                }),
            },
            ToolDefinition {
                name: "execute_script".to_string(),
                description: "Write a script to a temporary file and execute it.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "script": {
                            "type": "string",
                            "description": "The script content to execute."
                        },
                        "interpreter": {
                            "type": "string",
                            "description": "Interpreter to use (default: bash)."
                        },
                        "timeout_ms": {
                            "type": "integer",
                            "description": "Optional timeout in milliseconds."
                        }
                    },
                    "required": ["script"]
                }),
            },
            ToolDefinition {
                name: "dry_run".to_string(),
                description:
                    "Analyse a command without executing it. Returns the parsed binary, arguments, a description, and risk level."
                        .to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The command to analyse."
                        }
                    },
                    "required": ["command"]
                }),
            },
        ]
    }

    /// Dispatch a tool call by name, deserialising parameters from `params`.
    pub async fn handle_tool_call(
        &self,
        tool_name: &str,
        params: Value,
    ) -> anyhow::Result<Value> {
        match tool_name {
            "execute" => {
                let command = params["command"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing required parameter: command"))?;
                let timeout_ms = params["timeout_ms"].as_u64();
                let cwd = params["cwd"].as_str();

                let result = self.executor.execute(command, timeout_ms, cwd).await?;
                Ok(serde_json::to_value(result)?)
            }
            "execute_script" => {
                let script = params["script"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing required parameter: script"))?;
                let interpreter = params["interpreter"].as_str();
                let timeout_ms = params["timeout_ms"].as_u64();

                let result = self
                    .executor
                    .execute_script(script, interpreter, timeout_ms)
                    .await?;
                Ok(serde_json::to_value(result)?)
            }
            "dry_run" => {
                let command = params["command"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing required parameter: command"))?;

                let result = self.executor.dry_run(command);
                Ok(serde_json::to_value(result)?)
            }
            _ => Err(anyhow::anyhow!("unknown tool: {tool_name}")),
        }
    }
}

impl Default for ShellServer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tool_definitions_count() {
        let defs = ShellServer::tool_definitions();
        assert_eq!(defs.len(), 3);
        let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"execute"));
        assert!(names.contains(&"execute_script"));
        assert!(names.contains(&"dry_run"));
    }

    #[test]
    fn tool_definitions_have_schemas() {
        for def in ShellServer::tool_definitions() {
            assert_eq!(def.input_schema["type"], "object");
            assert!(
                def.input_schema["properties"].is_object(),
                "{} should have properties",
                def.name
            );
            assert!(
                def.input_schema["required"].is_array(),
                "{} should have required array",
                def.name
            );
        }
    }

    #[tokio::test]
    async fn handle_execute() {
        let server = ShellServer::new();
        let result = server
            .handle_tool_call("execute", json!({ "command": "echo mcp_test" }))
            .await
            .expect("handle_tool_call should succeed");

        assert_eq!(result["success"], true);
        assert_eq!(result["exit_code"], 0);
        assert!(result["stdout"].as_str().unwrap().contains("mcp_test"));
    }

    #[tokio::test]
    async fn handle_execute_script() {
        let server = ShellServer::new();
        let result = server
            .handle_tool_call(
                "execute_script",
                json!({ "script": "echo script_via_mcp" }),
            )
            .await
            .expect("handle_tool_call should succeed");

        assert_eq!(result["success"], true);
        assert!(result["stdout"]
            .as_str()
            .unwrap()
            .contains("script_via_mcp"));
    }

    #[tokio::test]
    async fn handle_dry_run() {
        let server = ShellServer::new();
        let result = server
            .handle_tool_call("dry_run", json!({ "command": "rm -rf /" }))
            .await
            .expect("handle_tool_call should succeed");

        assert_eq!(result["binary"], "rm");
        assert_eq!(result["risk_level"], "high");
    }

    #[tokio::test]
    async fn handle_unknown_tool() {
        let server = ShellServer::new();
        let result = server
            .handle_tool_call("nonexistent", json!({}))
            .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("unknown tool"));
    }

    #[tokio::test]
    async fn handle_missing_required_param() {
        let server = ShellServer::new();
        let result = server
            .handle_tool_call("execute", json!({}))
            .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("command"));
    }
}
