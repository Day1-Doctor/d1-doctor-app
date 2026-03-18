use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::task_types::{Artifact, CreateTaskRequest, TaskFilter, TaskSpec, TaskStatus};

/// The Task Engine manages the lifecycle of tasks and their artifacts.
///
/// All operations are concurrency-safe via `RwLock`-guarded internal state.
/// State transitions are validated against the task FSM before being applied.
pub struct TaskEngine {
    tasks: Arc<RwLock<HashMap<String, TaskSpec>>>,
    /// Artifacts grouped by task ID.
    artifacts: Arc<RwLock<HashMap<String, Vec<Artifact>>>>,
}

impl TaskEngine {
    /// Create a new, empty Task Engine.
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            artifacts: Arc::new(RwLock::new(HashMap::new())),
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
        Ok(task.clone())
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
        Ok(task.clone())
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
        Ok(task.clone())
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
        Ok(task.clone())
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
        Ok(task.clone())
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
        tasks.insert(child.id.clone(), child);

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
