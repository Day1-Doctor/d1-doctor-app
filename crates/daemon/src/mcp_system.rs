//! MCP system server — wraps [`SystemOps`] into a tool-dispatch interface.
//!
//! Exposes 10 tools (package_search, package_install, package_remove,
//! service_status, service_control, config_read, config_set, env_get,
//! env_set, network_check) with JSON Schema definitions and a single
//! `handle_tool_call` dispatcher.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::system_ops::SystemOps;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Definition of a single MCP tool, including its JSON Schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// The MCP system server that holds a [`SystemOps`] instance and dispatches
/// tool calls.
pub struct SystemServer {
    ops: SystemOps,
}

impl SystemServer {
    pub fn new() -> Self {
        SystemServer {
            ops: SystemOps::new(),
        }
    }

    /// Return the list of all tool definitions with JSON Schema descriptions.
    pub fn tool_definitions(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "package_search".into(),
                description: "Search for packages by name using the system package manager (brew on macOS, apt on Linux)".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Package name or search term"
                        }
                    },
                    "required": ["name"]
                }),
            },
            ToolDefinition {
                name: "package_install".into(),
                description: "Install a package by name using the system package manager".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Package name to install"
                        }
                    },
                    "required": ["name"]
                }),
            },
            ToolDefinition {
                name: "package_remove".into(),
                description: "Remove/uninstall a package by name".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Package name to remove"
                        }
                    },
                    "required": ["name"]
                }),
            },
            ToolDefinition {
                name: "service_status".into(),
                description: "Query the status of a system service (launchctl on macOS, systemctl on Linux)".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Service name or label"
                        }
                    },
                    "required": ["name"]
                }),
            },
            ToolDefinition {
                name: "service_control".into(),
                description: "Control a system service: start, stop, or restart".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Service name or label"
                        },
                        "action": {
                            "type": "string",
                            "enum": ["start", "stop", "restart"],
                            "description": "Action to perform"
                        }
                    },
                    "required": ["name", "action"]
                }),
            },
            ToolDefinition {
                name: "config_read".into(),
                description: "Read and parse a configuration file (TOML or JSON) into a JSON value".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to the config file"
                        },
                        "format": {
                            "type": "string",
                            "enum": ["toml", "json"],
                            "description": "File format (toml or json)"
                        }
                    },
                    "required": ["path", "format"]
                }),
            },
            ToolDefinition {
                name: "config_set".into(),
                description: "Set a value in a configuration file using dotted key notation (e.g. 'section.key')".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to the config file"
                        },
                        "key": {
                            "type": "string",
                            "description": "Dotted key path (e.g. 'server.port')"
                        },
                        "value": {
                            "type": "string",
                            "description": "Value to set (parsed as int/float/bool/string automatically)"
                        }
                    },
                    "required": ["path", "key", "value"]
                }),
            },
            ToolDefinition {
                name: "env_get".into(),
                description: "Get an environment variable from the daemon process".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "key": {
                            "type": "string",
                            "description": "Environment variable name"
                        }
                    },
                    "required": ["key"]
                }),
            },
            ToolDefinition {
                name: "env_set".into(),
                description: "Set an environment variable in the daemon process (does not persist across process restarts)".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "key": {
                            "type": "string",
                            "description": "Environment variable name"
                        },
                        "value": {
                            "type": "string",
                            "description": "Value to set"
                        }
                    },
                    "required": ["key", "value"]
                }),
            },
            ToolDefinition {
                name: "network_check".into(),
                description: "Run a network diagnostic check: ping, dns lookup, port connectivity, or internet connectivity test".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "check_type": {
                            "type": "string",
                            "enum": ["ping", "dns", "port", "connectivity"],
                            "description": "Type of network check"
                        },
                        "target": {
                            "type": "string",
                            "description": "Target hostname, IP, or host:port (ignored for connectivity check)"
                        }
                    },
                    "required": ["check_type", "target"]
                }),
            },
        ]
    }

    /// Dispatch a tool call by name with the given JSON parameters.
    ///
    /// Returns a JSON value with the tool result on success.
    pub async fn handle_tool_call(&self, tool: &str, params: &Value) -> Result<Value> {
        match tool {
            "package_search" => {
                let name = param_str(params, "name")?;
                let results = self.ops.package_search(&name).await?;
                Ok(serde_json::to_value(results)?)
            }
            "package_install" => {
                let name = param_str(params, "name")?;
                let output = self.ops.package_install(&name).await?;
                Ok(json!({ "output": output }))
            }
            "package_remove" => {
                let name = param_str(params, "name")?;
                let output = self.ops.package_remove(&name).await?;
                Ok(json!({ "output": output }))
            }
            "service_status" => {
                let name = param_str(params, "name")?;
                let info = self.ops.service_status(&name).await?;
                Ok(serde_json::to_value(info)?)
            }
            "service_control" => {
                let name = param_str(params, "name")?;
                let action = param_str(params, "action")?;
                let output = self.ops.service_control(&name, &action).await?;
                Ok(json!({ "output": output }))
            }
            "config_read" => {
                let path = param_str(params, "path")?;
                let format = param_str(params, "format")?;
                let val = self.ops.config_read(&path, &format).await?;
                Ok(val)
            }
            "config_set" => {
                let path = param_str(params, "path")?;
                let key = param_str(params, "key")?;
                let value = param_str(params, "value")?;
                self.ops.config_set(&path, &key, &value).await?;
                Ok(json!({ "success": true }))
            }
            "env_get" => {
                let key = param_str(params, "key")?;
                let val = self.ops.env_get(&key)?;
                Ok(json!({ "value": val }))
            }
            "env_set" => {
                let key = param_str(params, "key")?;
                let value = param_str(params, "value")?;
                self.ops.env_set(&key, &value)?;
                Ok(json!({ "success": true }))
            }
            "network_check" => {
                let check_type = param_str(params, "check_type")?;
                let target = param_str(params, "target")?;
                let result = self.ops.network_check(&check_type, &target).await?;
                Ok(serde_json::to_value(result)?)
            }
            unknown => anyhow::bail!("unknown tool: {unknown}"),
        }
    }
}

