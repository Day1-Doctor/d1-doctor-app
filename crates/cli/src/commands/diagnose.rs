//! `d1-doctor diagnose` subcommand

use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct DiagnoseArgs {
    /// Focus area: "memory", "cpu", "disk", or "all" (default)
    #[arg(long, default_value = "all")]
    pub focus: String,

    /// Skip plan approval and run immediately
    #[arg(long, short = 'y')]
    pub yes: bool,
}

pub async fn handle(args: &DiagnoseArgs) -> Result<()> {
    let request_text = match args.focus.as_str() {
        "memory" => "run memory diagnostics and report usage".to_string(),
        "cpu"    => "run cpu diagnostics and report load".to_string(),
        "disk"   => "run disk diagnostics and report space".to_string(),
        _        => "run system diagnostics: memory, cpu, disk, and running processes".to_string(),
    };
    println!("Diagnostics request: {request_text}");
    println!("(Connect to daemon to execute — use with running daemon)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_diagnose_all_no_panic() {
        let args = DiagnoseArgs { focus: "all".to_string(), yes: false };
        let result = handle(&args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_diagnose_memory_no_panic() {
        let args = DiagnoseArgs { focus: "memory".to_string(), yes: true };
        let result = handle(&args).await;
        assert!(result.is_ok());
    }
}
