// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};

const DAEMON_BASE_URL: &str = "http://localhost:3030";

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub version: Option<String>,
    pub active_tasks: u32,
    pub credits_remaining: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendRequestResponse {
    pub task_id: String,
}

#[tauri::command]
async fn connect_daemon() -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| e.to_string())?;

    match client.get(format!("{}/health", DAEMON_BASE_URL)).send().await {
        Ok(resp) if resp.status().is_success() => Ok("connected".to_string()),
        Ok(resp) => Err(format!("daemon returned HTTP {}", resp.status())),
        Err(e) => Err(format!("cannot reach daemon: {}", e)),
    }
}

#[tauri::command]
async fn get_status() -> Result<DaemonStatus, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| e.to_string())?;

    let health_resp = client
        .get(format!("{}/health", DAEMON_BASE_URL))
        .send()
        .await
        .map_err(|e| format!("daemon unreachable: {}", e))?;

    if !health_resp.status().is_success() {
        return Ok(DaemonStatus {
            running: false,
            version: None,
            active_tasks: 0,
            credits_remaining: None,
        });
    }

    let health: serde_json::Value = health_resp.json().await.unwrap_or_default();

    Ok(DaemonStatus {
        running: true,
        version: health.get("version").and_then(|v| v.as_str()).map(String::from),
        active_tasks: health
            .get("active_tasks")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        credits_remaining: None,
    })
}

#[tauri::command]
async fn send_request(request: String) -> Result<String, String> {
    if request.trim().is_empty() {
        return Err("request cannot be empty".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let body = serde_json::json!({ "description": request.trim() });

    let resp = client
        .post(format!("{}/api/tasks", DAEMON_BASE_URL))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("failed to send request: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("daemon error {}: {}", status, text));
    }

    let payload: SendRequestResponse = resp
        .json()
        .await
        .map_err(|e| format!("invalid response from daemon: {}", e))?;

    Ok(payload.task_id)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            connect_daemon,
            get_status,
            send_request,
        ])
        .run(tauri::generate_context!())
        .expect("error while running d1-doctor desktop application");
}
