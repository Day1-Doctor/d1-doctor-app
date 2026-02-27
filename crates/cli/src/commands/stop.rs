use anyhow::Result;
use colored::Colorize;

pub async fn execute() -> Result<()> {
    // Read PID file
    let pid_path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("No home directory"))?
        .join(".d1doctor")
        .join("daemon.pid");

    if !pid_path.exists() {
        println!(
            "{} Daemon is not running (no PID file found)",
            "○".dimmed()
        );
        return Ok(());
    }

    let pid_str = std::fs::read_to_string(&pid_path)?;
    let pid: u32 = pid_str
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid PID in {}", pid_path.display()))?;

    // Send SIGTERM
    #[cfg(unix)]
    {
        use std::io;
        let result = unsafe { libc::kill(pid as i32, libc::SIGTERM) };
        if result != 0 {
            let err = io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::ESRCH) {
                println!(
                    "{} Daemon process {} not found, cleaning up PID file",
                    "⚠".yellow(),
                    pid
                );
                let _ = std::fs::remove_file(&pid_path);
                return Ok(());
            }
            return Err(anyhow::anyhow!(
                "Failed to send SIGTERM to {pid}: {err}"
            ));
        }
    }

    println!("{} Daemon stopped (PID {})", "✓".green(), pid);
    let _ = std::fs::remove_file(&pid_path);
    Ok(())
}
