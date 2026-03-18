use super::*;
use rusqlite::params;
use uuid::Uuid;

/// Helper: open a fresh in-memory DB with migrations applied.
fn setup() -> DbHandle {
    init_memory().expect("failed to init in-memory db")
}

/// Helper: return current ISO 8601 timestamp.
fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

// ---------------------------------------------------------------------------
// Migration tests
// ---------------------------------------------------------------------------

#[test]
fn migration_creates_all_tables() {
    let db = setup();
    let conn = db.lock().unwrap();

    let expected_tables = [
        "schema_version",
        "agents",
        "tasks",
        "artifacts",
        "tool_executions",
        "session_costs",
    ];

    for table in &expected_tables {
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name=?1",
                params![table],
                |row| row.get(0),
            )
            .unwrap_or(false);

        assert!(exists, "table '{}' should exist after migration", table);
    }
}

#[test]
fn migration_is_idempotent() {
    let conn = rusqlite::Connection::open_in_memory().expect("failed to open in-memory db");

    // Run migrations twice — should not error.
    let first = migrations::run_migrations(&conn).expect("first migration failed");
    let second = migrations::run_migrations(&conn).expect("second migration failed");

    assert_eq!(first, 1, "first run should apply 1 migration");
    assert_eq!(second, 0, "second run should apply 0 migrations");

    // Verify version was recorded.
    let version: i64 = conn
        .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
            row.get(0)
        })
        .expect("failed to query schema_version");
    assert_eq!(version, 1);
}

// ---------------------------------------------------------------------------
// CRUD tests — insert and query each table
// ---------------------------------------------------------------------------

#[test]
fn crud_agents() {
    let db = setup();
    let conn = db.lock().unwrap();

    let id = Uuid::new_v4().to_string();
    let ts = now();

    conn.execute(
        "INSERT INTO agents (id, name, role, framework, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, "Dr. Bob", "orchestrator", "builtin", "idle", ts, ts],
    )
    .expect("insert agent");

    let name: String = conn
        .query_row(
            "SELECT name FROM agents WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .expect("query agent");

    assert_eq!(name, "Dr. Bob");
}

#[test]
fn crud_tasks() {
    let db = setup();
    let conn = db.lock().unwrap();

    // Insert prerequisite agent.
    let agent_id = Uuid::new_v4().to_string();
    let ts = now();
    conn.execute(
        "INSERT INTO agents (id, name, role, framework, created_at, updated_at)
         VALUES (?1, 'a', 'coder', 'generic', ?2, ?2)",
        params![agent_id, ts],
    )
    .unwrap();

    let task_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO tasks (id, title, status, agent_id, priority, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![task_id, "Write tests", "pending", agent_id, 1, ts],
    )
    .expect("insert task");

    let title: String = conn
        .query_row(
            "SELECT title FROM tasks WHERE id = ?1",
            params![task_id],
            |row| row.get(0),
        )
        .expect("query task");

    assert_eq!(title, "Write tests");
}

#[test]
fn crud_artifacts() {
    let db = setup();
    let conn = db.lock().unwrap();

    let agent_id = Uuid::new_v4().to_string();
    let task_id = Uuid::new_v4().to_string();
    let artifact_id = Uuid::new_v4().to_string();
    let ts = now();

    conn.execute(
        "INSERT INTO agents (id, name, role, framework, created_at, updated_at)
         VALUES (?1, 'a', 'writer', 'generic', ?2, ?2)",
        params![agent_id, ts],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO tasks (id, title, agent_id, created_at)
         VALUES (?1, 't', ?2, ?3)",
        params![task_id, agent_id, ts],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO artifacts (id, task_id, agent_id, type, name, path, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            artifact_id,
            task_id,
            agent_id,
            "document",
            "report.md",
            "/tmp/report.md",
            ts
        ],
    )
    .expect("insert artifact");

    let atype: String = conn
        .query_row(
            "SELECT type FROM artifacts WHERE id = ?1",
            params![artifact_id],
            |row| row.get(0),
        )
        .expect("query artifact");

    assert_eq!(atype, "document");
}

#[test]
fn crud_tool_executions() {
    let db = setup();
    let conn = db.lock().unwrap();

    let agent_id = Uuid::new_v4().to_string();
    let exec_id = Uuid::new_v4().to_string();
    let ts = now();

    conn.execute(
        "INSERT INTO agents (id, name, role, framework, created_at, updated_at)
         VALUES (?1, 'a', 'operator', 'generic', ?2, ?2)",
        params![agent_id, ts],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO tool_executions (id, agent_id, tool_name, risk_level, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![exec_id, agent_id, "shell_exec", "high", "success", ts],
    )
    .expect("insert tool_execution");

    let tool: String = conn
        .query_row(
            "SELECT tool_name FROM tool_executions WHERE id = ?1",
            params![exec_id],
            |row| row.get(0),
        )
        .expect("query tool_execution");

    assert_eq!(tool, "shell_exec");
}

#[test]
fn crud_session_costs() {
    let db = setup();
    let conn = db.lock().unwrap();

    let agent_id = Uuid::new_v4().to_string();
    let cost_id = Uuid::new_v4().to_string();
    let ts = now();

    conn.execute(
        "INSERT INTO agents (id, name, role, framework, created_at, updated_at)
         VALUES (?1, 'a', 'analyst', 'claude_sdk', ?2, ?2)",
        params![agent_id, ts],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO session_costs (id, agent_id, provider, model, tokens_in, tokens_out, cost_usd, cost_dd, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![cost_id, agent_id, "anthropic", "claude-opus-4-6", 1000, 500, 0.045, 4.5, ts],
    )
    .expect("insert session_cost");

    let model: String = conn
        .query_row(
            "SELECT model FROM session_costs WHERE id = ?1",
            params![cost_id],
            |row| row.get(0),
        )
        .expect("query session_cost");

    assert_eq!(model, "claude-opus-4-6");
}

#[test]
fn indexes_are_created() {
    let db = setup();
    let conn = db.lock().unwrap();

    let expected_indexes = [
        "idx_tasks_agent",
        "idx_tasks_parent",
        "idx_tasks_status",
        "idx_artifacts_task",
        "idx_tool_exec_agent",
        "idx_session_costs_agent",
    ];

    for idx in &expected_indexes {
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='index' AND name=?1",
                params![idx],
                |row| row.get(0),
            )
            .unwrap_or(false);

        assert!(exists, "index '{}' should exist after migration", idx);
    }
}
