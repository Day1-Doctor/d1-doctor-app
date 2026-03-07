//! `d1 account` — account management commands.

use std::io::{self, Write};

use d1_common::Config;

/// Run the account deletion flow.
///
/// 1. Prompt the user to type DELETE for confirmation.
/// 2. Send DELETE /api/account to the local daemon (which relays to the platform).
/// 3. Clear local credentials on success.
pub async fn run_delete() -> anyhow::Result<()> {
    println!("WARNING: This will permanently delete your account and all associated data.");
    println!("This action cannot be undone.\n");
    print!("Type DELETE to confirm: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input != "DELETE" {
        println!("Account deletion cancelled.");
        return Ok(());
    }

    let config = Config::load().unwrap_or_default();
    let daemon_url = format!("http://127.0.0.1:{}/api/account", config.daemon_port);

    // Load the access token for the Authorization header.
    let token = load_access_token()?;

    println!("Deleting account...");

    let client = reqwest::Client::new();
    let resp = client
        .delete(&daemon_url)
        .header("Authorization", format!("Bearer {}", token))
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            // Clear local credentials.
            clear_local_credentials()?;
            println!("Account deleted successfully. All data has been removed.");
            println!("Local credentials cleared.");
            Ok(())
        }
        Ok(r) => {
            let status = r.status();
            let body = r.text().await.unwrap_or_default();
            anyhow::bail!(
                "Account deletion failed (HTTP {}): {}",
                status.as_u16(),
                body
            );
        }
        Err(e) => {
            anyhow::bail!(
                "Could not reach the daemon at {}. Is it running?\nError: {}",
                daemon_url,
                e
            );
        }
    }
}

/// Read the access token from the local credentials file.
fn load_access_token() -> anyhow::Result<String> {
    let path = credentials_path()?;
    let content = std::fs::read_to_string(&path).map_err(|_| {
        anyhow::anyhow!(
            "No stored credentials found. Run \'d1 auth login\' first."
        )
    })?;
    let creds: serde_json::Value = serde_json::from_str(&content)?;
    creds
        .get("access_token")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| anyhow::anyhow!("Invalid credentials file: missing access_token"))
}

/// Remove the local credentials file.
fn clear_local_credentials() -> anyhow::Result<()> {
    let path = credentials_path()?;
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

/// Resolve `~/.d1-doctor/credentials.json`.
fn credentials_path() -> anyhow::Result<std::path::PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| anyhow::anyhow!("Could not determine home directory"))?;
    Ok(std::path::PathBuf::from(home)
        .join(".d1-doctor")
        .join("credentials.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_path_not_empty() {
        // Should resolve to a non-empty path when HOME is set.
        let path = credentials_path();
        assert!(path.is_ok());
        let p = path.unwrap();
        assert!(p.to_str().unwrap().contains(".d1-doctor"));
        assert!(p.to_str().unwrap().contains("credentials.json"));
    }

    #[test]
    fn test_load_access_token_missing_file() {
        // With a non-existent credentials file, should return an error.
        std::env::set_var("HOME", "/tmp/nonexistent-d1-test-dir");
        let result = load_access_token();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No stored credentials"));
    }

    #[test]
    fn test_clear_local_credentials_nonexistent() {
        // Clearing when no file exists should succeed silently.
        std::env::set_var("HOME", "/tmp/nonexistent-d1-test-dir");
        let result = clear_local_credentials();
        assert!(result.is_ok());
    }
}
