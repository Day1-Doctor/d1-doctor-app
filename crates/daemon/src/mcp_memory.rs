//! MCP tool-server wrapper around [`MemoryStore`].
//!
//! Exposes every public method of `MemoryStore` as a JSON-schema-typed tool
//! that can be invoked via `handle_tool_call(name, params)`.

use std::sync::Arc;

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

use crate::memory_store::{MemoryScope, MemoryStore};

// ---------------------------------------------------------------------------
// ToolDefinition
// ---------------------------------------------------------------------------

/// Descriptor for a single MCP tool — name, human description, and JSON Schema
/// for the input parameters.
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

// ---------------------------------------------------------------------------
// MemoryServer
// ---------------------------------------------------------------------------

/// MCP server that wraps a [`MemoryStore`] and dispatches tool calls to it.
pub struct MemoryServer {
    store: Arc<MemoryStore>,
}

impl MemoryServer {
    /// Create a new `MemoryServer` wrapping the given store.
    pub fn new(store: Arc<MemoryStore>) -> Self {
        Self { store }
    }

    // -- Tool catalogue -----------------------------------------------------

    /// Return the full set of tool definitions exposed by this server.
    pub fn tool_definitions(&self) -> Vec<ToolDefinition> {
        vec![
            // -- recall -----------------------------------------------------
            ToolDefinition {
                name: "recall".into(),
                description: "Search memory across one or all scopes.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Free-text search query."
                        },
                        "scope": {
                            "type": "string",
                            "enum": ["profile", "session", "task", "agent", "all"],
                            "description": "Which memory scope to search."
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results.",
                            "default": 10
                        }
                    },
                    "required": ["query", "scope"]
                }),
            },
            // -- recall_profile ---------------------------------------------
            ToolDefinition {
                name: "recall_profile".into(),
                description: "Return all profile entries for a given category.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "category": {
                            "type": "string",
                            "description": "Profile category to retrieve (e.g. \"system\", \"user\")."
                        }
                    },
                    "required": ["category"]
                }),
            },
            // -- recall_similar_tasks ---------------------------------------
            ToolDefinition {
                name: "recall_similar_tasks".into(),
                description: "FTS5-ranked search over task memory.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Free-text search query."
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results.",
                            "default": 5
                        }
                    },
                    "required": ["query"]
                }),
            },
            // -- recall_error_patterns --------------------------------------
            ToolDefinition {
                name: "recall_error_patterns".into(),
                description: "FTS5 search scoped to error_patterns and fix_patterns columns.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Error text to search for."
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results.",
                            "default": 5
                        }
                    },
                    "required": ["query"]
                }),
            },
            // -- store_profile ----------------------------------------------
            ToolDefinition {
                name: "store_profile".into(),
                description: "Insert or replace a profile memory entry.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "category": {
                            "type": "string",
                            "description": "Category (e.g. \"system\", \"user\")."
                        },
                        "key": {
                            "type": "string",
                            "description": "Profile key."
                        },
                        "value": {
                            "type": "string",
                            "description": "Profile value."
                        },
                        "source": {
                            "type": "string",
                            "description": "Who set this value (e.g. \"agent\", \"user\")."
                        }
                    },
                    "required": ["category", "key", "value", "source"]
                }),
            },
            // -- store_session ----------------------------------------------
            ToolDefinition {
                name: "store_session".into(),
                description: "Append an event to session memory.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "Session identifier."
                        },
                        "step_number": {
                            "type": "integer",
                            "description": "Step number within the session."
                        },
                        "agent_name": {
                            "type": "string",
                            "description": "Name of the agent."
                        },
                        "event_type": {
                            "type": "string",
                            "description": "Event type (e.g. \"action\", \"observation\")."
                        },
                        "content": {
                            "type": "string",
                            "description": "Event content."
                        },
                        "metadata": {
                            "type": "string",
                            "description": "Optional JSON metadata.",
                            "nullable": true
                        }
                    },
                    "required": ["session_id", "step_number", "agent_name", "event_type", "content"]
                }),
            },
            // -- store_task_outcome -----------------------------------------
            ToolDefinition {
                name: "store_task_outcome".into(),
                description: "Record a completed task outcome with procedure and error details.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "task_description": {
                            "type": "string",
                            "description": "What the task was."
                        },
                        "category": {
                            "type": "string",
                            "description": "Task category."
                        },
                        "outcome": {
                            "type": "string",
                            "description": "Result (e.g. \"success\", \"failure\")."
                        },
                        "procedure_steps": {
                            "type": "string",
                            "description": "JSON array of steps taken."
                        },
                        "error_patterns": {
                            "type": "string",
                            "description": "JSON array of error patterns encountered."
                        },
                        "fix_patterns": {
                            "type": "string",
                            "description": "JSON array of fixes applied."
                        },
                        "duration_seconds": {
                            "type": "integer",
                            "description": "Duration in seconds."
                        },
                        "system_context": {
                            "type": "string",
                            "description": "JSON object with system info."
                        },
                        "session_id": {
                            "type": "string",
                            "description": "Session that produced this outcome."
                        }
                    },
                    "required": [
                        "task_description", "category", "outcome",
                        "procedure_steps", "error_patterns", "fix_patterns",
                        "duration_seconds", "system_context", "session_id"
                    ]
                }),
            },
            // -- store_agent_learning ---------------------------------------
            ToolDefinition {
                name: "store_agent_learning".into(),
                description: "Record an agent learning or insight.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "agent_name": {
                            "type": "string",
                            "description": "Name of the agent."
                        },
                        "memory_type": {
                            "type": "string",
                            "description": "Type of memory (e.g. \"preference\", \"skill\", \"fact\")."
                        },
                        "content": {
                            "type": "string",
                            "description": "The learning content."
                        },
                        "confidence": {
                            "type": "number",
                            "description": "Confidence score (0.0 – 1.0)."
                        }
                    },
                    "required": ["agent_name", "memory_type", "content", "confidence"]
                }),
            },
            // -- forget -----------------------------------------------------
            ToolDefinition {
                name: "forget".into(),
                description: "Delete a memory record by id and table.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Record id to delete."
                        },
                        "table": {
                            "type": "string",
                            "enum": ["profile_memory", "session_memory", "task_memory", "agent_memory"],
                            "description": "Table to delete from."
                        }
                    },
                    "required": ["id", "table"]
                }),
            },
        ]
    }

    // -- Dispatch -----------------------------------------------------------

    /// Route a tool call to the appropriate [`MemoryStore`] method.
    ///
    /// Returns the result as a JSON `Value` suitable for sending over MCP.
    pub fn handle_tool_call(&self, tool: &str, params: Value) -> Result<Value> {
        match tool {
            "recall" => self.handle_recall(params),
            "recall_profile" => self.handle_recall_profile(params),
            "recall_similar_tasks" => self.handle_recall_similar_tasks(params),
            "recall_error_patterns" => self.handle_recall_error_patterns(params),
            "store_profile" => self.handle_store_profile(params),
            "store_session" => self.handle_store_session(params),
            "store_task_outcome" => self.handle_store_task_outcome(params),
            "store_agent_learning" => self.handle_store_agent_learning(params),
            "forget" => self.handle_forget(params),
            _ => bail!("unknown tool: {tool}"),
        }
    }

    // -- Individual handlers ------------------------------------------------

    fn handle_recall(&self, params: Value) -> Result<Value> {
        let query = param_str(&params, "query")?;
        let scope = parse_scope(param_str(&params, "scope")?)?;
        let limit = param_i32_or(&params, "limit", 10);

        let entries = self.store.recall(query, scope, limit)?;
        let items: Vec<Value> = entries
            .into_iter()
            .map(|e| {
                json!({
                    "id": e.id,
                    "level": e.level,
                    "content": e.content,
                    "metadata": e.metadata,
                    "confidence": e.confidence,
                    "created_at": e.created_at,
                })
            })
            .collect();
        Ok(json!({ "entries": items }))
    }

    fn handle_recall_profile(&self, params: Value) -> Result<Value> {
        let category = param_str(&params, "category")?;
        let entries = self.store.recall_profile(category)?;
        let items: Vec<Value> = entries
            .into_iter()
            .map(|e| {
                json!({
                    "id": e.id,
                    "category": e.category,
                    "key": e.key,
                    "value": e.value,
                    "confidence": e.confidence,
                    "source": e.source,
                    "created_at": e.created_at,
                    "updated_at": e.updated_at,
                })
            })
            .collect();
        Ok(json!({ "entries": items }))
    }

    fn handle_recall_similar_tasks(&self, params: Value) -> Result<Value> {
        let query = param_str(&params, "query")?;
        let limit = param_i32_or(&params, "limit", 5);
        let entries = self.store.recall_similar_tasks(query, limit)?;
        Ok(json!({ "entries": task_entries_to_json(entries) }))
    }

    fn handle_recall_error_patterns(&self, params: Value) -> Result<Value> {
        let query = param_str(&params, "query")?;
        let limit = param_i32_or(&params, "limit", 5);
        let entries = self.store.recall_error_patterns(query, limit)?;
        Ok(json!({ "entries": task_entries_to_json(entries) }))
    }

    fn handle_store_profile(&self, params: Value) -> Result<Value> {
        let category = param_str(&params, "category")?;
        let key = param_str(&params, "key")?;
        let value = param_str(&params, "value")?;
        let source = param_str(&params, "source")?;
        let id = self.store.store_profile(category, key, value, source)?;
        Ok(json!({ "id": id }))
    }

    fn handle_store_session(&self, params: Value) -> Result<Value> {
        let session_id = param_str(&params, "session_id")?;
        let step_number = param_i32(&params, "step_number")?;
        let agent_name = param_str(&params, "agent_name")?;
        let event_type = param_str(&params, "event_type")?;
        let content = param_str(&params, "content")?;
        let metadata = params
            .get("metadata")
            .and_then(|v| v.as_str());

        let id =
            self.store
                .store_session(session_id, step_number, agent_name, event_type, content, metadata)?;
        Ok(json!({ "id": id }))
    }

    fn handle_store_task_outcome(&self, params: Value) -> Result<Value> {
        let task_description = param_str(&params, "task_description")?;
        let category = param_str(&params, "category")?;
        let outcome = param_str(&params, "outcome")?;
        let procedure_steps = param_str(&params, "procedure_steps")?;
        let error_patterns = param_str(&params, "error_patterns")?;
        let fix_patterns = param_str(&params, "fix_patterns")?;
        let duration_seconds = param_i64(&params, "duration_seconds")?;
        let system_context = param_str(&params, "system_context")?;
        let session_id = param_str(&params, "session_id")?;

        let id = self.store.store_task_outcome(
            task_description,
            category,
            outcome,
            procedure_steps,
            error_patterns,
            fix_patterns,
            duration_seconds,
            system_context,
            session_id,
        )?;
        Ok(json!({ "id": id }))
    }

    fn handle_store_agent_learning(&self, params: Value) -> Result<Value> {
        let agent_name = param_str(&params, "agent_name")?;
        let memory_type = param_str(&params, "memory_type")?;
        let content = param_str(&params, "content")?;
        let confidence = params
            .get("confidence")
            .and_then(|v| v.as_f64())
            .context("missing or invalid param: confidence")?;

        let id = self
            .store
            .store_agent_learning(agent_name, memory_type, content, confidence)?;
        Ok(json!({ "id": id }))
    }

    fn handle_forget(&self, params: Value) -> Result<Value> {
        let id = param_str(&params, "id")?;
        let table = param_str(&params, "table")?;
        self.store.forget(id, table)?;
        Ok(json!({ "ok": true }))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract a required `&str` parameter from a JSON object.
fn param_str<'a>(params: &'a Value, key: &str) -> Result<&'a str> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .with_context(|| format!("missing or invalid param: {key}"))
}

