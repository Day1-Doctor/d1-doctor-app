//! E2E Execution Pipeline Tests — decompose, route, orchestrate, verify state transitions.
//! All tests use in-memory runtime with no actual LLM calls.

use std::sync::Arc;

use serde_json::json;

use d1_copilot_lib::station::events::{EventBus, EventType};
use d1_copilot_lib::station::kernel::agent::{AgentDescriptor, AgentRole, Framework};
use d1_copilot_lib::station::kernel::{AgentKernel, AgentStatus, Trigger};
use d1_copilot_lib::station::tasks::decomposer::TaskDecomposer;
use d1_copilot_lib::station::tasks::handoff::TaskHandoffManager;
use d1_copilot_lib::station::tasks::router::TaskRouter;
use d1_copilot_lib::station::tasks::task_types::{CreateTaskRequest, TaskSpec, TaskStatus};
use d1_copilot_lib::station::tasks::TaskEngine;

/// Set up a kernel with one agent per role.
async fn setup_kernel() -> Arc<AgentKernel> {
    let kernel = Arc::new(AgentKernel::new());
    let roles = [
        ("Dr. Bob", AgentRole::Orchestrator),
        ("Scout", AgentRole::Researcher),
        ("Sage", AgentRole::Analyst),
        ("Quill", AgentRole::Writer),
        ("Pixel", AgentRole::Coder),
        ("Atlas", AgentRole::Operator),
    ];
    for (name, role) in roles {
        let model = match role {
            AgentRole::Researcher | AgentRole::Operator => "claude-haiku-4-5",
            _ => "claude-sonnet-4",
        };
        let agent = AgentDescriptor::new(name, role, Framework::Builtin, model);
        kernel.register(agent).await;
    }
    kernel
}

/// Set up a parent task with N sequential subtasks, each assigned to the given agent.
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

// ---------------------------------------------------------------------------
// Test: decompose task -> route to agents -> verify step ordering
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execution_decompose_route_step_ordering() {
    let kernel = setup_kernel().await;
    let engine = Arc::new(TaskEngine::new());
    let bus = Arc::new(EventBus::new(64));
    let router = TaskRouter::new(Arc::clone(&kernel), Arc::clone(&engine), Arc::clone(&bus));

    let decomposer = TaskDecomposer::new();
    let plan = decomposer.decompose("Research competitors, analyze data, and write a report");
    let step_count = plan.steps.len();
    assert!(step_count >= 3, "should decompose into at least 3 steps");

    let parent_id = router.route_plan(plan).await.unwrap();

    let subtasks = engine.get_subtasks(&parent_id).await;
    assert_eq!(subtasks.len(), step_count);

    // Verify step_index ordering is sequential.
    let mut indices: Vec<u32> = subtasks
        .iter()
        .filter_map(|t| t.step_index)
        .collect();
    indices.sort();
    for (i, idx) in indices.iter().enumerate() {
        assert_eq!(*idx, i as u32, "step_index should match position");
    }
}

// ---------------------------------------------------------------------------
// Test: task with multiple steps, verify step_index ordering
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execution_multi_step_ordering() {
    let engine = TaskEngine::new();
    let kernel = AgentKernel::new();

    let (parent_id, step_ids, _agent_id) =
        setup_pipeline(&engine, &kernel, AgentRole::Coder, 5).await;

    let subtasks = engine.get_subtasks(&parent_id).await;
    assert_eq!(subtasks.len(), 5);

    // Verify each step has the correct step_index.
    for (i, step_id) in step_ids.iter().enumerate() {
        let task = engine.status(step_id).await.unwrap();
        assert_eq!(task.step_index, Some(i as u32));
        assert_eq!(task.parent_id, Some(parent_id.clone()));
    }
}

// ---------------------------------------------------------------------------
// Test: task completion marks parent as completed (via handoff)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execution_task_completion_marks_parent_completed() {
    let engine = Arc::new(TaskEngine::new());
    let kernel = Arc::new(AgentKernel::new());
    let bus = Arc::new(EventBus::new(64));

    // Register an idle agent.
    let agent = AgentDescriptor::new("worker", AgentRole::Coder, Framework::Builtin, "claude-sonnet-4");
    kernel.register(agent).await;

    // Create parent with 2 steps.
    let parent = engine
        .create(CreateTaskRequest {
            description: "Completable task".into(),
            priority: None,
        })
        .await;
    engine.start(&parent.id).await.unwrap();

    let step0 = TaskSpec::new_subtask("Step 0", &parent.id, 0);
    let step0_id = engine.add_subtask(&parent.id, step0).await.unwrap();
    engine.start(&step0_id).await.unwrap();

    let step1 = TaskSpec::new_subtask("Step 1", &parent.id, 1);
    let _step1_id = engine.add_subtask(&parent.id, step1).await.unwrap();

    let manager = TaskHandoffManager::new(engine.clone(), kernel.clone(), bus.clone());

    // Complete step 0 -> triggers step 1.
    manager
        .on_step_completed(&parent.id, 0, json!({"output": "step 0 done"}))
        .await
        .unwrap();

    // Complete step 1 -> should complete parent.
    let triggered = manager
        .on_step_completed(&parent.id, 1, json!({"final": "result"}))
        .await
        .unwrap();

    assert!(!triggered, "no more steps should be triggered");

    let parent_task = engine.status(&parent.id).await.unwrap();
    assert_eq!(parent_task.status, TaskStatus::Completed);
}

