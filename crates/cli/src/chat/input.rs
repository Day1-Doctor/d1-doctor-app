//! User input handling with multi-line (paste mode) support.

use std::io::{self, BufRead, Write};

use anyhow::Result;

/// Parsed user input.
pub enum UserInput {
    /// A complete message to send.
    Message(String),
    /// User wants to exit (Ctrl+D or /exit).
    Exit,
}

/// Read a complete user input from the terminal.
pub fn read_user_input() -> Result<UserInput> {
    print!("\n\x1b[1;36myou>\x1b[0m ");
    io::stdout().flush()?;

    let stdin = io::stdin();
    let mut first_line = String::new();

    let bytes_read = stdin.lock().read_line(&mut first_line)?;
    if bytes_read == 0 {
        return Ok(UserInput::Exit);
    }

    let trimmed = first_line.trim();

    if trimmed == "/exit" || trimmed == "/quit" {
        return Ok(UserInput::Exit);
    }

    if trimmed == "{" {
        return read_multiline();
    }

    Ok(UserInput::Message(first_line.trim().to_string()))
}

fn read_multiline() -> Result<UserInput> {
    println!(
        "\x1b[2m{}\x1b[0m",
        crate::i18n::t("chat.paste_mode")
    );

    let stdin = io::stdin();
    let mut lines = Vec::new();

    loop {
        print!("\x1b[2m...\x1b[0m ");
        io::stdout().flush()?;

        let mut line = String::new();
        let bytes_read = stdin.lock().read_line(&mut line)?;

        if bytes_read == 0 {
            return Ok(UserInput::Exit);
        }

        if line.trim() == "}" {
            break;
        }

        lines.push(line.trim_end_matches('\n').to_string());
    }

    let message = lines.join("\n");
    if message.trim().is_empty() {
        Ok(UserInput::Message(String::new()))
    } else {
        Ok(UserInput::Message(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_input_variants() {
        let msg = UserInput::Message("hello".to_string());
        let exit = UserInput::Exit;

        match msg {
            UserInput::Message(s) => assert_eq!(s, "hello"),
            UserInput::Exit => panic!("Expected Message"),
        }

        match exit {
            UserInput::Exit => {}
            UserInput::Message(_) => panic!("Expected Exit"),
        }
    }
}
