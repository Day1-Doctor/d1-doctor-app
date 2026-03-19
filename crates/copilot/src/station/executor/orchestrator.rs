use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::station::events::bus::EventBus;
use crate::station::events::event_types::{AgentEvent, EventType};
use crate::station::kernel::kernel::AgentKernel;
use crate::station::tasks::handoff::TaskHandoffManager;
use crate::station::tasks::task_engine::TaskEngine;

use super::agent_executor::AgentExecutor;
use super::step_runner::StepRunner;

/// Orchestrates the full task execution pipeline: execute steps sequentially
/// via the [`StepRunner`], hand off outputs between steps via the
/// [`TaskHandoffManager`], and mark the parent task complete or failed.
///
/// The orchestrator expects that a parent task has already been decomposed
/// and routed (subtasks exist with `agent_id` assignments) before
/// [`orchestrate`] or [`orchestrate_parallel`] is called.
pub struct TaskOrchestrator {
    /// Retained for direct `execute_step` calls in future extensions.
    #[allow(dead_code)]
    executor: Arc<AgentExecutor>,
    step_runner: Arc<StepRunner>,
    task_engine: Arc<TaskEngine>,
    kernel: Arc<AgentKernel>,
    event_bus: Arc<EventBus>,
    handoff_manager: Arc<TaskHandoffManager>,
}

impl TaskOrchestrator {
    /// Create a new orchestrator wired to all station subsystems.
    pub fn new(
        executor: Arc<AgentExecutor>,
        step_runner: Arc<StepRunner>,
        task_engine: Arc<TaskEngine>,
        kernel: Arc<AgentKernel>,
        event_bus: Arc<EventBus>,
        handoff_manager: Arc<TaskHandoffManager>,
    ) -> Self {
        Self {
            executor,
            step_runner,
            task_engine,
            kernel,
            event_bus,
            handoff_manager,
        }
    }

    /// Orchestrate sequential execution of all subtasks under `parent_task_id`.
    ///
    /// Given a parent task that has already been decomposed and routed:
    /// 1. Get all subtasks from the task engine, sorted by `step_index`.
    /// 2. For each subtask (sequentially):
    ///    a. Find the assigned agent from the kernel.
    ///    b. Start the subtask via the task engine.
    ///    c. Execute via `step_runner.run_step()`.
    ///    d. On success: call `handoff_manager.on_step_completed()` to chain output.
    ///    e. On failure: mark the subtask as Failed, emit an error event,
    ///       and mark the parent as Failed.
    /// 3. After all subtasks complete the parent is marked complete by
    ///    the handoff manager.
    pub async fn orchestrate(&self, parent_task_id: &str) -> Result<(), String> {
        let mut subtasks = self.task_engine.get_subtasks(parent_task_id).await;
        if subtasks.is_empty() {
            return Err(format!(
                "no subtasks found for parent {}",
                parent_task_id
            ));
        }

        // Sort by step_index ascending.
        subtasks.sort_by_key(|t| t.step_index.unwrap_or(u32::MAX));

        for subtask in &subtasks {
            let step_index = subtask.step_index.unwrap_or(0);

            // Find the assigned agent.
            let agent = match &subtask.agent_id {
                Some(agent_id) => match self.kernel.get_agent(agent_id).await {
                    Some(a) => a,
                    None => {
                        let err = format!("agent {} not found for step {}", agent_id, step_index);
                        self.fail_step_and_parent(parent_task_id, &subtask.id, step_index, &err)
                            .await?;
                        return Err(err);
                    }
                },
                None => {
                    let err = format!("no agent assigned to step {}", step_index);
                    self.fail_step_and_parent(parent_task_id, &subtask.id, step_index, &err)
                        .await?;
                    return Err(err);
                }
            };

            // Start the subtask (Pending -> Running).
            self.task_engine.start(&subtask.id).await?;

            // Execute via step runner (with retry logic).
            match self.step_runner.run_step(subtask, &agent).await {
                Ok(result) => {
                    // Hand off output to the next step (or complete the parent).
                    let output = serde_json::json!({ "output": result.output });
                    self.handoff_manager
                        .on_step_completed(parent_task_id, step_index, output)
                        .await?;
                }
                Err(err) => {
                    self.fail_step_and_parent(parent_task_id, &subtask.id, step_index, &err)
                        .await?;
                    return Err(err);
                }
            }
        }

        Ok(())
    }

