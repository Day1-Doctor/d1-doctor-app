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
    fn test_tail_offset_calculation() {
        // Simulate: 10 lines, tail=3 → start index = 7
        let lines: Vec<&str> = (0..10).map(|_| "line").collect();
        let tail: u32 = 3;
        let start = if lines.len() > tail as usize {
            lines.len() - tail as usize
        } else {
            0
        };
        assert_eq!(start, 7);
    }

    #[test]
    fn test_tail_offset_clamps_at_zero() {
        // Simulate: 2 lines, tail=50 → start = 0 (don't go negative)
        let lines: Vec<&str> = vec!["a", "b"];
        let tail: u32 = 50;
        let start = if lines.len() > tail as usize {
            lines.len() - tail as usize
        } else {
            0
        };
        assert_eq!(start, 0);
    }
}
