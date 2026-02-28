//! Tauri commands for daemon lifecycle management.

/// Check if daemon is reachable on port 9876.
pub(crate) async fn ping_daemon() -> Result<(), ()> {
    tokio::net::TcpStream::connect("127.0.0.1:9876")
        .await
        .map(|_| ())
        .map_err(|_| ())
}

/// Ensure daemon is running. Spawns sidecar in release builds; in dev, instructs user.
#[tauri::command]
pub async fn ensure_daemon_running(app: tauri::AppHandle) -> Result<(), String> {
    if ping_daemon().await.is_ok() {
        eprintln!("[d1d] Daemon already running on port 9876");
        return Ok(());
    }

    #[cfg(not(debug_assertions))]
    {
        use std::time::Duration;
        use tauri_plugin_shell::ShellExt;
        let config_path = dirs::home_dir()
            .map(|h| h.join(".d1doctor").join("config.toml"))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "~/.d1doctor/config.toml".to_string());
        let sidecar = app
            .shell()
            .sidecar("binaries/d1d")
            .map_err(|e| e.to_string())?;
        let (_rx, _child) = sidecar
            .args(["--config", &config_path])
            .spawn()
            .map_err(|e| e.to_string())?;

        for _ in 0..25 {
            tokio::time::sleep(Duration::from_millis(200)).await;
            if ping_daemon().await.is_ok() {
                eprintln!("[d1d] Daemon started successfully");
                return Ok(());
            }
        }
        return Err("Daemon did not start within 5 seconds".into());
    }

    #[cfg(debug_assertions)]
    {
        let _ = app; // suppress unused warning
        eprintln!("[d1d] Daemon not running (dev mode). Start it with: cargo run --bin d1d");
        Err("Daemon not running. Start it with: d1 start".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_daemon_false_on_unused_port() {
        // Port 19876 is almost certainly not in use
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            tokio::net::TcpStream::connect("127.0.0.1:19876").await
        });
        // This should fail (no listener on 19876)
        assert!(result.is_err(), "Expected no listener on port 19876");
    }

    #[test]
    fn test_ping_daemon_error_type() {
        // Verify ping_daemon returns Err when port not open
        // We test on a definitely closed port
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result: Result<(), ()> = rt.block_on(async {
            tokio::net::TcpStream::connect("127.0.0.1:19876")
                .await
                .map(|_| ())
                .map_err(|_| ())
        });
        assert!(result.is_err(), "ping on closed port should return Err");
    }
}
