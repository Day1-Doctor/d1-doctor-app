//! CLI command definitions and handlers.

pub mod account;
pub mod gateway;
pub mod status;

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
    /// AI gateway management
    Gateway {
        #[command(subcommand)]
        command: GatewayCommands,
    },
    /// Show credit balance and recent usage
    Credits,
    /// Account management
    Account {
        #[command(subcommand)]
        command: AccountCommands,
    },
}

#[derive(Subcommand)]
pub enum AccountCommands {
    /// Permanently delete your account and all associated data
    Delete,
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
        Commands::Run { target } => crate::chat::run_interactive(target).await,
        Commands::Install { package } => {
            println!(
                "{}",
                crate::i18n::t_args("commands.installing", &[("package", &package)])
            );
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
        Commands::Account { command } => match command {
            AccountCommands::Delete => account::run_delete().await,
        },
    }
}