    /// Orchestrate parallel execution of subtasks under `parent_task_id`.
    ///
    /// Independent subtasks (those with no `depends_on` equivalent -- i.e.
    /// `step_index == 0` or first batch) run concurrently via `tokio::spawn`.
    /// Dependent subtasks wait for their prerequisite step_index to complete.
    ///
    /// For this initial implementation, tasks are grouped into waves:
    /// - Wave 0: all subtasks at step_index 0 (no dependencies)
    /// - Wave 1: all subtasks at step_index 1 (depend on wave 0)
    /// - etc.
    ///
    /// Within each wave, tasks run concurrently.
    pub async fn orchestrate_parallel(&self, parent_task_id: &str) -> Result<(), String> {
        let mut subtasks = self.task_engine.get_subtasks(parent_task_id).await;
        if subtasks.is_empty() {
            return Err(format!(
                "no subtasks found for parent {}",
                parent_task_id
            ));
        }

        // Sort by step_index ascending.
        subtasks.sort_by_key(|t| t.step_index.unwrap_or(u32::MAX));

        // Group subtasks into waves by step_index.
        let mut waves: Vec<Vec<usize>> = Vec::new();
        let mut current_index: Option<u32> = None;
        for (i, subtask) in subtasks.iter().enumerate() {
            let idx = subtask.step_index.unwrap_or(0);
            if current_index != Some(idx) {
                waves.push(Vec::new());
                current_index = Some(idx);
            }
            waves.last_mut().unwrap().push(i);
        }

        #[allow(unused_assignments)]
        let mut last_output = serde_json::json!({});

        for wave in &waves {
            if wave.len() == 1 {
                // Single task in wave -- run sequentially (same as orchestrate).
                let task_idx = wave[0];
                let subtask = &subtasks[task_idx];
                let step_index = subtask.step_index.unwrap_or(0);

                let agent = match &subtask.agent_id {
                    Some(agent_id) => match self.kernel.get_agent(agent_id).await {
                        Some(a) => a,
                        None => {
                            let err =
                                format!("agent {} not found for step {}", agent_id, step_index);
                            self.fail_step_and_parent(
                                parent_task_id,
                                &subtask.id,
                                step_index,
                                &err,
                            )
                            .await?;
                            return Err(err);
                        }
                    },
                    None => {
                        let err = format!("no agent assigned to step {}", step_index);
                        self.fail_step_and_parent(
                            parent_task_id,
                            &subtask.id,
                            step_index,
                            &err,
                        )
                        .await?;
                        return Err(err);
                    }
                };

                self.task_engine.start(&subtask.id).await?;

                match self.step_runner.run_step(subtask, &agent).await {
                    Ok(result) => {
                        last_output = serde_json::json!({ "output": result.output });
                        self.handoff_manager
                            .on_step_completed(parent_task_id, step_index, last_output.clone())
                            .await?;
                    }
                    Err(err) => {
                        self.fail_step_and_parent(
                            parent_task_id,
                            &subtask.id,
                            step_index,
                            &err,
                        )
                        .await?;
                        return Err(err);
                    }
                }
            } else {
                // Multiple tasks in wave -- run concurrently.
                let mut handles = Vec::new();

                for &task_idx in wave {
                    let subtask = subtasks[task_idx].clone();
                    let step_index = subtask.step_index.unwrap_or(0);
                    let kernel = Arc::clone(&self.kernel);
                    let step_runner = Arc::clone(&self.step_runner);
                    let task_engine = Arc::clone(&self.task_engine);

                    let handle = tokio::spawn(async move {
                        let agent = match &subtask.agent_id {
                            Some(agent_id) => match kernel.get_agent(agent_id).await {
                                Some(a) => a,
                                None => {
                                    return Err(format!(
                                        "agent {} not found for step {}",
                                        agent_id, step_index
                                    ));
                                }
                            },
                            None => {
                                return Err(format!(
                                    "no agent assigned to step {}",
                                    step_index
                                ));
                            }
                        };

                        task_engine.start(&subtask.id).await?;

                        match step_runner.run_step(&subtask, &agent).await {
                            Ok(result) => Ok((subtask.id.clone(), step_index, result)),
                            Err(err) => Err(err),
                        }
                    });

                    handles.push((task_idx, handle));
                }

                // Await all concurrent tasks.
                for (task_idx, handle) in handles {
                    let subtask = &subtasks[task_idx];
                    let step_index = subtask.step_index.unwrap_or(0);

                    match handle.await {
                        Ok(Ok((_id, si, result))) => {
                            last_output = serde_json::json!({ "output": result.output });
                            self.handoff_manager
                                .on_step_completed(parent_task_id, si, last_output.clone())
                                .await?;
                        }
                        Ok(Err(err)) => {
                            self.fail_step_and_parent(
                                parent_task_id,
                                &subtask.id,
                                step_index,
                                &err,
                            )
                            .await?;
                            return Err(err);
                        }
                        Err(join_err) => {
                            let err = format!("task join error: {}", join_err);
                            self.fail_step_and_parent(
                                parent_task_id,
                                &subtask.id,
                                step_index,
                                &err,
                            )
                            .await?;
                            return Err(err);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Mark a subtask as failed, emit an error event, and fail the parent task.
    async fn fail_step_and_parent(
        &self,
        parent_task_id: &str,
        subtask_id: &str,
        step_index: u32,
        error: &str,
    ) -> Result<(), String> {
        // Attempt to fail the subtask. It may still be Pending if start() hasn't
        // been called yet, which is fine -- TaskEngine::fail accepts both
        // Running and Pending states.
        let _ = self.task_engine.fail(subtask_id, error).await;

        // Emit error event.
        let event = AgentEvent {
            id: Uuid::new_v4().to_string(),
            agent_id: "orchestrator".to_string(),
            timestamp: Utc::now(),
            event_type: EventType::TaskStepCompleted {
                task_id: parent_task_id.to_string(),
                step_index,
                result: serde_json::json!({ "error": error }),
            },
        };
        self.event_bus.publish(event).await;

        // Fail the parent task.
        let _ = self.task_engine.fail(parent_task_id, error).await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::station::costs::cost_tracker::CostTracker;
    use crate::station::kernel::agent::{AgentDescriptor, AgentRole, Framework};
    use crate::station::llm::client::LlmClient;
    use crate::station::permissions::PermissionEngine;
    use crate::station::skills::skill_registry::SkillRegistry;
    use crate::station::tasks::task_types::{CreateTaskRequest, TaskSpec, TaskStatus};
    use tokio::sync::RwLock;

    /// Build the full set of shared subsystems for testing.
    struct TestHarness {
        task_engine: Arc<TaskEngine>,
        kernel: Arc<AgentKernel>,
        event_bus: Arc<EventBus>,
        orchestrator: TaskOrchestrator,
    }

    async fn setup() -> TestHarness {
        let llm_client = Arc::new(RwLock::new(LlmClient::new(
            "https://gateway.day1.doctor",
        )));
        let kernel = Arc::new(AgentKernel::new());
        let event_bus = Arc::new(EventBus::new(64));
        let cost_tracker = Arc::new(CostTracker::new());
        let permission_engine = Arc::new(PermissionEngine::new());
        let skill_registry = Arc::new(SkillRegistry::new());
        let task_engine = Arc::new(TaskEngine::new());

        let executor = Arc::new(AgentExecutor::new(
            llm_client,
            kernel.clone(),
            event_bus.clone(),
            cost_tracker,
            permission_engine,
            skill_registry,
        ));

        let step_runner = Arc::new(StepRunner::new(executor.clone(), 0));

        let handoff_manager = Arc::new(TaskHandoffManager::new(
            task_engine.clone(),
            kernel.clone(),
            event_bus.clone(),
        ));

        let orchestrator = TaskOrchestrator::new(
            executor,
            step_runner,
            task_engine.clone(),
            kernel.clone(),
            event_bus.clone(),
            handoff_manager,
        );

        TestHarness {
            task_engine,
            kernel,
            event_bus,
            orchestrator,
        }
    }

    /// Set up a parent task with N sequential subtasks, each assigned to the
    /// given agent. The parent is started (Running); subtasks are left Pending.
    async fn setup_pipeline(
        engine: &TaskEngine,
        kernel: &AgentKernel,
        agent_role: AgentRole,
        n: u32,
    ) -> (String, Vec<String>, String) {
        let agent = AgentDescriptor::new(
            "test-worker",
            agent_role,
            Framework::Builtin,
            "claude-sonnet-4",
        );
        let agent_id = agent.id.clone();
        kernel.register(agent).await;

        let parent = engine
            .create(CreateTaskRequest {
                description: "Parent task".to_string(),
                priority: None,
            })
            .await;
        engine.start(&parent.id).await.unwrap();

        let mut step_ids = Vec::new();
        for i in 0..n {
            let mut step = TaskSpec::new_subtask(&format!("Step {}", i), &parent.id, i);
            step.agent_id = Some(agent_id.clone());
            let id = engine.add_subtask(&parent.id, step).await.unwrap();
            step_ids.push(id);
        }

        (parent.id, step_ids, agent_id)
    }

    #[tokio::test]
    async fn test_orchestrate_fails_without_subtasks() {
        let h = setup().await;
        let parent = h
            .task_engine
            .create(CreateTaskRequest {
                description: "Empty".to_string(),
                priority: None,
            })
            .await;
        h.task_engine.start(&parent.id).await.unwrap();

        let result = h.orchestrator.orchestrate(&parent.id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no subtasks"));
    }

    #[tokio::test]
    async fn test_step_failure_marks_parent_failed() {
        let h = setup().await;
        let (parent_id, step_ids, _agent_id) =
            setup_pipeline(&h.task_engine, &h.kernel, AgentRole::Coder, 2).await;

        // The orchestrator will try to execute via the LLM client, which will
        // fail because there's no real gateway. This tests the error handling
        // path: step fails -> parent fails.
        let result = h.orchestrator.orchestrate(&parent_id).await;
        assert!(result.is_err(), "orchestrate should fail without gateway");

        // The first subtask should be marked Failed.
        let step0 = h.task_engine.status(&step_ids[0]).await.unwrap();
        assert_eq!(step0.status, TaskStatus::Failed);

        // The parent should also be Failed.
        let parent = h.task_engine.status(&parent_id).await.unwrap();
        assert_eq!(parent.status, TaskStatus::Failed);
    }

    #[tokio::test]
    async fn test_error_event_emitted_on_failure() {
        let h = setup().await;
        let mut rx = h.event_bus.subscribe();

        let (parent_id, _step_ids, _agent_id) =
            setup_pipeline(&h.task_engine, &h.kernel, AgentRole::Coder, 1).await;

        let _ = h.orchestrator.orchestrate(&parent_id).await;

        // Drain events and look for an error event.
        let mut found_error_event = false;
        while let Ok(event) = rx.try_recv() {
            if let EventType::TaskStepCompleted { result, .. } = &event.event_type {
                if result.get("error").is_some() {
                    found_error_event = true;
                    break;
                }
            }
        }

        assert!(found_error_event, "should have emitted an error event");
    }

    #[tokio::test]
    async fn test_orchestrate_no_agent_assigned_fails() {
        let h = setup().await;

        // Create parent + subtask without assigning an agent.
        let parent = h
            .task_engine
            .create(CreateTaskRequest {
                description: "No agent".to_string(),
                priority: None,
            })
            .await;
        h.task_engine.start(&parent.id).await.unwrap();

        let step = TaskSpec::new_subtask("Unassigned step", &parent.id, 0);
        h.task_engine.add_subtask(&parent.id, step).await.unwrap();

        let result = h.orchestrator.orchestrate(&parent.id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no agent assigned"));

        // Parent should be failed.
        let parent = h.task_engine.status(&parent.id).await.unwrap();
        assert_eq!(parent.status, TaskStatus::Failed);
    }

    #[tokio::test]
    async fn test_orchestrate_parallel_fails_without_subtasks() {
        let h = setup().await;
        let parent = h
            .task_engine
            .create(CreateTaskRequest {
                description: "Empty parallel".to_string(),
                priority: None,
            })
            .await;
        h.task_engine.start(&parent.id).await.unwrap();

        let result = h.orchestrator.orchestrate_parallel(&parent.id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no subtasks"));
    }

    #[tokio::test]
    async fn test_parallel_step_failure_marks_parent_failed() {
        let h = setup().await;
        let (parent_id, step_ids, _agent_id) =
            setup_pipeline(&h.task_engine, &h.kernel, AgentRole::Writer, 2).await;

        let result = h.orchestrator.orchestrate_parallel(&parent_id).await;
        assert!(result.is_err(), "should fail without real gateway");

        // First step should be Failed.
        let step0 = h.task_engine.status(&step_ids[0]).await.unwrap();
        assert_eq!(step0.status, TaskStatus::Failed);

        // Parent should be Failed.
        let parent = h.task_engine.status(&parent_id).await.unwrap();
        assert_eq!(parent.status, TaskStatus::Failed);
    }

    #[tokio::test]
    async fn test_orchestrator_construction() {
        let h = setup().await;
        // Verify the orchestrator was constructed successfully with all deps.
        // This is a smoke test ensuring the wiring is correct.
        let parent = h
            .task_engine
            .create(CreateTaskRequest {
                description: "Smoke test".to_string(),
                priority: None,
            })
            .await;
        assert_eq!(parent.status, TaskStatus::Pending);
    }
}
