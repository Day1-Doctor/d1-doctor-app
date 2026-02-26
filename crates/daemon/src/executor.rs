//! Shell command executor with timeout and output capture.
use anyhow::Result;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::time::timeout;

pub struct Executor;

#[derive(Debug)]
pub struct ExecResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub timed_out: bool,
}

impl Executor {
    pub fn new() -> Self {
        Self
    }

    /// Execute a shell command with a timeout in seconds.
    pub async fn run(&self, command: &str, timeout_secs: u64) -> Result<ExecResult> {
        let start = Instant::now();
        let deadline = Duration::from_secs(timeout_secs);

        let mut child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let result = timeout(deadline, async {
            let mut stdout_buf = Vec::new();
            let mut stderr_buf = Vec::new();

            if let Some(mut stdout) = child.stdout.take() {
                stdout.read_to_end(&mut stdout_buf).await.ok();
            }
            if let Some(mut stderr) = child.stderr.take() {
                stderr.read_to_end(&mut stderr_buf).await.ok();
            }

            let status = child.wait().await?;
            Ok::<(Vec<u8>, Vec<u8>, std::process::ExitStatus), anyhow::Error>(
                (stdout_buf, stderr_buf, status)
            )
        })
        .await;

        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok((stdout, stderr, status))) => Ok(ExecResult {
                success: status.success(),
                stdout: String::from_utf8_lossy(&stdout).to_string(),
                stderr: String::from_utf8_lossy(&stderr).to_string(),
                exit_code: status.code().unwrap_or(-1),
                duration_ms,
                timed_out: false,
            }),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                child.kill().await.ok();
                Ok(ExecResult {
                    success: false,
                    stdout: String::new(),
                    stderr: format!("Command timed out after {timeout_secs}s"),
                    exit_code: -1,
                    duration_ms,
                    timed_out: true,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_echo_command() {
        let exec = Executor::new();
        let result = exec.run("echo hello", 5).await.unwrap();
        assert!(result.success);
        assert_eq!(result.stdout.trim(), "hello");
        assert_eq!(result.exit_code, 0);
    }

    #[tokio::test]
    async fn test_execute_failing_command() {
        let exec = Executor::new();
        let result = exec.run("false", 5).await.unwrap();
        assert!(!result.success);
        assert_ne!(result.exit_code, 0);
    }

    #[tokio::test]
    async fn test_timeout_is_enforced() {
        let exec = Executor::new();
        let result = exec.run("sleep 10", 1).await;
        assert!(result.is_err() || result.unwrap().timed_out);
    }
}
