//! MCP Filesystem Server — wraps [`FilesystemOps`] as MCP tools.
//!
//! Provides JSON-Schema tool definitions and a dispatch function
//! for the 8 filesystem tools: read_file, write_file, edit_file,
//! glob, grep, list_directory, diff, backup.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::filesystem::FilesystemOps;

// ----------------------------------------------------------------
// ToolDefinition (local, matches MCP pattern from D1D-11)
// ----------------------------------------------------------------

/// An MCP tool definition with JSON Schema for inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name (e.g., "read_file").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// JSON Schema describing the input parameters.
    pub input_schema: Value,
}

// ----------------------------------------------------------------
// FilesystemServer
// ----------------------------------------------------------------

/// MCP server that dispatches filesystem tool calls to [`FilesystemOps`].
pub struct FilesystemServer {
    ops: FilesystemOps,
}

impl FilesystemServer {
    /// Create a new `FilesystemServer` wrapping the given `FilesystemOps`.
    pub fn new(ops: FilesystemOps) -> Self {
        Self { ops }
    }

    /// Return the JSON Schema tool definitions for all 8 filesystem tools.
    pub fn tool_definitions() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "read_file".to_string(),
                description: "Read a file with line numbers. Supports optional offset and limit for partial reads. Maximum file size: 1 MB.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file (absolute or relative to workspace root)"
                        },
                        "offset": {
                            "type": "integer",
                            "description": "0-based line offset to start reading from",
                            "minimum": 0
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of lines to return",
                            "minimum": 1
                        }
                    },
                    "required": ["path"],
                    "additionalProperties": false
                }),
            },
            ToolDefinition {
                name: "write_file".to_string(),
                description: "Write content to a file. Creates parent directories as needed. If the file already exists, a backup is created automatically before overwriting.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file (absolute or relative to workspace root)"
                        },
                        "content": {
                            "type": "string",
                            "description": "Content to write to the file"
                        }
                    },
                    "required": ["path", "content"],
                    "additionalProperties": false
                }),
            },
            ToolDefinition {
                name: "edit_file".to_string(),
                description: "Replace an exact string in a file. The old_string must appear exactly once. A backup is created before editing.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file (absolute or relative to workspace root)"
                        },
                        "old_string": {
                            "type": "string",
                            "description": "Exact string to find and replace (must be unique in the file)"
                        },
                        "new_string": {
                            "type": "string",
                            "description": "Replacement string"
                        }
                    },
                    "required": ["path", "old_string", "new_string"],
                    "additionalProperties": false
                }),
            },
            ToolDefinition {
                name: "glob".to_string(),
                description: "Find files matching a glob pattern under the workspace. Supports ** for recursive matching.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "Glob pattern (e.g., '**/*.rs', 'src/*.txt')"
                        },
                        "path": {
                            "type": "string",
                            "description": "Base directory for the search (default: workspace root)"
                        }
                    },
                    "required": ["pattern"],
                    "additionalProperties": false
                }),
            },
            ToolDefinition {
                name: "grep".to_string(),
                description: "Search for a regex pattern in files. Returns matching lines with optional context.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "Regular expression pattern to search for"
                        },
                        "path": {
                            "type": "string",
                            "description": "File or directory to search in (default: workspace root)"
                        },
                        "context_lines": {
                            "type": "integer",
                            "description": "Number of context lines to show before and after each match",
                            "minimum": 0,
                            "default": 0
                        }
                    },
                    "required": ["pattern"],
                    "additionalProperties": false
                }),
            },
            ToolDefinition {
                name: "list_directory".to_string(),
                description: "List directory contents in a tree-style format with depth control.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Directory to list (default: workspace root)"
                        },
                        "depth": {
                            "type": "integer",
                            "description": "Maximum depth of the tree listing (default: 2)",
                            "minimum": 1,
                            "default": 2
                        }
                    },
                    "additionalProperties": false
                }),
            },
            ToolDefinition {
                name: "diff".to_string(),
                description: "Show a line-by-line diff between two files.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path_a": {
                            "type": "string",
                            "description": "Path to the first file"
                        },
                        "path_b": {
                            "type": "string",
                            "description": "Path to the second file"
                        }
                    },
                    "required": ["path_a", "path_b"],
                    "additionalProperties": false
                }),
            },
            ToolDefinition {
                name: "backup".to_string(),
                description: "Create a timestamped backup copy of a file in the backup directory.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file to backup"
                        }
                    },
                    "required": ["path"],
                    "additionalProperties": false
                }),
            },
        ]
    }

    /// Dispatch an MCP tool call to the appropriate filesystem operation.
    pub async fn handle_tool_call(&self, tool: &str, params: Value) -> Result<Value> {
        match tool {
            "read_file" => {
                let path = params["path"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing required parameter: path"))?;
                let offset = params["offset"].as_u64().map(|v| v as usize);
                let limit = params["limit"].as_u64().map(|v| v as usize);

                let result = self.ops.read_file(path, offset, limit)?;
                Ok(json!({ "content": result }))
            }

            "write_file" => {
                let path = params["path"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing required parameter: path"))?;
                let content = params["content"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing required parameter: content"))?;

                let result = self.ops.write_file(path, content)?;
                Ok(json!({ "message": result }))
            }

            "edit_file" => {
                let path = params["path"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing required parameter: path"))?;
                let old_string = params["old_string"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing required parameter: old_string"))?;
                let new_string = params["new_string"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing required parameter: new_string"))?;

                let result = self.ops.edit_file(path, old_string, new_string)?;
                Ok(json!({ "message": result }))
            }

            "glob" => {
                let pattern = params["pattern"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing required parameter: pattern"))?;
                let path = params["path"].as_str();

                let files = self.ops.glob_files(pattern, path)?;
                Ok(json!({ "files": files }))
            }

            "grep" => {
                let pattern = params["pattern"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing required parameter: pattern"))?;
                let path = params["path"].as_str();
                let context_lines = params["context_lines"].as_u64().map(|v| v as usize);

                let result = self.ops.grep(pattern, path, context_lines)?;
                Ok(json!({ "matches": result }))
            }

            "list_directory" => {
                let path = params["path"].as_str();
                let depth = params["depth"].as_u64().map(|v| v as usize);

                let result = self.ops.list_directory(path, depth)?;
                Ok(json!({ "tree": result }))
            }

            "diff" => {
                let path_a = params["path_a"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing required parameter: path_a"))?;
                let path_b = params["path_b"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing required parameter: path_b"))?;

                let result = self.ops.diff(path_a, path_b)?;
                Ok(json!({ "diff": result }))
            }

            "backup" => {
                let path = params["path"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing required parameter: path"))?;

                let result = self.ops.backup(path)?;
                Ok(json!({ "message": result }))
            }

            _ => bail!("unknown filesystem tool: {}", tool),
        }
    }
}

// ================================================================
// Tests
// ================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_server() -> (tempfile::TempDir, FilesystemServer) {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        let backup_dir = tmp.path().join("backups");
        let ops = FilesystemOps::new(tmp.path().to_path_buf(), backup_dir);
        let server = FilesystemServer::new(ops);
        (tmp, server)
    }

    #[test]
    fn test_tool_definitions_count() {
        let defs = FilesystemServer::tool_definitions();
        assert_eq!(defs.len(), 8);

        let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"read_file"));
        assert!(names.contains(&"write_file"));
        assert!(names.contains(&"edit_file"));
        assert!(names.contains(&"glob"));
        assert!(names.contains(&"grep"));
        assert!(names.contains(&"list_directory"));
        assert!(names.contains(&"diff"));
        assert!(names.contains(&"backup"));
    }

    #[test]
    fn test_tool_definitions_have_schemas() {
        for def in FilesystemServer::tool_definitions() {
            assert!(
                def.input_schema.is_object(),
                "tool {} has no input_schema object",
                def.name
            );
            assert_eq!(
                def.input_schema["type"], "object",
                "tool {} schema type should be 'object'",
                def.name
            );
        }
    }

    #[tokio::test]
    async fn test_dispatch_read_file() {
        let (tmp, server) = setup_test_server();
        let file = tmp.path().join("test.txt");
        fs::write(&file, "hello\nworld\n").unwrap();

        let result = server
            .handle_tool_call("read_file", json!({ "path": file.to_str().unwrap() }))
            .await
            .unwrap();

        let content = result["content"].as_str().unwrap();
        assert!(content.contains("hello"));
        assert!(content.contains("world"));
    }

    #[tokio::test]
    async fn test_dispatch_write_file() {
        let (tmp, server) = setup_test_server();
        let file = tmp.path().join("out.txt");

        let result = server
            .handle_tool_call(
                "write_file",
                json!({
                    "path": file.to_str().unwrap(),
                    "content": "test content"
                }),
            )
            .await
            .unwrap();

        assert!(result["message"].as_str().unwrap().contains("wrote"));
        assert_eq!(fs::read_to_string(&file).unwrap(), "test content");
    }

    #[tokio::test]
    async fn test_dispatch_edit_file() {
        let (tmp, server) = setup_test_server();
        let file = tmp.path().join("edit.txt");
        fs::write(&file, "foo bar baz").unwrap();

        let result = server
            .handle_tool_call(
                "edit_file",
                json!({
                    "path": file.to_str().unwrap(),
                    "old_string": "bar",
                    "new_string": "qux"
                }),
            )
            .await
            .unwrap();

        assert!(result["message"]
            .as_str()
            .unwrap()
            .contains("replaced 1 occurrence"));
        assert_eq!(fs::read_to_string(&file).unwrap(), "foo qux baz");
    }

    #[tokio::test]
    async fn test_dispatch_glob() {
        let (tmp, server) = setup_test_server();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::write(tmp.path().join("src/a.rs"), "").unwrap();
        fs::write(tmp.path().join("src/b.rs"), "").unwrap();
        fs::write(tmp.path().join("readme.md"), "").unwrap();

        let result = server
            .handle_tool_call("glob", json!({ "pattern": "**/*.rs" }))
            .await
            .unwrap();

        let files = result["files"].as_array().unwrap();
        assert_eq!(files.len(), 2);
    }

    #[tokio::test]
    async fn test_dispatch_grep() {
        let (tmp, server) = setup_test_server();
        fs::write(tmp.path().join("search.txt"), "alpha\nbeta\ngamma\n").unwrap();

        let result = server
            .handle_tool_call(
                "grep",
                json!({
                    "pattern": "beta",
                    "path": tmp.path().join("search.txt").to_str().unwrap()
                }),
            )
            .await
            .unwrap();

        let matches = result["matches"].as_str().unwrap();
        assert!(matches.contains("beta"));
    }

    #[tokio::test]
    async fn test_dispatch_list_directory() {
        let (tmp, server) = setup_test_server();
        fs::create_dir_all(tmp.path().join("sub")).unwrap();
        fs::write(tmp.path().join("sub/file.txt"), "").unwrap();

        let result = server
            .handle_tool_call("list_directory", json!({ "depth": 2 }))
            .await
            .unwrap();

        let tree = result["tree"].as_str().unwrap();
        assert!(tree.contains("sub/"));
        assert!(tree.contains("file.txt"));
    }

    #[tokio::test]
    async fn test_dispatch_diff() {
        let (tmp, server) = setup_test_server();
        let a = tmp.path().join("a.txt");
        let b = tmp.path().join("b.txt");
        fs::write(&a, "line1\nline2\n").unwrap();
        fs::write(&b, "line1\nline3\n").unwrap();

        let result = server
            .handle_tool_call(
                "diff",
                json!({
                    "path_a": a.to_str().unwrap(),
                    "path_b": b.to_str().unwrap()
                }),
            )
            .await
            .unwrap();

        let diff = result["diff"].as_str().unwrap();
        assert!(diff.contains("---"));
        assert!(diff.contains("+++"));
    }

    #[tokio::test]
    async fn test_dispatch_backup() {
        let (tmp, server) = setup_test_server();
        let file = tmp.path().join("backup_me.txt");
        fs::write(&file, "important data").unwrap();

        let result = server
            .handle_tool_call("backup", json!({ "path": file.to_str().unwrap() }))
            .await
            .unwrap();

        assert!(result["message"].as_str().unwrap().contains("backed up to"));
    }

    #[tokio::test]
    async fn test_dispatch_unknown_tool() {
        let (_tmp, server) = setup_test_server();
        let result = server.handle_tool_call("nonexistent", json!({})).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("unknown filesystem tool"));
    }
}
