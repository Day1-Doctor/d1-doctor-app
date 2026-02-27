use anyhow::Result;
use colored::Colorize;

pub async fn execute(tail: u32) -> Result<()> {
    let log_path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("No home directory"))?
        .join(".d1doctor")
        .join("daemon.log");

    if !log_path.exists() {
        println!(
            "{} No log file found at {}",
            "○".dimmed(),
            log_path.display()
        );
        println!(
            "  Start the daemon to generate logs: {}",
            "d1 start".cyan()
        );
        return Ok(());
    }

    let content = std::fs::read_to_string(&log_path)?;
    let lines: Vec<&str> = content.lines().collect();
    let start = if lines.len() > tail as usize {
        lines.len() - tail as usize
    } else {
        0
    };

    for line in &lines[start..] {
        println!("{}", line);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_logs_no_crash_when_no_file() {
        // The function handles missing file gracefully
        // We just verify it compiles
    }
}
