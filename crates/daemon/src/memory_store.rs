//! Typed CRUD and recall operations over the local memory database.
//!
//! [`MemoryStore`] wraps a [`LocalDb`] and provides high-level methods for
//! storing profile facts, session events, task outcomes, and agent learnings,
//! plus FTS5-powered recall queries.

use std::sync::Arc;

use anyhow::{Context, Result};
use rusqlite::params;
use tracing::debug;
use uuid::Uuid;

use crate::local_db::LocalDb;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Which memory table(s) a recall query should search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryScope {
    Profile,
    Session,
    Task,
    Agent,
    All,
}

/// Generic memory entry returned by [`MemoryStore::recall`].
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub id: String,
    pub level: String,
    pub content: String,
    pub metadata: Option<String>,
    pub confidence: f64,
    pub created_at: String,
}

/// A row from `profile_memory`.
#[derive(Debug, Clone)]
pub struct ProfileEntry {
    pub id: String,
    pub category: String,
    pub key: String,
    pub value: String,
    pub confidence: f64,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

/// A row from `task_memory`.
#[derive(Debug, Clone)]
pub struct TaskEntry {
    pub id: String,
    pub task_description: String,
    pub task_category: String,
    pub outcome: String,
    pub procedure_steps: String,
    pub error_patterns: String,
    pub fix_patterns: String,
    pub duration_seconds: i64,
    pub session_id: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// MemoryStore
// ---------------------------------------------------------------------------

/// High-level CRUD + recall wrapper around [`LocalDb`].
pub struct MemoryStore {
    db: Arc<LocalDb>,
}

impl MemoryStore {
    /// Create a new `MemoryStore` backed by the given database.
    pub fn new(db: Arc<LocalDb>) -> Self {
        Self { db }
    }

    // -- Store methods ------------------------------------------------------

