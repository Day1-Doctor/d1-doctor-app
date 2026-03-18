use std::sync::Arc;

use crate::station::events::{AgentEvent, EventBus, EventType};
use crate::station::kernel::{AgentKernel, AgentRole, AgentStatus};
use crate::station::tasks::task_types::{CreateTaskRequest, TaskSpec};
use crate::station::tasks::TaskEngine;

use super::decomposer::DecomposedPlan;

/// Routes decomposed plans to agents by matching suggested roles to
/// registered agents in the kernel.
///
/// The router:
/// 1. Creates a parent task from the original description.
/// 2. Creates sub-tasks for each planned step.
/// 3. Assigns each sub-task to the best available agent of the matching role.
/// 4. Emits routing events on the event bus.
pub struct TaskRouter {
    kernel: Arc<AgentKernel>,
    task_engine: Arc<TaskEngine>,
    event_bus: Arc<EventBus>,
}

impl TaskRouter {
    pub fn new(
        kernel: Arc<AgentKernel>,
        task_engine: Arc<TaskEngine>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            kernel,
            task_engine,
            event_bus,
        }
    }

    /// Route a decomposed plan: create parent task + sub-tasks, assign to agents.
    ///
    /// Returns the parent task ID on success.
    pub async fn route_plan(&self, plan: DecomposedPlan) -> Result<String, String> {
        // 1. Create parent task.
        let parent = self
            .task_engine
            .create(CreateTaskRequest {
                description: plan.original_description.clone(),
                priority: Some(0),
            })
            .await;
        let parent_id = parent.id.clone();
        self.task_engine.start(&parent_id).await?;

        // 2. Create sub-tasks and assign to agents.
        for step in &plan.steps {
            let role = map_role(&step.suggested_role);
            let agent = self.find_available_agent(role).await;

            let mut subtask = TaskSpec::new_subtask(&step.title, &parent_id, step.step_index);
            if let Some(ref agent_id) = agent {
                subtask.agent_id = Some(agent_id.clone());
            }

            let subtask_id = self.task_engine.add_subtask(&parent_id, subtask).await?;

            // Emit a routing event for observability.
            let event = AgentEvent {
                id: uuid::Uuid::new_v4().to_string(),
                agent_id: agent.unwrap_or_else(|| "unassigned".to_string()),
                timestamp: chrono::Utc::now(),
                event_type: EventType::TaskStepCompleted {
                    task_id: subtask_id,
                    step_index: step.step_index,
                    result: serde_json::json!({ "status": "routed" }),
                },
            };
            self.event_bus.publish(event).await;
        }

        Ok(parent_id)
    }

    /// Find the best available agent for the given role.
    ///
    /// Prefers idle agents, then falls back to any agent of that role.
    async fn find_available_agent(&self, role: AgentRole) -> Option<String> {
        let agents = self.kernel.agents_by_role(role).await;
        // Prefer idle agents, then any agent of that role.
        agents
            .iter()
            .find(|a| a.status == AgentStatus::Idle)
            .or_else(|| agents.first())
            .map(|a| a.id.clone())
    }
}

