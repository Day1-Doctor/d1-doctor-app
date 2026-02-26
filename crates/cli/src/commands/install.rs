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

#[cfg(test)]
mod tests {
    use super::*;
    use d1_common::proto::{
        Envelope, MessageType, UserRequest, PlanProposal, PlanStep, PlanApproval, ApprovalAction,
    };
    use futures::{SinkExt, StreamExt};
    use prost::Message as ProstMessage;
    use tokio::net::TcpListener;
    use tokio_tungstenite::{accept_async, tungstenite::Message as WsMessage};

    // ─── helpers ──────────────────────────────────────────────────────────────

    /// Spawn a mock WebSocket server on a random port.
    /// `handler` receives `(sink, stream)` for one connection.
    /// Returns the `ws://…` URL to connect to.
    async fn mock_server<H, F>(handler: H) -> String
    where
        H: FnOnce(
                futures::stream::SplitSink<
                    tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
                    WsMessage,
                >,
                futures::stream::SplitStream<
                    tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
                >,
            ) -> F
            + Send
            + 'static,
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("ws://127.0.0.1:{}/ws", addr.port());

        tokio::spawn(async move {
            let (tcp, _) = listener.accept().await.unwrap();
            let ws = accept_async(tcp).await.unwrap();
            let (sink, stream) = ws.split();
            handler(sink, stream).await;
        });

