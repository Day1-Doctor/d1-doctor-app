//! Tauri commands for reading task history from the local daemon SQLite database.
//!
//! Database: ~/.d1doctor/d1doctor.db  (created and owned by the daemon)
//! Schema:   tasks(id, session_id, request, status, created_at)
//! Access:   read-only, WAL mode compatible concurrent reads

use rusqlite::{Connection, OpenFlags};
use serde::Serialize;

/// A minimal task summary returned to the frontend.
#[derive(Debug, Serialize)]
pub struct TaskSummary {
    pub id: String,
    pub title: String,
    pub status: String, // "pending" | "running" | "completed" | "failed"
    pub created_at: i64, // Unix timestamp (seconds)
}

fn db_path() -> Result<std::path::PathBuf, String> {
    let home = dirs::home_dir()
        .ok_or_else(|| "Cannot determine home directory".to_string())?;
    Ok(home.join(".d1doctor").join("d1doctor.db"))
}

/// Return up to 20 most recent tasks from the daemon's local SQLite database.
///
/// Returns an empty array when:
/// - The database file doesn't exist (fresh install, daemon never run)
/// - The database is locked/busy (daemon is writing; 500 ms timeout)
/// - Any other error (logged, not propagated to UI)
#[tauri::command]
pub async fn list_recent_tasks() -> Vec<TaskSummary> {
    let path = match db_path() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[tasks] Failed to locate db: {e}");
            return vec![];
        }
    };

    if !path.exists() {
        return vec![]; // Fresh install — daemon hasn't created DB yet
    }

    // Open read-only; bundled SQLite handles WAL concurrent access
    let conn = match Connection::open_with_flags(
        &path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[tasks] Failed to open db: {e}");
            return vec![];
        }
    };

    // 500 ms busy timeout so we don't block the UI if daemon is mid-write
    if let Err(e) = conn.busy_timeout(std::time::Duration::from_millis(500)) {
        eprintln!("[tasks] Failed to set busy timeout: {e}");
        return vec![];
    }

    let mut stmt = match conn.prepare(
        "SELECT id, request, status, created_at \
         FROM tasks \
         ORDER BY created_at DESC \
         LIMIT 20",
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[tasks] Failed to prepare query: {e}");
            return vec![];
        }
    };

    let rows = stmt.query_map([], |row| {
        Ok(TaskSummary {
            id: row.get(0)?,
            title: row.get(1)?,
            status: row.get(2)?,
            created_at: row.get(3)?,
        })
    });

    match rows {
        Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            eprintln!("[tasks] Query failed: {e}");
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_path_contains_d1doctor() {
        let path = db_path().unwrap();
        assert!(path.to_string_lossy().contains(".d1doctor"));
        assert_eq!(path.file_name().unwrap(), "d1doctor.db");
    }

    #[tokio::test]
    async fn list_recent_tasks_never_panics() {
        // The function must not panic even if DB doesn't exist or has wrong schema.
        // Result can be empty Vec or populated Vec.
        let result = list_recent_tasks().await;
        assert!(result.len() <= 20);
    }
}
