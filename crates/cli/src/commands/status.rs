use crate::daemon_client::{ping_daemon, DaemonClient};
use anyhow::Result;
use colored::Colorize;

pub async fn execute(json: bool) -> Result<()> {
    if !ping_daemon().await {
        if json {
            println!("{}", serde_json::json!({"status": "not_running"}));
        } else {
            println!("{} Daemon is not running. Start it with: {}", "✗".red(), "d1 start".cyan());
        }
        return Ok(());
    }

    let mut client = DaemonClient::connect().await?;
    // Send heartbeat and wait for daemon.status response
    client
        .send("heartbeat", serde_json::json!({"ping": true}))
        .await?;

    // Wait for daemon.status message (it's sent immediately on connect)
    let mut status_payload = None;
    for _ in 0..5 {
        if let Some(msg) = client.recv_timeout(2).await? {
            if msg.msg_type == "daemon.status" {
                status_payload = Some(msg.payload);
                break;
            }
        }
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&status_payload.unwrap_or_default())?
        );
    } else {
        print_status_table(status_payload.as_ref());
    }
    Ok(())
}

pub fn print_status_table(payload: Option<&serde_json::Value>) {
    println!("{}", "Day 1 Doctor Daemon".bold());
    if let Some(p) = payload {
        let version = p["daemon_version"].as_str().unwrap_or("unknown");
        let orch = p["orchestrator_connected"].as_bool().unwrap_or(false);
        let tasks = p["active_tasks"].as_u64().unwrap_or(0);
        println!("  {:<16} {}", "Version".dimmed(), version);
        println!("  {:<16} {}", "Status".dimmed(), "● connected".green());
        println!(
            "  {:<16} {}",
            "Orchestrator".dimmed(),
            if orch {
                "● connected".green().to_string()
            } else {
                "○ disconnected".red().to_string()
            }
        );
        println!("  {:<16} {}", "Active Tasks".dimmed(), tasks);
    } else {
        println!("  {}", "(no status data received)".dimmed());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_status_table_no_crash() {
        // Should not panic with None
        print_status_table(None);
    }

    #[test]
    fn test_print_status_table_with_data() {
        let payload = serde_json::json!({
            "daemon_version": "0.4.1",
            "orchestrator_connected": true,
            "active_tasks": 2
        });
        print_status_table(Some(&payload));
        // No assertion needed — just verify no panic
    }
}
