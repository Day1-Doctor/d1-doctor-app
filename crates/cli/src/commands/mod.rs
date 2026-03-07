//! CLI command definitions and handlers.

pub mod gateway;
pub mod status;

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
    /// AI gateway management
    Gateway {
        #[command(subcommand)]
        command: GatewayCommands,
    },
    /// Show credit balance and recent usage
    Credits,
}

#[derive(Subcommand)]
pub enum GatewayCommands {
    /// Show gateway health and connection info
    Status,
    /// List available LLM models with pricing
    Models,
}

pub async fn handle(cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::Install { package } => {
            println!("Installing {}...", package);
            todo!("Implement install command")
        }
        Commands::Diagnose => todo!("Implement diagnose"),
        Commands::Files => todo!("Implement files"),
        Commands::Status => status::run().await,
        Commands::Upgrade => todo!("Implement upgrade"),
        Commands::Gateway { command } => match command {
            GatewayCommands::Status => gateway::run_status().await,
            GatewayCommands::Models => gateway::run_models().await,
        },
        Commands::Credits => {
            crate::credits::print_credits();
            Ok(())
        }
    }
}
