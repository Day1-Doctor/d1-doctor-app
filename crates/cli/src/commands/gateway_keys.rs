//! `d1 gateway keys` — API key management (list, create, revoke).

use serde::{Deserialize, Serialize};

use super::gateway::api_client;

/// API key as returned by the platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub key_prefix: String,
    pub name: String,
    pub is_active: bool,
    pub rate_limit_rpm: Option<i32>,
    pub last_used_at: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
}

/// Response when creating a new key (includes the full plaintext key).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKeyResponse {
    pub id: String,
    pub plaintext_key: String,
    pub key_prefix: String,
    pub name: String,
    pub created_at: Option<String>,
}

/// List all API keys for the current user.
pub async fn run_list() -> anyhow::Result<()> {
    let (client, base_url, token) = api_client()?;

    let resp = client
        .get(format!("{}/api/v1/api-keys", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!(
            "{}",
            crate::i18n::t_args(
                "gateway.keys.list_failed",
                &[("status", &status.to_string()), ("body", &body)]
            )
        );
    }

    let keys: Vec<ApiKey> = resp.json().await?;
    print_keys(&keys);
    Ok(())
}

/// Create a new API key.
pub async fn run_create(name: &str) -> anyhow::Result<()> {
    let (client, base_url, token) = api_client()?;

    let resp = client
        .post(format!("{}/api/v1/api-keys", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "name": name }))
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!(
            "{}",
            crate::i18n::t_args(
                "gateway.keys.create_failed",
                &[("status", &status.to_string()), ("body", &body)]
            )
        );
    }

    let key: CreateKeyResponse = resp.json().await?;
    println!();
    println!(
        "{}",
        crate::i18n::t_args("gateway.keys.created_title", &[("name", &key.name)])
    );
    println!();
    println!("  {}", key.plaintext_key);
    println!();
    println!("{}", crate::i18n::t("gateway.keys.created_warning"));
    Ok(())
}

/// Revoke (delete) an API key by ID.
pub async fn run_revoke(key_id: &str) -> anyhow::Result<()> {
    let (client, base_url, token) = api_client()?;

    let resp = client
        .delete(format!("{}/api/v1/api-keys/{}", base_url, key_id))
        .header("Authorization", format!("Bearer {}", token))
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await?;

    if resp.status().as_u16() == 204 || resp.status().is_success() {
        println!(
            "{}",
            crate::i18n::t_args("gateway.keys.revoked", &[("id", key_id)])
        );
    } else if resp.status().as_u16() == 404 {
        anyhow::bail!(
            "{}",
            crate::i18n::t_args("gateway.keys.not_found", &[("id", key_id)])
        );
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!(
            "{}",
            crate::i18n::t_args(
                "gateway.keys.revoke_failed",
                &[("status", &status.to_string()), ("body", &body)]
            )
        );
    }

    Ok(())
}

/// Print API keys in a formatted table.
fn print_keys(keys: &[ApiKey]) {
    if keys.is_empty() {
        println!("{}", crate::i18n::t("gateway.keys.none"));
        println!();
        println!("{}", crate::i18n::t("gateway.keys.create_hint"));
        return;
    }

    println!(
        "{:<38} {:<16} {:<14} {:<10} {}",
        crate::i18n::t("gateway.keys.col_id"),
        crate::i18n::t("gateway.keys.col_name"),
        crate::i18n::t("gateway.keys.col_prefix"),
        crate::i18n::t("gateway.keys.col_status"),
        crate::i18n::t("gateway.keys.col_created"),
    );
    println!("{}", "-".repeat(96));

    for key in keys {
        let status_str = if key.is_active {
            crate::i18n::t("gateway.keys.active")
        } else {
            crate::i18n::t("gateway.keys.revoked_label")
        };
        // Truncate created_at to date only
        let created = key
            .created_at
            .split('T')
            .next()
            .unwrap_or(&key.created_at);
        println!(
            "{:<38} {:<16} {:<14} {:<10} {}",
            key.id, key.name, key.key_prefix, status_str, created
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_deserialization() {
        let json = r#"{
            "id": "key-1",
            "key_prefix": "d1d_sk_a3f2",
            "name": "Cursor",
            "is_active": true,
            "rate_limit_rpm": null,
            "last_used_at": null,
            "expires_at": null,
            "created_at": "2026-03-10T00:00:00Z"
        }"#;
        let key: ApiKey = serde_json::from_str(json).unwrap();
        assert_eq!(key.id, "key-1");
        assert_eq!(key.name, "Cursor");
        assert!(key.is_active);
    }

    #[test]
    fn test_create_key_response_deserialization() {
        let json = r#"{
            "id": "key-1",
            "plaintext_key": "d1d_sk_abc123def456",
            "key_prefix": "d1d_sk_abc1",
            "name": "Test",
            "created_at": "2026-03-10T00:00:00Z"
        }"#;
        let resp: CreateKeyResponse = serde_json::from_str(json).unwrap();
        assert!(resp.plaintext_key.starts_with("d1d_sk_"));
    }

    #[test]
    fn test_print_keys_empty() {
        crate::i18n::init("en");
        print_keys(&[]);
    }

    #[test]
    fn test_print_keys_with_data() {
        crate::i18n::init("en");
        let keys = vec![ApiKey {
            id: "abc-123".into(),
            key_prefix: "d1d_sk_a3f2".into(),
            name: "Cursor".into(),
            is_active: true,
            rate_limit_rpm: None,
            last_used_at: None,
            expires_at: None,
            created_at: "2026-03-10T00:00:00Z".into(),
        }];
        print_keys(&keys);
    }
}
