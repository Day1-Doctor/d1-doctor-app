//! E2E Task Handoff Tests — step-to-step output chaining, sequential pipelines, parent completion.
//! All tests use in-memory runtime with no actual LLM calls.

use std::sync::Arc;

use serde_json::json;

use d1_copilot_lib::station::events::{EventBus, EventType};
use d1_copilot_lib::station::kernel::agent::{AgentDescriptor, AgentRole, Framework};
use d1_copilot_lib::station::kernel::AgentKernel;
use d1_copilot_lib::station::tasks::handoff::TaskHandoffManager;
use d1_copilot_lib::station::tasks::task_types::{CreateTaskRequest, TaskSpec, TaskStatus};
use d1_copilot_lib::station::tasks::TaskEngine;

/// Set up a parent task with N sequential subtasks and start step 0.
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

// ---------------------------------------------------------------------------
// Test: step A completes -> output passed to step B -> step B starts
// ---------------------------------------------------------------------------

#[tokio::test]
async fn handoff_step_a_output_to_step_b() {
    let engine = Arc::new(TaskEngine::new());
    let kernel = Arc::new(AgentKernel::new());
    let bus = Arc::new(EventBus::new(64));

    // Register an idle agent for assignment.
    let agent = AgentDescriptor::new("worker", AgentRole::Coder, Framework::Builtin, "claude-sonnet-4");
    kernel.register(agent).await;

    let (parent_id, step_ids) = setup_pipeline(&engine, 2).await;
    let manager = TaskHandoffManager::new(engine.clone(), kernel.clone(), bus.clone());

    // Complete step 0 with output.
    let output_0 = json!({"research": "competitor data"});
    let triggered = manager
        .on_step_completed(&parent_id, 0, output_0.clone())
        .await
        .unwrap();

    assert!(triggered, "should trigger the next step");

    // Step 0 should be Completed with the output stored.
    let step0 = engine.status(&step_ids[0]).await.unwrap();
    assert_eq!(step0.status, TaskStatus::Completed);
    assert_eq!(step0.output, Some(output_0));

    // Step 1 should be Running.
    let step1 = engine.status(&step_ids[1]).await.unwrap();
    assert_eq!(step1.status, TaskStatus::Running);
}

// ---------------------------------------------------------------------------
// Test: 3-step sequential pipeline completes correctly
// ---------------------------------------------------------------------------

#[tokio::test]
async fn handoff_three_step_pipeline_completes() {
    let engine = Arc::new(TaskEngine::new());
    let kernel = Arc::new(AgentKernel::new());
    let bus = Arc::new(EventBus::new(64));

    let agent = AgentDescriptor::new("worker", AgentRole::Writer, Framework::Builtin, "claude-sonnet-4");
    kernel.register(agent).await;

    let (parent_id, step_ids) = setup_pipeline(&engine, 3).await;
    let manager = TaskHandoffManager::new(engine.clone(), kernel.clone(), bus.clone());

    // Step 0 -> Step 1.
    let output_0 = json!({"step": 0, "data": "research results"});
    let triggered = manager
        .on_step_completed(&parent_id, 0, output_0.clone())
        .await
        .unwrap();
    assert!(triggered, "should trigger step 1");

    // Step 1 -> Step 2.
    let output_1 = json!({"step": 1, "data": "analysis results"});
    let triggered = manager
        .on_step_completed(&parent_id, 1, output_1.clone())
        .await
        .unwrap();
    assert!(triggered, "should trigger step 2");

    // Step 2 -> Parent complete.
    let output_2 = json!({"step": 2, "data": "final report"});
    let triggered = manager
        .on_step_completed(&parent_id, 2, output_2.clone())
        .await
        .unwrap();
    assert!(!triggered, "no more steps; parent should complete");

    // Verify all steps completed.
    for (i, step_id) in step_ids.iter().enumerate() {
        let step = engine.status(step_id).await.unwrap();
        assert_eq!(step.status, TaskStatus::Completed, "step {} should be Completed", i);
    }

    // Verify parent completed with final output.
    let parent = engine.status(&parent_id).await.unwrap();
    assert_eq!(parent.status, TaskStatus::Completed);
    assert_eq!(parent.output, Some(output_2));
}

// ---------------------------------------------------------------------------
// Test: final step completion marks parent done
// ---------------------------------------------------------------------------

