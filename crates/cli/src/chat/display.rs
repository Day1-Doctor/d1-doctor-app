//! Terminal display helpers for the chat session.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::connection::ConnectionTarget;

/// Print welcome banner when starting a chat session.
pub fn print_welcome(session_id: &str, target: &ConnectionTarget) {
    println!("\x1b[1;32m--- Day 1 Doctor Chat ---\x1b[0m");
    println!("Session: {}", &session_id[..8]);
    println!("Connected to: {}", target);
    println!();
    println!("  Type a message and press Enter to send.");
    println!("  Multi-line: start with '{{', end with '}}'.");
    println!("  Ctrl+C to cancel response, Ctrl+D or /exit to quit.");
    println!("\x1b[2m{}\x1b[0m", "-".repeat(40));
}

/// Print the agent's response.
pub fn print_response(response: &str) {
    println!();
    println!("\x1b[1;33mdr.bob>\x1b[0m {}", response);
}

/// Print cancellation notice.
pub fn print_cancelled() {
    println!();
    println!("\x1b[2m(response cancelled)\x1b[0m");
}

/// Print an error message.
pub fn print_error(err: &anyhow::Error) {
    eprintln!();
    eprintln!("\x1b[1;31merror:\x1b[0m {}", err);
}

/// Print goodbye message.
pub fn print_goodbye() {
    println!();
    println!("\x1b[2mGoodbye!\x1b[0m");
}

/// Show a typing indicator (spinner) in a background thread.
/// Returns an `Arc<AtomicBool>` cancel token — set to `true` to stop.
pub fn show_typing_indicator() -> Arc<AtomicBool> {
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_clone = cancel.clone();

    std::thread::spawn(move || {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut i = 0;

        while !cancel_clone.load(Ordering::Relaxed) {
            print!("\r\x1b[2m{} thinking...\x1b[0m", frames[i % frames.len()]);
            let _ = io::stdout().flush();
            i += 1;
            std::thread::sleep(std::time::Duration::from_millis(80));
        }

        // Clear the spinner line
        print!("\r\x1b[K");
        let _ = io::stdout().flush();
    });

    cancel
}

/// Stop the typing indicator.
pub fn stop_typing_indicator(cancel: Arc<AtomicBool>) {
    cancel.store(true, Ordering::Relaxed);
    // Brief sleep to let the spinner thread clean up
    std::thread::sleep(std::time::Duration::from_millis(100));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typing_indicator_lifecycle() {
        let cancel = show_typing_indicator();
        std::thread::sleep(std::time::Duration::from_millis(200));
        stop_typing_indicator(cancel);
        // If we get here without hanging, the indicator lifecycle works
    }
}
