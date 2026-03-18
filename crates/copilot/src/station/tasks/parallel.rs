use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tokio::sync::RwLock;

use super::task_types::{TaskSpec, TaskStatus};

/// Describes a sub-task together with its assigned agent and dependency list.
#[derive(Debug, Clone)]
pub struct ParallelTask {
    pub spec: TaskSpec,
    pub agent_id: String,
    /// IDs of tasks that must complete before this one starts.
    pub depends_on: Vec<String>,
}

/// Event emitted as parallel tasks complete.
#[derive(Debug, Clone)]
pub enum ParallelEvent {
    TaskStarted { task_id: String, agent_id: String },
    TaskCompleted { task_id: String },
    TaskFailed { task_id: String, error: String },
}

/// Executes a set of sub-tasks concurrently, respecting dependency edges.
///
/// Independent tasks are spawned immediately via `tokio::spawn`. Tasks with
/// `depends_on` wait until all their dependencies have completed successfully
/// before being dispatched.
pub struct ParallelExecutor {
    events: Arc<RwLock<Vec<ParallelEvent>>>,
}

impl ParallelExecutor {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Execute tasks in parallel, respecting dependency ordering.
    ///
    /// Each `ParallelTask` carries a `TaskSpec`, an `agent_id`, and a list of
    /// task IDs it depends on. Independent tasks run concurrently; dependent
    /// tasks wait for their prerequisites to finish.
    ///
    /// Returns a `Vec<Result<TaskSpec, String>>` in the same order as the input.
    pub async fn execute_parallel(
        &self,
        tasks: Vec<ParallelTask>,
    ) -> Vec<Result<TaskSpec, String>> {
        if tasks.is_empty() {
            return Vec::new();
        }

        let task_count = tasks.len();

        // Build index: task_id -> position in input vec.
        let _id_to_index: HashMap<String, usize> = tasks
            .iter()
            .enumerate()
            .map(|(i, t)| (t.spec.id.clone(), i))
            .collect();

        // Shared state for completed task IDs.
        let completed: Arc<RwLock<HashSet<String>>> = Arc::new(RwLock::new(HashSet::new()));
        let failed: Arc<RwLock<HashSet<String>>> = Arc::new(RwLock::new(HashSet::new()));

        // Channel for collecting results.
        let (result_tx, mut result_rx) =
            tokio::sync::mpsc::channel::<(usize, Result<TaskSpec, String>)>(task_count);

        // Notify channel: each completed task broadcasts its ID so waiters wake up.
        let (notify_tx, _) = tokio::sync::broadcast::channel::<String>(task_count * 2);

        let events = Arc::clone(&self.events);

        for (idx, ptask) in tasks.into_iter().enumerate() {
            let completed = Arc::clone(&completed);
            let failed = Arc::clone(&failed);
            let result_tx = result_tx.clone();
            let notify_tx = notify_tx.clone();
            let mut notify_rx = notify_tx.subscribe();
            let events = Arc::clone(&events);

            tokio::spawn(async move {
                // Wait for dependencies.
                if !ptask.depends_on.is_empty() {
                    loop {
                        // Check if all dependencies are satisfied.
                        let comp = completed.read().await;
                        let fail = failed.read().await;

                        // If any dependency failed, this task fails too.
                        let dep_failed: Vec<String> = ptask
                            .depends_on
                            .iter()
                            .filter(|d| fail.contains(d.as_str()))
                            .cloned()
                            .collect();
                        if !dep_failed.is_empty() {
                            drop(comp);
                            drop(fail);
                            let err =
                                format!("dependency failed: {}", dep_failed.join(", "));
                            events.write().await.push(ParallelEvent::TaskFailed {
                                task_id: ptask.spec.id.clone(),
                                error: err.clone(),
                            });
                            let _ = result_tx.send((idx, Err(err))).await;
                            let _ = notify_tx.send(ptask.spec.id.clone());
                            return;
                        }

                        let all_done = ptask.depends_on.iter().all(|d| comp.contains(d.as_str()));
                        drop(comp);
                        drop(fail);

                        if all_done {
                            break;
                        }

                        // Wait for a notification that some task completed.
                        let _ = notify_rx.recv().await;
                    }
                }

                // Emit started event.
                events.write().await.push(ParallelEvent::TaskStarted {
                    task_id: ptask.spec.id.clone(),
                    agent_id: ptask.agent_id.clone(),
                });

                // Simulate agent FSM execution.
                // In production this would delegate to the actual agent runtime.
                let result = run_agent_task(ptask.spec.clone(), &ptask.agent_id).await;

                match &result {
                    Ok(_) => {
                        completed.write().await.insert(ptask.spec.id.clone());
                        events.write().await.push(ParallelEvent::TaskCompleted {
                            task_id: ptask.spec.id.clone(),
                        });
                    }
                    Err(e) => {
                        failed.write().await.insert(ptask.spec.id.clone());
                        events.write().await.push(ParallelEvent::TaskFailed {
                            task_id: ptask.spec.id.clone(),
                            error: e.clone(),
                        });
                    }
                }

                let _ = result_tx.send((idx, result)).await;
                let _ = notify_tx.send(ptask.spec.id.clone());
            });
        }

        // Drop the original sender so the channel closes when all spawned tasks finish.
        drop(result_tx);

        // Collect all results.
        let mut results: Vec<Option<Result<TaskSpec, String>>> = vec![None; task_count];
        while let Some((idx, result)) = result_rx.recv().await {
            results[idx] = Some(result);
        }

        // Unwrap Options — all slots should be filled.
        results
            .into_iter()
            .enumerate()
            .map(|(i, r)| {
                r.unwrap_or_else(|| {
                    Err(format!("task at index {} did not produce a result", i))
                })
            })
            .collect()
    }

