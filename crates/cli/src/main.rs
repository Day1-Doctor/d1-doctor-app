//! Day 1 Doctor — CLI Client

mod cli;
mod commands;
mod daemon_client;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run {
            task,
            approve,
            no_approve,
            json,
        } => commands::run::execute(task, approve, no_approve, json).await?,
        Commands::Start => commands::start::execute().await?,
        Commands::Stop => commands::stop::execute().await?,
        Commands::Status { json } => commands::status::execute(json).await?,
        Commands::Logs { tail } => commands::logs::execute(tail).await?,
        Commands::Tasks { task_id, json, all } => {
            commands::tasks::execute(task_id, json, all).await?
        }
        Commands::Config { action } => commands::config::execute(action).await?,
        Commands::Doctor { fix } => commands::doctor::execute(fix).await?,
    }
    Ok(())
}