#[tokio::test]
async fn handoff_final_step_marks_parent_done() {
    let engine = Arc::new(TaskEngine::new());
    let kernel = Arc::new(AgentKernel::new());
    let bus = Arc::new(EventBus::new(64));

    // Single-step pipeline.
    let (parent_id, _step_ids) = setup_pipeline(&engine, 1).await;
    let manager = TaskHandoffManager::new(engine.clone(), kernel.clone(), bus.clone());

    let final_output = json!({"done": true, "artifacts": ["report.md"]});
    let triggered = manager
        .on_step_completed(&parent_id, 0, final_output.clone())
        .await
        .unwrap();

    assert!(!triggered, "single step — should not trigger next");

    let parent = engine.status(&parent_id).await.unwrap();
    assert_eq!(parent.status, TaskStatus::Completed);
    assert_eq!(parent.output, Some(final_output));
}

// ---------------------------------------------------------------------------
// Test: step_completed events emitted for each handoff
// ---------------------------------------------------------------------------

#[tokio::test]
async fn handoff_events_emitted_for_each_step() {
    let engine = Arc::new(TaskEngine::new());
    let kernel = Arc::new(AgentKernel::new());
    let bus = Arc::new(EventBus::new(64));
    let mut rx = bus.subscribe();

    let (parent_id, _step_ids) = setup_pipeline(&engine, 2).await;
    let manager = TaskHandoffManager::new(engine.clone(), kernel.clone(), bus.clone());

    // Complete step 0.
    let output = json!({"result": "step 0 output"});
    manager
        .on_step_completed(&parent_id, 0, output.clone())
        .await
        .unwrap();

    // Should receive a TaskStepCompleted event.
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
        other => panic!("expected TaskStepCompleted, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Test: nonexistent step returns error
// ---------------------------------------------------------------------------

#[tokio::test]
async fn handoff_nonexistent_step_returns_error() {
    let engine = Arc::new(TaskEngine::new());
    let kernel = Arc::new(AgentKernel::new());
    let bus = Arc::new(EventBus::new(64));

    let (parent_id, _step_ids) = setup_pipeline(&engine, 1).await;
    let manager = TaskHandoffManager::new(engine.clone(), kernel.clone(), bus.clone());

    // Step index 99 does not exist.
    let result = manager
        .on_step_completed(&parent_id, 99, json!({}))
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

// ---------------------------------------------------------------------------
// Test: idle agent gets assigned to next step
// ---------------------------------------------------------------------------

#[tokio::test]
async fn handoff_idle_agent_assigned_to_next_step() {
    let engine = Arc::new(TaskEngine::new());
    let kernel = Arc::new(AgentKernel::new());
    let bus = Arc::new(EventBus::new(64));

    let agent = AgentDescriptor::new("worker-1", AgentRole::Coder, Framework::Builtin, "claude-sonnet-4");
    let agent_id = agent.id.clone();
    kernel.register(agent).await;

    let (parent_id, step_ids) = setup_pipeline(&engine, 2).await;
    let manager = TaskHandoffManager::new(engine.clone(), kernel.clone(), bus.clone());

    manager
        .on_step_completed(&parent_id, 0, json!({}))
        .await
        .unwrap();

    // The idle agent should have been assigned to step 1.
    let agent_desc = kernel.get_agent(&agent_id).await.unwrap();
    assert_eq!(agent_desc.current_task_id, Some(step_ids[1].clone()));
}

// ---------------------------------------------------------------------------
// Test: output stored on completed step
// ---------------------------------------------------------------------------

#[tokio::test]
async fn handoff_output_stored_on_completed_step() {
    let engine = Arc::new(TaskEngine::new());
    let kernel = Arc::new(AgentKernel::new());
    let bus = Arc::new(EventBus::new(64));

    let (parent_id, step_ids) = setup_pipeline(&engine, 2).await;
    let manager = TaskHandoffManager::new(engine.clone(), kernel.clone(), bus.clone());

    let output = json!({"analysis": "important data", "confidence": 0.95});
    manager
        .on_step_completed(&parent_id, 0, output.clone())
        .await
        .unwrap();

    let step0 = engine.status(&step_ids[0]).await.unwrap();
    assert_eq!(step0.output, Some(output));
    assert!(step0.completed_at.is_some());
}
