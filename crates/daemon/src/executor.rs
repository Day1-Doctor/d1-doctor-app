//! Local command executor with timeout enforcement and output limits.
//!
//! Provides three execution modes:
//! - `execute` — run a shell command with captured output
//! - `execute_script` — write a script to a temp file and execute it
//! - `dry_run` — parse a command and describe what it would do without running it

use serde::{Deserialize, Serialize};
use std::time::Instant;
use tokio::process::Command;

/// Maximum output size in bytes (50 KB). Output exceeding this limit is truncated.
const MAX_OUTPUT_BYTES: usize = 50 * 1024;

/// Default command timeout in milliseconds (30 seconds).
const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// Commands considered destructive for risk assessment in dry-run mode.
const DESTRUCTIVE_COMMANDS: &[&str] = &[
    "rm", "rmdir", "mkfs", "dd", "shred", "kill", "killall", "pkill",
    "shutdown", "reboot", "halt", "poweroff", "fdisk", "parted",
    "chmod", "chown", "chgrp", "mv", "truncate",
];

/// Commands that modify system state but are generally lower risk.
const MUTATING_COMMANDS: &[&str] = &[
    "cp", "mkdir", "touch", "ln", "install", "sed", "awk",
    "tee", "patch", "git", "npm", "cargo", "pip", "brew",
    "apt", "yum", "dnf", "pacman",
];

/// Result of executing a command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub timed_out: bool,
}

/// Result of a dry-run analysis of a command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunResult {
    pub command: String,
    pub binary: String,
    pub args: Vec<String>,
    pub description: String,
    pub risk_level: String,
}

/// Configurable command executor with timeout and output-size enforcement.
pub struct Executor {
    /// Default timeout applied when the caller does not specify one (milliseconds).
    pub default_timeout_ms: u64,
    /// Maximum number of bytes kept from stdout/stderr before truncation.
    pub max_output_bytes: usize,
}

impl Default for Executor {
    fn default() -> Self {
        Self {
            default_timeout_ms: DEFAULT_TIMEOUT_MS,
            max_output_bytes: MAX_OUTPUT_BYTES,
        }
    }
}

impl Executor {
    /// Create a new executor with the given defaults.
    pub fn new(default_timeout_ms: u64, max_output_bytes: usize) -> Self {
        Self {
            default_timeout_ms,
            max_output_bytes,
        }
    }

    /// Execute a shell command, capturing stdout and stderr.
    ///
    /// The command is run via `sh -c` so shell features (pipes, redirects, etc.)
    /// are available. Output is truncated to `max_output_bytes` and execution is
    /// aborted if `timeout_ms` (or the default) elapses.
    pub async fn execute(
        &self,
        command: &str,
        timeout_ms: Option<u64>,
        cwd: Option<&str>,
    ) -> anyhow::Result<ExecResult> {
        let timeout = std::time::Duration::from_millis(
            timeout_ms.unwrap_or(self.default_timeout_ms),
        );

        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let start = Instant::now();

        let result = tokio::time::timeout(timeout, cmd.output()).await;

        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(output)) => {
                let stdout = truncate_output(&output.stdout, self.max_output_bytes);
                let stderr = truncate_output(&output.stderr, self.max_output_bytes);
                let exit_code = output.status.code().unwrap_or(-1);

                Ok(ExecResult {
                    success: output.status.success(),
                    stdout,
                    stderr,
                    exit_code,
                    duration_ms,
                    timed_out: false,
                })
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("failed to execute command: {e}")),
            Err(_) => {
                // Timeout elapsed — the child process is dropped (killed) automatically.
                Ok(ExecResult {
                    success: false,
                    stdout: String::new(),
                    stderr: format!("command timed out after {timeout_ms} ms",
                        timeout_ms = timeout.as_millis()),
                    exit_code: -1,
                    duration_ms,
                    timed_out: true,
                })
            }
        }
    }

    /// Execute a script by writing it to a temporary file and invoking the
    /// given interpreter (default: `bash`).
    ///
    /// The temp file is cleaned up after execution regardless of outcome.
    pub async fn execute_script(
        &self,
        script: &str,
        interpreter: Option<&str>,
        timeout_ms: Option<u64>,
    ) -> anyhow::Result<ExecResult> {
        let interp = interpreter.unwrap_or("bash");

        // Create a unique temp file path.
        let tmp_dir = std::env::temp_dir();
        let file_name = format!("d1-exec-{}.sh", uuid::Uuid::new_v4());
        let tmp_path = tmp_dir.join(file_name);

        // Write the script content.
        tokio::fs::write(&tmp_path, script).await?;

        // Make executable (best-effort on Unix).
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o755);
            tokio::fs::set_permissions(&tmp_path, perms).await?;
        }

        let command = format!("{} {}", interp, tmp_path.display());
        let result = self.execute(&command, timeout_ms, None).await;

        // Clean up temp file regardless of execution outcome.
        let _ = tokio::fs::remove_file(&tmp_path).await;

        result
    }

    /// Analyse a command without executing it.
    ///
    /// Returns the parsed binary name, arguments, a human-readable description,
    /// and a risk level (`"low"`, `"medium"`, or `"high"`).
    pub fn dry_run(&self, command: &str) -> DryRunResult {
        let parts = shell_split(command);
        let (binary, args) = if parts.is_empty() {
            (String::new(), Vec::new())
        } else {
            (parts[0].clone(), parts[1..].to_vec())
        };

        // Extract just the binary name (strip path prefix).
        let binary_name = binary
            .rsplit('/')
            .next()
            .unwrap_or(&binary)
            .to_string();

        let risk_level = assess_risk(&binary_name, &args);
        let description = describe_command(&binary_name, &args);

        DryRunResult {
            command: command.to_string(),
            binary: binary_name,
            args,
            description,
            risk_level,
        }
    }
}

