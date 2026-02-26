//! Local SQLite database for daemon state persistence.
//!
//! Stores session history, task records, and audit log locally
//! so the daemon can report on past activity without cloud connectivity.

use rusqlite::{Connection, params};
use anyhow::Result;

pub struct LocalDb {
    pub conn: Connection,
}

impl LocalDb {
    /// Open (or create) the SQLite database at `path`.
    /// Enables WAL mode and foreign keys, then creates tables if needed.
    /// Pass `:memory:` for an in-memory database (tests).
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Performance and safety settings
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             PRAGMA synchronous=NORMAL;",
        )?;

        let db = Self { conn };
        db.create_schema()?;
        Ok(db)
    }

    fn create_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id          TEXT PRIMARY KEY,
                user_id     TEXT NOT NULL,
                started_at  INTEGER NOT NULL DEFAULT (unixepoch()),
                ended_at    INTEGER,
                status      TEXT NOT NULL DEFAULT 'active'
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id          TEXT PRIMARY KEY,
                session_id  TEXT NOT NULL REFERENCES sessions(id),
                request     TEXT NOT NULL,
                status      TEXT NOT NULL DEFAULT 'pending',
                created_at  INTEGER NOT NULL DEFAULT (unixepoch())
            );

            CREATE TABLE IF NOT EXISTS commands (
                id           TEXT PRIMARY KEY,
                task_id      TEXT NOT NULL REFERENCES tasks(id),
                command_type TEXT NOT NULL,
                target       TEXT NOT NULL,
                args_json    TEXT NOT NULL DEFAULT '[]',
                result_json  TEXT,
                executed_at  INTEGER NOT NULL DEFAULT (unixepoch())
            );

            CREATE TABLE IF NOT EXISTS audit_log (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id  TEXT NOT NULL REFERENCES sessions(id),
                action      TEXT NOT NULL,
                detail      TEXT NOT NULL,
                risk_tier   TEXT NOT NULL DEFAULT 'LOW',
                timestamp   INTEGER NOT NULL DEFAULT (unixepoch())
            );",
        )?;
        Ok(())
    }

    pub fn insert_task(&self, id: &str, session_id: &str, request: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tasks (id, session_id, request) VALUES (?1, ?2, ?3)",
            params![id, session_id, request],
        )?;
        Ok(())
    }

    pub fn append_audit_log(
        &self,
        session_id: &str,
        action: &str,
        detail: &str,
        risk_tier: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO audit_log (session_id, action, detail, risk_tier)
             VALUES (?1, ?2, ?3, ?4)",
            params![session_id, action, detail, risk_tier],
        )?;
        Ok(())
    }

    pub fn insert_session(&self, id: &str, user_id: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sessions (id, user_id, status) VALUES (?1, ?2, 'active')",
            params![id, user_id],
        )?;
        Ok(())
    }

    pub fn end_session(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET status='completed', ended_at=unixepoch() WHERE id=?1",
            params![id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn in_memory_db() -> LocalDb {
        LocalDb::open(":memory:").expect("in-memory db should open")
    }

    #[test]
    fn test_open_creates_schema() {
        let db = in_memory_db();
        // Verify sessions table exists by inserting a row
        let result = db.conn.execute(
            "INSERT INTO sessions (id, user_id, status) VALUES (?1, ?2, ?3)",
            rusqlite::params!["sess-1", "user-1", "active"],
        );
        assert!(result.is_ok(), "sessions table should exist after open");
    }

    #[test]
    fn test_insert_and_get_task() {
        let db = in_memory_db();
        db.conn.execute(
            "INSERT INTO sessions (id, user_id, status) VALUES (?1, ?2, ?3)",
            rusqlite::params!["sess-1", "user-1", "active"],
        ).unwrap();

        db.insert_task("task-1", "sess-1", "install docker").unwrap();

        let count: i64 = db.conn
            .query_row("SELECT COUNT(*) FROM tasks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_append_audit_log() {
        let db = in_memory_db();
        db.conn.execute(
            "INSERT INTO sessions (id, user_id, status) VALUES (?1, ?2, ?3)",
            rusqlite::params!["sess-1", "user-1", "active"],
        ).unwrap();

        db.append_audit_log("sess-1", "INSTALL", "docker", "LOW").unwrap();

        let count: i64 = db.conn
            .query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_session_and_end_session() {
        let db = in_memory_db();
        db.insert_session("sess-2", "user-2").unwrap();
        db.end_session("sess-2").unwrap();

        let status: String = db.conn
            .query_row("SELECT status FROM sessions WHERE id='sess-2'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(status, "completed");
    }
}
