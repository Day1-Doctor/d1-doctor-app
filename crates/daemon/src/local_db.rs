//! Local SQLite database for daemon state persistence.
//! Schema: tasks, task_steps, daemon_log
//! Spec: LocalStack_v2.4.1_Spec.md §2.4

use anyhow::Result;
use rusqlite::{Connection, OptionalExtension};

pub struct LocalDb {
    pub(crate) conn: Connection,
}

#[derive(Debug)]
pub struct Task {
    pub task_id: String,
    pub input: String,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub result_json: Option<String>,
}

#[derive(Debug)]
pub struct TaskStep {
    pub step_id: String,
    pub task_id: String,
    pub step_order: i64,
    pub description: String,
    pub status: String,
    pub result_json: Option<String>,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
}

impl LocalDb {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS tasks (
                task_id     TEXT PRIMARY KEY,
                input       TEXT NOT NULL,
                status      TEXT NOT NULL DEFAULT 'submitted',
                created_at  INTEGER NOT NULL,
                updated_at  INTEGER NOT NULL,
                result_json TEXT
            );
            CREATE TABLE IF NOT EXISTS task_steps (
                step_id      TEXT PRIMARY KEY,
                task_id      TEXT NOT NULL REFERENCES tasks(task_id),
                step_order   INTEGER NOT NULL,
                description  TEXT NOT NULL,
                status       TEXT NOT NULL DEFAULT 'pending',
                result_json  TEXT,
                started_at   INTEGER,
                completed_at INTEGER
            );
            CREATE TABLE IF NOT EXISTS daemon_log (
                id    INTEGER PRIMARY KEY AUTOINCREMENT,
                level TEXT NOT NULL,
                msg   TEXT NOT NULL,
                ts    INTEGER NOT NULL
            );
        ")?;
        Ok(())
    }

    pub fn insert_task(&self, task_id: &str, input: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp_millis();
        self.conn.execute(
            "INSERT INTO tasks (task_id, input, status, created_at, updated_at) VALUES (?1, ?2, 'submitted', ?3, ?3)",
            rusqlite::params![task_id, input, now],
        )?;
        Ok(())
    }

    pub fn get_task(&self, task_id: &str) -> Result<Option<Task>> {
        self.conn.query_row(
            "SELECT task_id, input, status, created_at, updated_at, result_json FROM tasks WHERE task_id = ?1",
            [task_id],
            |row| Ok(Task {
                task_id: row.get(0)?,
                input: row.get(1)?,
                status: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                result_json: row.get(5)?,
            }),
        ).optional().map_err(Into::into)
    }

    pub fn update_task_status(&self, task_id: &str, status: &str, result_json: Option<&str>) -> Result<()> {
        let now = chrono::Utc::now().timestamp_millis();
        self.conn.execute(
            "UPDATE tasks SET status = ?1, result_json = ?2, updated_at = ?3 WHERE task_id = ?4",
            rusqlite::params![status, result_json, now, task_id],
        )?;
        Ok(())
    }

    pub fn list_tasks_recent(&self, limit: u32) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT task_id, input, status, created_at, updated_at, result_json FROM tasks ORDER BY created_at DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map([limit], |row| Ok(Task {
            task_id: row.get(0)?,
            input: row.get(1)?,
            status: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
            result_json: row.get(5)?,
        }))?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }

    pub fn insert_step(&self, step_id: &str, task_id: &str, order: i64, description: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO task_steps (step_id, task_id, step_order, description, status) VALUES (?1, ?2, ?3, ?4, 'pending')",
            rusqlite::params![step_id, task_id, order, description],
        )?;
        Ok(())
    }

    pub fn update_step_status(&self, step_id: &str, status: &str, result_json: Option<&str>) -> Result<()> {
        let now = chrono::Utc::now().timestamp_millis();
        self.conn.execute(
            "UPDATE task_steps SET status = ?1, result_json = ?2, completed_at = ?3 WHERE step_id = ?4",
            rusqlite::params![status, result_json, now, step_id],
        )?;
        Ok(())
    }

    pub fn get_steps(&self, task_id: &str) -> Result<Vec<TaskStep>> {
        let mut stmt = self.conn.prepare(
            "SELECT step_id, task_id, step_order, description, status, result_json, started_at, completed_at FROM task_steps WHERE task_id = ?1 ORDER BY step_order"
        )?;
        let rows = stmt.query_map([task_id], |row| Ok(TaskStep {
            step_id: row.get(0)?,
            task_id: row.get(1)?,
            step_order: row.get(2)?,
            description: row.get(3)?,
            status: row.get(4)?,
            result_json: row.get(5)?,
            started_at: row.get(6)?,
            completed_at: row.get(7)?,
        }))?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }

    pub fn append_log(&self, level: &str, msg: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp_millis();
        self.conn.execute(
            "INSERT INTO daemon_log (level, msg, ts) VALUES (?1, ?2, ?3)",
            rusqlite::params![level, msg, now],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> LocalDb {
        LocalDb::open(":memory:").unwrap()
    }

    #[test]
    fn test_schema_created() {
        let db = test_db();
        let count: i64 = db.conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='tasks'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_and_get_task() {
        let db = test_db();
        db.insert_task("tsk_001", "install openclaw").unwrap();
        let task = db.get_task("tsk_001").unwrap().unwrap();
        assert_eq!(task.task_id, "tsk_001");
        assert_eq!(task.input, "install openclaw");
        assert_eq!(task.status, "submitted");
    }

    #[test]
    fn test_update_task_status() {
        let db = test_db();
        db.insert_task("tsk_002", "setup rust").unwrap();
        db.update_task_status("tsk_002", "executing", None).unwrap();
        let task = db.get_task("tsk_002").unwrap().unwrap();
        assert_eq!(task.status, "executing");
    }

    #[test]
    fn test_insert_step() {
        let db = test_db();
        db.insert_task("tsk_003", "configure git").unwrap();
        db.insert_step("step_001", "tsk_003", 1, "Check git installed").unwrap();
        let steps = db.get_steps("tsk_003").unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].description, "Check git installed");
    }

    #[test]
    fn test_list_tasks_recent() {
        let db = test_db();
        db.insert_task("tsk_a", "task a").unwrap();
        db.insert_task("tsk_b", "task b").unwrap();
        let tasks = db.list_tasks_recent(10).unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn test_get_nonexistent_task_returns_none() {
        let db = test_db();
        let task = db.get_task("nonexistent").unwrap();
        assert!(task.is_none());
    }

    #[test]
    fn test_daemon_log_append() {
        let db = test_db();
        db.append_log("info", "daemon started").unwrap();
        let count: i64 = db.conn.query_row(
            "SELECT count(*) FROM daemon_log",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }
}
