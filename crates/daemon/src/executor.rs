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

    /// Perform a file operation: FILE_READ, FILE_WRITE, FILE_MOVE, or FILE_DELETE.
    pub async fn run_file_op(
        &self,
        action: &str,
        path: &str,
        destination: Option<&str>,
        content: Option<&str>,
    ) -> anyhow::Result<ExecResult> {
        let start = Instant::now();
        match action {
            "FILE_READ" => {
                let data = tokio::fs::read_to_string(path)
                    .await
                    .map_err(|e| anyhow::anyhow!("FILE_READ failed for {path}: {e}"))?;
                Ok(ExecResult {
                    success: true,
                    stdout: data,
                    stderr: String::new(),
                    exit_code: 0,
                    duration_ms: start.elapsed().as_millis() as u64,
                    timed_out: false,
                })
            }
            "FILE_WRITE" => {
                let bytes = content.unwrap_or("").as_bytes();
                tokio::fs::write(path, bytes)
                    .await
                    .map_err(|e| anyhow::anyhow!("FILE_WRITE failed for {path}: {e}"))?;
                Ok(ExecResult {
                    success: true,
                    stdout: format!("Written {} bytes to {path}", bytes.len()),
                    stderr: String::new(),
                    exit_code: 0,
                    duration_ms: start.elapsed().as_millis() as u64,
                    timed_out: false,
                })
            }
            "FILE_MOVE" => {
                let dest = destination
                    .ok_or_else(|| anyhow::anyhow!("FILE_MOVE requires a destination path"))?;
                tokio::fs::rename(path, dest)
                    .await
                    .map_err(|e| anyhow::anyhow!("FILE_MOVE failed {path} -> {dest}: {e}"))?;
                Ok(ExecResult {
                    success: true,
                    stdout: format!("Moved {path} -> {dest}"),
                    stderr: String::new(),
                    exit_code: 0,
                    duration_ms: start.elapsed().as_millis() as u64,
                    timed_out: false,
                })
            }
            "FILE_DELETE" => {
                tokio::fs::remove_file(path)
                    .await
                    .map_err(|e| anyhow::anyhow!("FILE_DELETE failed for {path}: {e}"))?;
                Ok(ExecResult {
                    success: true,
                    stdout: format!("Deleted {path}"),
                    stderr: String::new(),
                    exit_code: 0,
                    duration_ms: start.elapsed().as_millis() as u64,
                    timed_out: false,
                })
            }
            other => Err(anyhow::anyhow!("Unknown file action: {other}")),
        }
    }

    /// Collect system information (OS, CPU count, memory) as a JSON string.
    pub async fn collect_system_info(&self) -> anyhow::Result<ExecResult> {
        let start = Instant::now();

        // sysinfo is synchronous — use spawn_blocking to avoid blocking the async runtime
        let info = tokio::task::spawn_blocking(|| {
            use sysinfo::System;

            let mut sys = System::new_all();
            sys.refresh_all();

            let os = System::name().unwrap_or_else(|| "unknown".to_string());
            let os_version = System::os_version().unwrap_or_else(|| "unknown".to_string());
            let cpu_count = sys.cpus().len() as u64;
            let memory_total_mb = sys.total_memory() / (1024 * 1024);
            let memory_used_mb = sys.used_memory() / (1024 * 1024);

            serde_json::json!({
                "os": os,
                "os_version": os_version,
                "cpu_count": cpu_count,
                "memory_total_mb": memory_total_mb,
                "memory_used_mb": memory_used_mb,
            })
        })
        .await
        .map_err(|e| anyhow::anyhow!("spawn_blocking error: {e}"))?;

        Ok(ExecResult {
            success: true,
            stdout: info.to_string(),
            stderr: String::new(),
            exit_code: 0,
            duration_ms: start.elapsed().as_millis() as u64,
            timed_out: false,
        })
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

    // ─── B13: File operation tests ──────────────────────────────────────────

    #[tokio::test]
    async fn test_file_write_and_read() {
        let executor = Executor::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hello.txt").to_str().unwrap().to_string();

        // Write
        let write_result = executor
            .run_file_op("FILE_WRITE", &path, None, Some("hello world"))
            .await
            .unwrap();
        assert!(write_result.success);

        // Read back
        let read_result = executor
            .run_file_op("FILE_READ", &path, None, None)
            .await
            .unwrap();
        assert!(read_result.success);
        assert_eq!(read_result.stdout.trim(), "hello world");
    }

    #[tokio::test]
    async fn test_file_move() {
        let executor = Executor::new();
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src.txt").to_str().unwrap().to_string();
        let dst = dir.path().join("dst.txt").to_str().unwrap().to_string();

        tokio::fs::write(&src, b"move me").await.unwrap();

        let result = executor
            .run_file_op("FILE_MOVE", &src, Some(&dst), None)
            .await
            .unwrap();
        assert!(result.success);
        assert!(tokio::fs::metadata(&dst).await.is_ok());
        assert!(tokio::fs::metadata(&src).await.is_err());
    }

    #[tokio::test]
    async fn test_file_delete() {
        let executor = Executor::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("delete_me.txt").to_str().unwrap().to_string();

        tokio::fs::write(&path, b"bye").await.unwrap();

        let result = executor
            .run_file_op("FILE_DELETE", &path, None, None)
            .await
            .unwrap();
        assert!(result.success);
        assert!(tokio::fs::metadata(&path).await.is_err());
    }

    // ─── B14: System info test ───────────────────────────────────────────────

    #[tokio::test]
    async fn test_collect_system_info() {
        let executor = Executor::new();
        let result = executor.collect_system_info().await.unwrap();
        assert!(result.success);
        let info: serde_json::Value = serde_json::from_str(&result.stdout).unwrap();
        assert!(info.get("os").and_then(|v| v.as_str()).is_some());
        assert!(info.get("cpu_count").and_then(|v| v.as_u64()).map_or(false, |n| n > 0));
        assert!(info.get("memory_total_mb").and_then(|v| v.as_u64()).map_or(false, |n| n > 0));
    }
}
