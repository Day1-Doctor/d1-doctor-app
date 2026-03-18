//! E2E Task Pipeline Tests — create tasks, decompose, route, verify subtasks.
//! All tests use in-memory runtime with no actual LLM calls.

use std::sync::Arc;

use d1_copilot_lib::station::events::EventBus;
use d1_copilot_lib::station::kernel::agent::{AgentDescriptor, AgentRole, Framework};
use d1_copilot_lib::station::kernel::AgentKernel;
use d1_copilot_lib::station::tasks::decomposer::TaskDecomposer;
use d1_copilot_lib::station::tasks::router::TaskRouter;
use d1_copilot_lib::station::tasks::task_types::{CreateTaskRequest, TaskStatus};
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
        let agent = AgentDescriptor::new(name, role, Framework::Builtin);
        kernel.register(agent).await;
    }
    kernel
}

#[tokio::test]
async fn pipeline_create_task_and_check_status() {
    let engine = TaskEngine::new();

    let task = engine
        .create(CreateTaskRequest {
            description: "Test task".into(),
            priority: Some(1),
        })
        .await;

    assert_eq!(task.status, TaskStatus::Pending);
    assert_eq!(task.title, "Test task");
    assert_eq!(task.priority, 1);

    let fetched = engine.status(&task.id).await.unwrap();
    assert_eq!(fetched.id, task.id);
}

#[tokio::test]
async fn pipeline_task_lifecycle() {
    let engine = TaskEngine::new();

    let task = engine
        .create(CreateTaskRequest {
            description: "Lifecycle test".into(),
            priority: None,
        })
        .await;

    // Start
    let started = engine.start(&task.id).await.unwrap();
    assert_eq!(started.status, TaskStatus::Running);

    // Complete
    let completed = engine
        .complete(&task.id, serde_json::json!({"result": "done"}))
        .await
        .unwrap();
    assert_eq!(completed.status, TaskStatus::Completed);
    assert!(completed.output.is_some());
}

#[tokio::test]
async fn pipeline_decompose_research_and_write() {
    let decomposer = TaskDecomposer::new();
    let plan = decomposer.decompose("Research AI agents and write a report");

    assert!(plan.steps.len() >= 2);

    let roles: Vec<&str> = plan
        .steps
        .iter()
        .map(|s| s.suggested_role.as_str())
        .collect();
    assert!(roles.contains(&"researcher"));
    assert!(roles.contains(&"writer"));
}

#[tokio::test]
async fn pipeline_route_creates_subtasks() {
    let kernel = setup_kernel().await;
    let engine = Arc::new(TaskEngine::new());
    let bus = Arc::new(EventBus::new(64));
    let router = TaskRouter::new(Arc::clone(&kernel), Arc::clone(&engine), Arc::clone(&bus));

    let decomposer = TaskDecomposer::new();
    let plan = decomposer.decompose("Research competitors and write a comparison");

    let parent_id = router.route_plan(plan).await.unwrap();

    let parent = engine.status(&parent_id).await.unwrap();
    assert_eq!(parent.status, TaskStatus::Running);

    let subtasks = engine.get_subtasks(&parent_id).await;
    assert!(subtasks.len() >= 2, "expected at least 2 subtasks");

    for sub in &subtasks {
        assert_eq!(sub.status, TaskStatus::Pending);
        assert_eq!(sub.parent_id, Some(parent_id.clone()));
    }
}

#[tokio::test]
async fn pipeline_subtasks_have_agents_assigned() {
    let kernel = setup_kernel().await;
    let engine = Arc::new(TaskEngine::new());
    let bus = Arc::new(EventBus::new(64));
    let router = TaskRouter::new(Arc::clone(&kernel), Arc::clone(&engine), Arc::clone(&bus));

    let decomposer = TaskDecomposer::new();
    let plan = decomposer.decompose("Research, analyze, and write a report");

    let parent_id = router.route_plan(plan).await.unwrap();
    let subtasks = engine.get_subtasks(&parent_id).await;

    for sub in &subtasks {
        assert!(
            sub.agent_id.is_some(),
            "subtask '{}' should have an agent assigned",
            sub.title
        );
    }
}

#[tokio::test]
async fn pipeline_task_fail_transition() {
    let engine = TaskEngine::new();

    let task = engine
        .create(CreateTaskRequest {
            description: "Failing task".into(),
            priority: None,
        })
        .await;

    engine.start(&task.id).await.unwrap();

    let failed = engine.fail(&task.id, "something went wrong").await.unwrap();
    assert_eq!(failed.status, TaskStatus::Failed);
    assert!(failed.output.is_some());
}

#[tokio::test]
async fn pipeline_task_cancel_from_pending() {
    let engine = TaskEngine::new();

    let task = engine
        .create(CreateTaskRequest {
            description: "To be cancelled".into(),
            priority: None,
        })
        .await;

    let cancelled = engine.cancel(&task.id).await.unwrap();
    assert_eq!(cancelled.status, TaskStatus::Cancelled);
}

#[tokio::test]
async fn pipeline_count_by_status() {
    let engine = TaskEngine::new();

    let t1 = engine
        .create(CreateTaskRequest {
            description: "task 1".into(),
            priority: None,
        })
        .await;
    let t2 = engine
        .create(CreateTaskRequest {
            description: "task 2".into(),
            priority: None,
        })
        .await;
    engine
        .create(CreateTaskRequest {
            description: "task 3".into(),
            priority: None,
        })
        .await;

    engine.start(&t1.id).await.unwrap();
    engine.start(&t2.id).await.unwrap();
    engine
        .complete(&t1.id, serde_json::json!({}))
        .await
        .unwrap();

    let counts = engine.count_by_status().await;
    assert_eq!(counts.get(&TaskStatus::Pending), Some(&1));
    assert_eq!(counts.get(&TaskStatus::Running), Some(&1));
    assert_eq!(counts.get(&TaskStatus::Completed), Some(&1));
}

#[tokio::test]
async fn pipeline_events_emitted_during_routing() {
    let kernel = setup_kernel().await;
    let engine = Arc::new(TaskEngine::new());
    let bus = Arc::new(EventBus::new(64));

    let mut rx = bus.subscribe();

    let router = TaskRouter::new(Arc::clone(&kernel), Arc::clone(&engine), Arc::clone(&bus));

    let decomposer = TaskDecomposer::new();
    let plan = decomposer.decompose("Research AI agents");
    let step_count = plan.steps.len();

    router.route_plan(plan).await.unwrap();

    // Should receive one event per routed step
    for _ in 0..step_count {
        let event = rx.try_recv().expect("should receive routing event");
        match event.event_type {
            d1_copilot_lib::station::events::EventType::TaskStepCompleted { result, .. } => {
                assert_eq!(result["status"], "routed");
            }
            _ => panic!("expected TaskStepCompleted event"),
        }
    }
}