    /// Retrieve all events emitted during execution.
    pub async fn get_events(&self) -> Vec<ParallelEvent> {
        self.events.read().await.clone()
    }
}

impl Default for ParallelExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Simulate running a task with an agent. In the real implementation this
/// would invoke the agent FSM via the appropriate `FrameworkAdapter`.
async fn run_agent_task(mut spec: TaskSpec, _agent_id: &str) -> Result<TaskSpec, String> {
    spec.status = TaskStatus::Running;
    spec.started_at = Some(chrono::Utc::now());

    // Simulate a small amount of work.
    tokio::task::yield_now().await;

    spec.status = TaskStatus::Completed;
    spec.completed_at = Some(chrono::Utc::now());
    spec.output = Some(serde_json::json!({ "status": "done" }));
    Ok(spec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::station::tasks::task_types::TaskSpec;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_task(title: &str) -> TaskSpec {
        TaskSpec {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            status: TaskStatus::Pending,
            agent_id: None,
            parent_id: None,
            step_index: None,
            priority: 0,
            input: None,
            output: None,
            sub_tasks: Vec::new(),
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_two_independent_tasks_run_concurrently() {
        let executor = ParallelExecutor::new();

        let task_a = make_task("Task A");
        let task_b = make_task("Task B");

        let tasks = vec![
            ParallelTask {
                spec: task_a.clone(),
                agent_id: "agent-1".to_string(),
                depends_on: vec![],
            },
            ParallelTask {
                spec: task_b.clone(),
                agent_id: "agent-2".to_string(),
                depends_on: vec![],
            },
        ];

        let results = executor.execute_parallel(tasks).await;

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());

        let r0 = results[0].as_ref().unwrap();
        let r1 = results[1].as_ref().unwrap();
        assert_eq!(r0.status, TaskStatus::Completed);
        assert_eq!(r1.status, TaskStatus::Completed);

        // Both tasks should have started events.
        let events = executor.get_events().await;
        let started_count = events
            .iter()
            .filter(|e| matches!(e, ParallelEvent::TaskStarted { .. }))
            .count();
        assert_eq!(started_count, 2);
    }

    #[tokio::test]
    async fn test_dependent_task_waits_for_prerequisite() {
        let executor = ParallelExecutor::new();

        let task_a = make_task("Prerequisite");
        let task_b = make_task("Dependent");

        let a_id = task_a.id.clone();

        let tasks = vec![
            ParallelTask {
                spec: task_a,
                agent_id: "agent-1".to_string(),
                depends_on: vec![],
            },
            ParallelTask {
                spec: task_b,
                agent_id: "agent-2".to_string(),
                depends_on: vec![a_id.clone()],
            },
        ];

        let results = executor.execute_parallel(tasks).await;

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());

        // Verify ordering via events: task_a started before task_b.
        let events = executor.get_events().await;
        let started_events: Vec<&ParallelEvent> = events
            .iter()
            .filter(|e| matches!(e, ParallelEvent::TaskStarted { .. }))
            .collect();
        assert_eq!(started_events.len(), 2);

        // First started event should be for the prerequisite.
        match &started_events[0] {
            ParallelEvent::TaskStarted { task_id, .. } => {
                assert_eq!(task_id, &a_id);
            }
            _ => panic!("expected TaskStarted"),
        }
    }

    #[tokio::test]
    async fn test_all_results_collected() {
        let executor = ParallelExecutor::new();

        let tasks: Vec<ParallelTask> = (0..5)
            .map(|i| ParallelTask {
                spec: make_task(&format!("Task {}", i)),
                agent_id: format!("agent-{}", i),
                depends_on: vec![],
            })
            .collect();

        let results = executor.execute_parallel(tasks).await;
        assert_eq!(results.len(), 5);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[tokio::test]
    async fn test_empty_task_list() {
        let executor = ParallelExecutor::new();
        let results = executor.execute_parallel(vec![]).await;
        assert!(results.is_empty());
    }
}
