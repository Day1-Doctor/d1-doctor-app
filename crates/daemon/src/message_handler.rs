//! Message routing for the daemon WebSocket event loop.

use anyhow::Result;
use prost::Message as ProstMessage;
use d1_common::proto::{
    Envelope, MessageType, Command, CommandResult, ProgressUpdate,
};
use crate::executor::Executor;
use uuid::Uuid;

pub async fn handle_command_message(envelope: &Envelope) -> Result<Vec<Envelope>> {
    let cmd = Command::decode(envelope.payload.as_slice())?;
    let session_id = envelope.session_id.clone();

    let executor = Executor::new();
    let exec_result = executor
        .run(&cmd.shell_command, ((cmd.timeout_ms / 1000).max(5)) as u64)
        .await?;

    let mut cmd_result = CommandResult::default();
    cmd_result.command_id = cmd.id.clone();
    cmd_result.task_id = cmd.task_id.clone();
    cmd_result.success = exec_result.success;
    cmd_result.stdout = exec_result.stdout.clone();
    cmd_result.stderr = exec_result.stderr.clone();
    cmd_result.exit_code = exec_result.exit_code;
    cmd_result.duration_ms = exec_result.duration_ms as i64;

    let result_env = make_envelope(
        &session_id,
        MessageType::CommandResult,
        cmd_result.encode_to_vec(),
    );

    let mut progress = ProgressUpdate::default();
    progress.task_id = cmd.task_id.clone();
    progress.step_number = cmd.step_number;
    progress.message = if exec_result.success {
        format!("Step {} completed: {}", cmd.step_number, cmd.shell_command)
    } else {
        format!("Step {} failed: {}", cmd.step_number, exec_result.stderr.trim())
    };
    progress.percent_complete = 0;

    let progress_env = make_envelope(
        &session_id,
        MessageType::ProgressUpdate,
        progress.encode_to_vec(),
    );

    Ok(vec![progress_env, result_env])
}

fn make_envelope(session_id: &str, msg_type: MessageType, payload: Vec<u8>) -> Envelope {
    let mut env = Envelope::default();
    env.id = Uuid::new_v4().to_string();
    env.session_id = session_id.to_string();
    env.timestamp_ms = now_ms();
    env.r#type = msg_type as i32;
    env.payload = payload;
    env
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
