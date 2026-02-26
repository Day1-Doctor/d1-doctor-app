//! Day 1 Doctor — CLI Client

mod auth_token;
mod commands;
mod auth;
mod credits;
mod tui;

use clap::Parser;

#[derive(Parser)]
#[command(name = "d1-doctor")]
#[command(about = "Day 1 Doctor — AI-powered system setup assistant")]
struct Cli {
    #[command(subcommand)]
    command: Option<commands::Commands>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Some(cmd) => commands::handle(cmd).await?,
        None => {
            println!("Day 1 Doctor CLI v{}", env!("CARGO_PKG_VERSION"));
            println!("Run `d1-doctor --help` for usage");
        }
    }
    
    Ok(())
}
