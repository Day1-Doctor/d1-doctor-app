//! Interactive CLI chat session (`d1 run`).
//!
//! Provides a REPL-style chat interface that connects to the daemon
//! via local socket or cloud WebSocket and streams agent responses.

mod connection;
mod display;
mod history;
mod input;

use anyhow::Result;
use d1_common::Config;

pub use connection::{ChatConnection, ConnectionTarget};
pub use history::SessionHistory;

/// Run the interactive chat session.
pub async fn run_interactive(target: Option<String>) -> Result<()> {
    let config = Config::load().unwrap_or_default();

    let target = match target {
        Some(url) => ConnectionTarget::Cloud(url),
        None => ConnectionTarget::Local(config.daemon_port),
    };

    let session_id = uuid::Uuid::new_v4().to_string();
    let mut history = SessionHistory::new(&session_id)?;
    let mut conn = ChatConnection::connect(&target).await?;

    display::print_welcome(&session_id, &target);

    loop {
        match input::read_user_input() {
            Ok(input::UserInput::Message(text)) => {
                if text.trim().is_empty() {
                    continue;
                }
                history.add_user_message(&text)?;

                let cancel_token = display::show_typing_indicator();
                match conn
                    .send_and_stream(&session_id, &text, &cancel_token)
                    .await
                {
                    Ok(response) => {
                        display::stop_typing_indicator(cancel_token);
                        display::print_response(&response);
                        history.add_agent_response(&response)?;
                    }
                    Err(e) if is_cancelled(&e) => {
                        display::stop_typing_indicator(cancel_token);
                        display::print_cancelled();
                    }
                    Err(e) => {
                        display::stop_typing_indicator(cancel_token);
                        display::print_error(&e);
                    }
                }
            }
            Ok(input::UserInput::Exit) => {
                display::print_goodbye();
                break;
            }
            Err(e) => {
                display::print_error(&e);
                break;
            }
        }
    }

    conn.disconnect().await?;
    history.finalize()?;
    Ok(())
}

fn is_cancelled(err: &anyhow::Error) -> bool {
    err.to_string().contains("cancelled")
}
