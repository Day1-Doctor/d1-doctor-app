//! `d1-doctor install <package>` command.

use anyhow::{Context, Result};
use futures::{SinkExt, StreamExt};
use prost::Message as ProstMessage;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use d1_common::proto::{
    Envelope, MessageType, UserRequest, PlanProposal, PlanApproval, ApprovalAction,
};

const DAEMON_WS_URL: &str = "ws://127.0.0.1:3030/ws";

pub async fn handle(package: &str) -> Result<()> {
    run_install(package, DAEMON_WS_URL, /*auto_approve=*/ false).await
}

pub async fn run_install(package: &str, ws_url: &str, auto_approve: bool) -> Result<()> {
    println!("Connecting to d1-doctor daemon...");

    let (mut ws, _) = connect_async(ws_url)
        .await
        .context("Could not connect to daemon. Is `d1-daemon` running?")?;

    // Build and send UserRequest
    let mut user_req = UserRequest::default();
    user_req.text = format!("install {}", package);

    let mut env = Envelope::default();
    env.id = uuid_v4();
    env.session_id = uuid_v4();
    env.timestamp_ms = now_ms();
    env.r#type = MessageType::UserRequest as i32;
    env.payload = user_req.encode_to_vec();

    ws.send(Message::Binary(env.encode_to_vec()))
        .await
        .context("Failed to send UserRequest to daemon")?;

    println!("Waiting for plan from orchestrator...");

    // Receive PLAN_PROPOSAL
    let msg = ws
        .next()
        .await
        .context("Daemon closed connection before sending plan")?
        .context("WebSocket error receiving plan")?;

    let bytes = match msg {
        Message::Binary(b) => b,
        other => anyhow::bail!("Expected binary message, got: {:?}", other),
    };

    let response_env = Envelope::decode(bytes.as_slice())
        .context("Failed to decode Envelope from daemon")?;

    if response_env.r#type != MessageType::PlanProposal as i32 {
        anyhow::bail!(
            "Expected PLAN_PROPOSAL (type={}), got type={}",
            MessageType::PlanProposal as i32,
            response_env.r#type
        );
    }

    let proposal = PlanProposal::decode(response_env.payload.as_slice())
        .context("Failed to decode PlanProposal")?;

    // Display the plan
    println!();
    println!("Plan: {}", proposal.summary);
    println!("Steps:");
    for step in &proposal.steps {
        println!("  {}. [{}] {}", step.step_number, step.agent_name, step.description);
    }
    println!("Estimated credits: {}", proposal.estimated_credits);
    println!();

    // Prompt or use auto mode
    let approved = if auto_approve {
        println!("(auto-approve enabled)");
        true
    } else {
        prompt_approval()?
    };

    // Send PLAN_APPROVAL
    let mut approval = PlanApproval::default();
    approval.task_id = proposal.task_id.clone();
    approval.action = if approved {
        ApprovalAction::Approve as i32
    } else {
        ApprovalAction::Reject as i32
    };

    let mut approval_env = Envelope::default();
    approval_env.id = uuid_v4();
    approval_env.session_id = response_env.session_id.clone();
    approval_env.timestamp_ms = now_ms();
    approval_env.r#type = MessageType::PlanApproval as i32;
    approval_env.payload = approval.encode_to_vec();

    ws.send(Message::Binary(approval_env.encode_to_vec()))
        .await
        .context("Failed to send PlanApproval")?;

    if approved {
        println!("Plan approved — execution started.");
        println!("Watch progress with: d1-doctor status");
    } else {
        println!("Plan rejected. Nothing was installed.");
    }

    Ok(())
}

fn prompt_approval() -> Result<bool> {
    use std::io::Write;
    print!("Approve this plan? [Y/n]: ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_lowercase();
    Ok(trimmed.is_empty() || trimmed == "y" || trimmed == "yes")
}

fn uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
