//! CLI authentication commands — OAuth callback flow with local credential storage.

use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum AuthCommand {
    /// Authenticate with Day 1 Doctor via browser OAuth flow
    Login,
    /// Clear stored credentials
    Logout,
    /// Display current authentication status
    Whoami,
}

#[derive(Serialize, Deserialize)]
struct StoredCredentials {
    access_token: String,
    refresh_token: String,
    email: String,
    expires_at: i64,
}

/// Resolve the credentials file path: `~/.d1-doctor/credentials.json`.
fn credentials_path() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| anyhow::anyhow!("Could not determine home directory"))?;
    Ok(PathBuf::from(home)
        .join(".d1-doctor")
        .join("credentials.json"))
}

fn load_credentials() -> anyhow::Result<StoredCredentials> {
    let path = credentials_path()?;
    let content = std::fs::read_to_string(&path).map_err(|_| {
        anyhow::anyhow!("No stored credentials found. Run 'd1-doctor auth login' to authenticate.")
    })?;
    let creds: StoredCredentials = serde_json::from_str(&content)?;
    Ok(creds)
}

fn store_credentials(creds: &StoredCredentials) -> anyhow::Result<()> {
    let path = credentials_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(creds)?;
    std::fs::write(&path, &content)?;
    // Restrict file permissions on Unix so only the owner can read.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn clear_credentials() -> anyhow::Result<()> {
    let path = credentials_path()?;
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

pub async fn handle_auth(cmd: AuthCommand) -> anyhow::Result<()> {
    match cmd {
        AuthCommand::Login => login().await,
        AuthCommand::Logout => logout().await,
        AuthCommand::Whoami => whoami().await,
    }
}

async fn login() -> anyhow::Result<()> {
    // Bind a local server on a random available port for the OAuth callback.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let redirect_uri = format!("http://127.0.0.1:{}/callback", port);

    let auth_url = format!(
        "https://auth.day1doctor.com/authorize?redirect_uri={}&response_type=code",
        redirect_uri
    );

    println!("Opening browser for authentication...");
    open_browser(&auth_url)?;
    println!("Waiting for authentication callback on port {port}...");

    // Accept exactly one connection (the OAuth redirect).
    let (stream, _) = listener.accept().await?;
    stream.readable().await?;
    let mut buf = vec![0u8; 4096];
    let n = stream.try_read(&mut buf)?;
    let request = String::from_utf8_lossy(&buf[..n]);

    let code = parse_auth_code(&request)
        .ok_or_else(|| anyhow::anyhow!("Failed to extract authorization code from callback"))?;

    // Respond to the browser so the user sees a success page.
    let html = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
        <html><body><h1>Authentication successful!</h1>\
        <p>You can close this tab and return to the terminal.</p></body></html>";
    stream.writable().await?;
    let _ = stream.try_write(html.as_bytes());

    // Exchange the authorization code for tokens.
    let client = reqwest::Client::new();
    let resp = client
        .post("https://auth.day1doctor.com/token")
        .json(&serde_json::json!({
            "grant_type": "authorization_code",
            "code": code,
            "redirect_uri": redirect_uri,
        }))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Token exchange failed ({}): {}", status, body);
    }

    let creds: StoredCredentials = resp.json().await?;
    let email = creds.email.clone();
    store_credentials(&creds)?;

    println!("Successfully logged in as {email}");
    Ok(())
}

async fn logout() -> anyhow::Result<()> {
    clear_credentials()?;
    println!("Logged out successfully.");
    Ok(())
}

async fn whoami() -> anyhow::Result<()> {
    match load_credentials() {
        Ok(creds) => {
            println!("Logged in as: {}", creds.email);
            let now = chrono::Utc::now().timestamp();
            if now > creds.expires_at {
                println!(
                    "Warning: Token has expired. Run 'd1-doctor auth login' to re-authenticate."
                );
            }
        }
        Err(_) => {
            println!("Not logged in. Run 'd1-doctor auth login' to authenticate.");
        }
    }
    Ok(())
}

/// Open a URL in the default system browser.
fn open_browser(url: &str) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .spawn()?;
    }
    Ok(())
}

/// Extract the `code` query parameter from an HTTP request line like
/// `GET /callback?code=abc123 HTTP/1.1`.
fn parse_auth_code(request: &str) -> Option<String> {
    let first_line = request.lines().next()?;
    let path = first_line.split_whitespace().nth(1)?;
    let query = path.split('?').nth(1)?;
    for param in query.split('&') {
        let mut kv = param.splitn(2, '=');
        if kv.next()? == "code" {
            return kv.next().map(String::from);
        }
    }
    None
}