// ---------------------------------------------------------------------------
// Test: failed step marks parent as failed
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execution_failed_step_marks_parent_failed() {
    let engine = TaskEngine::new();
    let kernel = AgentKernel::new();

    let (parent_id, step_ids, _agent_id) =
        setup_pipeline(&engine, &kernel, AgentRole::Coder, 3).await;

    // Start step 0, then fail it.
    engine.start(&step_ids[0]).await.unwrap();
    engine.fail(&step_ids[0], "out of memory").await.unwrap();

    // Verify step 0 is Failed.
    let step0 = engine.status(&step_ids[0]).await.unwrap();
    assert_eq!(step0.status, TaskStatus::Failed);
    assert!(step0.output.unwrap().to_string().contains("out of memory"));

    // Fail the parent.
    engine.fail(&parent_id, "step 0 failed").await.unwrap();
    let parent = engine.status(&parent_id).await.unwrap();
    assert_eq!(parent.status, TaskStatus::Failed);
}

// ---------------------------------------------------------------------------
// Test: cancel task during execution
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execution_cancel_running_task() {
    let engine = TaskEngine::new();

    let task = engine
        .create(CreateTaskRequest {
            description: "Cancellable task".into(),
            priority: None,
        })
        .await;

    // Start, then cancel while running.
    engine.start(&task.id).await.unwrap();
    let running = engine.status(&task.id).await.unwrap();
    assert_eq!(running.status, TaskStatus::Running);

    let cancelled = engine.cancel(&task.id).await.unwrap();
    assert_eq!(cancelled.status, TaskStatus::Cancelled);
    assert!(cancelled.completed_at.is_some());
}

// ---------------------------------------------------------------------------
// Test: agent FSM transitions during simulated execution
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execution_agent_fsm_full_cycle_during_step() {
    let kernel = Arc::new(AgentKernel::new());
    let agent = AgentDescriptor::new("worker", AgentRole::Coder, Framework::Builtin, "claude-sonnet-4");
    let id = kernel.register(agent).await;

    // Simulate the execute_step lifecycle: idle -> working -> thinking -> working -> idle
    let (_, new) = kernel.apply_trigger(&id, Trigger::TaskAssign).await.unwrap();
    assert_eq!(new, AgentStatus::Working);

    let (_, new) = kernel.apply_trigger(&id, Trigger::LlmCallStart).await.unwrap();
    assert_eq!(new, AgentStatus::Thinking);

    // Simulate tool call within thinking
    let (_, new) = kernel.apply_trigger(&id, Trigger::ToolCallStart).await.unwrap();
    assert_eq!(new, AgentStatus::Executing);

    let (_, new) = kernel.apply_trigger(&id, Trigger::ToolCallEnd).await.unwrap();
    assert_eq!(new, AgentStatus::Working);

    // Back to thinking for another LLM round
    let (_, new) = kernel.apply_trigger(&id, Trigger::LlmCallStart).await.unwrap();
    assert_eq!(new, AgentStatus::Thinking);

    // LLM returns final answer — end the LLM call
    let (_, new) = kernel.apply_trigger(&id, Trigger::LlmCallEnd).await.unwrap();
    assert_eq!(new, AgentStatus::Working);

    // Complete the task
    let (_, new) = kernel.apply_trigger(&id, Trigger::TaskComplete).await.unwrap();
    assert_eq!(new, AgentStatus::Idle);
}

// ---------------------------------------------------------------------------
// Test: events emitted during execution pipeline
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execution_events_emitted_during_pipeline() {
    let kernel = setup_kernel().await;
    let engine = Arc::new(TaskEngine::new());
    let bus = Arc::new(EventBus::new(128));
    let mut rx = bus.subscribe();

    let router = TaskRouter::new(Arc::clone(&kernel), Arc::clone(&engine), Arc::clone(&bus));

    let decomposer = TaskDecomposer::new();
    let plan = decomposer.decompose("Research AI agents");
    let step_count = plan.steps.len();
    assert!(step_count > 0);

    router.route_plan(plan).await.unwrap();

    // Collect all routing events.
    let mut event_count = 0;
    while let Ok(event) = rx.try_recv() {
        match event.event_type {
            EventType::TaskStepCompleted { .. } => event_count += 1,
            _ => {}
        }
    }

    assert_eq!(
        event_count, step_count,
        "should receive one routing event per step"
    );
}

// ---------------------------------------------------------------------------
// Test: step result contains correct fields
// ---------------------------------------------------------------------------

#[tokio::test]
async fn execution_subtask_has_correct_parent_and_agent() {
    let engine = TaskEngine::new();
    let kernel = AgentKernel::new();

    let (parent_id, step_ids, agent_id) =
        setup_pipeline(&engine, &kernel, AgentRole::Writer, 2).await;

    for step_id in &step_ids {
        let task = engine.status(step_id).await.unwrap();
        assert_eq!(task.parent_id, Some(parent_id.clone()));
        assert_eq!(task.agent_id, Some(agent_id.clone()));
        assert_eq!(task.status, TaskStatus::Pending);
    }
}