/// Truncate raw output bytes to a UTF-8 string of at most `max_bytes`.
fn truncate_output(bytes: &[u8], max_bytes: usize) -> String {
    if bytes.len() <= max_bytes {
        String::from_utf8_lossy(bytes).into_owned()
    } else {
        let truncated = &bytes[..max_bytes];
        let s = String::from_utf8_lossy(truncated).into_owned();
        format!("{s}\n... [output truncated at {max_bytes} bytes]")
    }
}

/// Minimal shell-style word splitting (handles double/single quotes).
fn shell_split(input: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            ' ' | '\t' if !in_single && !in_double => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            '\\' if !in_single => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            _ => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Determine risk level based on binary name and arguments.
fn assess_risk(binary: &str, args: &[String]) -> String {
    if DESTRUCTIVE_COMMANDS.contains(&binary) {
        // `rm -rf /` is higher risk than `rm file.txt`
        let has_force = args.iter().any(|a| a.contains('f') && a.starts_with('-'));
        let has_recursive = args.iter().any(|a| {
            (a.contains('r') || a.contains('R')) && a.starts_with('-')
        });
        if has_force || has_recursive {
            return "high".to_string();
        }
        return "medium".to_string();
    }
    if MUTATING_COMMANDS.contains(&binary) {
        return "medium".to_string();
    }
    "low".to_string()
}

/// Build a brief human-readable description of what the command would do.
fn describe_command(binary: &str, args: &[String]) -> String {
    let args_summary = if args.is_empty() {
        String::new()
    } else if args.len() <= 5 {
        format!(" with args: {}", args.join(" "))
    } else {
        format!(" with {} args (first: {})", args.len(), args[0])
    };

    let verb = if DESTRUCTIVE_COMMANDS.contains(&binary) {
        "Destructive command"
    } else if MUTATING_COMMANDS.contains(&binary) {
        "Mutating command"
    } else {
        "Command"
    };

    format!("{verb}: `{binary}`{args_summary}")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn execute_simple_echo() {
        let executor = Executor::default();
        let result = executor
            .execute("echo hello", None, None)
            .await
            .expect("execute should succeed");

        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout.trim(), "hello");
        assert!(result.stderr.is_empty() || result.stderr.trim().is_empty());
        assert!(!result.timed_out);
    }

    #[tokio::test]
    async fn execute_with_cwd() {
        let executor = Executor::default();
        let result = executor
            .execute("pwd", None, Some("/tmp"))
            .await
            .expect("execute should succeed");

        assert!(result.success);
        // On macOS /tmp is a symlink to /private/tmp
        let pwd = result.stdout.trim();
        assert!(
            pwd == "/tmp" || pwd == "/private/tmp",
            "unexpected pwd: {pwd}"
        );
    }

    #[tokio::test]
    async fn execute_timeout() {
        let executor = Executor::default();
        let result = executor
            .execute("sleep 10", Some(200), None)
            .await
            .expect("execute should return timeout result");

        assert!(!result.success);
        assert!(result.timed_out);
        assert!(result.duration_ms < 5000, "should not wait full 10s");
    }

    #[tokio::test]
    async fn execute_nonzero_exit() {
        let executor = Executor::default();
        let result = executor
            .execute("exit 42", None, None)
            .await
            .expect("execute should succeed even with non-zero exit");

        assert!(!result.success);
        assert_eq!(result.exit_code, 42);
        assert!(!result.timed_out);
    }

    #[tokio::test]
    async fn execute_output_truncation() {
        // Executor with a very small output limit.
        let executor = Executor::new(DEFAULT_TIMEOUT_MS, 32);
        let result = executor
            .execute("echo 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'", None, None)
            .await
            .expect("execute should succeed");

        assert!(result.success);
        assert!(result.stdout.contains("truncated"));
    }

    #[tokio::test]
    async fn execute_script_simple() {
        let executor = Executor::default();
        let script = "#!/bin/bash\necho script_output\nexit 0";
        let result = executor
            .execute_script(script, None, None)
            .await
            .expect("execute_script should succeed");

        assert!(result.success);
        assert_eq!(result.stdout.trim(), "script_output");
        assert_eq!(result.exit_code, 0);
    }

    #[tokio::test]
    async fn execute_script_with_interpreter() {
        let executor = Executor::default();
        let script = "echo 'from sh'";
        let result = executor
            .execute_script(script, Some("sh"), None)
            .await
            .expect("execute_script with sh should succeed");

        assert!(result.success);
        assert_eq!(result.stdout.trim(), "from sh");
    }

    #[tokio::test]
    async fn execute_script_cleans_up_temp() {
        let executor = Executor::default();
        // Run a script that prints its own path so we can verify cleanup.
        let script = "echo $0";
        let result = executor
            .execute_script(script, Some("bash"), None)
            .await
            .expect("execute_script should succeed");

        assert!(result.success);
        let script_path = result.stdout.trim();
        // Temp file should have been removed.
        assert!(
            !std::path::Path::new(script_path).exists(),
            "temp script file should be cleaned up"
        );
    }

    #[test]
    fn dry_run_simple_command() {
        let executor = Executor::default();
        let result = executor.dry_run("ls -la /tmp");

        assert_eq!(result.binary, "ls");
        assert_eq!(result.args, vec!["-la", "/tmp"]);
        assert_eq!(result.risk_level, "low");
        assert!(result.description.contains("ls"));
    }

    #[test]
    fn dry_run_destructive_command() {
        let executor = Executor::default();
        let result = executor.dry_run("rm -rf /some/path");

        assert_eq!(result.binary, "rm");
        assert_eq!(result.risk_level, "high");
        assert!(result.description.contains("Destructive"));
    }

    #[test]
    fn dry_run_medium_risk() {
        let executor = Executor::default();
        let result = executor.dry_run("rm file.txt");

        assert_eq!(result.binary, "rm");
        assert_eq!(result.risk_level, "medium");
    }

    #[test]
    fn dry_run_mutating_command() {
        let executor = Executor::default();
        let result = executor.dry_run("git commit -m 'test'");

        assert_eq!(result.binary, "git");
        assert_eq!(result.risk_level, "medium");
        assert!(result.description.contains("Mutating"));
    }

    #[test]
    fn dry_run_empty_command() {
        let executor = Executor::default();
        let result = executor.dry_run("");

        assert!(result.binary.is_empty());
        assert!(result.args.is_empty());
        assert_eq!(result.risk_level, "low");
    }

    #[test]
    fn shell_split_handles_quotes() {
        let parts = shell_split(r#"echo "hello world" 'foo bar'"#);
        assert_eq!(parts, vec!["echo", "hello world", "foo bar"]);
    }

    #[test]
    fn shell_split_handles_escapes() {
        let parts = shell_split(r#"echo hello\ world"#);
        assert_eq!(parts, vec!["echo", "hello world"]);
    }
}
