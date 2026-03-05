//! Local SQLite database for daemon state persistence.
//!
//! Manages the memory schema: profile_memory, session_memory, task_memory,
//! agent_memory, task_memory_fts (FTS5), and audit_log tables.

use rusqlite::Connection;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Local SQLite database handle for the daemon.
pub struct LocalDb {
    conn: Connection,
}

impl LocalDb {
    /// Opens (or creates) the SQLite database at the given path,
    /// enables WAL mode, and runs the idempotent schema migration.
    pub fn open(path: &str) -> anyhow::Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
            debug!(?parent, "Ensured database directory exists");
        }

        let conn = Connection::open(path)?;
        info!("Opened SQLite database at {}", path);

        // Enable WAL mode for concurrent reads
        conn.pragma_update(None, "journal_mode", "WAL")?;
        debug!("WAL mode enabled");

        let db = Self { conn };
        db.init_schema()?;
        info!("Database schema initialized");

        Ok(db)
    }

    /// Opens an in-memory database (useful for testing).
    #[cfg(test)]
    pub fn open_in_memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Returns a reference to the underlying connection.
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Runs the idempotent schema migration (CREATE TABLE IF NOT EXISTS).
    fn init_schema(&self) -> anyhow::Result<()> {
        self.conn.execute_batch(SCHEMA_SQL)?;
        debug!("Schema migration completed");
        Ok(())
    }
}

/// Idempotent schema DDL — safe to run on every startup.
const SCHEMA_SQL: &str = r#"
-- Profile memory: long-lived key/value facts about the user/environment
CREATE TABLE IF NOT EXISTS profile_memory (
    id          TEXT PRIMARY KEY,
    category    TEXT NOT NULL,
    key         TEXT NOT NULL,
    value       TEXT NOT NULL,
    confidence  REAL NOT NULL DEFAULT 1.0,
    source      TEXT NOT NULL DEFAULT 'user',
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(category, key)
);

