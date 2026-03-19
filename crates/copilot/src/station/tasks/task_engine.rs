use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::task_types::{Artifact, CreateTaskRequest, TaskFilter, TaskSpec, TaskStatus};
use crate::station::db::DbHandle;

/// The Task Engine manages the lifecycle of tasks and their artifacts.
///
/// All operations are concurrency-safe via `RwLock`-guarded internal state.
/// State transitions are validated against the task FSM before being applied.
/// An optional `DbHandle` enables write-behind persistence to SQLite so that
/// tasks survive app restarts. The in-memory HashMap remains the primary store
/// during runtime.
pub struct TaskEngine {
    tasks: Arc<RwLock<HashMap<String, TaskSpec>>>,
    /// Artifacts grouped by task ID.
    artifacts: Arc<RwLock<HashMap<String, Vec<Artifact>>>>,
    /// Optional SQLite handle for write-behind persistence.
    db: Option<DbHandle>,
}

impl TaskEngine {
    /// Create a new, empty Task Engine.
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            artifacts: Arc::new(RwLock::new(HashMap::new())),
            db: None,
        }
    }

    /// Create a new Task Engine backed by an SQLite database.
    pub fn with_db(db: DbHandle) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            artifacts: Arc::new(RwLock::new(HashMap::new())),
            db: Some(db),
        }
    }

    /// Load non-completed tasks from the database into the in-memory store.
    ///
    /// Call this once at startup to restore pending/running/paused tasks from
    /// the previous session. Completed, failed, and cancelled tasks are not
    /// loaded to keep the working set small.
    pub async fn load_from_db(&self) -> Result<usize, String> {
        let db = match &self.db {
            Some(db) => db,
            None => return Ok(0),
        };

        // Collect all rows while holding the db mutex, then release it.
        let loaded_tasks: Vec<TaskSpec> = {
            let conn = db.lock().map_err(|e| format!("db lock failed: {e}"))?;
            let mut stmt = conn
                .prepare(
                    "SELECT id, title, status, agent_id, parent_id, step_index,
                            priority, input, output, created_at, started_at, completed_at
                     FROM tasks
                     WHERE status NOT IN ('completed', 'failed', 'cancelled')",
                )
                .map_err(|e| format!("prepare failed: {e}"))?;

            let rows = stmt
                .query_map([], |row| {
                    let status_str: String = row.get(2)?;
                    let status = match status_str.as_str() {
                        "pending" => TaskStatus::Pending,
                        "running" => TaskStatus::Running,
                        "paused" => TaskStatus::Paused,
                        _ => TaskStatus::Pending,
                    };

                    let input_str: Option<String> = row.get(7)?;
                    let input = input_str.and_then(|s| serde_json::from_str(&s).ok());

                    let output_str: Option<String> = row.get(8)?;
                    let output = output_str.and_then(|s| serde_json::from_str(&s).ok());

                    let created_str: String = row.get(9)?;
                    let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now());

                    let started_str: Option<String> = row.get(10)?;
                    let started_at = started_str.and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                    });

                    let completed_str: Option<String> = row.get(11)?;
                    let completed_at = completed_str.and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                    });

                    Ok(TaskSpec {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        status,
                        agent_id: row.get(3)?,
                        parent_id: row.get(4)?,
                        step_index: row.get::<_, Option<i32>>(5)?.map(|v| v as u32),
                        priority: row.get(6)?,
                        input,
                        output,
                        sub_tasks: Vec::new(),
                        started_at,
                        completed_at,
                        created_at,
                    })
                })
                .map_err(|e| format!("query failed: {e}"))?;

            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("row read failed: {e}"))?
        };

        let count = loaded_tasks.len();
        let mut tasks = self.tasks.write().await;
        for task in loaded_tasks {
            tasks.insert(task.id.clone(), task);
        }

        // Rebuild sub_tasks references from parent_id links.
        let parent_child: Vec<(String, String)> = tasks
            .values()
            .filter_map(|t| t.parent_id.as_ref().map(|pid| (pid.clone(), t.id.clone())))
            .collect();
        for (parent_id, child_id) in parent_child {
            if let Some(parent) = tasks.get_mut(&parent_id) {
                if !parent.sub_tasks.contains(&child_id) {
                    parent.sub_tasks.push(child_id);
                }
            }
        }

        Ok(count)
    }

    /// Persist a task to SQLite (write-behind). Errors are logged, not propagated.
    fn persist_task(&self, task: &TaskSpec) {
        let db = match &self.db {
            Some(db) => db,
            None => return,
        };

        let conn = match db.lock() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("failed to acquire db lock for task persist: {e}");
                return;
            }
        };

        let status = match task.status {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Paused => "paused",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
            TaskStatus::Cancelled => "cancelled",
        };

        let input = task.input.as_ref().map(|v| v.to_string());
        let output = task.output.as_ref().map(|v| v.to_string());
        let created_at = task.created_at.to_rfc3339();
        let started_at = task.started_at.map(|dt| dt.to_rfc3339());
        let completed_at = task.completed_at.map(|dt| dt.to_rfc3339());
        let step_index = task.step_index.map(|v| v as i32);

        let result = conn.execute(
            "INSERT INTO tasks (id, title, status, agent_id, parent_id, step_index,
                               priority, input, output, created_at, started_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                agent_id = excluded.agent_id,
                parent_id = excluded.parent_id,
                input = excluded.input,
                output = excluded.output,
                started_at = excluded.started_at,
                completed_at = excluded.completed_at",
            rusqlite::params![
                task.id,
                task.title,
                status,
                task.agent_id,
                task.parent_id,
                step_index,
                task.priority,
                input,
                output,
                created_at,
                started_at,
                completed_at,
            ],
        );

        if let Err(e) = result {
            tracing::warn!(task_id = %task.id, "failed to persist task: {e}");
        }
    }

    /// Create a new task from a description.
    pub async fn create(&self, request: CreateTaskRequest) -> TaskSpec {
        let task = TaskSpec {
            id: Uuid::new_v4().to_string(),
            title: request.description,
            status: TaskStatus::Pending,
            agent_id: None,
            parent_id: None,
            step_index: None,
            priority: request.priority.unwrap_or(0),
            input: None,
            output: None,
            sub_tasks: Vec::new(),
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
        };
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id.clone(), task.clone());
        drop(tasks);
        self.persist_task(&task);
        task
    }

    /// Start a task (transition Pending -> Running).
    pub async fn start(&self, task_id: &str) -> Result<TaskSpec, String> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| format!("task not found: {}", task_id))?;

        if task.status != TaskStatus::Pending {
            return Err(format!(
                "cannot start task in {:?} status (expected pending)",
                task.status
            ));
        }

        task.status = TaskStatus::Running;
        task.started_at = Some(Utc::now());
        let snapshot = task.clone();
        drop(tasks);
        self.persist_task(&snapshot);
        Ok(snapshot)
    }

    /// Pause a running task (transition Running -> Paused).
    pub async fn pause(&self, task_id: &str) -> Result<TaskSpec, String> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| format!("task not found: {}", task_id))?;

        if task.status != TaskStatus::Running {
            return Err(format!(
                "cannot pause task in {:?} status (expected running)",
                task.status
            ));
        }

        task.status = TaskStatus::Paused;
        let snapshot = task.clone();
        drop(tasks);
        self.persist_task(&snapshot);
        Ok(snapshot)
    }

    /// Cancel a task (valid from Pending or Running).
    pub async fn cancel(&self, task_id: &str) -> Result<TaskSpec, String> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| format!("task not found: {}", task_id))?;

        if task.status != TaskStatus::Pending && task.status != TaskStatus::Running {
            return Err(format!(
                "cannot cancel task in {:?} status (expected pending or running)",
                task.status
            ));
        }

        task.status = TaskStatus::Cancelled;
        task.completed_at = Some(Utc::now());
        let snapshot = task.clone();
        drop(tasks);
        self.persist_task(&snapshot);
        Ok(snapshot)
    }

    /// Complete a task with output (transition Running -> Completed).
    pub async fn complete(
        &self,
        task_id: &str,
        output: serde_json::Value,
    ) -> Result<TaskSpec, String> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| format!("task not found: {}", task_id))?;

        if task.status != TaskStatus::Running {
            return Err(format!(
                "cannot complete task in {:?} status (expected running)",
                task.status
            ));
        }

        task.status = TaskStatus::Completed;
        task.output = Some(output);
        task.completed_at = Some(Utc::now());
        let snapshot = task.clone();
        drop(tasks);
        self.persist_task(&snapshot);
        Ok(snapshot)
    }

    /// Fail a task with an error message (valid from Running or Pending).
    pub async fn fail(&self, task_id: &str, error: &str) -> Result<TaskSpec, String> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| format!("task not found: {}", task_id))?;

        if task.status != TaskStatus::Running && task.status != TaskStatus::Pending {
            return Err(format!(
                "cannot fail task in {:?} status (expected running or pending)",
                task.status
            ));
        }

        task.status = TaskStatus::Failed;
        task.output = Some(serde_json::json!({ "error": error }));
        task.completed_at = Some(Utc::now());
        let snapshot = task.clone();
        drop(tasks);
        self.persist_task(&snapshot);
        Ok(snapshot)
    }

    /// Get a task by ID.
    pub async fn status(&self, task_id: &str) -> Option<TaskSpec> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// List tasks with optional filters.
    pub async fn list(&self, filter: Option<TaskFilter>) -> Vec<TaskSpec> {
        let tasks = self.tasks.read().await;
        let mut result: Vec<TaskSpec> = tasks
            .values()
            .filter(|task| {
                if let Some(ref f) = filter {
                    if let Some(status) = f.status {
                        if task.status != status {
                            return false;
                        }
                    }
                    if let Some(ref agent_id) = f.agent_id {
                        if task.agent_id.as_deref() != Some(agent_id.as_str()) {
                            return false;
                        }
                    }
                    if let Some(ref parent_id) = f.parent_id {
                        if task.parent_id.as_deref() != Some(parent_id.as_str()) {
                            return false;
                        }
                    }
                }
                true
            })
            .cloned()
            .collect();
        // Sort by created_at for deterministic ordering.
        result.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        result
    }

    /// Add a sub-task to a parent task. Returns the sub-task ID.
    pub async fn add_subtask(&self, parent_id: &str, subtask: TaskSpec) -> Result<String, String> {
        let mut tasks = self.tasks.write().await;
        let parent = tasks
            .get_mut(parent_id)
            .ok_or_else(|| format!("parent task not found: {}", parent_id))?;

        let subtask_id = subtask.id.clone();
        parent.sub_tasks.push(subtask_id.clone());

        // Store the sub-task itself with parent_id set.
        let mut child = subtask;
        child.parent_id = Some(parent_id.to_string());
        let child_snapshot = child.clone();
        tasks.insert(child.id.clone(), child);
        drop(tasks);
        self.persist_task(&child_snapshot);

        Ok(subtask_id)
    }

    /// Get all sub-tasks of a parent task.
    pub async fn get_subtasks(&self, parent_id: &str) -> Vec<TaskSpec> {
        let tasks = self.tasks.read().await;
        let parent = match tasks.get(parent_id) {
            Some(p) => p,
            None => return Vec::new(),
        };

        let mut subtasks: Vec<TaskSpec> = parent
            .sub_tasks
            .iter()
            .filter_map(|id| tasks.get(id).cloned())
            .collect();
        subtasks.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        subtasks
    }

    /// Add an artifact to a task. Returns the artifact ID.
    pub async fn add_artifact(&self, artifact: Artifact) -> String {
        let id = artifact.id.clone();
        let task_id = artifact.task_id.clone();
        let mut artifacts = self.artifacts.write().await;
        artifacts.entry(task_id).or_default().push(artifact);
        id
    }

    /// Get all artifacts for a task.
    pub async fn get_artifacts(&self, task_id: &str) -> Vec<Artifact> {
        let artifacts = self.artifacts.read().await;
        artifacts.get(task_id).cloned().unwrap_or_default()
    }

    /// Get task count grouped by status.
    pub async fn count_by_status(&self) -> HashMap<TaskStatus, usize> {
        let tasks = self.tasks.read().await;
        let mut counts: HashMap<TaskStatus, usize> = HashMap::new();
        for task in tasks.values() {
            *counts.entry(task.status).or_insert(0) += 1;
        }
        counts
    }
}

impl Default for TaskEngine {
    fn default() -> Self {
        Self::new()
    }
}
