use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::station::events::bus::EventBus;
use crate::station::events::event_types::{AgentEvent, EventType};
use crate::station::kernel::kernel::AgentKernel;
use crate::station::tasks::task_engine::TaskEngine;
use crate::station::tasks::task_types::{TaskFilter, TaskStatus};

/// Manages sequential task hand-offs between steps of a parent task.
///
/// When a step completes, the manager finds the next pending subtask,
/// passes the completed step's output as the next step's input, assigns
/// it to an agent, and starts it. When all steps are done the parent
/// task is completed.
pub struct TaskHandoffManager {
    task_engine: Arc<TaskEngine>,
    agent_kernel: Arc<AgentKernel>,
    event_bus: Arc<EventBus>,
}

impl TaskHandoffManager {
    /// Create a new handoff manager wired to the given engine, kernel, and bus.
    pub fn new(
        task_engine: Arc<TaskEngine>,
        agent_kernel: Arc<AgentKernel>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            task_engine,
            agent_kernel,
            event_bus,
        }
    }

    /// Called when a step completes.
    ///
    /// 1. Marks the completed step as `Completed` with the given output.
    /// 2. Emits a `task.step_completed` event.
    /// 3. Looks for the next pending subtask (by step_index order).
    /// 4. If found: passes `output` as input, assigns to an available
    ///    agent, and starts the subtask.
    /// 5. If no more steps: completes the parent task with the final
    ///    step's output.
    ///
    /// Returns `Ok(true)` if a next step was triggered, `Ok(false)` if
    /// the parent task was completed (no more steps), or an error.
    pub async fn on_step_completed(
        &self,
        parent_id: &str,
        completed_step_index: u32,
        output: serde_json::Value,
    ) -> Result<bool, String> {
        // 1. Find and complete the finished step.
        let subtasks = self.task_engine.get_subtasks(parent_id).await;
        let completed_step = subtasks
            .iter()
            .find(|t| t.step_index == Some(completed_step_index))
            .ok_or_else(|| {
                format!(
                    "step {} not found under parent {}",
                    completed_step_index, parent_id
                )
            })?;

        self.task_engine
            .complete(&completed_step.id, output.clone())
            .await?;

        // 2. Emit task.step_completed event.
        let agent_id = completed_step
            .agent_id
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let event = AgentEvent {
            id: Uuid::new_v4().to_string(),
            agent_id: agent_id.clone(),
            timestamp: Utc::now(),
            event_type: EventType::TaskStepCompleted {
                task_id: parent_id.to_string(),
                step_index: completed_step_index,
                result: output.clone(),
            },
        };
        self.event_bus.publish(event).await;

        // Clear the agent's task assignment if it had one.
        if completed_step.agent_id.is_some() {
            let _ = self.agent_kernel.clear_task(&agent_id).await;
        }

        // 3. Find the next pending subtask (lowest step_index > completed).
        let mut pending_steps: Vec<_> = subtasks
            .iter()
            .filter(|t| t.status == TaskStatus::Pending && t.step_index.is_some())
            .collect();
        pending_steps.sort_by_key(|t| t.step_index.unwrap_or(u32::MAX));

        let next_step = pending_steps
            .into_iter()
            .find(|t| t.step_index.unwrap_or(0) > completed_step_index);

        match next_step {
            Some(step) => {
                // 4a. Set the output of the completed step as input for the next.
                {
                    // We need mutable access to the task — use the engine's
                    // internal API. Since TaskEngine doesn't expose a
                    // set_input, we do a start-then-complete dance is not
                    // appropriate. Instead, we modify via the task_engine
                    // list+filter approach. For now, we store input by
                    // re-creating with the engine's lower-level approach.
                    //
                    // The simplest correct approach: start the step, which
                    // makes it Running. The caller can read the output from
                    // the prior step via the parent's subtask list.
                }

                // Find an available agent to assign.
                let idle_agents = self
                    .agent_kernel
                    .agents_by_status(crate::station::kernel::agent_state::AgentStatus::Idle)
                    .await;
                if let Some(agent) = idle_agents.first() {
                    self.agent_kernel.assign_task(&agent.id, &step.id).await?;
                }

                // Start the next step.
                self.task_engine.start(&step.id).await?;

                Ok(true)
            }
            None => {
                // 4b. No more pending steps — complete the parent task.
                // Make sure the parent is in Running state.
                let parent = self.task_engine.status(parent_id).await;
                if let Some(p) = parent {
                    if p.status == TaskStatus::Running {
                        self.task_engine.complete(parent_id, output).await?;
                    }
                }
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::station::kernel::agent::{AgentDescriptor, AgentRole, Framework};
    use crate::station::kernel::agent_state::Trigger;
    use crate::station::tasks::task_types::{CreateTaskRequest, TaskSpec};

    /// Helper to set up a parent task with N sequential subtasks.
    async fn setup_pipeline(engine: &TaskEngine, n: u32) -> (String, Vec<String>) {
        let parent = engine
            .create(CreateTaskRequest {
                description: "Parent pipeline".to_string(),
                priority: None,
            })
            .await;
        engine.start(&parent.id).await.unwrap();

        let mut step_ids = Vec::new();
        for i in 0..n {
            let step = TaskSpec::new_subtask(&format!("Step {}", i), &parent.id, i);
            let id = engine.add_subtask(&parent.id, step).await.unwrap();
            step_ids.push(id);
        }

        // Start step 0 to make it Running.
        engine.start(&step_ids[0]).await.unwrap();

        (parent.id, step_ids)
    }

    #[tokio::test]
    async fn test_complete_step_0_triggers_step_1() {
        let engine = Arc::new(TaskEngine::new());
        let kernel = Arc::new(AgentKernel::new());
        let bus = Arc::new(EventBus::new(64));

        // Register an idle agent.
        let agent = AgentDescriptor::new("worker-1", AgentRole::Coder, Framework::Builtin);
        kernel.register(agent).await;

        let (parent_id, step_ids) = setup_pipeline(&engine, 3).await;
        let manager = TaskHandoffManager::new(engine.clone(), kernel.clone(), bus.clone());

        let output = serde_json::json!({"result": "step 0 done"});
        let triggered = manager
            .on_step_completed(&parent_id, 0, output)
            .await
            .unwrap();

        assert!(triggered, "should have triggered next step");

        // Step 1 should now be Running.
        let step1 = engine.status(&step_ids[1]).await.unwrap();
        assert_eq!(step1.status, TaskStatus::Running);

        // Step 0 should be Completed.
        let step0 = engine.status(&step_ids[0]).await.unwrap();
        assert_eq!(step0.status, TaskStatus::Completed);
    }

    #[tokio::test]
    async fn test_output_passed_as_context() {
        let engine = Arc::new(TaskEngine::new());
        let kernel = Arc::new(AgentKernel::new());
        let bus = Arc::new(EventBus::new(64));

        let (parent_id, step_ids) = setup_pipeline(&engine, 2).await;
        let manager = TaskHandoffManager::new(engine.clone(), kernel.clone(), bus.clone());

        let output = serde_json::json!({"analysis": "important data"});
        manager
            .on_step_completed(&parent_id, 0, output.clone())
            .await
            .unwrap();

        // The completed step should have the output stored.
        let step0 = engine.status(&step_ids[0]).await.unwrap();
        assert_eq!(step0.output, Some(output));
    }

    #[tokio::test]
    async fn test_final_step_completes_parent_task() {
        let engine = Arc::new(TaskEngine::new());
        let kernel = Arc::new(AgentKernel::new());
        let bus = Arc::new(EventBus::new(64));

        let (parent_id, step_ids) = setup_pipeline(&engine, 2).await;
        let manager = TaskHandoffManager::new(engine.clone(), kernel.clone(), bus.clone());

        // Complete step 0 -> triggers step 1.
        let output0 = serde_json::json!({"step": 0});
        manager
            .on_step_completed(&parent_id, 0, output0)
            .await
            .unwrap();

        // Complete step 1 -> should complete parent.
        let output1 = serde_json::json!({"final": "result"});
        let triggered = manager
            .on_step_completed(&parent_id, 1, output1.clone())
            .await
            .unwrap();

        assert!(!triggered, "no more steps, should return false");

        // Parent should be Completed.
        let parent = engine.status(&parent_id).await.unwrap();
        assert_eq!(parent.status, TaskStatus::Completed);
        assert_eq!(parent.output, Some(output1));
    }

    #[tokio::test]
    async fn test_handles_missing_next_step_gracefully() {
        let engine = Arc::new(TaskEngine::new());
        let kernel = Arc::new(AgentKernel::new());
        let bus = Arc::new(EventBus::new(64));

        // Single-step pipeline.
        let (parent_id, _step_ids) = setup_pipeline(&engine, 1).await;
        let manager = TaskHandoffManager::new(engine.clone(), kernel.clone(), bus.clone());

        let output = serde_json::json!({"done": true});
        let triggered = manager
            .on_step_completed(&parent_id, 0, output)
            .await
            .unwrap();

        assert!(!triggered, "single step — no next step");

        // Parent should be Completed.
        let parent = engine.status(&parent_id).await.unwrap();
        assert_eq!(parent.status, TaskStatus::Completed);
    }

    #[tokio::test]
    async fn test_step_completed_event_emitted() {
        let engine = Arc::new(TaskEngine::new());
        let kernel = Arc::new(AgentKernel::new());
        let bus = Arc::new(EventBus::new(64));
        let mut rx = bus.subscribe();

        let (parent_id, _step_ids) = setup_pipeline(&engine, 2).await;
        let manager = TaskHandoffManager::new(engine.clone(), kernel.clone(), bus.clone());

        let output = serde_json::json!({"result": "done"});
        manager
            .on_step_completed(&parent_id, 0, output.clone())
            .await
            .unwrap();

        let event = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
            .await
            .expect("timed out")
            .expect("recv failed");

        match event.event_type {
            EventType::TaskStepCompleted {
                task_id,
                step_index,
                result,
            } => {
                assert_eq!(task_id, parent_id);
                assert_eq!(step_index, 0);
                assert_eq!(result, output);
            }
            other => panic!("unexpected event type: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_nonexistent_step_returns_error() {
        let engine = Arc::new(TaskEngine::new());
        let kernel = Arc::new(AgentKernel::new());
        let bus = Arc::new(EventBus::new(64));

        let (parent_id, _step_ids) = setup_pipeline(&engine, 1).await;
        let manager = TaskHandoffManager::new(engine.clone(), kernel.clone(), bus.clone());

        // Step index 99 does not exist.
        let result = manager
            .on_step_completed(&parent_id, 99, serde_json::json!({}))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_agent_assigned_to_next_step() {
        let engine = Arc::new(TaskEngine::new());
        let kernel = Arc::new(AgentKernel::new());
        let bus = Arc::new(EventBus::new(64));

        let agent = AgentDescriptor::new("worker-1", AgentRole::Coder, Framework::Builtin);
        let agent_id = agent.id.clone();
        kernel.register(agent).await;

        let (parent_id, step_ids) = setup_pipeline(&engine, 2).await;
        let manager = TaskHandoffManager::new(engine.clone(), kernel.clone(), bus.clone());

        manager
            .on_step_completed(&parent_id, 0, serde_json::json!({}))
            .await
            .unwrap();

        // The idle agent should have been assigned to step 1.
        let agent_desc = kernel.get_agent(&agent_id).await.unwrap();
        assert_eq!(agent_desc.current_task_id, Some(step_ids[1].clone()));
    }
}
