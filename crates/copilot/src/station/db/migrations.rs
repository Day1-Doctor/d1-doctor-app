/// Migration runner for the Day1 Copilot SQLite database.
///
/// Tracks applied migrations via a `schema_version` table and applies
/// pending migrations in order.
use rusqlite::Connection;

use super::schema;

/// SQL to bootstrap the migration tracking table.
const CREATE_SCHEMA_VERSION: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    version     INTEGER PRIMARY KEY,
    name        TEXT NOT NULL,
    applied_at  TEXT NOT NULL
);
"#;

/// A migration definition.
struct Migration {
    version: i64,
    name: &'static str,
    statements: &'static [&'static str],
}

/// All registered migrations, in order.
const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "v3_0_agent_tables",
    statements: schema::V1_STATEMENTS,
}];

/// Ensure the schema_version table exists.
fn ensure_version_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(CREATE_SCHEMA_VERSION)
}

/// Return the highest applied migration version, or 0 if none.
fn current_version(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    )
}

/// Run all pending migrations inside a transaction.
///
/// Returns the number of migrations applied.
pub fn run_migrations(conn: &Connection) -> rusqlite::Result<usize> {
    ensure_version_table(conn)?;
    let current = current_version(conn)?;
    let mut applied = 0;

    for migration in MIGRATIONS {
        if migration.version <= current {
            continue;
        }

        let tx = conn.unchecked_transaction()?;

        // Execute each statement in the migration.
        for sql in migration.statements {
            tx.execute_batch(sql)?;
        }

        // Record that this migration was applied.
        let now = chrono::Utc::now().to_rfc3339();
        tx.execute(
            "INSERT INTO schema_version (version, name, applied_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![migration.version, migration.name, now],
        )?;

        tx.commit()?;
        applied += 1;

        tracing::info!(
            version = migration.version,
            name = migration.name,
            "applied migration"
        );
    }

    Ok(applied)
}
