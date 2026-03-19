/// Audit trail writer for tool executions and LLM calls.
///
/// All writes are non-blocking: failures are logged but never propagate
/// to the caller, ensuring that audit logging cannot break step execution.
use std::sync::Arc;

use crate::station::db::DbHandle;

/// A single tool execution record for the audit trail.
pub struct ToolExecAudit {
    pub agent_id: String,
    pub task_id: Option<String>,
    pub tool_name: String,
    pub input: Option<String>,
    pub output: Option<String>,
    pub approved: bool,
    pub duration_ms: u64,
}

/// A single LLM call record for the audit trail.
pub struct LlmCallAudit {
    pub agent_id: String,
    pub task_id: Option<String>,
    pub model: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub cost_dd: Option<f64>,
}

/// Non-blocking writer that inserts audit records into SQLite.
///
/// Holds an optional [`DbHandle`]; when `None`, all writes are silently
/// skipped (useful in tests or when the DB is not initialized).
#[derive(Clone)]
pub struct AuditWriter {
    db: Option<DbHandle>,
}

impl AuditWriter {
    /// Create a writer backed by an SQLite database.
    pub fn new(db: DbHandle) -> Self {
        Self { db: Some(db) }
    }

    /// Create a no-op writer that discards all audit records.
    pub fn noop() -> Self {
        Self { db: None }
    }