/// Extract a required `i32` parameter.
fn param_i32(params: &Value, key: &str) -> Result<i32> {
    params
        .get(key)
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .with_context(|| format!("missing or invalid param: {key}"))
}

/// Extract an optional `i32` parameter with a default value.
fn param_i32_or(params: &Value, key: &str, default: i32) -> i32 {
    params
        .get(key)
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .unwrap_or(default)
}

/// Extract a required `i64` parameter.
fn param_i64(params: &Value, key: &str) -> Result<i64> {
    params
        .get(key)
        .and_then(|v| v.as_i64())
        .with_context(|| format!("missing or invalid param: {key}"))
}

/// Parse a scope string into a [`MemoryScope`].
fn parse_scope(s: &str) -> Result<MemoryScope> {
    match s {
        "profile" => Ok(MemoryScope::Profile),
        "session" => Ok(MemoryScope::Session),
        "task" => Ok(MemoryScope::Task),
        "agent" => Ok(MemoryScope::Agent),
        "all" => Ok(MemoryScope::All),
        other => bail!("invalid scope: {other}"),
    }
}

/// Convert a `Vec<TaskEntry>` into a JSON array.
fn task_entries_to_json(entries: Vec<crate::memory_store::TaskEntry>) -> Vec<Value> {
    entries
        .into_iter()
        .map(|e| {
            json!({
                "id": e.id,
                "task_description": e.task_description,
                "task_category": e.task_category,
                "outcome": e.outcome,
                "procedure_steps": e.procedure_steps,
                "error_patterns": e.error_patterns,
                "fix_patterns": e.fix_patterns,
                "duration_seconds": e.duration_seconds,
                "session_id": e.session_id,
                "created_at": e.created_at,
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::LocalDb;

    /// Helper: create a `MemoryServer` backed by an in-memory database.
    fn test_server() -> MemoryServer {
        let db = LocalDb::open_in_memory().expect("in-memory db");
        let store = Arc::new(MemoryStore::new(Arc::new(db)));
        MemoryServer::new(store)
    }

    // -- tool_definitions ---------------------------------------------------

    #[test]
    fn test_tool_definitions_count_and_names() {
        let server = test_server();
        let defs = server.tool_definitions();
        assert_eq!(defs.len(), 9, "expected 9 tool definitions");

        let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"recall"));
        assert!(names.contains(&"recall_profile"));
        assert!(names.contains(&"recall_similar_tasks"));
        assert!(names.contains(&"recall_error_patterns"));
        assert!(names.contains(&"store_profile"));
        assert!(names.contains(&"store_session"));
        assert!(names.contains(&"store_task_outcome"));
        assert!(names.contains(&"store_agent_learning"));
        assert!(names.contains(&"forget"));
    }

    #[test]
    fn test_tool_definitions_have_valid_schemas() {
        let server = test_server();
        for def in server.tool_definitions() {
            assert!(
                def.input_schema.get("type").is_some(),
                "tool {} missing 'type' in schema",
                def.name
            );
            assert!(
                def.input_schema.get("properties").is_some(),
                "tool {} missing 'properties' in schema",
                def.name
            );
        }
    }

    // -- dispatch: store_profile + recall -----------------------------------

    #[test]
    fn test_dispatch_store_profile_and_recall() {
        let server = test_server();

        // Store a profile entry via tool call.
        let result = server
            .handle_tool_call(
                "store_profile",
                json!({
                    "category": "system",
                    "key": "os",
                    "value": "macOS",
                    "source": "agent"
                }),
            )
            .unwrap();
        assert!(result.get("id").is_some(), "store_profile should return an id");

        // Recall it.
        let result = server
            .handle_tool_call(
                "recall",
                json!({
                    "query": "macOS",
                    "scope": "profile",
                    "limit": 5
                }),
            )
            .unwrap();
        let entries = result["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0]["content"].as_str().unwrap().contains("macOS"));
    }

    // -- dispatch: recall_profile -------------------------------------------

    #[test]
    fn test_dispatch_recall_profile() {
        let server = test_server();

        server
            .handle_tool_call(
                "store_profile",
                json!({
                    "category": "env",
                    "key": "shell",
                    "value": "zsh",
                    "source": "agent"
                }),
            )
            .unwrap();

        let result = server
            .handle_tool_call("recall_profile", json!({ "category": "env" }))
            .unwrap();
        let entries = result["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["key"], "shell");
        assert_eq!(entries[0]["value"], "zsh");
    }

    // -- dispatch: store_session --------------------------------------------

    #[test]
    fn test_dispatch_store_session() {
        let server = test_server();

        let result = server
            .handle_tool_call(
                "store_session",
                json!({
                    "session_id": "sess-1",
                    "step_number": 0,
                    "agent_name": "dr_bob",
                    "event_type": "action",
                    "content": "ran npm install"
                }),
            )
            .unwrap();
        assert!(result.get("id").is_some());

        // With optional metadata.
        let result2 = server
            .handle_tool_call(
                "store_session",
                json!({
                    "session_id": "sess-1",
                    "step_number": 1,
                    "agent_name": "dr_bob",
                    "event_type": "observation",
                    "content": "install succeeded",
                    "metadata": "{\"exit_code\":0}"
                }),
            )
            .unwrap();
        assert!(result2.get("id").is_some());
    }

    // -- dispatch: store_task_outcome + recall_similar_tasks -----------------

    #[test]
    fn test_dispatch_store_task_outcome_and_recall() {
        let server = test_server();

        server
            .handle_tool_call(
                "store_task_outcome",
                json!({
                    "task_description": "Fix broken npm install",
                    "category": "dependency",
                    "outcome": "success",
                    "procedure_steps": "[\"rm node_modules\",\"npm ci\"]",
                    "error_patterns": "[\"ERESOLVE\"]",
                    "fix_patterns": "[\"--legacy-peer-deps\"]",
                    "duration_seconds": 42,
                    "system_context": "{\"node\":\"18\"}",
                    "session_id": "sess-1"
                }),
            )
            .unwrap();

        let result = server
            .handle_tool_call(
                "recall_similar_tasks",
                json!({ "query": "npm", "limit": 5 }),
            )
            .unwrap();
        let entries = result["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["task_description"], "Fix broken npm install");
    }

    // -- dispatch: recall_error_patterns ------------------------------------

    #[test]
    fn test_dispatch_recall_error_patterns() {
        let server = test_server();

        server
            .handle_tool_call(
                "store_task_outcome",
                json!({
                    "task_description": "Fix npm install",
                    "category": "dependency",
                    "outcome": "success",
                    "procedure_steps": "[\"npm ci\"]",
                    "error_patterns": "[\"ERESOLVE could not resolve\"]",
                    "fix_patterns": "[\"use --legacy-peer-deps\"]",
                    "duration_seconds": 10,
                    "system_context": "{}",
                    "session_id": "s1"
                }),
            )
            .unwrap();

        let result = server
            .handle_tool_call(
                "recall_error_patterns",
                json!({ "query": "ERESOLVE", "limit": 5 }),
            )
            .unwrap();
        let entries = result["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["task_description"], "Fix npm install");
    }

    // -- dispatch: store_agent_learning -------------------------------------

    #[test]
    fn test_dispatch_store_agent_learning() {
        let server = test_server();

        let result = server
            .handle_tool_call(
                "store_agent_learning",
                json!({
                    "agent_name": "dr_bob",
                    "memory_type": "preference",
                    "content": "User prefers verbose output",
                    "confidence": 0.8
                }),
            )
            .unwrap();
        assert!(result.get("id").is_some());

        // Verify via recall.
        let recall = server
            .handle_tool_call(
                "recall",
                json!({ "query": "verbose", "scope": "agent", "limit": 10 }),
            )
            .unwrap();
        let entries = recall["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0]["content"].as_str().unwrap().contains("verbose"));
    }

    // -- dispatch: forget ---------------------------------------------------

    #[test]
    fn test_dispatch_forget() {
        let server = test_server();

        // Store something first.
        let result = server
            .handle_tool_call(
                "store_agent_learning",
                json!({
                    "agent_name": "dr_bob",
                    "memory_type": "fact",
                    "content": "temporary note",
                    "confidence": 0.5
                }),
            )
            .unwrap();
        let id = result["id"].as_str().unwrap();

        // Forget it.
        let forget_result = server
            .handle_tool_call(
                "forget",
                json!({ "id": id, "table": "agent_memory" }),
            )
            .unwrap();
        assert_eq!(forget_result["ok"], true);

        // Verify it's gone.
        let recall = server
            .handle_tool_call(
                "recall",
                json!({ "query": "temporary note", "scope": "agent", "limit": 10 }),
            )
            .unwrap();
        let entries = recall["entries"].as_array().unwrap();
        assert!(entries.is_empty(), "record should have been deleted");
    }

    // -- dispatch: unknown tool ---------------------------------------------

    #[test]
    fn test_dispatch_unknown_tool() {
        let server = test_server();
        let result = server.handle_tool_call("nonexistent", json!({}));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unknown tool"), "error: {err}");
    }

    // -- dispatch: recall with default limit --------------------------------

    #[test]
    fn test_dispatch_recall_default_limit() {
        let server = test_server();

        // recall without explicit limit should use default (10).
        let result = server
            .handle_tool_call(
                "recall",
                json!({ "query": "anything", "scope": "all" }),
            )
            .unwrap();
        assert!(result.get("entries").is_some());
    }
}
