/// SQL schema definitions for Day1 Copilot v3.0 tables.

/// Agent registration and state.
pub const CREATE_AGENTS: &str = r#"
CREATE TABLE IF NOT EXISTS agents (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    role        TEXT NOT NULL,
    framework   TEXT NOT NULL,
    adapter_config TEXT,
    status      TEXT NOT NULL DEFAULT 'idle',
    trust_score REAL DEFAULT 0.5,
    sprite_id   TEXT,
    room        TEXT DEFAULT 'main',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
"#;

/// Task management.
pub const CREATE_TASKS: &str = r#"
CREATE TABLE IF NOT EXISTS tasks (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'pending',
    agent_id    TEXT REFERENCES agents(id),
    parent_id   TEXT REFERENCES tasks(id),
    step_index  INTEGER,
    priority    INTEGER DEFAULT 0,
    input       TEXT,
    output      TEXT,
    started_at  TEXT,
    completed_at TEXT,
    created_at  TEXT NOT NULL
);
"#;

/// Artifacts produced by tasks.
pub const CREATE_ARTIFACTS: &str = r#"
CREATE TABLE IF NOT EXISTS artifacts (
    id          TEXT PRIMARY KEY,
    task_id     TEXT NOT NULL REFERENCES tasks(id),
    agent_id    TEXT NOT NULL REFERENCES agents(id),
    type        TEXT NOT NULL,
    name        TEXT NOT NULL,
    path        TEXT NOT NULL,
    mime_type   TEXT,
    size_bytes  INTEGER,
    created_at  TEXT NOT NULL
);
"#;

/// Tool execution audit log.
pub const CREATE_TOOL_EXECUTIONS: &str = r#"
CREATE TABLE IF NOT EXISTS tool_executions (
    id          TEXT PRIMARY KEY,
    agent_id    TEXT NOT NULL REFERENCES agents(id),
    task_id     TEXT REFERENCES tasks(id),
    tool_name   TEXT NOT NULL,
    params      TEXT,
    result      TEXT,
    risk_level  TEXT NOT NULL,
    approved_by TEXT,
    duration_ms INTEGER,
    status      TEXT NOT NULL,
    created_at  TEXT NOT NULL
);
"#;

/// Session cost tracking.
pub const CREATE_SESSION_COSTS: &str = r#"
CREATE TABLE IF NOT EXISTS session_costs (
    id          TEXT PRIMARY KEY,
    agent_id    TEXT NOT NULL REFERENCES agents(id),
    task_id     TEXT REFERENCES tasks(id),
    provider    TEXT NOT NULL,
    model       TEXT NOT NULL,
    tokens_in   INTEGER NOT NULL DEFAULT 0,
    tokens_out  INTEGER NOT NULL DEFAULT 0,
    cost_usd    REAL NOT NULL DEFAULT 0,
    cost_dd     REAL NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL
);
"#;

/// Indexes for query performance.
pub const CREATE_INDEXES: &str = r#"
CREATE INDEX IF NOT EXISTS idx_tasks_agent ON tasks(agent_id);
CREATE INDEX IF NOT EXISTS idx_tasks_parent ON tasks(parent_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_artifacts_task ON artifacts(task_id);
CREATE INDEX IF NOT EXISTS idx_tool_exec_agent ON tool_executions(agent_id);
CREATE INDEX IF NOT EXISTS idx_session_costs_agent ON session_costs(agent_id);
"#;

/// All V1 migration statements in order.
pub const V1_STATEMENTS: &[&str] = &[
    CREATE_AGENTS,
    CREATE_TASKS,
    CREATE_ARTIFACTS,
    CREATE_TOOL_EXECUTIONS,
    CREATE_SESSION_COSTS,
    CREATE_INDEXES,
];

// ---------------------------------------------------------------------------
// V2 — LLM call audit trail (D1D-265)
// ---------------------------------------------------------------------------

/// LLM call audit log for cost tracking and debugging.
pub const CREATE_LLM_CALLS: &str = r#"
CREATE TABLE IF NOT EXISTS llm_calls (
    id              TEXT PRIMARY KEY,
    agent_id        TEXT NOT NULL,
    task_id         TEXT,
    model           TEXT NOT NULL,
    prompt_tokens   INTEGER,
    completion_tokens INTEGER,
    cost_dd         REAL,
    created_at      TEXT NOT NULL
);
"#;

/// Indexes for the llm_calls table.
pub const CREATE_LLM_CALLS_INDEXES: &str = r#"
CREATE INDEX IF NOT EXISTS idx_llm_calls_agent ON llm_calls(agent_id);
CREATE INDEX IF NOT EXISTS idx_llm_calls_task ON llm_calls(task_id);
"#;

/// All V2 migration statements in order.
pub const V2_STATEMENTS: &[&str] = &[CREATE_LLM_CALLS, CREATE_LLM_CALLS_INDEXES];
