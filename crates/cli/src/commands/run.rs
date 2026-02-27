use crate::daemon_client::{DaemonClient, TaskSubmitPayload};
use anyhow::Result;
use colored::Colorize;
use uuid::Uuid;

pub async fn execute(task: String, approve: bool, _no_approve: bool, json: bool) -> Result<()> {
    let mut client = DaemonClient::connect().await?;

    let task_id = format!("tsk_{}", &Uuid::new_v4().to_string()[..8]);
    client
        .send(
            "task.submit",
            TaskSubmitPayload {
                task_id: task_id.clone(),
                input: task.clone(),
                context: serde_json::json!({"cwd": null, "env": {}}),
            },
        )
        .await?;

    println!("{} Task submitted: {}", "◉".cyan(), task.bold());
    println!("  Task ID: {}", task_id.dimmed());

    // Stream events
    loop {
        match client.recv_timeout(120).await {
            Ok(Some(msg)) => {
                if json {
                    println!("{}", serde_json::to_string(&msg.payload)?);
                    if msg.msg_type == "task.completed" || msg.msg_type == "task.failed" {
                        break;
                    }
                    continue;
                }

                match msg.msg_type.as_str() {
                    "daemon.status" => {} // ignore
                    "plan.proposed" => {
                        handle_plan_proposed(&msg.payload, &mut client, &task_id, approve).await?;
                    }
                    "step.started" => {
                        let step_num = msg.payload["step_number"].as_u64().unwrap_or(0);
                        let desc = msg.payload["description"].as_str().unwrap_or("");
                        println!(
                            "  {} {}  {}",
                            format!("{step_num}").dimmed(),
                            "◉".cyan(),
                            desc
                        );
                    }
                    "step.completed" => {
                        let step_num = msg.payload["step_number"].as_u64().unwrap_or(0);
                        let secs = msg.payload["duration_seconds"].as_f64().unwrap_or(0.0);
                        println!(
                            "  {} {}  ({:.1}s)",
                            format!("{step_num}").dimmed(),
                            "✓".green(),
                            secs
                        );
                    }
                    "step.failed" => {
                        let desc = msg.payload["description"].as_str().unwrap_or("step");
                        println!("  {} {}", "✗".red(), desc);
                    }
                    "agent.message" => {
                        let content = msg.payload["message"].as_str().unwrap_or("");
                        println!("    {}", content.dimmed());
                    }
                    "task.completed" => {
                        let summary =
                            msg.payload["summary"].as_str().unwrap_or("Task complete.");
                        println!("\n{} {}", "✓".green().bold(), summary.bold());
                        break;
                    }
                    "task.failed" => {
                        let error = msg.payload["error"]["message"]
                            .as_str()
                            .unwrap_or("Unknown error");
                        println!("\n{} Task failed: {}", "✗".red().bold(), error);
                        std::process::exit(1);
                    }
                    "error" => {
                        let code = msg.payload["code"].as_str().unwrap_or("unknown");
                        eprintln!("{} Protocol error: {}", "✗".red(), code);
                        break;
                    }
                    _ => {} // ignore unknown message types
                }
            }
            Ok(None) => break,
            Err(e) => {
                eprintln!("{} {}", "Error:".red(), e);
                break;
            }
        }
    }
    Ok(())
}

async fn handle_plan_proposed(
    payload: &serde_json::Value,
    client: &mut DaemonClient,
    task_id: &str,
    auto_approve: bool,
) -> Result<()> {
    let plan_id = payload["plan_id"].as_str().unwrap_or("").to_string();
    let summary = payload["summary"].as_str().unwrap_or("Execute task");
    let steps = payload["steps"].as_array().cloned().unwrap_or_default();
    let risk = payload["risk_tier"].as_str().unwrap_or("MEDIUM");

    println!("\n{}", "━━━ Plan Proposed ━━━".bold());
    println!("  {}", summary);
    println!("  Risk: {}", format_risk(risk));
    println!("  Steps:");
    for (i, step) in steps.iter().enumerate() {
        let desc = step["description"].as_str().unwrap_or("step");
        let step_risk = step["risk_tier"].as_str().unwrap_or("LOW");
        println!("    {}. {} [{}]", i + 1, desc, format_risk(step_risk));
    }
    println!();

    let action =
        if auto_approve || !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
            println!("{} Auto-approving plan (--approve flag)", "→".cyan());
            "APPROVE"
        } else {
            // Interactive TUI
            print_tui_prompt();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            match input.trim().to_lowercase().as_str() {
                "y" | "yes" | "" => "APPROVE",
                _ => "REJECT",
            }
        };

    client
        .send(
            "plan.approve",
            serde_json::json!({
                "task_id": task_id,
                "plan_id": plan_id,
                "action": action,
                "modifications": null
            }),
        )
        .await?;

    println!(
        "{} Plan {}",
        if action == "APPROVE" {
            "✓".green()
        } else {
            "✗".red()
        },
        if action == "APPROVE" {
            "approved"
        } else {
            "rejected"
        }
    );
    Ok(())
}

fn print_tui_prompt() {
    println!("Approve this plan? [Y/n] ");
}

fn format_risk(risk: &str) -> colored::ColoredString {
    match risk {
        "LOW" => risk.green(),
        "MEDIUM" => risk.yellow(),
        "HIGH" => risk.red(),
        _ => risk.white(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_risk_low() {
        let result = format_risk("LOW");
        assert!(result.to_string().contains("LOW"));
    }

    #[test]
    fn test_format_risk_high() {
        let result = format_risk("HIGH");
        assert!(result.to_string().contains("HIGH"));
    }
}
