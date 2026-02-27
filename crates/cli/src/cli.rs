use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "d1", about = "Day 1 Doctor CLI", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Submit a task, stream output to terminal
    Run {
        /// Task description
        task: String,
        /// Auto-approve plans without TUI
        #[arg(long)]
        approve: bool,
        /// Never approve (cancel on plan)
        #[arg(long)]
        no_approve: bool,
        /// Output raw JSON events
        #[arg(long)]
        json: bool,
    },
    /// Start local daemon
    Start,
    /// Stop local daemon
    Stop,
    /// Show daemon status and connection info
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Tail daemon logs
    Logs {
        #[arg(long, default_value = "50")]
        tail: u32,
    },
    /// List recent tasks or show task detail
    Tasks {
        /// Optional task ID to inspect
        task_id: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        all: bool,
    },
    /// Read or write config
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Self-diagnostic check
    Doctor {
        /// Auto-fix LOW-risk issues
        #[arg(long)]
        fix: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Read config value
    Get { key: Option<String> },
    /// Set config value
    Set { key: String, value: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_command_parsed() {
        let args = Cli::try_parse_from(["d1", "run", "install openclaw"]).unwrap();
        assert!(matches!(args.command, Commands::Run { ref task, .. } if task == "install openclaw"));
    }

    #[test]
    fn test_status_command_parsed() {
        let args = Cli::try_parse_from(["d1", "status"]).unwrap();
        assert!(matches!(args.command, Commands::Status { .. }));
    }

    #[test]
    fn test_start_command_parsed() {
        let args = Cli::try_parse_from(["d1", "start"]).unwrap();
        assert!(matches!(args.command, Commands::Start));
    }

    #[test]
    fn test_run_with_approve_flag() {
        let args = Cli::try_parse_from(["d1", "run", "test task", "--approve"]).unwrap();
        assert!(matches!(args.command, Commands::Run { approve: true, .. }));
    }

    #[test]
    fn test_config_get_parsed() {
        let args = Cli::try_parse_from(["d1", "config", "get", "orchestrator.url"]).unwrap();
        assert!(matches!(args.command, Commands::Config { action: ConfigAction::Get { .. } }));
    }

    #[test]
    fn test_doctor_fix_flag() {
        let args = Cli::try_parse_from(["d1", "doctor", "--fix"]).unwrap();
        assert!(matches!(args.command, Commands::Doctor { fix: true }));
    }
}
