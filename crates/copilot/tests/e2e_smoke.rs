//! E2E Smoke Tests — verify that the Station Runtime can be created
//! and responds to basic operations without any actual LLM calls.

use std::sync::Arc;

use d1_copilot_lib::station::events::{AgentEvent, EventBus, EventType};
use d1_copilot_lib::station::kernel::AgentKernel;
use d1_copilot_lib::station::runtime::BuiltinRuntime;

/// Helper: create an in-memory runtime with kernel + event bus.
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
async fn smoke_runtime_creates_successfully() {
    let (runtime, _kernel, _bus) = make_runtime();
    assert_eq!(runtime.gateway_url(), "https://gateway.day1.doctor/v1");
}

#[tokio::test]
async fn smoke_runtime_initializes_six_agents() {
    let (runtime, kernel, _bus) = make_runtime();
    let ids = runtime.initialize().await.unwrap();
    assert_eq!(ids.len(), 6, "should register exactly 6 agents");
    assert_eq!(kernel.agent_count().await, 6);
}

#[tokio::test]
async fn smoke_event_bus_subscribe_and_receive() {
    let bus = EventBus::new(64);
    let mut rx = bus.subscribe();

    let event = AgentEvent {
        id: "test-evt-1".to_string(),
        agent_id: "agent-1".to_string(),
        timestamp: chrono::Utc::now(),
        event_type: EventType::AgentStateChanged {
            from: "idle".to_string(),
            to: "working".to_string(),
        },
    };

    bus.publish(event).await;

    let received = rx.try_recv().expect("should receive published event");
    assert_eq!(received.id, "test-evt-1");
    assert_eq!(received.agent_id, "agent-1");
}

#[tokio::test]
async fn smoke_event_bus_history_works() {
    let bus = EventBus::new(64);

    for i in 0..5 {
        let event = AgentEvent {
            id: format!("hist-{}", i),
            agent_id: "agent-1".to_string(),
            timestamp: chrono::Utc::now(),
            event_type: EventType::AgentStateChanged {
                from: "idle".to_string(),
                to: "working".to_string(),
            },
        };
        bus.publish(event).await;
    }

    let history = bus.history(10).await;
    assert_eq!(history.len(), 5);
    assert_eq!(history[0].id, "hist-0");
    assert_eq!(history[4].id, "hist-4");
}
