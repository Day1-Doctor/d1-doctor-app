//! CLI command definitions and handlers.

pub mod account;
pub mod gateway;
pub mod gateway_keys;
pub mod gateway_setup;
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
    Models {
        #[command(subcommand)]
        command: Option<ModelsSubcommand>,
    },
    /// Manage API keys
    Keys {
        #[command(subcommand)]
        command: KeysCommands,
    },
    /// Show DD credit balance
    Balance,
    /// Open the top-up page to add credits
    Topup,
    /// Show API usage summary
    Usage {
        /// Number of days to show usage for (default: 7)
        #[arg(long, default_value = "7")]
        days: u32,
    },
    /// Auto-configure an AI coding app to use Day1 Doctor gateway
    Setup {
        /// App to configure: cursor, continue, openclaw, or --generic
        app: String,
    },
}

#[derive(Subcommand)]
pub enum ModelsSubcommand {
    /// Show detailed info about a specific model
    Info {
        /// Model alias (e.g. gpt-4o, claude-sonnet, dr-bob)
        alias: String,
    },
}

#[derive(Subcommand)]
pub enum KeysCommands {
    /// List all API keys
    List,
    /// Create a new API key
    Create {
        /// Friendly name for the key
        #[arg(long, default_value = "Default")]
        name: String,
    },
    /// Revoke (delete) an API key
    Revoke {
        /// Key ID to revoke
        key_id: String,
    },
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
            GatewayCommands::Models { command } => match command {
                None => gateway::run_models_list().await,
                Some(ModelsSubcommand::Info { alias }) => gateway::run_models_info(&alias).await,
            },
            GatewayCommands::Keys { command } => match command {
                KeysCommands::List => gateway_keys::run_list().await,
                KeysCommands::Create { name } => gateway_keys::run_create(&name).await,
                KeysCommands::Revoke { key_id } => gateway_keys::run_revoke(&key_id).await,
            },
            GatewayCommands::Balance => gateway::run_balance().await,
            GatewayCommands::Topup => gateway::run_topup().await,
            GatewayCommands::Usage { days } => gateway::run_usage(days).await,
            GatewayCommands::Setup { app } => gateway_setup::run_setup(&app).await,
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