/// Map a role string (from the decomposer) to the kernel's `AgentRole` enum.
fn map_role(role_str: &str) -> AgentRole {
    match role_str {
        "researcher" => AgentRole::Researcher,
        "analyst" => AgentRole::Analyst,
        "writer" => AgentRole::Writer,
        "coder" => AgentRole::Coder,
        "operator" => AgentRole::Operator,
        _ => AgentRole::Orchestrator,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::station::kernel::agent::{AgentDescriptor, Framework};
    use crate::station::tasks::decomposer::TaskDecomposer;
    use crate::station::tasks::task_types::TaskStatus;

    /// Helper: build a kernel with one agent per role.
    async fn setup_kernel() -> Arc<AgentKernel> {
        let kernel = Arc::new(AgentKernel::new());
        let roles = [
            ("researcher-1", AgentRole::Researcher),
            ("analyst-1", AgentRole::Analyst),
            ("writer-1", AgentRole::Writer),
            ("coder-1", AgentRole::Coder),
            ("operator-1", AgentRole::Operator),
            ("orchestrator-1", AgentRole::Orchestrator),
        ];
        for (name, role) in roles {
            let agent = AgentDescriptor::new(name, role, Framework::Builtin);
            kernel.register(agent).await;
        }
        kernel
    }

    #[tokio::test]
    async fn test_route_plan() {
        let kernel = setup_kernel().await;
        let task_engine = Arc::new(TaskEngine::new());
        let event_bus = Arc::new(EventBus::new(64));
        let router = TaskRouter::new(
            Arc::clone(&kernel),
            Arc::clone(&task_engine),
            Arc::clone(&event_bus),
        );

        let decomposer = TaskDecomposer::new();
        let plan = decomposer.decompose("Research AI agents and write a report");

        let parent_id = router.route_plan(plan).await.unwrap();

        // Parent task should exist and be running.
        let parent = task_engine.status(&parent_id).await.unwrap();
        assert_eq!(parent.status, TaskStatus::Running);

        // Should have sub-tasks.
        let subtasks = task_engine.get_subtasks(&parent_id).await;
        assert!(
            subtasks.len() >= 2,
            "expected at least 2 subtasks, got {}",
            subtasks.len()
        );

        // All sub-tasks should be pending with parent_id set.
        for sub in &subtasks {
            assert_eq!(sub.status, TaskStatus::Pending);
            assert_eq!(sub.parent_id, Some(parent_id.clone()));
        }
    }

    #[tokio::test]
    async fn test_agent_assignment() {
        let kernel = setup_kernel().await;
        let task_engine = Arc::new(TaskEngine::new());
        let event_bus = Arc::new(EventBus::new(64));
        let router = TaskRouter::new(
            Arc::clone(&kernel),
            Arc::clone(&task_engine),
            Arc::clone(&event_bus),
        );

        let decomposer = TaskDecomposer::new();
        let plan =
            decomposer.decompose("Research competitors, analyze them, and write a comparison");

        let parent_id = router.route_plan(plan).await.unwrap();
        let subtasks = task_engine.get_subtasks(&parent_id).await;
        assert_eq!(subtasks.len(), 3);

        // Each sub-task should have an agent assigned.
        for sub in &subtasks {
            assert!(
                sub.agent_id.is_some(),
                "sub-task '{}' should have an agent assigned",
                sub.title
            );
        }

        // Verify agent roles match: the assigned agents' roles should correspond
        // to the decomposer's suggested roles.
        let researcher_subtask = &subtasks[0]; // step 0 = researcher
        let analyst_subtask = &subtasks[1]; // step 1 = analyst
        let writer_subtask = &subtasks[2]; // step 2 = writer

        let r_agent = kernel
            .get_agent(researcher_subtask.agent_id.as_ref().unwrap())
            .await
            .unwrap();
        assert_eq!(r_agent.role, AgentRole::Researcher);

        let a_agent = kernel
            .get_agent(analyst_subtask.agent_id.as_ref().unwrap())
            .await
            .unwrap();
        assert_eq!(a_agent.role, AgentRole::Analyst);

        let w_agent = kernel
            .get_agent(writer_subtask.agent_id.as_ref().unwrap())
            .await
            .unwrap();
        assert_eq!(w_agent.role, AgentRole::Writer);
    }

    #[tokio::test]
    async fn test_route_simple_task_fallback() {
        let kernel = setup_kernel().await;
        let task_engine = Arc::new(TaskEngine::new());
        let event_bus = Arc::new(EventBus::new(64));
        let router = TaskRouter::new(
            Arc::clone(&kernel),
            Arc::clone(&task_engine),
            Arc::clone(&event_bus),
        );

        let decomposer = TaskDecomposer::new();
        let plan = decomposer.decompose("Hello world");

        let parent_id = router.route_plan(plan).await.unwrap();
        let subtasks = task_engine.get_subtasks(&parent_id).await;
        assert_eq!(subtasks.len(), 1);

        // Fallback: assigned to orchestrator role
        let agent = kernel
            .get_agent(subtasks[0].agent_id.as_ref().unwrap())
            .await
            .unwrap();
        assert_eq!(agent.role, AgentRole::Orchestrator);
    }

    #[tokio::test]
    async fn test_route_no_matching_agent() {
        // Kernel with only a coder — no researcher, writer, etc.
        let kernel = Arc::new(AgentKernel::new());
        let coder = AgentDescriptor::new("coder-only", AgentRole::Coder, Framework::Builtin);
        kernel.register(coder).await;

        let task_engine = Arc::new(TaskEngine::new());
        let event_bus = Arc::new(EventBus::new(64));
        let router = TaskRouter::new(
            Arc::clone(&kernel),
            Arc::clone(&task_engine),
            Arc::clone(&event_bus),
        );

        let decomposer = TaskDecomposer::new();
        let plan = decomposer.decompose("Research AI and write a report");

        let parent_id = router.route_plan(plan).await.unwrap();
        let subtasks = task_engine.get_subtasks(&parent_id).await;

        // Sub-tasks should still be created even without matching agents.
        assert!(subtasks.len() >= 2);

        // Some sub-tasks may have no agent assigned (researcher, writer roles
        // not present in kernel).
        let unassigned_count = subtasks.iter().filter(|s| s.agent_id.is_none()).count();
        assert!(
            unassigned_count >= 1,
            "at least one sub-task should be unassigned when no matching agent exists"
        );
    }

    #[tokio::test]
    async fn test_event_bus_receives_routing_events() {
        let kernel = setup_kernel().await;
        let task_engine = Arc::new(TaskEngine::new());
        let event_bus = Arc::new(EventBus::new(64));

        let mut rx = event_bus.subscribe();

        let router = TaskRouter::new(
            Arc::clone(&kernel),
            Arc::clone(&task_engine),
            Arc::clone(&event_bus),
        );

        let decomposer = TaskDecomposer::new();
        let plan = decomposer.decompose("Research AI agents");
        let step_count = plan.steps.len();

        router.route_plan(plan).await.unwrap();

        // We should receive one event per routed step.
        for _ in 0..step_count {
            let event = rx.try_recv().expect("should receive routing event");
            match event.event_type {
                EventType::TaskStepCompleted { result, .. } => {
                    assert_eq!(result["status"], "routed");
                }
                _ => panic!("expected TaskStepCompleted event"),
            }
        }
    }

    #[tokio::test]
    async fn test_prefers_idle_agent() {
        let kernel = Arc::new(AgentKernel::new());

        // Register two coders.
        let coder_a = AgentDescriptor::new("coder-busy", AgentRole::Coder, Framework::Builtin);
        let coder_b = AgentDescriptor::new("coder-idle", AgentRole::Coder, Framework::Builtin);
        let id_a = kernel.register(coder_a).await;
        let _id_b = kernel.register(coder_b).await;

        // Make coder-busy Working.
        kernel
            .apply_trigger(&id_a, crate::station::kernel::Trigger::TaskAssign)
            .await
            .unwrap();

        let task_engine = Arc::new(TaskEngine::new());
        let event_bus = Arc::new(EventBus::new(64));
        let router = TaskRouter::new(
            Arc::clone(&kernel),
            Arc::clone(&task_engine),
            Arc::clone(&event_bus),
        );

        let decomposer = TaskDecomposer::new();
        let plan = decomposer.decompose("Build a web server");

        let parent_id = router.route_plan(plan).await.unwrap();
        let subtasks = task_engine.get_subtasks(&parent_id).await;

        // The coder step should be assigned to the idle coder, not the busy one.
        let coder_subtask = subtasks
            .iter()
            .find(|s| s.title == "Implement code")
            .expect("should have a coder sub-task");

        let assigned_agent = kernel
            .get_agent(coder_subtask.agent_id.as_ref().unwrap())
            .await
            .unwrap();
        assert_eq!(assigned_agent.status, AgentStatus::Idle);
        assert_eq!(assigned_agent.name, "coder-idle");
    }

    #[test]
    fn test_map_role() {
        assert_eq!(map_role("researcher"), AgentRole::Researcher);
        assert_eq!(map_role("analyst"), AgentRole::Analyst);
        assert_eq!(map_role("writer"), AgentRole::Writer);
        assert_eq!(map_role("coder"), AgentRole::Coder);
        assert_eq!(map_role("operator"), AgentRole::Operator);
        assert_eq!(map_role("orchestrator"), AgentRole::Orchestrator);
        assert_eq!(map_role("unknown"), AgentRole::Orchestrator);
    }
}
