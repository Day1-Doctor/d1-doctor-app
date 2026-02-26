//! Integration tests for the install command using a mock WebSocket server.

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};

async fn spawn_mock_orchestrator() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();

        // Receive UserRequest envelope
        let msg = ws.next().await.unwrap().unwrap();
        assert!(msg.is_binary(), "expected binary protobuf message");

        // Build a minimal PLAN_PROPOSAL response
        let response = build_plan_proposal_bytes("task-001");
        ws.send(Message::Binary(response)).await.unwrap();

        // Receive PLAN_APPROVAL (type=4)
        let _approval = ws.next().await.unwrap().unwrap();
    });

    addr
}

fn build_plan_proposal_bytes(task_id: &str) -> Vec<u8> {
    use prost::Message as _;
    use d1_common::proto::{Envelope, MessageType, PlanProposal, PlanStep};

    let mut proposal = PlanProposal::default();
    proposal.task_id = task_id.to_string();
    proposal.summary = "Install node via brew".to_string();
    proposal.estimated_credits = 1.0;
    let mut step = PlanStep::default();
    step.step_number = 1;
    step.description = "Run brew install node".to_string();
    step.agent_name = "executor".to_string();
    proposal.steps.push(step);

    let mut env = Envelope::default();
    env.id = "env-001".to_string();
    env.session_id = "sess-test".to_string();
    env.timestamp_ms = 0;
    env.r#type = MessageType::PlanProposal as i32;
    env.payload = proposal.encode_to_vec();
    env.encode_to_vec()
}

#[tokio::test]
async fn test_install_sends_user_request_and_receives_plan() {
    let addr = spawn_mock_orchestrator().await;
    let url = format!("ws://{}/ws", addr);

    let result = d1_doctor_cli::commands::install::run_install(
        "node",
        &url,
        /*auto_approve=*/ true,
    )
    .await;

    assert!(result.is_ok(), "install returned error: {:?}", result);
}

#[tokio::test]
async fn test_install_sends_reject_on_auto_reject() {
    let addr = spawn_mock_orchestrator().await;
    let url = format!("ws://{}/ws", addr);

    let result = d1_doctor_cli::commands::install::run_install(
        "node",
        &url,
        /*auto_approve=*/ false,
    )
    .await;

    assert!(result.is_ok(), "install reject returned error: {:?}", result);
}
