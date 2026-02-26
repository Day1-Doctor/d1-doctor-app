//! CLI command definitions and handlers.

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
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
}

pub async fn handle(cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::Install { package } => {
            println!("Installing {}...", package);
            todo!("Implement install command")
        }
        Commands::Diagnose => todo!("Implement diagnose"),
        Commands::Files => todo!("Implement files"),
        Commands::Status => todo!("Implement status"),
        Commands::Upgrade => todo!("Implement upgrade"),
    }
}
