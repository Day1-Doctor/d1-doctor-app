use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A task managed by the Task Engine.
///
/// Tasks represent discrete units of work assigned to agents. They follow
/// a state-machine lifecycle: Pending -> Running -> Completed/Failed/Cancelled,
/// with an optional Paused state while Running.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub agent_id: Option<String>,
    pub parent_id: Option<String>,
    pub step_index: Option<u32>,
    pub priority: i32,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
    /// IDs of child tasks.
    pub sub_tasks: Vec<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl TaskSpec {
    /// Create a new sub-task with defaults, suitable for use with `TaskEngine::add_subtask`.
    pub fn new_subtask(title: &str, parent_id: &str, step_index: u32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            status: TaskStatus::Pending,
            agent_id: None,
            parent_id: Some(parent_id.to_string()),
            step_index: Some(step_index),
            priority: 0,
            input: None,
            output: None,
            sub_tasks: Vec::new(),
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
        }
    }
}

/// The lifecycle status of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// An artifact produced by an agent during task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub task_id: String,
    pub agent_id: String,
    /// One of: document, code, chart, image, data, log.
    pub artifact_type: String,
    pub name: String,
    pub path: String,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub created_at: DateTime<Utc>,
}

/// Request payload for creating a new task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    /// Natural language task description.
    pub description: String,
    pub priority: Option<i32>,
}

/// Filter criteria for listing tasks.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub agent_id: Option<String>,
    pub parent_id: Option<String>,
}
