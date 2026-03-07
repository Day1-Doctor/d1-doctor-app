//! Terminal display helpers for the chat session.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::connection::ConnectionTarget;

/// Print welcome banner when starting a chat session.
pub fn print_welcome(session_id: &str, target: &ConnectionTarget) {
    println!(
        "\x1b[1;32m{}\x1b[0m",
        crate::i18n::t("chat.welcome_title")
    );
    println!(
        "{}",
        crate::i18n::t_args("chat.session_label", &[("id", &session_id[..8])])
    );
    println!(
        "{}",
        crate::i18n::t_args("chat.connected_to", &[("target", &target.to_string())])
    );
    println!();
    println!("{}", crate::i18n::t("chat.input_hint"));
    println!("{}", crate::i18n::t("chat.multiline_hint"));
    println!("{}", crate::i18n::t("chat.exit_hint"));
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
    println!(
        "\x1b[2m{}\x1b[0m",
        crate::i18n::t("chat.response_cancelled")
    );
}

/// Print an error message.
pub fn print_error(err: &anyhow::Error) {
    eprintln!();
    eprintln!("\x1b[1;31merror:\x1b[0m {}", err);
}

/// Print goodbye message.
pub fn print_goodbye() {
    println!();
    println!("\x1b[2m{}\x1b[0m", crate::i18n::t("chat.goodbye"));
}

/// Show a typing indicator (spinner) in a background thread.
pub fn show_typing_indicator() -> Arc<AtomicBool> {
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_clone = cancel.clone();

    let thinking_msg = crate::i18n::t("chat.thinking");

    std::thread::spawn(move || {
        let frames = [
            "\u{280B}", "\u{2819}", "\u{2839}", "\u{2838}",
            "\u{283C}", "\u{2834}", "\u{2826}", "\u{2827}",
            "\u{2807}", "\u{280F}",
        ];
        let mut i = 0;

        while !cancel_clone.load(Ordering::Relaxed) {
            print!(
                "\r\x1b[2m{} {}\x1b[0m",
                frames[i % frames.len()],
                thinking_msg
            );
            let _ = io::stdout().flush();
            i += 1;
            std::thread::sleep(std::time::Duration::from_millis(80));
        }

        print!("\r\x1b[K");
        let _ = io::stdout().flush();
    });

    cancel
}

/// Stop the typing indicator.
pub fn stop_typing_indicator(cancel: Arc<AtomicBool>) {
    cancel.store(true, Ordering::Relaxed);
    std::thread::sleep(std::time::Duration::from_millis(100));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typing_indicator_lifecycle() {
        crate::i18n::init("en");
        let cancel = show_typing_indicator();
        std::thread::sleep(std::time::Duration::from_millis(200));
        stop_typing_indicator(cancel);
    }
}
