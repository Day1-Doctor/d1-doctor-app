//! `d1-doctor files` subcommand — FILE_READ, FILE_WRITE, FILE_MOVE, FILE_DELETE

use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct FilesArgs {
    #[command(subcommand)]
    pub action: FilesAction,
}

#[derive(Subcommand, Debug)]
pub enum FilesAction {
    /// List files in a directory
    List {
        /// Directory path to list
        path: String,
    },
    /// Read a file's contents
    Read {
        /// Path to the file
        path: String,
    },
    /// Write content to a file
    Write {
        /// Path to the file
        path: String,
        /// Content to write
        #[arg(long, short)]
        content: String,
    },
}

pub async fn handle(args: &FilesArgs) -> Result<()> {
    let request_text = match &args.action {
        FilesAction::List { path } => format!("list files in directory {path}"),
        FilesAction::Read { path } => format!("read file {path}"),
        FilesAction::Write { path, content } => format!("write to file {path} with content: {content}"),
    };
    println!("Files request: {request_text}");
    println!("(Connect to daemon to execute — use with running daemon)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_files_list_handle_no_panic() {
        let args = FilesArgs { action: FilesAction::List { path: "/tmp".to_string() } };
        let result = handle(&args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_files_read_handle_no_panic() {
        let args = FilesArgs { action: FilesAction::Read { path: "/tmp/nonexistent.txt".to_string() } };
        let result = handle(&args).await;
        assert!(result.is_ok());
    }
}
