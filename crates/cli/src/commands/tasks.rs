use crate::daemon_client::DaemonClient;
use anyhow::Result;
use colored::Colorize;

pub async fn execute(task_id: Option<String>, json: bool, _all: bool) -> Result<()> {
    // Query daemon for task list via REST-like WS message
    // For now: connect, send a tasks.list custom message, wait for response
    // If daemon doesn't support it yet, show a helpful message
    let mut client = DaemonClient::connect().await.unwrap_or_else(|e| {
        eprintln!("{} {}", "Error:".red(), e);
        std::process::exit(1);
    });

    client
        .send(
            "tasks.list",
            serde_json::json!({
                "task_id": task_id,
                "limit": 20
            }),
        )
        .await?;

    match client.recv_timeout(3).await {
        Ok(Some(msg)) if msg.msg_type == "tasks.list.response" => {
            if json {
                println!("{}", serde_json::to_string_pretty(&msg.payload)?);
            } else {
                print_tasks_table(&msg.payload);
            }
        }
        Ok(Some(msg)) if msg.msg_type == "error" => {
            // Daemon may not support tasks.list yet
            eprintln!(
                "{} Daemon returned: {}",
                "⚠".yellow(),
                msg.payload["message"].as_str().unwrap_or("unknown error")
            );
        }
        _ => {
            println!(
                "{} No task data available (daemon may not support tasks.list yet)",
                "○".dimmed()
            );
        }
    }
    Ok(())
}

fn print_tasks_table(payload: &serde_json::Value) {
    let tasks = payload["tasks"].as_array();
    match tasks {
        Some(tasks) if tasks.is_empty() => println!("{}", "No recent tasks".dimmed()),
        Some(tasks) => {
            println!(
                "{:<12} {:<8} {:<30} {}",
                "TASK ID".bold(),
                "STATUS".bold(),
                "INPUT".bold(),
                "CREATED".bold()
            );
            for task in tasks {
                let id = task["task_id"].as_str().unwrap_or("-");
                let status = task["status"].as_str().unwrap_or("-");
                let input = task["input"].as_str().unwrap_or("-");
                let created = task["created_at"].as_str().unwrap_or("-");
                println!(
                    "{:<12} {:<8} {:<30} {}",
                    id,
                    status,
                    &input[..input.len().min(30)],
                    created
                );
            }
        }
        None => println!("{}", "No task data in response".dimmed()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_tasks_table_empty() {
        let payload = serde_json::json!({"tasks": []});
        print_tasks_table(&payload); // should not panic
    }

    #[test]
    fn test_print_tasks_table_with_tasks() {
        let payload = serde_json::json!({
            "tasks": [
                {"task_id": "tsk_abc123", "status": "completed", "input": "install openclaw", "created_at": "2026-02-27"}
            ]
        });
        print_tasks_table(&payload); // should not panic
    }
}