    /// Insert or replace a profile memory entry.
    ///
    /// On conflict (`category`, `key` pair already exists) the old value is
    /// logged to `audit_log` and the row is replaced.  Returns the new row id.
    pub fn store_profile(
        &self,
        category: &str,
        key: &str,
        value: &str,
        source: &str,
    ) -> Result<String> {
        let conn = self.db.conn();
        let id = Uuid::new_v4().to_string();

        // Check for an existing row so we can audit the replacement.
        let existing: Option<(String, String)> = conn
            .query_row(
                "SELECT id, value FROM profile_memory WHERE category = ?1 AND key = ?2",
                params![category, key],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        if let Some((old_id, old_value)) = existing {
            // Audit the replacement.
            let audit_id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO audit_log (id, table_name, record_id, agent_name, old_value, new_value)
                 VALUES (?1, 'profile_memory', ?2, ?3, ?4, ?5)",
                params![audit_id, old_id, source, old_value, value],
            )
            .context("audit_log insert for profile replacement")?;
            debug!(old_id, %category, %key, "Audited profile memory replacement");
        }

        conn.execute(
            "INSERT OR REPLACE INTO profile_memory (id, category, key, value, confidence, source, updated_at)
             VALUES (?1, ?2, ?3, ?4, 1.0, ?5, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
            params![id, category, key, value, source],
        )
        .context("store_profile insert")?;

        debug!(%id, %category, %key, "Stored profile memory");
        Ok(id)
    }

    /// Append an event to session memory.  Returns the new row id.
    pub fn store_session(
        &self,
        session_id: &str,
        step_number: i32,
        agent_name: &str,
        event_type: &str,
        content: &str,
        metadata: Option<&str>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        self.db
            .conn()
            .execute(
                "INSERT INTO session_memory (id, session_id, step_number, agent_name, event_type, content, metadata)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![id, session_id, step_number, agent_name, event_type, content, metadata],
            )
            .context("store_session insert")?;

        debug!(%id, %session_id, step_number, "Stored session memory");
        Ok(id)
    }

    /// Record a completed task outcome.  The FTS5 index is automatically
    /// updated via the database trigger.  Returns the new row id.
    pub fn store_task_outcome(
        &self,
        task_description: &str,
        category: &str,
        outcome: &str,
        procedure_steps: &str,
        error_patterns: &str,
        fix_patterns: &str,
        duration: i64,
        system_context: &str,
        session_id: &str,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        self.db
            .conn()
            .execute(
                "INSERT INTO task_memory
                    (id, task_description, task_category, outcome, procedure_steps,
                     error_patterns, fix_patterns, duration_seconds, system_context, session_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    id,
                    task_description,
                    category,
                    outcome,
                    procedure_steps,
                    error_patterns,
                    fix_patterns,
                    duration,
                    system_context,
                    session_id,
                ],
            )
            .context("store_task_outcome insert")?;

        debug!(%id, %category, "Stored task outcome");
        Ok(id)
    }

    /// Record an agent learning.  Returns the new row id.
    pub fn store_agent_learning(
        &self,
        agent_name: &str,
        memory_type: &str,
        content: &str,
        confidence: f64,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        self.db
            .conn()
            .execute(
                "INSERT INTO agent_memory (id, agent_name, memory_type, content, confidence)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, agent_name, memory_type, content, confidence],
            )
            .context("store_agent_learning insert")?;

        debug!(%id, %agent_name, %memory_type, "Stored agent learning");
        Ok(id)
    }

    // -- Recall methods -----------------------------------------------------

    /// General-purpose recall across one or all memory scopes.
    ///
    /// * `Task` and `Agent` scopes use FTS5 MATCH.
    /// * `Profile` uses LIKE on key and value columns.
    /// * `Session` uses LIKE on content.
    /// * `All` unions results from every scope.
    pub fn recall(&self, query: &str, scope: MemoryScope, limit: i32) -> Result<Vec<MemoryEntry>> {
        match scope {
            MemoryScope::Profile => self.recall_profile_generic(query, limit),
            MemoryScope::Session => self.recall_session_generic(query, limit),
            MemoryScope::Task => self.recall_task_generic(query, limit),
            MemoryScope::Agent => self.recall_agent_generic(query, limit),
            MemoryScope::All => {
                let per = std::cmp::max(limit / 4, 1);
                let mut entries = Vec::new();
                entries.extend(self.recall_profile_generic(query, per)?);
                entries.extend(self.recall_session_generic(query, per)?);
                entries.extend(self.recall_task_generic(query, per)?);
                entries.extend(self.recall_agent_generic(query, per)?);
                entries.truncate(limit as usize);
                Ok(entries)
            }
        }
    }

    /// Return all profile entries for the given `category`.
    pub fn recall_profile(&self, category: &str) -> Result<Vec<ProfileEntry>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, category, key, value, confidence, source, created_at, updated_at
             FROM profile_memory WHERE category = ?1 ORDER BY key",
        )?;

