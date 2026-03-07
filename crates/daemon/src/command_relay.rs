//! Daemon command relay — routes cloud command requests through security
//! classification, handles approval flow, executes locally, and returns results.
//!
//! Protocol messages (cloud ↔ daemon):
//! - `command.request`   — cloud asks daemon to run a command
//! - `command.accepted`  — daemon will execute (LOW risk auto-approved)
//! - `command.rejected`  — daemon refused (BLOCKED or user denied)
//! - `command.stdout`    — streaming stdout/stderr chunk
//! - `command.completed` — execution finished with exit code + duration

use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::executor::Executor;
use crate::security::{PermissionDecision, SecurityLayer};

// ---------------------------------------------------------------------------
// Protocol types
// ---------------------------------------------------------------------------

/// Inbound command request from the cloud orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequest {
    /// Unique identifier for this command invocation.
    pub id: String,
    /// The command type: `shell_exec`, `file_read`, `file_write`, `system_info`.
    pub command_type: String,
    /// Shell command string (for `shell_exec`) or structured payload.
    pub payload: serde_json::Value,
    /// Optional working directory override.
    #[serde(default)]
    pub cwd: Option<String>,
    /// Optional timeout in milliseconds.
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

/// Outbound message sent from the daemon back to the cloud.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CommandResponse {
    /// Daemon accepted the command and will execute it.
    #[serde(rename = "command.accepted")]
    Accepted { command_id: String },

    /// Daemon rejected the command (blocked or user denied).
    #[serde(rename = "command.rejected")]
    Rejected { command_id: String, reason: String },

    /// Streaming stdout/stderr chunk during execution.
    #[serde(rename = "command.stdout")]
    Stdout { command_id: String, data: String },

    /// Execution completed.
    #[serde(rename = "command.completed")]
    Completed {
        command_id: String,
        success: bool,
        exit_code: i32,
        stdout: String,
        stderr: String,
        duration_ms: u64,
    },
}

// ---------------------------------------------------------------------------
// ApprovalHandler trait
// ---------------------------------------------------------------------------

/// Trait for prompting the user when a command requires approval.
///
/// Implementations forward the approval prompt to a connected app or CLI client.
#[async_trait::async_trait]
pub trait ApprovalHandler: Send + Sync {
    /// Ask the user whether `command` should be allowed to run.
    /// Returns `true` if approved, `false` if denied.
    async fn request_approval(&self, command_id: &str, command: &str, reason: &str) -> bool;
}

/// Default handler that auto-denies (used when no client is connected).
pub struct DenyAllApprovalHandler;

#[async_trait::async_trait]
impl ApprovalHandler for DenyAllApprovalHandler {
    async fn request_approval(&self, command_id: &str, _command: &str, reason: &str) -> bool {
        warn!(command_id, reason, "No approval handler — auto-denying");
        false
    }
}

// ---------------------------------------------------------------------------
// CommandRelay
// ---------------------------------------------------------------------------

/// Orchestrates the full command lifecycle:
/// receive → classify → approve/deny → execute → respond.
pub struct CommandRelay {
    security: SecurityLayer,
    executor: Executor,
    approval_handler: Arc<dyn ApprovalHandler>,
}

impl CommandRelay {
    /// Create a new relay with default security and executor settings.
    pub fn new() -> Self {
        Self {
            security: SecurityLayer::new(),
            executor: Executor::default(),
            approval_handler: Arc::new(DenyAllApprovalHandler),
        }
    }

    /// Create a relay with a custom approval handler.
    pub fn with_approval_handler(approval_handler: Arc<dyn ApprovalHandler>) -> Self {
        Self {
            security: SecurityLayer::new(),
            executor: Executor::default(),
            approval_handler,
        }
    }

    /// Create a relay with fully custom components (primarily for testing).
    pub fn with_components(
        security: SecurityLayer,
        executor: Executor,
        approval_handler: Arc<dyn ApprovalHandler>,
    ) -> Self {
        Self {
            security,
            executor,
            approval_handler,
        }
    }

