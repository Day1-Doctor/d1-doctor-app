//! Authentication commands — Sprint 2: token persistence + device-code stub.
use crate::auth_token::TokenStore;
use d1_common::config_dir;

pub async fn login() -> anyhow::Result<()> {
    println!("Day 1 Doctor — Authentication");
    println!();
    println!("To activate your device, visit:");
    println!("  https://day1doctor.com/activate");
    println!();
    println!("(Full device-code flow coming in Sprint 5)");
    println!();

    use std::io::Write;
    print!("Paste your Supabase JWT (or press Enter to skip): ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let token = input.trim().to_string();

    if !token.is_empty() {
        let store = TokenStore {
            access_token: token,
            user_id: "local-user".to_string(),
        };
        let token_path = config_dir().join("token.json");
        store.save(&token_path)?;
        println!("Token saved to {}", token_path.display());
    }
    Ok(())
}

pub async fn logout() -> anyhow::Result<()> {
    let token_path = config_dir().join("token.json");
    if token_path.exists() {
        std::fs::remove_file(&token_path)?;
        println!("Logged out. Credentials removed.");
    } else {
        println!("Not logged in.");
    }
    Ok(())
}

pub fn load_token() -> Option<TokenStore> {
    let token_path = config_dir().join("token.json");
    TokenStore::try_load(&token_path)
}
