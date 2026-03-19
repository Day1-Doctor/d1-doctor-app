use serde::{Deserialize, Serialize};

/// Parsed OAuth callback data from a `day1copilot://auth/callback?...` URL.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthCallback {
    pub token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
}

/// Parse an OAuth callback URL.
///
/// Accepted format: `day1copilot://auth/callback?token=...&refresh_token=...&expires_in=...`
///
/// The `token` (or `access_token`) query parameter is required.
/// `refresh_token` and `expires_in` are optional.
pub fn parse_callback_url(raw: &str) -> Result<AuthCallback, String> {
    let parsed = url::Url::parse(raw).map_err(|e| format!("Invalid callback URL: {}", e))?;

    let token = parsed
        .query_pairs()
        .find(|(k, _)| k == "token" || k == "access_token")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| "Missing token in callback URL".to_string())?;

    if token.is_empty() {
        return Err("Token is empty in callback URL".to_string());
    }

    let refresh_token = parsed
        .query_pairs()
        .find(|(k, _)| k == "refresh_token")
        .map(|(_, v)| v.to_string())
        .filter(|v| !v.is_empty());

    let expires_in = parsed
        .query_pairs()
        .find(|(k, _)| k == "expires_in")
        .and_then(|(_, v)| v.parse::<u64>().ok());

    Ok(AuthCallback {
        token,
        refresh_token,
        expires_in,
    })
}

/// Read the stored auth data (token + optional refresh token) from
/// `~/.day1copilot/auth.json`.
pub fn read_auth_file() -> Result<serde_json::Value, String> {
    let auth_file = dirs::home_dir()
        .ok_or("Cannot determine home directory")?
        .join(".day1copilot/auth.json");
    if !auth_file.exists() {
        return Err("Auth file not found".to_string());
    }
    let content = std::fs::read_to_string(&auth_file).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

/// Write auth data (token + optional refresh token) to `~/.day1copilot/auth.json`.
pub fn write_auth_file(token: &str, refresh_token: Option<&str>) -> Result<(), String> {
    let config_dir = dirs::home_dir()
        .ok_or("Cannot determine home directory")?
        .join(".day1copilot");
    std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    let auth_file = config_dir.join("auth.json");

    let mut auth_data = serde_json::json!({
        "token": token,
        "token_type": "jwt",
    });

    if let Some(rt) = refresh_token {
        auth_data["refresh_token"] = serde_json::Value::String(rt.to_string());
    }

    std::fs::write(
        &auth_file,
        serde_json::to_string_pretty(&auth_data).unwrap(),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_callback_url_with_token() {
        let url = "day1copilot://auth/callback?token=jwt_abc123";
        let result = parse_callback_url(url).unwrap();
        assert_eq!(result.token, "jwt_abc123");
        assert!(result.refresh_token.is_none());
        assert!(result.expires_in.is_none());
    }

    #[test]
    fn test_parse_callback_url_with_access_token() {
        let url = "day1copilot://auth/callback?access_token=jwt_xyz789";
        let result = parse_callback_url(url).unwrap();
        assert_eq!(result.token, "jwt_xyz789");
    }

    #[test]
    fn test_parse_callback_url_with_all_fields() {
        let url = "day1copilot://auth/callback?token=jwt_abc&refresh_token=rt_def&expires_in=3600";
        let result = parse_callback_url(url).unwrap();
        assert_eq!(result.token, "jwt_abc");
        assert_eq!(result.refresh_token.as_deref(), Some("rt_def"));
        assert_eq!(result.expires_in, Some(3600));
    }

    #[test]
    fn test_parse_callback_url_missing_token() {
        let url = "day1copilot://auth/callback?refresh_token=rt_def";
        let result = parse_callback_url(url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing token"));
    }

    #[test]
    fn test_parse_callback_url_empty_token() {
        let url = "day1copilot://auth/callback?token=";
        let result = parse_callback_url(url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_parse_callback_url_invalid_url() {
        let result = parse_callback_url("not a url at all");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid callback URL"));
    }

    #[test]
    fn test_parse_callback_url_no_query_params() {
        let url = "day1copilot://auth/callback";
        let result = parse_callback_url(url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing token"));
    }

    #[test]
    fn test_parse_callback_url_expires_in_non_numeric() {
        let url = "day1copilot://auth/callback?token=jwt_abc&expires_in=notanumber";
        let result = parse_callback_url(url).unwrap();
        assert_eq!(result.token, "jwt_abc");
        assert!(result.expires_in.is_none()); // gracefully ignored
    }

    #[test]
    fn test_parse_callback_url_empty_refresh_token_filtered() {
        let url = "day1copilot://auth/callback?token=jwt_abc&refresh_token=";
        let result = parse_callback_url(url).unwrap();
        assert_eq!(result.token, "jwt_abc");
        assert!(result.refresh_token.is_none()); // empty string filtered out
    }
}