    /// Process a [`CommandRequest`] through the full lifecycle.
    ///
    /// Returns a channel receiver that yields one or more [`CommandResponse`]
    /// messages (accepted/rejected, optional stdout chunks, completed).
    pub async fn execute(&self, request: CommandRequest) -> mpsc::Receiver<CommandResponse> {
        let (tx, rx) = mpsc::channel(32);

        match request.command_type.as_str() {
            "shell_exec" => self.handle_shell_exec(request, tx).await,
            "file_read" => self.handle_file_read(request, tx).await,
            _ => {
                let _ = tx
                    .send(CommandResponse::Rejected {
                        command_id: request.id,
                        reason: format!("Unknown command type: {}", request.command_type),
                    })
                    .await;
            }
        }

        rx
    }

    // -- shell_exec --------------------------------------------------------

    async fn handle_shell_exec(&self, request: CommandRequest, tx: mpsc::Sender<CommandResponse>) {
        let command_str = match request.payload.as_str() {
            Some(s) => s.to_string(),
            None => match request.payload.get("command").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => {
                    let _ = tx
                        .send(CommandResponse::Rejected {
                            command_id: request.id,
                            reason: "Missing 'command' in payload".to_string(),
                        })
                        .await;
                    return;
                }
            },
        };

        // 1. Security classification
        let decision = self.security.check_permission(&command_str);
        debug!(command_id = %request.id, ?decision, "Security decision");

        match decision {
            PermissionDecision::Deny { reason } => {
                let _ = tx
                    .send(CommandResponse::Rejected {
                        command_id: request.id,
                        reason,
                    })
                    .await;
                return;
            }
            PermissionDecision::RequireApproval { reason } => {
                let approved = self
                    .approval_handler
                    .request_approval(&request.id, &command_str, &reason)
                    .await;
                if !approved {
                    let _ = tx
                        .send(CommandResponse::Rejected {
                            command_id: request.id,
                            reason: format!("User denied: {reason}"),
                        })
                        .await;
                    return;
                }
            }
            PermissionDecision::Allow | PermissionDecision::AllowWithLogging => {
                // Proceed
            }
        }

        // 2. Accepted
        let _ = tx
            .send(CommandResponse::Accepted {
                command_id: request.id.clone(),
            })
            .await;

        // 3. Execute
        info!(command_id = %request.id, %command_str, "Executing command");
        let result = self
            .executor
            .execute(&command_str, request.timeout_ms, request.cwd.as_deref())
            .await;

        match result {
            Ok(exec) => {
                let _ = tx
                    .send(CommandResponse::Completed {
                        command_id: request.id,
                        success: exec.success,
                        exit_code: exec.exit_code,
                        stdout: exec.stdout,
                        stderr: exec.stderr,
                        duration_ms: exec.duration_ms,
                    })
                    .await;
            }
            Err(e) => {
                let _ = tx
                    .send(CommandResponse::Completed {
                        command_id: request.id,
                        success: false,
                        exit_code: -1,
                        stdout: String::new(),
                        stderr: e.to_string(),
                        duration_ms: 0,
                    })
                    .await;
            }
        }
    }

    // -- file_read ---------------------------------------------------------

    async fn handle_file_read(&self, request: CommandRequest, tx: mpsc::Sender<CommandResponse>) {
        let path = match request.payload.as_str() {
            Some(s) => s.to_string(),
            None => match request.payload.get("path").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => {
                    let _ = tx
                        .send(CommandResponse::Rejected {
                            command_id: request.id,
                            reason: "Missing 'path' in payload".to_string(),
                        })
                        .await;
                    return;
                }
            },
        };

        // Validate path is within sandbox
        if let Err(e) = self.security.validate_path(&path) {
            let _ = tx
                .send(CommandResponse::Rejected {
                    command_id: request.id,
                    reason: e.to_string(),
                })
                .await;
            return;
        }

        // Accepted
        let _ = tx
            .send(CommandResponse::Accepted {
                command_id: request.id.clone(),
            })
            .await;

        // Read the file
        let start = Instant::now();
        match tokio::fs::read_to_string(&path).await {
            Ok(contents) => {
                let _ = tx
                    .send(CommandResponse::Completed {
                        command_id: request.id,
                        success: true,
                        exit_code: 0,
                        stdout: contents,
                        stderr: String::new(),
                        duration_ms: start.elapsed().as_millis() as u64,
                    })
                    .await;
            }
            Err(e) => {
                let _ = tx
                    .send(CommandResponse::Completed {
                        command_id: request.id,
                        success: false,
                        exit_code: 1,
                        stdout: String::new(),
                        stderr: e.to_string(),
                        duration_ms: start.elapsed().as_millis() as u64,
                    })
                    .await;
            }
        }
    }
}

