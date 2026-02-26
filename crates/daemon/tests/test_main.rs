//! Tests for daemon message handling: COMMAND → COMMAND_RESULT + PROGRESS_UPDATE

use prost::Message as ProstMessage;
use d1_common::proto::{
    Envelope, MessageType, Command, CommandResult, ProgressUpdate,
};

fn make_command_envelope(session_id: &str, task_id: &str, command: &str, step: i32) -> Vec<u8> {
    let mut cmd = Command::default();
    cmd.id = "cmd-001".to_string();
    cmd.task_id = task_id.to_string();
    cmd.step_number = step;
    cmd.shell_command = command.to_string();
    cmd.timeout_ms = 5000;

    let mut env = Envelope::default();
    env.id = "env-001".to_string();
    env.session_id = session_id.to_string();
    env.timestamp_ms = 0;
    env.r#type = MessageType::Command as i32;
    env.payload = cmd.encode_to_vec();
    env.encode_to_vec()
}

fn decode_envelope(bytes: &[u8]) -> Envelope {
    Envelope::decode(bytes).expect("Failed to decode Envelope")
}

#[tokio::test]
async fn test_handle_command_returns_command_result() {
    use d1_daemon::message_handler::handle_command_message;

    let cmd_bytes = make_command_envelope("sess-001", "task-001", "echo sprint3", 1);
    let env = decode_envelope(&cmd_bytes);

    let responses = handle_command_message(&env).await.unwrap();

    assert!(
        responses.iter().any(|r| r.r#type == MessageType::CommandResult as i32),
        "Expected COMMAND_RESULT in responses"
    );

    let result_env = responses
        .iter()
        .find(|r| r.r#type == MessageType::CommandResult as i32)
        .unwrap();
    let result = CommandResult::decode(result_env.payload.as_slice()).unwrap();
    assert!(result.success, "echo should succeed");
    assert_eq!(result.command_id, "cmd-001");
    assert!(result.stdout.contains("sprint3"), "stdout should contain 'sprint3'");
}

#[tokio::test]
async fn test_handle_command_returns_progress_update() {
    use d1_daemon::message_handler::handle_command_message;

    let cmd_bytes = make_command_envelope("sess-001", "task-001", "echo progress_test", 2);
    let env = decode_envelope(&cmd_bytes);

    let responses = handle_command_message(&env).await.unwrap();

    assert!(
        responses.iter().any(|r| r.r#type == MessageType::ProgressUpdate as i32),
        "Expected PROGRESS_UPDATE in responses"
    );

    let progress_env = responses
        .iter()
        .find(|r| r.r#type == MessageType::ProgressUpdate as i32)
        .unwrap();
    let progress = ProgressUpdate::decode(progress_env.payload.as_slice()).unwrap();
    assert_eq!(progress.task_id, "task-001");
    assert_eq!(progress.step_number, 2);
}

#[tokio::test]
async fn test_handle_failed_command_result_is_not_success() {
    use d1_daemon::message_handler::handle_command_message;

    let cmd_bytes = make_command_envelope("sess-001", "task-001", "false", 1);
    let env = decode_envelope(&cmd_bytes);

    let responses = handle_command_message(&env).await.unwrap();
    let result_env = responses
        .iter()
        .find(|r| r.r#type == MessageType::CommandResult as i32)
        .unwrap();
    let result = CommandResult::decode(result_env.payload.as_slice()).unwrap();
    assert!(!result.success, "`false` command should fail");
    assert_ne!(result.exit_code, 0);
}
