use chrono::Utc;
use tokio::sync::broadcast;

use super::{AgentEvent, EventBus, EventType};

/// Helper to create a test event with the given agent_id and event_type.
fn make_event(agent_id: &str, event_type: EventType) -> AgentEvent {
    AgentEvent {
        id: uuid::Uuid::new_v4().to_string(),
        agent_id: agent_id.to_string(),
        timestamp: Utc::now(),
        event_type,
    }
}

fn state_change(from: &str, to: &str) -> EventType {
    EventType::AgentStateChanged {
        from: from.to_string(),
        to: to.to_string(),
    }
}

#[tokio::test]
async fn test_publish_subscribe() {
    let bus = EventBus::new(64);
    let mut rx = bus.subscribe();

    let evt = make_event("agent-1", state_change("idle", "running"));
    bus.publish(evt.clone()).await;

    let received = rx.recv().await.unwrap();
    assert_eq!(received.agent_id, "agent-1");
    match &received.event_type {
        EventType::AgentStateChanged { from, to } => {
            assert_eq!(from, "idle");
            assert_eq!(to, "running");
        }
        other => panic!("unexpected event type: {:?}", other),
    }
}

#[tokio::test]
async fn test_multiple_subscribers() {
    let bus = EventBus::new(64);
    let mut rx1 = bus.subscribe();
    let mut rx2 = bus.subscribe();
    let mut rx3 = bus.subscribe();

    let evt = make_event("agent-2", state_change("running", "done"));
    bus.publish(evt.clone()).await;

    for rx in [&mut rx1, &mut rx2, &mut rx3] {
        let received = rx.recv().await.unwrap();
        assert_eq!(received.agent_id, "agent-2");
    }

    assert_eq!(bus.subscriber_count(), 3);
}

#[tokio::test]
async fn test_concurrent_publishers() {
    let bus = std::sync::Arc::new(EventBus::new(64));
    let mut rx = bus.subscribe();

    let bus1 = bus.clone();
    let bus2 = bus.clone();

    let h1 = tokio::spawn(async move {
        bus1.publish(make_event("agent-a", state_change("idle", "running")))
            .await;
    });
    let h2 = tokio::spawn(async move {
        bus2.publish(make_event(
            "agent-b",
            EventType::TokenStream {
                delta: 10,
                total: 100,
            },
        ))
        .await;
    });

    h1.await.unwrap();
    h2.await.unwrap();

    let mut ids = Vec::new();
    // We expect exactly 2 events -- read with a short timeout so the test
    // does not hang if something goes wrong.
    for _ in 0..2 {
        match tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv()).await {
            Ok(Ok(e)) => ids.push(e.agent_id),
            Ok(Err(broadcast::error::RecvError::Lagged(_))) => {
                panic!("receiver lagged");
            }
            Ok(Err(broadcast::error::RecvError::Closed)) => {
                panic!("channel closed");
            }
            Err(_) => panic!("timed out waiting for event"),
        }
    }
    ids.sort();
    assert_eq!(ids, vec!["agent-a", "agent-b"]);
}

#[tokio::test]
async fn test_history() {
    let bus = EventBus::new(64);

    for i in 0..5 {
        bus.publish(make_event(
            &format!("agent-{}", i),
            state_change("idle", "running"),
        ))
        .await;
    }

    let hist = bus.history(10).await;
    assert_eq!(hist.len(), 5);
    assert_eq!(hist[0].agent_id, "agent-0");
    assert_eq!(hist[4].agent_id, "agent-4");

    // Request fewer than available.
    let hist2 = bus.history(3).await;
    assert_eq!(hist2.len(), 3);
    assert_eq!(hist2[0].agent_id, "agent-2");
}

#[tokio::test]
async fn test_history_for_agent() {
    let bus = EventBus::new(64);

    bus.publish(make_event("alice", state_change("idle", "running")))
        .await;
    bus.publish(make_event("bob", state_change("idle", "running")))
        .await;
    bus.publish(make_event(
        "alice",
        EventType::TokenStream {
            delta: 5,
            total: 50,
        },
    ))
    .await;
    bus.publish(make_event("bob", state_change("running", "done")))
        .await;
    bus.publish(make_event("alice", state_change("running", "done")))
        .await;

    let alice_events = bus.history_for_agent("alice", 10).await;
    assert_eq!(alice_events.len(), 3);
    for e in &alice_events {
        assert_eq!(e.agent_id, "alice");
    }

    let bob_events = bus.history_for_agent("bob", 10).await;
    assert_eq!(bob_events.len(), 2);

    // With a tighter limit.
    let alice_1 = bus.history_for_agent("alice", 1).await;
    assert_eq!(alice_1.len(), 1);
    // Should be the most recent alice event.
    assert_eq!(alice_1[0].agent_id, "alice");
}

#[tokio::test]
async fn test_history_cap() {
    let bus = EventBus::with_max_history(64, 5);

    for i in 0..10 {
        bus.publish(make_event(
            &format!("agent-{}", i),
            state_change("idle", "running"),
        ))
        .await;
    }

    let hist = bus.history(100).await;
    assert_eq!(hist.len(), 5);
    // Oldest surviving event should be agent-5 (0..4 were evicted).
    assert_eq!(hist[0].agent_id, "agent-5");
    assert_eq!(hist[4].agent_id, "agent-9");
}

#[tokio::test]
async fn test_event_serialization() {
    let evt = AgentEvent {
        id: "evt-001".to_string(),
        agent_id: "agent-x".to_string(),
        timestamp: chrono::DateTime::parse_from_rfc3339("2026-03-18T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc),
        event_type: EventType::AgentStateChanged {
            from: "idle".to_string(),
            to: "running".to_string(),
        },
    };

    let json = serde_json::to_value(&evt).unwrap();
    assert_eq!(json["id"], "evt-001");
    assert_eq!(json["agent_id"], "agent-x");
    assert_eq!(json["event_type"]["type"], "agent.state_changed");
    assert_eq!(json["event_type"]["payload"]["from"], "idle");
    assert_eq!(json["event_type"]["payload"]["to"], "running");

    // Round-trip.
    let deserialized: AgentEvent = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.id, "evt-001");
    match &deserialized.event_type {
        EventType::AgentStateChanged { from, to } => {
            assert_eq!(from, "idle");
            assert_eq!(to, "running");
        }
        other => panic!("unexpected: {:?}", other),
    }

    // Verify other variants serialize correctly.
    let cost_evt = make_event(
        "agent-y",
        EventType::CostUpdated {
            session_tokens: 1500,
            session_cost_dd: 0.42,
        },
    );
    let cost_json = serde_json::to_value(&cost_evt).unwrap();
    assert_eq!(cost_json["event_type"]["type"], "cost.updated");
    assert_eq!(cost_json["event_type"]["payload"]["session_tokens"], 1500);
}