/// Extract a string parameter from a JSON object, returning an error if
/// missing or not a string.
fn param_str(params: &Value, key: &str) -> Result<String> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .with_context(|| format!("missing or invalid parameter: {key}"))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definitions_count() {
        let server = SystemServer::new();
        let defs = server.tool_definitions();
        assert_eq!(defs.len(), 10, "should expose exactly 10 tools");
    }

    #[test]
    fn test_tool_definitions_have_required_fields() {
        let server = SystemServer::new();
        for def in server.tool_definitions() {
            assert!(!def.name.is_empty(), "tool name should not be empty");
            assert!(
                !def.description.is_empty(),
                "tool description should not be empty"
            );
            assert!(
                def.input_schema.is_object(),
                "input_schema should be a JSON object"
            );
            assert!(
                def.input_schema.get("properties").is_some(),
                "input_schema should have 'properties'"
            );
            assert!(
                def.input_schema.get("required").is_some(),
                "input_schema should have 'required'"
            );
        }
    }

    #[tokio::test]
    async fn test_dispatch_env_get() {
        let server = SystemServer::new();

        // Set a known env var first
        std::env::set_var("D1_MCP_TEST_VAR", "test_value_42");

        let result = server
            .handle_tool_call("env_get", &json!({ "key": "D1_MCP_TEST_VAR" }))
            .await
            .unwrap();

        assert_eq!(result["value"], "test_value_42");

        std::env::remove_var("D1_MCP_TEST_VAR");
    }

    #[tokio::test]
    async fn test_dispatch_env_set() {
        let server = SystemServer::new();

        let result = server
            .handle_tool_call(
                "env_set",
                &json!({ "key": "D1_MCP_TEST_SET_VAR", "value": "hello" }),
            )
            .await
            .unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(std::env::var("D1_MCP_TEST_SET_VAR").unwrap(), "hello");

        std::env::remove_var("D1_MCP_TEST_SET_VAR");
    }

    #[tokio::test]
    async fn test_dispatch_env_roundtrip() {
        let server = SystemServer::new();
        let key = "D1_MCP_ROUNDTRIP_TEST";

        server
            .handle_tool_call("env_set", &json!({ "key": key, "value": "round_trip" }))
            .await
            .unwrap();

        let result = server
            .handle_tool_call("env_get", &json!({ "key": key }))
            .await
            .unwrap();
        assert_eq!(result["value"], "round_trip");

        std::env::remove_var(key);
    }

    #[tokio::test]
    async fn test_dispatch_config_read_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.toml");
        tokio::fs::write(&path, "[section]\nkey = \"val\"\n")
            .await
            .unwrap();

        let server = SystemServer::new();
        let result = server
            .handle_tool_call(
                "config_read",
                &json!({ "path": path.to_str().unwrap(), "format": "toml" }),
            )
            .await
            .unwrap();

        assert_eq!(result["section"]["key"], "val");
    }

    #[tokio::test]
    async fn test_dispatch_config_set_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.toml");
        tokio::fs::write(&path, "[section]\nkey = \"val\"\n")
            .await
            .unwrap();

        let server = SystemServer::new();
        server
            .handle_tool_call(
                "config_set",
                &json!({
                    "path": path.to_str().unwrap(),
                    "key": "section.key",
                    "value": "new_val"
                }),
            )
            .await
            .unwrap();

        let result = server
            .handle_tool_call(
                "config_read",
                &json!({ "path": path.to_str().unwrap(), "format": "toml" }),
            )
            .await
            .unwrap();

        assert_eq!(result["section"]["key"], "new_val");
    }

    #[tokio::test]
    async fn test_dispatch_network_check_connectivity() {
        let server = SystemServer::new();
        let result = server
            .handle_tool_call(
                "network_check",
                &json!({ "check_type": "connectivity", "target": "" }),
            )
            .await
            .unwrap();

        // Should return a valid NetworkResult structure
        assert!(result.get("success").is_some());
        assert!(result.get("output").is_some());
    }

    #[tokio::test]
    async fn test_dispatch_unknown_tool() {
        let server = SystemServer::new();
        let result = server
            .handle_tool_call("nonexistent_tool", &json!({}))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_dispatch_missing_param() {
        let server = SystemServer::new();
        // env_get requires "key" parameter
        let result = server.handle_tool_call("env_get", &json!({})).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_param_str_extraction() {
        let params = json!({ "name": "nginx", "port": 80 });
        assert_eq!(param_str(&params, "name").unwrap(), "nginx");
        assert!(param_str(&params, "missing").is_err());
        // port is a number, not a string
        assert!(param_str(&params, "port").is_err());
    }
}
