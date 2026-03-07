//! CLI command definitions and handlers.

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Start an interactive chat session with Dr. Bob
    Run {
        /// WebSocket URL to connect to (defaults to local daemon)
        #[arg(long)]
        target: Option<String>,
    },
    /// Install and configure software
    Install { package: String },
    /// Run system diagnostics
    Diagnose,
    /// Manage files and directories
    Files,
    /// Show daemon and connection status
    Status,
    /// Upgrade Day 1 Doctor
    Upgrade,
    /// Manage authentication (login, logout, whoami)
    Auth {
        #[command(subcommand)]
        command: crate::auth::AuthCommand,
    },
}

pub async fn handle(cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::Run { target } => crate::chat::run_interactive(target).await,
        Commands::Install { package } => {
            println!("Installing {}...", package);
            todo!("Implement install command")
        }
        Commands::Diagnose => todo!("Implement diagnose"),
        Commands::Files => todo!("Implement files"),
        Commands::Status => todo!("Implement status"),
        Commands::Upgrade => todo!("Implement upgrade"),
        Commands::Auth { command } => crate::auth::handle_auth(command).await,
    }
}
