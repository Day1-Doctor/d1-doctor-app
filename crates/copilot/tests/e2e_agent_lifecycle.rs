//! E2E Agent Lifecycle Tests — register agents, apply triggers, verify state changes.
//! All tests use in-memory runtime with no actual LLM calls.

use std::sync::Arc;

use d1_copilot_lib::station::events::EventBus;
use d1_copilot_lib::station::kernel::agent::{AgentDescriptor, AgentRole, Framework};
use d1_copilot_lib::station::kernel::{AgentKernel, AgentStatus, Trigger};
use d1_copilot_lib::station::runtime::BuiltinRuntime;

fn make_kernel() -> Arc<AgentKernel> {
    Arc::new(AgentKernel::new())
}

fn make_runtime() -> (BuiltinRuntime, Arc<AgentKernel>, Arc<EventBus>) {
    let kernel = Arc::new(AgentKernel::new());
    let event_bus = Arc::new(EventBus::new(128));
    let runtime = BuiltinRuntime::new(
        Arc::clone(&kernel),
        Arc::clone(&event_bus),
        "https://gateway.day1.doctor/v1",
    );
    (runtime, kernel, event_bus)
}

#[tokio::test]
async fn lifecycle_register_and_deregister() {
    let kernel = make_kernel();
    let agent = AgentDescriptor::new("test-agent", AgentRole::Coder, Framework::Builtin);
    let id = agent.id.clone();

    kernel.register(agent).await;
    assert_eq!(kernel.agent_count().await, 1);

    let removed = kernel.deregister(&id).await.unwrap();
    assert_eq!(removed.name, "test-agent");
    assert_eq!(kernel.agent_count().await, 0);
}

#[tokio::test]
async fn lifecycle_full_fsm_cycle() {
    let kernel = make_kernel();
    let agent = AgentDescriptor::new("worker", AgentRole::Writer, Framework::Generic);
    let id = kernel.register(agent).await;

    // idle -> working
    let (old, new) = kernel.apply_trigger(&id, Trigger::TaskAssign).await.unwrap();
    assert_eq!(old, AgentStatus::Idle);
    assert_eq!(new, AgentStatus::Working);

    // working -> thinking
    let (_, new) = kernel.apply_trigger(&id, Trigger::LlmCallStart).await.unwrap();
    assert_eq!(new, AgentStatus::Thinking);

    // thinking -> executing
    let (_, new) = kernel.apply_trigger(&id, Trigger::ToolCallStart).await.unwrap();
    assert_eq!(new, AgentStatus::Executing);

    // executing -> working
    let (_, new) = kernel.apply_trigger(&id, Trigger::ToolCallEnd).await.unwrap();
    assert_eq!(new, AgentStatus::Working);

    // working -> idle (complete)
    let (_, new) = kernel.apply_trigger(&id, Trigger::TaskComplete).await.unwrap();
    assert_eq!(new, AgentStatus::Idle);
}

#[tokio::test]
async fn lifecycle_error_and_recovery() {
    let kernel = make_kernel();
    let agent = AgentDescriptor::new("fragile", AgentRole::Coder, Framework::Builtin);
    let id = kernel.register(agent).await;

    // idle -> working
    kernel.apply_trigger(&id, Trigger::TaskAssign).await.unwrap();

    // working -> error
    let (_, new) = kernel
        .apply_trigger(
            &id,
            Trigger::ErrorOccurred {
                message: "connection timeout".into(),
            },
        )
        .await
        .unwrap();
    assert_eq!(new, AgentStatus::Error);

    // error -> idle (resume)
    let (_, new) = kernel.apply_trigger(&id, Trigger::Resume).await.unwrap();
    assert_eq!(new, AgentStatus::Idle);
}

#[tokio::test]
async fn lifecycle_approval_flow() {
    let kernel = make_kernel();
    let agent = AgentDescriptor::new("cautious", AgentRole::Operator, Framework::Builtin);
    let id = kernel.register(agent).await;

    // idle -> working
    kernel.apply_trigger(&id, Trigger::TaskAssign).await.unwrap();

    // working -> paused (approval needed)
    let (_, new) = kernel.apply_trigger(&id, Trigger::ApprovalNeeded).await.unwrap();
    assert_eq!(new, AgentStatus::Paused);

    // paused -> working (approved)
    let (_, new) = kernel.apply_trigger(&id, Trigger::ApprovalGranted).await.unwrap();
    assert_eq!(new, AgentStatus::Working);
}

#[tokio::test]
async fn lifecycle_invalid_transition_rejected() {
    let kernel = make_kernel();
    let agent = AgentDescriptor::new("strict", AgentRole::Analyst, Framework::Builtin);
    let id = kernel.register(agent).await;

    // Cannot go from idle directly to thinking
    let result = kernel.apply_trigger(&id, Trigger::LlmCallStart).await;
    assert!(result.is_err());

    // Agent should still be idle after rejected transition
    let a = kernel.get_agent(&id).await.unwrap();
    assert_eq!(a.status, AgentStatus::Idle);
}

#[tokio::test]
async fn lifecycle_builtin_runtime_agents_have_correct_roles() {
    let (runtime, kernel, _bus) = make_runtime();
    runtime.initialize().await.unwrap();

    let agents = kernel.list_agents().await;
    assert_eq!(agents.len(), 6);

    let roles: Vec<AgentRole> = {
        let mut r: Vec<_> = agents.iter().map(|a| a.role).collect();
        r.sort_by_key(|role| format!("{:?}", role));
        r
    };

    // Should have one of each role
    assert!(roles.contains(&AgentRole::Orchestrator));
    assert!(roles.contains(&AgentRole::Researcher));
    assert!(roles.contains(&AgentRole::Analyst));
    assert!(roles.contains(&AgentRole::Writer));
    assert!(roles.contains(&AgentRole::Coder));
    assert!(roles.contains(&AgentRole::Operator));
}
