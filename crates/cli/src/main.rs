//! Day 1 Doctor — CLI Client

mod auth;
mod chat;
mod commands;
mod credits;
pub mod i18n;
mod tui;
pub mod version_check;

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
    i18n::init_auto();

    let cli = Cli::parse();

    match cli.command {
        Some(cmd) => commands::handle(cmd).await?,
        None => {
            println!(
                "{}",
                i18n::t_args(
                    "app.version_line",
                    &[("version", env!("CARGO_PKG_VERSION"))]
                )
            );
            println!("{}", i18n::t("app.usage_hint"));
        }
    }

    Ok(())
}
