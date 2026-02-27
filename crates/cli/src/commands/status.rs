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
    println!("{}", "─".repeat(48).dimmed());

    if let Some(p) = payload {
        let version = p["daemon_version"].as_str().unwrap_or("unknown");
        let orch_connected = p["orchestrator_connected"].as_bool().unwrap_or(false);
        let orch_url = p["orchestrator_url"].as_str().unwrap_or("(not configured)");
        let tasks = p["active_tasks"].as_u64().unwrap_or(0);

        println!("  {:<18} {}", "Version".dimmed(), version);
        println!("  {:<18} {}", "Status".dimmed(), "● connected".green());
        println!(
            "  {:<18} {}",
            "Orchestrator".dimmed(),
            if orch_connected {
                format!("● {}", orch_url).green().to_string()
            } else {
                format!("○ {} (disconnected)", orch_url).red().to_string()
            }
        );
        println!("  {:<18} {}", "Daemon port".dimmed(), "9876");
        println!("  {:<18} {}", "Active tasks".dimmed(), tasks);
    } else {
        println!(
            "  {}",
            "(no status data received — daemon may not be running)".dimmed()
        );
    }
    println!("{}", "─".repeat(48).dimmed());
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