impl Default for CommandRelay {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// Approval handler that always approves.
    struct AlwaysApprove;

    #[async_trait::async_trait]
    impl ApprovalHandler for AlwaysApprove {
        async fn request_approval(&self, _id: &str, _cmd: &str, _reason: &str) -> bool {
            true
        }
    }

    /// Approval handler that always denies.
    struct AlwaysDeny;

    #[async_trait::async_trait]
    impl ApprovalHandler for AlwaysDeny {
        async fn request_approval(&self, _id: &str, _cmd: &str, _reason: &str) -> bool {
            false
        }
    }

    /// Approval handler that records whether it was called.
    struct RecordingHandler {
        called: AtomicBool,
        approve: bool,
    }

    impl RecordingHandler {
        fn new(approve: bool) -> Self {
            Self {
                called: AtomicBool::new(false),
                approve,
            }
        }

        fn was_called(&self) -> bool {
            self.called.load(Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl ApprovalHandler for RecordingHandler {
        async fn request_approval(&self, _id: &str, _cmd: &str, _reason: &str) -> bool {
            self.called.store(true, Ordering::SeqCst);
            self.approve
        }
    }

    fn make_request(id: &str, cmd_type: &str, payload: serde_json::Value) -> CommandRequest {
        CommandRequest {
            id: id.to_string(),
            command_type: cmd_type.to_string(),
            payload,
            cwd: None,
            timeout_ms: Some(5000),
        }
    }

    async fn collect_responses(mut rx: mpsc::Receiver<CommandResponse>) -> Vec<CommandResponse> {
        let mut out = Vec::new();
        while let Some(msg) = rx.recv().await {
            out.push(msg);
        }
        out
    }

    // -- Low-risk shell commands auto-execute --

    #[tokio::test]
    async fn low_risk_auto_executes() {
        let relay = CommandRelay::new();
        let req = make_request("t1", "shell_exec", serde_json::json!("echo hello"));
        let responses = collect_responses(relay.execute(req).await).await;

        assert_eq!(responses.len(), 2, "expected accepted + completed");
        assert!(
            matches!(&responses[0], CommandResponse::Accepted { command_id } if command_id == "t1")
        );
        match &responses[1] {
            CommandResponse::Completed {
                command_id,
                success,
                stdout,
                ..
            } => {
                assert_eq!(command_id, "t1");
                assert!(success);
                assert_eq!(stdout.trim(), "hello");
            }
            other => panic!("expected Completed, got {:?}", other),
        }
    }

    // -- Blocked commands are rejected --

    #[tokio::test]
    async fn blocked_command_rejected() {
        let relay = CommandRelay::new();
        let req = make_request("t2", "shell_exec", serde_json::json!("rm -rf /"));
        let responses = collect_responses(relay.execute(req).await).await;

        assert_eq!(responses.len(), 1);
        assert!(
            matches!(&responses[0], CommandResponse::Rejected { command_id, .. } if command_id == "t2")
        );
    }

    // -- High-risk with approval granted --

    #[tokio::test]
    async fn high_risk_approved_executes() {
        let relay = CommandRelay::with_approval_handler(Arc::new(AlwaysApprove));
        // "rm" is high-risk but we're removing a nonexistent file — safe for testing
        let req = make_request(
            "t3",
            "shell_exec",
            serde_json::json!("rm /tmp/d1_relay_test_nonexistent_file_xyz 2>/dev/null; echo done"),
        );
        let responses = collect_responses(relay.execute(req).await).await;

        // Should have accepted + completed (rm of nonexistent file is ok with 2>/dev/null)
        assert!(responses.len() >= 2);
        assert!(matches!(&responses[0], CommandResponse::Accepted { .. }));
        assert!(matches!(
            &responses.last().unwrap(),
            CommandResponse::Completed { .. }
        ));
    }

    // -- High-risk with approval denied --

    #[tokio::test]
    async fn high_risk_denied_rejected() {
        let relay = CommandRelay::with_approval_handler(Arc::new(AlwaysDeny));
        let req = make_request("t4", "shell_exec", serde_json::json!("sudo ls"));
        let responses = collect_responses(relay.execute(req).await).await;

        assert_eq!(responses.len(), 1);
        match &responses[0] {
            CommandResponse::Rejected { command_id, reason } => {
                assert_eq!(command_id, "t4");
                assert!(reason.contains("denied"), "reason: {}", reason);
            }
            other => panic!("expected Rejected, got {:?}", other),
        }
    }

    // -- Approval handler is only called for high-risk commands --

    #[tokio::test]
    async fn approval_not_called_for_low_risk() {
        let handler = Arc::new(RecordingHandler::new(true));
        let relay = CommandRelay::with_components(
            SecurityLayer::new(),
            Executor::default(),
            handler.clone(),
        );
        let req = make_request("t5", "shell_exec", serde_json::json!("echo safe"));
        let _ = collect_responses(relay.execute(req).await).await;

        assert!(
            !handler.was_called(),
            "approval handler should not be called for low-risk"
        );
    }

    // -- Unknown command type rejected --

    #[tokio::test]
    async fn unknown_command_type_rejected() {
        let relay = CommandRelay::new();
        let req = make_request("t6", "unknown_type", serde_json::json!({}));
        let responses = collect_responses(relay.execute(req).await).await;

        assert_eq!(responses.len(), 1);
        assert!(matches!(
            &responses[0],
            CommandResponse::Rejected { command_id, reason }
            if command_id == "t6" && reason.contains("Unknown command type")
        ));
    }

    // -- Missing payload rejected --

    #[tokio::test]
    async fn missing_command_in_payload_rejected() {
        let relay = CommandRelay::new();
        let req = make_request("t7", "shell_exec", serde_json::json!({"not_command": true}));
        let responses = collect_responses(relay.execute(req).await).await;

        assert_eq!(responses.len(), 1);
        assert!(matches!(
            &responses[0],
            CommandResponse::Rejected { command_id, reason }
            if command_id == "t7" && reason.contains("Missing")
        ));
    }

    // -- file_read within sandbox succeeds --

    #[tokio::test]
    async fn file_read_success() {
        // Create a temp file to read
        let tmp = std::env::temp_dir().join("d1_relay_test_read.txt");
        tokio::fs::write(&tmp, "relay test content").await.unwrap();

        let relay = CommandRelay::with_components(
            SecurityLayer::with_sandbox_root(std::env::temp_dir()),
            Executor::default(),
            Arc::new(DenyAllApprovalHandler),
        );
        let req = make_request("t8", "file_read", serde_json::json!(tmp.to_str().unwrap()));
        let responses = collect_responses(relay.execute(req).await).await;

        assert_eq!(responses.len(), 2);
        assert!(matches!(&responses[0], CommandResponse::Accepted { .. }));
        match &responses[1] {
            CommandResponse::Completed {
                success, stdout, ..
            } => {
                assert!(success);
                assert_eq!(stdout, "relay test content");
            }
            other => panic!("expected Completed, got {:?}", other),
        }

        let _ = tokio::fs::remove_file(&tmp).await;
    }

    // -- file_read outside sandbox rejected --

    #[tokio::test]
    async fn file_read_outside_sandbox_rejected() {
        let narrow_sandbox = std::env::temp_dir().join("d1_relay_narrow_sandbox");
        tokio::fs::create_dir_all(&narrow_sandbox).await.ok();

        let relay = CommandRelay::with_components(
            SecurityLayer::with_sandbox_root(narrow_sandbox.clone()),
            Executor::default(),
            Arc::new(DenyAllApprovalHandler),
        );
        let req = make_request("t9", "file_read", serde_json::json!("/etc/hosts"));
        let responses = collect_responses(relay.execute(req).await).await;

        assert_eq!(responses.len(), 1);
        assert!(matches!(&responses[0], CommandResponse::Rejected { .. }));

        let _ = tokio::fs::remove_dir(&narrow_sandbox).await;
    }

    // -- Command with structured payload --

    #[tokio::test]
    async fn shell_exec_structured_payload() {
        let relay = CommandRelay::new();
        let req = make_request(
            "t10",
            "shell_exec",
            serde_json::json!({"command": "echo structured"}),
        );
        let responses = collect_responses(relay.execute(req).await).await;

        assert_eq!(responses.len(), 2);
        match &responses[1] {
            CommandResponse::Completed {
                success, stdout, ..
            } => {
                assert!(success);
                assert_eq!(stdout.trim(), "structured");
            }
            other => panic!("expected Completed, got {:?}", other),
        }
    }
}