-- Session memory: ordered log of events within a session
CREATE TABLE IF NOT EXISTS session_memory (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL,
    step_number INTEGER NOT NULL,
    agent_name  TEXT NOT NULL,
    event_type  TEXT NOT NULL,
    content     TEXT NOT NULL,
    metadata    TEXT,  -- JSON
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_session_memory_session_step
    ON session_memory (session_id, step_number);

-- Task memory: outcomes and learned procedures from completed tasks
CREATE TABLE IF NOT EXISTS task_memory (
    id                TEXT PRIMARY KEY,
    task_description  TEXT NOT NULL,
    task_category     TEXT,
    outcome           TEXT,
    procedure_steps   TEXT,  -- JSON array
    error_patterns    TEXT,  -- JSON array
    fix_patterns      TEXT,  -- JSON array
    duration_seconds  INTEGER,
    system_context    TEXT,  -- JSON object
    session_id        TEXT,
    created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- FTS5 virtual table for full-text search over task memory
CREATE VIRTUAL TABLE IF NOT EXISTS task_memory_fts USING fts5(
    task_description,
    procedure_steps,
    error_patterns,
    fix_patterns,
    content='task_memory',
    content_rowid='rowid'
);

-- Triggers to keep FTS5 index in sync with task_memory
CREATE TRIGGER IF NOT EXISTS task_memory_ai AFTER INSERT ON task_memory BEGIN
    INSERT INTO task_memory_fts (rowid, task_description, procedure_steps, error_patterns, fix_patterns)
    VALUES (new.rowid, new.task_description, new.procedure_steps, new.error_patterns, new.fix_patterns);
END;

CREATE TRIGGER IF NOT EXISTS task_memory_ad AFTER DELETE ON task_memory BEGIN
    INSERT INTO task_memory_fts (task_memory_fts, rowid, task_description, procedure_steps, error_patterns, fix_patterns)
    VALUES ('delete', old.rowid, old.task_description, old.procedure_steps, old.error_patterns, old.fix_patterns);
END;

CREATE TRIGGER IF NOT EXISTS task_memory_au AFTER UPDATE ON task_memory BEGIN
    INSERT INTO task_memory_fts (task_memory_fts, rowid, task_description, procedure_steps, error_patterns, fix_patterns)
    VALUES ('delete', old.rowid, old.task_description, old.procedure_steps, old.error_patterns, old.fix_patterns);
    INSERT INTO task_memory_fts (rowid, task_description, procedure_steps, error_patterns, fix_patterns)
    VALUES (new.rowid, new.task_description, new.procedure_steps, new.error_patterns, new.fix_patterns);
END;

-- Agent memory: per-agent learned knowledge and preferences
CREATE TABLE IF NOT EXISTS agent_memory (
    id          TEXT PRIMARY KEY,
    agent_name  TEXT NOT NULL,
    memory_type TEXT NOT NULL,
    content     TEXT NOT NULL,
    confidence  REAL NOT NULL DEFAULT 1.0,
    use_count   INTEGER NOT NULL DEFAULT 0,
    last_used_at TEXT,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_agent_memory_agent_type
    ON agent_memory (agent_name, memory_type);

-- Audit log: immutable record of all memory mutations
CREATE TABLE IF NOT EXISTS audit_log (
    id          TEXT PRIMARY KEY,
    table_name  TEXT NOT NULL,
    record_id   TEXT NOT NULL,
    agent_name  TEXT,
    old_value   TEXT,
    new_value   TEXT,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Archive table for task_memory: stores original rows before compression
CREATE TABLE IF NOT EXISTS task_memory_archive (
    id                TEXT PRIMARY KEY,
    task_description  TEXT NOT NULL,
    task_category     TEXT,
    outcome           TEXT,
    procedure_steps   TEXT,  -- JSON array (original, uncompressed)
    error_patterns    TEXT,  -- JSON array
    fix_patterns      TEXT,  -- JSON array
    duration_seconds  INTEGER,
    system_context    TEXT,  -- JSON object
    session_id        TEXT,
    created_at        TEXT NOT NULL,
    archived_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Archive table for agent_memory: stores original rows before compression
CREATE TABLE IF NOT EXISTS agent_memory_archive (
    id          TEXT PRIMARY KEY,
    agent_name  TEXT NOT NULL,
    memory_type TEXT NOT NULL,
    content     TEXT NOT NULL,
    confidence  REAL NOT NULL DEFAULT 1.0,
    use_count   INTEGER NOT NULL DEFAULT 0,
    last_used_at TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    archived_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    /// Helper: list all user tables in the database.
    fn list_tables(db: &LocalDb) -> Vec<String> {
        let mut stmt = db
            .conn()
            .prepare("SELECT name FROM sqlite_master WHERE type IN ('table', 'trigger') ORDER BY name")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    #[test]
    fn test_schema_creation() {
        let db = LocalDb::open_in_memory().unwrap();
        let objects = list_tables(&db);

        // Tables
        assert!(objects.contains(&"profile_memory".to_string()), "missing profile_memory");
        assert!(objects.contains(&"session_memory".to_string()), "missing session_memory");
        assert!(objects.contains(&"task_memory".to_string()), "missing task_memory");
        assert!(objects.contains(&"agent_memory".to_string()), "missing agent_memory");
        assert!(objects.contains(&"audit_log".to_string()), "missing audit_log");
        assert!(objects.contains(&"task_memory_archive".to_string()), "missing task_memory_archive");
        assert!(objects.contains(&"agent_memory_archive".to_string()), "missing agent_memory_archive");

        // FTS5 virtual table (shows up as a table in sqlite_master)
        let mut stmt = db
            .conn()
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='task_memory_fts'")
            .unwrap();
        let _count: i64 = stmt.query_row([], |row| row.get(0)).unwrap_or(0);
        // FTS5 tables are present — just verify we can query them
        let result: Result<Vec<String>, _> = db
            .conn()
            .prepare("SELECT * FROM task_memory_fts WHERE task_memory_fts MATCH 'test' LIMIT 0")
            .map(|_| vec![]);
        assert!(result.is_ok(), "task_memory_fts should be queryable");

        // Triggers
        assert!(objects.contains(&"task_memory_ai".to_string()), "missing insert trigger");
        assert!(objects.contains(&"task_memory_ad".to_string()), "missing delete trigger");
        assert!(objects.contains(&"task_memory_au".to_string()), "missing update trigger");
    }

    #[test]
    fn test_schema_idempotent() {
        let db = LocalDb::open_in_memory().unwrap();
        // Running init_schema again should not fail
        db.init_schema().unwrap();
        db.init_schema().unwrap();
    }

    #[test]
    fn test_profile_memory_crud() {
        let db = LocalDb::open_in_memory().unwrap();
        let id = "pm-001";

        // INSERT
        db.conn()
            .execute(
                "INSERT INTO profile_memory (id, category, key, value, confidence, source)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, "system", "os", "macOS", 0.95, "agent"],
            )
            .unwrap();

        // SELECT
        let (cat, key, val, conf): (String, String, String, f64) = db
            .conn()
            .query_row(
                "SELECT category, key, value, confidence FROM profile_memory WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();

        assert_eq!(cat, "system");
        assert_eq!(key, "os");
        assert_eq!(val, "macOS");
        assert!((conf - 0.95).abs() < f64::EPSILON);

        // UPDATE
        db.conn()
            .execute(
                "UPDATE profile_memory SET value = ?1 WHERE id = ?2",
                params!["macOS 15", id],
            )
            .unwrap();

        let updated_val: String = db
            .conn()
            .query_row(
                "SELECT value FROM profile_memory WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(updated_val, "macOS 15");

        // DELETE
        let deleted = db
            .conn()
            .execute("DELETE FROM profile_memory WHERE id = ?1", params![id])
            .unwrap();
        assert_eq!(deleted, 1);
    }

    #[test]
    fn test_profile_memory_unique_constraint() {
        let db = LocalDb::open_in_memory().unwrap();

        db.conn()
            .execute(
                "INSERT INTO profile_memory (id, category, key, value) VALUES ('a', 'sys', 'os', 'mac')",
                [],
            )
            .unwrap();

        // Same (category, key) with different id should fail
        let result = db.conn().execute(
            "INSERT INTO profile_memory (id, category, key, value) VALUES ('b', 'sys', 'os', 'linux')",
            [],
        );
        assert!(result.is_err(), "UNIQUE(category, key) should be enforced");
    }

    #[test]
    fn test_session_memory_insert_and_index() {
        let db = LocalDb::open_in_memory().unwrap();

        for i in 0..3 {
            db.conn()
                .execute(
                    "INSERT INTO session_memory (id, session_id, step_number, agent_name, event_type, content)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        format!("sm-{}", i),
                        "sess-1",
                        i,
                        "dr_bob",
                        "action",
                        format!("step {}", i)
                    ],
                )
                .unwrap();
        }

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM session_memory WHERE session_id = 'sess-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_task_memory_fts5() {
        let db = LocalDb::open_in_memory().unwrap();

        db.conn()
            .execute(
                "INSERT INTO task_memory (id, task_description, procedure_steps, error_patterns, fix_patterns)
                 VALUES ('tm-1', 'Fix broken npm install', '[\"rm node_modules\",\"npm ci\"]',
                         '[\"ERESOLVE\"]', '[\"use --legacy-peer-deps\"]')",
                [],
            )
            .unwrap();

        db.conn()
            .execute(
                "INSERT INTO task_memory (id, task_description, procedure_steps, error_patterns, fix_patterns)
                 VALUES ('tm-2', 'Configure PostgreSQL connection', '[\"edit .env\"]',
                         '[\"ECONNREFUSED\"]', '[\"start pg service\"]')",
                [],
            )
            .unwrap();

        // FTS5 search should find the npm task
        let matched_id: String = db
            .conn()
            .query_row(
                "SELECT t.id FROM task_memory t
                 JOIN task_memory_fts f ON t.rowid = f.rowid
                 WHERE task_memory_fts MATCH 'npm'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(matched_id, "tm-1");

        // Search for ECONNREFUSED should find PostgreSQL task
        let matched_id2: String = db
            .conn()
            .query_row(
                "SELECT t.id FROM task_memory t
                 JOIN task_memory_fts f ON t.rowid = f.rowid
                 WHERE task_memory_fts MATCH 'ECONNREFUSED'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(matched_id2, "tm-2");
    }

    #[test]
    fn test_agent_memory_crud() {
        let db = LocalDb::open_in_memory().unwrap();

        db.conn()
            .execute(
                "INSERT INTO agent_memory (id, agent_name, memory_type, content, confidence)
                 VALUES ('am-1', 'dr_bob', 'preference', 'User prefers verbose output', 0.8)",
                [],
            )
            .unwrap();

        let (name, mtype, content): (String, String, String) = db
            .conn()
            .query_row(
                "SELECT agent_name, memory_type, content FROM agent_memory WHERE id = 'am-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert_eq!(name, "dr_bob");
        assert_eq!(mtype, "preference");
        assert_eq!(content, "User prefers verbose output");
    }

    #[test]
    fn test_audit_log_insert() {
        let db = LocalDb::open_in_memory().unwrap();

        db.conn()
            .execute(
                "INSERT INTO audit_log (id, table_name, record_id, agent_name, old_value, new_value)
                 VALUES ('al-1', 'profile_memory', 'pm-001', 'dr_bob', NULL, '{\"os\":\"macOS\"}')",
                [],
            )
            .unwrap();

        let (tbl, rid): (String, String) = db
            .conn()
            .query_row(
                "SELECT table_name, record_id FROM audit_log WHERE id = 'al-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(tbl, "profile_memory");
        assert_eq!(rid, "pm-001");
    }
}