    /// Record a tool execution. Failures are logged, never propagated.
    pub fn record_tool_execution(&self, audit: ToolExecAudit) {
        let db = match &self.db {
            Some(db) => Arc::clone(db),
            None => return,
        };

        let conn = match db.lock() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("audit: failed to acquire db lock for tool execution: {e}");
                return;
            }
        };

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let risk_level = if audit.approved { "auto" } else { "denied" };
        let status = if audit.approved { "success" } else { "denied" };

        let result = conn.execute(
            "INSERT INTO tool_executions (id, agent_id, task_id, tool_name, params, result, risk_level, approved_by, duration_ms, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                id,
                audit.agent_id,
                audit.task_id,
                audit.tool_name,
                audit.input,
                audit.output,
                risk_level,
                if audit.approved { "auto" } else { "user_denied" },
                audit.duration_ms as i64,
                status,
                now,
            ],
        );

        if let Err(e) = result {
            tracing::warn!(
                tool = %audit.tool_name,
                agent = %audit.agent_id,
                "audit: failed to record tool execution: {e}"
            );
        }
    }

    /// Record an LLM call. Failures are logged, never propagated.
    pub fn record_llm_call(&self, audit: LlmCallAudit) {
        let db = match &self.db {
            Some(db) => Arc::clone(db),
            None => return,
        };

        let conn = match db.lock() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("audit: failed to acquire db lock for llm call: {e}");
                return;
            }
        };

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let result = conn.execute(
            "INSERT INTO llm_calls (id, agent_id, task_id, model, prompt_tokens, completion_tokens, cost_dd, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                id,
                audit.agent_id,
                audit.task_id,
                audit.model,
                audit.prompt_tokens as i64,
                audit.completion_tokens as i64,
                audit.cost_dd,
                now,
            ],
        );

        if let Err(e) = result {
            tracing::warn!(
                model = %audit.model,
                agent = %audit.agent_id,
                "audit: failed to record llm call: {e}"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::station::db;

    fn setup_writer() -> (AuditWriter, DbHandle) {
        let db_handle = db::init_memory().expect("init in-memory db");
        // Insert stub agent rows so FK constraints are satisfied.
        {
            let conn = db_handle.lock().unwrap();
            let now = chrono::Utc::now().to_rfc3339();
            for id in &["agent-1", "agent-2", "agent-3"] {
                conn.execute(
                    "INSERT OR IGNORE INTO agents (id, name, role, framework, created_at, updated_at)
                     VALUES (?1, ?1, 'test', 'test', ?2, ?2)",
                    rusqlite::params![id, now],
                )
                .unwrap();
            }
        }
        let writer = AuditWriter::new(db_handle.clone());
        (writer, db_handle)
    }

    #[test]
    fn test_record_tool_execution_approved() {
        let (writer, db_handle) = setup_writer();

        writer.record_tool_execution(ToolExecAudit {
            agent_id: "agent-1".to_string(),
            task_id: None,
            tool_name: "read_file".to_string(),
            input: Some(r#"{"path":"test.txt"}"#.to_string()),
            output: Some(r#"{"content":"hello"}"#.to_string()),
            approved: true,
            duration_ms: 42,
        });

        let conn = db_handle.lock().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tool_executions WHERE agent_id = ?1",
                rusqlite::params!["agent-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        let tool_name: String = conn
            .query_row(
                "SELECT tool_name FROM tool_executions WHERE agent_id = ?1",
                rusqlite::params!["agent-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tool_name, "read_file");

        let duration: i64 = conn
            .query_row(
                "SELECT duration_ms FROM tool_executions WHERE agent_id = ?1",
                rusqlite::params!["agent-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(duration, 42);
    }

    #[test]
    fn test_record_tool_execution_denied() {
        let (writer, db_handle) = setup_writer();

        writer.record_tool_execution(ToolExecAudit {
            agent_id: "agent-2".to_string(),
            task_id: None,
            tool_name: "shell".to_string(),
            input: Some("rm -rf /".to_string()),
            output: None,
            approved: false,
            duration_ms: 0,
        });

        let conn = db_handle.lock().unwrap();
        let status: String = conn
            .query_row(
                "SELECT status FROM tool_executions WHERE agent_id = ?1",
                rusqlite::params!["agent-2"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "denied");

        let approved_by: String = conn
            .query_row(
                "SELECT approved_by FROM tool_executions WHERE agent_id = ?1",
                rusqlite::params!["agent-2"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(approved_by, "user_denied");
    }

    #[test]
    fn test_record_llm_call() {
        let (writer, db_handle) = setup_writer();

        writer.record_llm_call(LlmCallAudit {
            agent_id: "agent-1".to_string(),
            task_id: None,
            model: "claude-sonnet-4".to_string(),
            prompt_tokens: 1000,
            completion_tokens: 500,
            cost_dd: Some(4.5),
        });

        let conn = db_handle.lock().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM llm_calls WHERE agent_id = ?1",
                rusqlite::params!["agent-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        let (model, prompt, completion, cost): (String, i64, i64, f64) = conn
            .query_row(
                "SELECT model, prompt_tokens, completion_tokens, cost_dd FROM llm_calls WHERE agent_id = ?1",
                rusqlite::params!["agent-1"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(model, "claude-sonnet-4");
        assert_eq!(prompt, 1000);
        assert_eq!(completion, 500);
        assert!((cost - 4.5).abs() < 0.001);
    }

    #[test]
    fn test_record_llm_call_no_cost() {
        let (writer, db_handle) = setup_writer();

        writer.record_llm_call(LlmCallAudit {
            agent_id: "agent-3".to_string(),
            task_id: None,
            model: "claude-haiku-4-5".to_string(),
            prompt_tokens: 200,
            completion_tokens: 100,
            cost_dd: None,
        });

        let conn = db_handle.lock().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM llm_calls WHERE agent_id = ?1",
                rusqlite::params!["agent-3"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_multiple_tool_executions() {
        let (writer, db_handle) = setup_writer();

        for i in 0..5 {
            writer.record_tool_execution(ToolExecAudit {
                agent_id: "agent-1".to_string(),
                task_id: None,
                tool_name: format!("tool-{i}"),
                input: None,
                output: None,
                approved: true,
                duration_ms: i as u64 * 10,
            });
        }

        let conn = db_handle.lock().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tool_executions WHERE agent_id = ?1",
                rusqlite::params!["agent-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_noop_writer_does_not_panic() {
        let writer = AuditWriter::noop();

        // These should silently no-op without panicking.
        writer.record_tool_execution(ToolExecAudit {
            agent_id: "agent-1".to_string(),
            task_id: None,
            tool_name: "shell".to_string(),
            input: None,
            output: None,
            approved: true,
            duration_ms: 0,
        });

        writer.record_llm_call(LlmCallAudit {
            agent_id: "agent-1".to_string(),
            task_id: None,
            model: "test".to_string(),
            prompt_tokens: 0,
            completion_tokens: 0,
            cost_dd: None,
        });
    }

    #[test]
    fn test_multiple_llm_calls_same_agent() {
        let (writer, db_handle) = setup_writer();

        for _ in 0..3 {
            writer.record_llm_call(LlmCallAudit {
                agent_id: "agent-1".to_string(),
                task_id: None,
                model: "claude-sonnet-4".to_string(),
                prompt_tokens: 500,
                completion_tokens: 250,
                cost_dd: Some(2.0),
            });
        }

        let conn = db_handle.lock().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM llm_calls WHERE agent_id = ?1",
                rusqlite::params!["agent-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 3);
    }
}