        let rows = stmt
            .query_map(params![category], |row| {
                Ok(ProfileEntry {
                    id: row.get(0)?,
                    category: row.get(1)?,
                    key: row.get(2)?,
                    value: row.get(3)?,
                    confidence: row.get(4)?,
                    source: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    /// FTS5-ranked search over `task_memory`.
    pub fn recall_similar_tasks(&self, query: &str, limit: i32) -> Result<Vec<TaskEntry>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.task_description, COALESCE(t.task_category, ''),
                    COALESCE(t.outcome, ''), COALESCE(t.procedure_steps, ''),
                    COALESCE(t.error_patterns, ''), COALESCE(t.fix_patterns, ''),
                    COALESCE(t.duration_seconds, 0), COALESCE(t.session_id, ''),
                    t.created_at
             FROM task_memory t
             JOIN task_memory_fts f ON t.rowid = f.rowid
             WHERE task_memory_fts MATCH ?1
             ORDER BY f.rank
             LIMIT ?2",
        )?;

        let rows = stmt
            .query_map(params![query, limit], |row| {
                Ok(TaskEntry {
                    id: row.get(0)?,
                    task_description: row.get(1)?,
                    task_category: row.get(2)?,
                    outcome: row.get(3)?,
                    procedure_steps: row.get(4)?,
                    error_patterns: row.get(5)?,
                    fix_patterns: row.get(6)?,
                    duration_seconds: row.get(7)?,
                    session_id: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    /// FTS5 search scoped to `error_patterns` and `fix_patterns` columns.
    pub fn recall_error_patterns(&self, query: &str, limit: i32) -> Result<Vec<TaskEntry>> {
        let conn = self.db.conn();
        // Scope the MATCH to error_patterns and fix_patterns columns.
        let fts_query = format!("{{error_patterns fix_patterns}} : {query}");
        let mut stmt = conn.prepare(
            "SELECT t.id, t.task_description, COALESCE(t.task_category, ''),
                    COALESCE(t.outcome, ''), COALESCE(t.procedure_steps, ''),
                    COALESCE(t.error_patterns, ''), COALESCE(t.fix_patterns, ''),
                    COALESCE(t.duration_seconds, 0), COALESCE(t.session_id, ''),
                    t.created_at
             FROM task_memory t
             JOIN task_memory_fts f ON t.rowid = f.rowid
             WHERE task_memory_fts MATCH ?1
             ORDER BY f.rank
             LIMIT ?2",
        )?;

        let rows = stmt
            .query_map(params![fts_query, limit], |row| {
                Ok(TaskEntry {
                    id: row.get(0)?,
                    task_description: row.get(1)?,
                    task_category: row.get(2)?,
                    outcome: row.get(3)?,
                    procedure_steps: row.get(4)?,
                    error_patterns: row.get(5)?,
                    fix_patterns: row.get(6)?,
                    duration_seconds: row.get(7)?,
                    session_id: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    // -- Management methods -------------------------------------------------

    /// Delete a record from the specified table and log the deletion.
    pub fn forget(&self, id: &str, table: &str) -> Result<()> {
        let conn = self.db.conn();

        // Validate table name to prevent SQL injection (only our known tables).
        let valid_tables = [
            "profile_memory",
            "session_memory",
            "task_memory",
            "agent_memory",
        ];
        anyhow::ensure!(
            valid_tables.contains(&table),
            "invalid table name: {table}"
        );

        let deleted = conn
            .execute(
                &format!("DELETE FROM {table} WHERE id = ?1"),
                params![id],
            )
            .context("forget delete")?;

        if deleted > 0 {
            let audit_id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO audit_log (id, table_name, record_id, agent_name, old_value, new_value)
                 VALUES (?1, ?2, ?3, 'system', 'deleted', NULL)",
                params![audit_id, table, id],
            )
            .context("forget audit_log insert")?;
            debug!(%id, %table, "Forgot memory record");
        }

        Ok(())
    }

    /// Increment the use count and update `last_used_at` for an agent memory.
    pub fn increment_use_count(&self, agent_memory_id: &str) -> Result<()> {
        let updated = self
            .db
            .conn()
            .execute(
                "UPDATE agent_memory
                 SET use_count = use_count + 1,
                     last_used_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                     updated_at   = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id = ?1",
                params![agent_memory_id],
            )
            .context("increment_use_count update")?;

        anyhow::ensure!(updated == 1, "agent_memory row not found: {agent_memory_id}");
        debug!(%agent_memory_id, "Incremented use count");
        Ok(())
    }

    // -- Private helpers ----------------------------------------------------

    fn recall_profile_generic(&self, query: &str, limit: i32) -> Result<Vec<MemoryEntry>> {
        let conn = self.db.conn();
        let like = format!("%{query}%");
        let mut stmt = conn.prepare(
            "SELECT id, 'profile' AS level, key || '=' || value AS content,
                    NULL AS metadata, confidence, created_at
             FROM profile_memory
             WHERE key LIKE ?1 OR value LIKE ?1
             LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![like, limit], |row| {
                Ok(MemoryEntry {
                    id: row.get(0)?,
                    level: row.get(1)?,
                    content: row.get(2)?,
                    metadata: row.get(3)?,
                    confidence: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn recall_session_generic(&self, query: &str, limit: i32) -> Result<Vec<MemoryEntry>> {
        let conn = self.db.conn();
        let like = format!("%{query}%");
        let mut stmt = conn.prepare(
            "SELECT id, 'session' AS level, content,
                    metadata, 1.0 AS confidence, created_at
             FROM session_memory
             WHERE content LIKE ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![like, limit], |row| {
                Ok(MemoryEntry {
                    id: row.get(0)?,
                    level: row.get(1)?,
                    content: row.get(2)?,
                    metadata: row.get(3)?,
                    confidence: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn recall_task_generic(&self, query: &str, limit: i32) -> Result<Vec<MemoryEntry>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT t.id, 'task' AS level, t.task_description AS content,
                    t.outcome AS metadata, 1.0 AS confidence, t.created_at
             FROM task_memory t
             JOIN task_memory_fts f ON t.rowid = f.rowid
             WHERE task_memory_fts MATCH ?1
             ORDER BY f.rank
             LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![query, limit], |row| {
                Ok(MemoryEntry {
                    id: row.get(0)?,
                    level: row.get(1)?,
                    content: row.get(2)?,
                    metadata: row.get(3)?,
                    confidence: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn recall_agent_generic(&self, query: &str, limit: i32) -> Result<Vec<MemoryEntry>> {
        let conn = self.db.conn();
        let like = format!("%{query}%");
        let mut stmt = conn.prepare(
            "SELECT id, 'agent' AS level, content,
                    memory_type AS metadata, confidence, created_at
             FROM agent_memory
             WHERE content LIKE ?1
             ORDER BY confidence DESC
             LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![like, limit], |row| {
                Ok(MemoryEntry {
                    id: row.get(0)?,
                    level: row.get(1)?,
                    content: row.get(2)?,
                    metadata: row.get(3)?,
                    confidence: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a MemoryStore backed by an in-memory database.
    fn test_store() -> MemoryStore {
        let db = LocalDb::open_in_memory().expect("in-memory db");
        MemoryStore::new(Arc::new(db))
    }

    // -- store_profile ------------------------------------------------------

    #[test]
    fn test_store_profile_insert() {
        let store = test_store();
        let id = store.store_profile("system", "os", "macOS", "agent").unwrap();
        assert!(!id.is_empty());

        let entries = store.recall_profile("system").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "os");
        assert_eq!(entries[0].value, "macOS");
        assert_eq!(entries[0].source, "agent");
    }

    #[test]
    fn test_store_profile_replace_with_audit() {
        let store = test_store();

        // First insert
        store.store_profile("system", "os", "macOS", "agent").unwrap();

        // Replace — should audit the old value
        let id2 = store.store_profile("system", "os", "Linux", "user").unwrap();
        assert!(!id2.is_empty());

        // Verify replacement
        let entries = store.recall_profile("system").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].value, "Linux");

        // Verify audit log
        let count: i64 = store
            .db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM audit_log WHERE table_name = 'profile_memory'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "expected one audit_log entry for the replacement");
    }

    // -- store_session ------------------------------------------------------

    #[test]
    fn test_store_session_insert() {
        let store = test_store();

        let id = store
            .store_session("sess-1", 0, "dr_bob", "action", "ran npm install", None)
            .unwrap();
        assert!(!id.is_empty());

        let id2 = store
            .store_session(
                "sess-1",
                1,
                "dr_bob",
                "observation",
                "install succeeded",
                Some(r#"{"exit_code":0}"#),
            )
            .unwrap();
        assert!(!id2.is_empty());

        // Verify via recall
        let results = store.recall("npm install", MemoryScope::Session, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].level, "session");
    }

    // -- store_task_outcome -------------------------------------------------

    #[test]
    fn test_store_task_outcome_and_fts_recall() {
        let store = test_store();

        store
            .store_task_outcome(
                "Fix broken npm install",
                "dependency",
                "success",
                r#"["rm node_modules","npm ci"]"#,
                r#"["ERESOLVE"]"#,
                r#"["--legacy-peer-deps"]"#,
                42,
                r#"{"node":"18"}"#,
                "sess-1",
            )
            .unwrap();

        store
            .store_task_outcome(
                "Configure PostgreSQL connection",
                "database",
                "success",
                r#"["edit .env"]"#,
                r#"["ECONNREFUSED"]"#,
                r#"["start pg service"]"#,
                15,
                r#"{"pg":"15"}"#,
                "sess-2",
            )
            .unwrap();

        // FTS5 recall should find the npm task
        let results = store.recall_similar_tasks("npm", 5).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].task_description, "Fix broken npm install");
        assert_eq!(results[0].task_category, "dependency");
        assert_eq!(results[0].duration_seconds, 42);
    }

    // -- store_agent_learning -----------------------------------------------

    #[test]
    fn test_store_agent_learning() {
        let store = test_store();
        let id = store
            .store_agent_learning("dr_bob", "preference", "User prefers verbose output", 0.8)
            .unwrap();
        assert!(!id.is_empty());

        let results = store.recall("verbose", MemoryScope::Agent, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].level, "agent");
        assert!(results[0].content.contains("verbose"));
    }

    // -- recall (generic) ---------------------------------------------------

    #[test]
    fn test_recall_profile_scope() {
        let store = test_store();
        store.store_profile("env", "shell", "zsh", "agent").unwrap();
        store.store_profile("env", "editor", "nvim", "agent").unwrap();

        let results = store.recall("zsh", MemoryScope::Profile, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("zsh"));
    }

    #[test]
    fn test_recall_task_scope() {
        let store = test_store();
        store
            .store_task_outcome(
                "Deploy to production",
                "deployment",
                "success",
                r#"["docker build","docker push"]"#,
                r#"[]"#,
                r#"[]"#,
                120,
                "{}",
                "sess-5",
            )
            .unwrap();

        let results = store.recall("production", MemoryScope::Task, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].level, "task");
    }

    #[test]
    fn test_recall_all_scope() {
        let store = test_store();

        store.store_profile("env", "lang", "rust", "agent").unwrap();
        store
            .store_session("s1", 0, "bob", "note", "rust is great", None)
            .unwrap();
        store
            .store_task_outcome(
                "Compile rust project",
                "build",
                "success",
                r#"["cargo build"]"#,
                r#"[]"#,
                r#"[]"#,
                10,
                "{}",
                "s1",
            )
            .unwrap();
        store
            .store_agent_learning("bob", "fact", "User writes rust code", 0.9)
            .unwrap();

        let results = store.recall("rust", MemoryScope::All, 20).unwrap();
        assert!(results.len() >= 3, "expected results from multiple scopes, got {}", results.len());

        let levels: Vec<&str> = results.iter().map(|e| e.level.as_str()).collect();
        assert!(levels.contains(&"profile"), "missing profile result");
        assert!(levels.contains(&"session"), "missing session result");
        assert!(levels.contains(&"task"), "missing task result");
    }

    // -- recall_similar_tasks (FTS5 ranking) --------------------------------

    #[test]
    fn test_recall_similar_tasks_ranking() {
        let store = test_store();

        // Insert a task that mentions "docker" heavily
        store
            .store_task_outcome(
                "Docker build failure",
                "docker",
                "fixed",
                r#"["docker build","docker push","docker compose up"]"#,
                r#"["docker daemon not running"]"#,
                r#"["restart docker daemon"]"#,
                30,
                "{}",
                "s1",
            )
            .unwrap();

        // Insert a task that mentions "docker" only once
        store
            .store_task_outcome(
                "Setup CI pipeline with docker",
                "ci",
                "success",
                r#"["edit .github/workflows"]"#,
                r#"[]"#,
                r#"[]"#,
                60,
                "{}",
                "s2",
            )
            .unwrap();

        let results = store.recall_similar_tasks("docker", 10).unwrap();
        assert_eq!(results.len(), 2);
        // The first result should be the more relevant (higher docker density) task
        assert_eq!(
            results[0].task_description, "Docker build failure",
            "FTS5 rank should prefer the task with more matches"
        );
    }

    // -- recall_error_patterns ----------------------------------------------

    #[test]
    fn test_recall_error_patterns() {
        let store = test_store();

        store
            .store_task_outcome(
                "Fix npm install",
                "dependency",
                "success",
                r#"["npm ci"]"#,
                r#"["ERESOLVE could not resolve"]"#,
                r#"["use --legacy-peer-deps"]"#,
                10,
                "{}",
                "s1",
            )
            .unwrap();

        store
            .store_task_outcome(
                "Fix database connection",
                "database",
                "success",
                r#"["edit .env"]"#,
                r#"["ECONNREFUSED 5432"]"#,
                r#"["start postgresql service"]"#,
                5,
                "{}",
                "s2",
            )
            .unwrap();

        let results = store.recall_error_patterns("ERESOLVE", 5).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].task_description, "Fix npm install");

        let results2 = store.recall_error_patterns("ECONNREFUSED", 5).unwrap();
        assert_eq!(results2.len(), 1);
        assert_eq!(results2[0].task_description, "Fix database connection");
    }

    // -- forget -------------------------------------------------------------

    #[test]
    fn test_forget_with_audit() {
        let store = test_store();

        let id = store
            .store_agent_learning("dr_bob", "preference", "likes cats", 0.5)
            .unwrap();

        // Forget it
        store.forget(&id, "agent_memory").unwrap();

        // Verify deleted
        let count: i64 = store
            .db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM agent_memory WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "record should be deleted");

        // Verify audit log entry
        let audit_count: i64 = store
            .db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM audit_log WHERE record_id = ?1 AND table_name = 'agent_memory'",
                params![id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(audit_count, 1, "expected audit_log entry for deletion");
    }

    #[test]
    fn test_forget_invalid_table() {
        let store = test_store();
        let result = store.forget("some-id", "users; DROP TABLE audit_log;");
        assert!(result.is_err(), "should reject invalid table names");
    }

    // -- increment_use_count ------------------------------------------------

    #[test]
    fn test_increment_use_count() {
        let store = test_store();

        let id = store
            .store_agent_learning("dr_bob", "skill", "debugging rust", 0.9)
            .unwrap();

        // Initial state
        let (use_count, last_used): (i64, Option<String>) = store
            .db
            .conn()
            .query_row(
                "SELECT use_count, last_used_at FROM agent_memory WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(use_count, 0);
        assert!(last_used.is_none());

        // Increment
        store.increment_use_count(&id).unwrap();

        let (use_count2, last_used2): (i64, Option<String>) = store
            .db
            .conn()
            .query_row(
                "SELECT use_count, last_used_at FROM agent_memory WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(use_count2, 1);
        assert!(last_used2.is_some(), "last_used_at should be set after increment");

        // Increment again
        store.increment_use_count(&id).unwrap();

        let use_count3: i64 = store
            .db
            .conn()
            .query_row(
                "SELECT use_count FROM agent_memory WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(use_count3, 2);
    }

    #[test]
    fn test_increment_use_count_missing_id() {
        let store = test_store();
        let result = store.increment_use_count("nonexistent-id");
        assert!(result.is_err(), "should error for missing agent_memory id");
    }

    // -- recall_profile (typed) ---------------------------------------------

    #[test]
    fn test_recall_profile_by_category() {
        let store = test_store();

        store.store_profile("system", "os", "macOS", "agent").unwrap();
        store.store_profile("system", "arch", "arm64", "agent").unwrap();
        store.store_profile("user", "name", "Alice", "user").unwrap();

        let system_entries = store.recall_profile("system").unwrap();
        assert_eq!(system_entries.len(), 2);

        let user_entries = store.recall_profile("user").unwrap();
        assert_eq!(user_entries.len(), 1);
        assert_eq!(user_entries[0].value, "Alice");

        let empty = store.recall_profile("nonexistent").unwrap();
        assert!(empty.is_empty());
    }
}
