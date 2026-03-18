/// Database connection manager for Day1 Copilot.
///
/// Opens or creates an SQLite database at `~/.day1copilot/data.db`,
/// enables WAL mode, and runs pending migrations on startup.

pub mod migrations;
pub mod schema;

#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

/// Thread-safe handle to the SQLite connection.
pub type DbHandle = Arc<Mutex<Connection>>;

/// Default database directory name under the user's home.
const DB_DIR: &str = ".day1copilot";
/// Default database file name.
const DB_FILE: &str = "data.db";

/// Resolve the database file path: `~/.day1copilot/data.db`.
fn db_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "could not determine home directory".to_string())?;
    Ok(home.join(DB_DIR).join(DB_FILE))
}

/// Open (or create) the database, enable WAL, and run migrations.
///
/// Returns a thread-safe handle suitable for use across async tasks.
pub fn init() -> Result<DbHandle, String> {
    let path = db_path()?;

    // Ensure the parent directory exists.
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create db directory: {e}"))?;
    }

    let conn =
        Connection::open(&path).map_err(|e| format!("failed to open database: {e}"))?;

    // Enable WAL mode for better concurrent read performance.
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| format!("failed to enable WAL mode: {e}"))?;

    // Run pending migrations.
    let applied = migrations::run_migrations(&conn)
        .map_err(|e| format!("migration failed: {e}"))?;

    if applied > 0 {
        tracing::info!(applied, "database migrations complete");
    }

    Ok(Arc::new(Mutex::new(conn)))
}

/// Open an in-memory database with migrations applied. Useful for testing.
#[cfg(test)]
pub fn init_memory() -> Result<DbHandle, String> {
    let conn =
        Connection::open_in_memory().map_err(|e| format!("failed to open in-memory db: {e}"))?;

    migrations::run_migrations(&conn).map_err(|e| format!("migration failed: {e}"))?;

    Ok(Arc::new(Mutex::new(conn)))
}
