use crate::daemon_client::ping_daemon;
use anyhow::Result;
use colored::Colorize;

struct Check {
    name: &'static str,
    status: CheckStatus,
    message: String,
}

enum CheckStatus {
    Ok,
    Warning,
    Error,
}

pub async fn execute(fix: bool) -> Result<()> {
    let mut checks = Vec::new();

    // Check 1: Daemon running
    let daemon_running = ping_daemon().await;
    checks.push(Check {
        name: "Daemon (port 9876)",
        status: if daemon_running {
            CheckStatus::Ok
        } else {
            CheckStatus::Error
        },
        message: if daemon_running {
            "Running".to_string()
        } else {
            "Not running. Start with: d1 start".to_string()
        },
    });

    // Check 2: Config file
    let config_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".d1doctor/config.toml");
    let config_exists = config_path.exists();
    checks.push(Check {
        name: "Config file",
        status: if config_exists {
            CheckStatus::Ok
        } else {
            CheckStatus::Warning
        },
        message: if config_exists {
            config_path.display().to_string()
        } else {
            format!("{} not found", config_path.display())
        },
    });

    // Check 3: Rust toolchain
    let rust_ok = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    checks.push(Check {
        name: "Rust toolchain",
        status: if rust_ok {
            CheckStatus::Ok
        } else {
            CheckStatus::Warning
        },
        message: if rust_ok {
            "Installed".to_string()
        } else {
            "Not found (optional)".to_string()
        },
    });

    // Check 4: Python
    let python_ok = std::process::Command::new("python3")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    checks.push(Check {
        name: "Python 3",
        status: if python_ok {
            CheckStatus::Ok
        } else {
            CheckStatus::Warning
        },
        message: if python_ok {
            "Installed".to_string()
        } else {
            "Not found".to_string()
        },
    });

    // Print results
    println!("{}", "Day 1 Doctor — System Diagnostics".bold());
    println!();
    for check in &checks {
        let (icon, colored_name) = match check.status {
            CheckStatus::Ok => ("✓".green(), check.name.green()),
            CheckStatus::Warning => ("⚠".yellow(), check.name.yellow()),
            CheckStatus::Error => ("✗".red(), check.name.red()),
        };
        println!(
            "  {} {:<30} {}",
            icon,
            colored_name,
            check.message.dimmed()
        );
    }

    let errors = checks
        .iter()
        .filter(|c| matches!(c.status, CheckStatus::Error))
        .count();
    if errors > 0 {
        println!("\n{} {} issue(s) found", "⚠".yellow(), errors);
        if !fix {
            println!(
                "  Run {} to auto-fix LOW risk issues",
                "d1 doctor --fix".cyan()
            );
        }
    } else {
        println!("\n{} All checks passed", "✓".green());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_doctor_output_formatting() {
        // Just ensure the module compiles and functions are accessible
        // Real integration test would require running the command
    }
}
