//! Interactive CLI chat session (`d1 run`).
//!
//! Provides a REPL-style chat interface that connects to the daemon
//! via local socket or cloud WebSocket and streams agent responses.

mod connection;
mod display;
mod history;
mod input;

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use d1_common::Config;

pub use connection::{ChatConnection, ConnectionTarget};
pub use history::SessionHistory;

/// Run the interactive chat session.
pub async fn run_interactive(target: Option<String>) -> Result<()> {
    let config = Config::load().unwrap_or_default();

    // Spawn a non-blocking version check (fire-and-forget).
    let daemon_port = config.daemon_port;
    tokio::spawn(async move {
        crate::version_check::maybe_nudge(daemon_port).await;
    });

    let target = match target {
        Some(url) => ConnectionTarget::Cloud(url),
        None => ConnectionTarget::Local(config.daemon_port),
    };

    let session_id = uuid::Uuid::new_v4().to_string();
    let mut history = SessionHistory::new(&session_id)?;
    let mut conn = ChatConnection::connect(&target).await?;

    // Send session_init so the server knows our locale.
    let locale = std::env::var("LANG").unwrap_or_else(|_| "en".to_string());
    conn.send_session_init(&session_id, &locale).await?;

    display::print_welcome(&session_id, &target);

    loop {
        match input::read_user_input() {
            Ok(input::UserInput::Message(text)) => {
                if text.trim().is_empty() {
                    continue;
                }
                history.add_user_message(&text)?;

                let cancel_token = display::show_typing_indicator();
                let first_chunk_received = Arc::new(AtomicBool::new(false));
                let fc = first_chunk_received.clone();
                let ct = cancel_token.clone();

                match conn
                    .send_and_stream(&session_id, &text, &cancel_token, move |chunk| {
                        if !fc.swap(true, Ordering::Relaxed) {
                            // First chunk: stop typing indicator and print the prompt.
                            ct.store(true, Ordering::Relaxed);
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            print!("\r\x1b[K");
                            let _ = io::stdout().flush();
                            display::print_stream_start();
                        }
                        display::print_chunk(chunk);
                    })
                    .await
                {
                    Ok(response) => {
                        display::stop_typing_indicator(cancel_token);
                        if first_chunk_received.load(Ordering::Relaxed) {
                            display::print_stream_end();
                        } else {
                            // Non-streaming fallback (got AgentResponse, no chunks).
                            display::print_response(&response);
                        }
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