        url
    }

    /// Build a binary WsMessage containing a PlanProposal envelope.
    fn plan_proposal_msg(task_id: &str, steps: usize) -> WsMessage {
        let proposal = PlanProposal {
            task_id: task_id.to_string(),
            summary: format!("Install package using Homebrew ({} step(s))", steps),
            steps: (1..=(steps as i32))
                .map(|i| PlanStep {
                    step_number: i,
                    description: format!("Step {}", i),
                    agent_name: "install_agent".to_string(),
                })
                .collect(),
            estimated_credits: steps as f32 * 0.5,
        };
        let mut env = Envelope::default();
        env.id = "server-env-1".to_string();
        env.session_id = "sess-1".to_string();
        env.r#type = MessageType::PlanProposal as i32;
        env.payload = proposal.encode_to_vec();
        WsMessage::Binary(env.encode_to_vec())
    }

    /// Decode an `Envelope` from a binary WsMessage.
    fn decode_envelope(msg: WsMessage) -> Envelope {
        let bytes = match msg {
            WsMessage::Binary(b) => b,
            other => panic!("Expected binary message, got: {:?}", other),
        };
        Envelope::decode(bytes.as_slice()).expect("Failed to decode Envelope")
    }

    // ─── tests ────────────────────────────────────────────────────────────────

    /// Happy path: CLI sends UserRequest, server proposes plan, CLI auto-approves.
    #[tokio::test]
    async fn test_install_auto_approve_full_flow() {
        let url = mock_server(|mut sink, mut stream| async move {
            // 1. Receive UserRequest
            let env = decode_envelope(stream.next().await.unwrap().unwrap());
            assert_eq!(env.r#type, MessageType::UserRequest as i32);
            let req = UserRequest::decode(env.payload.as_slice()).unwrap();
            assert_eq!(req.text, "install node");

            // 2. Send PlanProposal
            sink.send(plan_proposal_msg("task-001", 2)).await.unwrap();

            // 3. Receive PlanApproval
            let env = decode_envelope(stream.next().await.unwrap().unwrap());
            assert_eq!(env.r#type, MessageType::PlanApproval as i32);
            let approval = PlanApproval::decode(env.payload.as_slice()).unwrap();
            assert_eq!(approval.task_id, "task-001");
            assert_eq!(approval.action, ApprovalAction::Approve as i32);
        })
        .await;

        run_install("node", &url, true).await.unwrap();
    }

    /// CLI sends UserRequest with the correct package name embedded.
    #[tokio::test]
    async fn test_install_user_request_text_contains_package() {
        let url = mock_server(|mut sink, mut stream| async move {
            let env = decode_envelope(stream.next().await.unwrap().unwrap());
            let req = UserRequest::decode(env.payload.as_slice()).unwrap();
            // Package name "postgresql@16" must appear in the text
            assert!(
                req.text.contains("postgresql@16"),
                "UserRequest text should contain the package name, got: {}",
                req.text
            );
            sink.send(plan_proposal_msg("task-002", 1)).await.unwrap();
            // consume approval
            let _ = stream.next().await;
        })
        .await;

        run_install("postgresql@16", &url, true).await.unwrap();
    }

    /// Plan with many steps: all steps are listed, approval still sent.
    #[tokio::test]
    async fn test_install_plan_with_four_steps() {
        let url = mock_server(|mut sink, mut stream| async move {
            let _ = stream.next().await; // consume UserRequest
            sink.send(plan_proposal_msg("task-004", 4)).await.unwrap();
            // Receive approval — verify it approves task-004
            let env = decode_envelope(stream.next().await.unwrap().unwrap());
            let approval = PlanApproval::decode(env.payload.as_slice()).unwrap();
            assert_eq!(approval.task_id, "task-004");
            assert_eq!(approval.action, ApprovalAction::Approve as i32);
        })
        .await;

        run_install("vscode", &url, true).await.unwrap();
    }

    /// Approval envelope echoes back the task_id from the proposal.
    #[tokio::test]
    async fn test_install_approval_echoes_task_id() {
        let task_id = "task-uuid-xyz-987";
        let url = mock_server({
            let task_id = task_id.to_string();
            move |mut sink, mut stream| async move {
                let _ = stream.next().await;
                sink.send(plan_proposal_msg(&task_id, 1)).await.unwrap();
                let env = decode_envelope(stream.next().await.unwrap().unwrap());
                let approval = PlanApproval::decode(env.payload.as_slice()).unwrap();
                assert_eq!(approval.task_id, task_id);
            }
        })
        .await;

        run_install("git", &url, true).await.unwrap();
    }

    /// Daemon not running: connection refused → clean error message.
    #[tokio::test]
    async fn test_install_connection_refused_returns_error() {
        // Port 19991 is almost certainly not listening
        let result = run_install("node", "ws://127.0.0.1:19991/ws", false).await;
        assert!(result.is_err());
        let msg = format!("{:#}", result.unwrap_err());
        assert!(
            msg.to_lowercase().contains("connect") || msg.to_lowercase().contains("daemon"),
            "Expected error about connection/daemon, got: {}",
            msg
        );
    }

    /// Server sends wrong message type (Error) instead of PlanProposal → error.
    #[tokio::test]
    async fn test_install_wrong_message_type_returns_error() {
        let url = mock_server(|mut sink, mut stream| async move {
            let _ = stream.next().await; // consume UserRequest
            let mut env = Envelope::default();
            env.r#type = MessageType::Error as i32;
            env.payload = b"internal server error".to_vec();
            sink.send(WsMessage::Binary(env.encode_to_vec()))
                .await
                .unwrap();
        })
        .await;

        let result = run_install("node", &url, false).await;
        assert!(result.is_err());
        let msg = format!("{:#}", result.unwrap_err());
        assert!(
            msg.contains("PLAN_PROPOSAL") || msg.contains("type="),
            "Expected error about wrong message type, got: {}",
            msg
        );
    }

    /// Server closes connection immediately after UserRequest → error.
    #[tokio::test]
    async fn test_install_server_closes_before_plan_returns_error() {
        let url = mock_server(|_sink, mut stream| async move {
            let _ = stream.next().await; // consume UserRequest, then drop everything
        })
        .await;

        let result = run_install("node", &url, false).await;
        assert!(result.is_err());
        let msg = format!("{:#}", result.unwrap_err());
        assert!(
            msg.to_lowercase().contains("closed") || msg.to_lowercase().contains("plan"),
            "Expected error about closed connection, got: {}",
            msg
        );
    }

    /// Server sends a text frame instead of binary → error.
    #[tokio::test]
    async fn test_install_text_frame_returns_error() {
        let url = mock_server(|mut sink, mut stream| async move {
            let _ = stream.next().await;
            sink.send(WsMessage::Text(
                r#"{"error":"unexpected text frame"}"#.to_string(),
            ))
            .await
            .unwrap();
        })
        .await;

        let result = run_install("node", &url, false).await;
        assert!(result.is_err());
        let msg = format!("{:#}", result.unwrap_err());
        assert!(
            msg.to_lowercase().contains("binary") || msg.to_lowercase().contains("expected"),
            "Expected error about non-binary message, got: {}",
            msg
        );
    }
}
