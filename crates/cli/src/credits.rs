//! Credit balance display and upgrade prompts.
//!
//! Sprint 5: full credit display via orchestrator /v1/credits endpoint.

use anyhow::Result;
use serde::Deserialize;

/// Default orchestrator API base URL.
pub const DEFAULT_ORCHESTRATOR_API_URL: &str = "http://localhost:8080";

/// Credit balance response from the orchestrator API.
#[derive(Debug, Clone, Deserialize)]
pub struct CreditBalance {
    pub daily_balance: i32,
    pub bonus_balance: i32,
    pub daily_used: i32,
    pub reset_at: String,
}

/// Fetch credit balance from the orchestrator API.
///
/// Calls `GET {api_url}/v1/credits` with Bearer token authentication.
pub async fn fetch_credits(api_url: &str, token: &str) -> Result<CreditBalance> {
    let url = format!("{}/v1/credits", api_url.trim_end_matches('/'));

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "Credits API returned status {}: {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        );
    }

    let balance: CreditBalance = resp.json().await?;
    Ok(balance)
}

/// Pretty-print the credit balance to stdout.
pub fn display_balance(balance: &CreditBalance) {
    println!("{}", format_balance(balance));
}

/// Format the credit balance as a human-readable string (testable).
pub fn format_balance(balance: &CreditBalance) -> String {
    let total = balance.daily_balance + balance.bonus_balance;
    format!(
        "Credit Balance:\n  Daily  : {remaining} / {total_daily} (used {used})\n  Bonus  : {bonus}\n  Total  : {total}\n  Resets : {reset}",
        remaining = balance.daily_balance,
        total_daily = balance.daily_balance + balance.daily_used,
        used = balance.daily_used,
        bonus = balance.bonus_balance,
        total = total,
        reset = balance.reset_at,
    )
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_balance_displays_all_fields() {
        let balance = CreditBalance {
            daily_balance: 80,
            bonus_balance: 20,
            daily_used: 20,
            reset_at: "2026-02-27T00:00:00Z".to_string(),
        };

        let output = format_balance(&balance);

        assert!(output.contains("80"), "should contain daily_balance");
        assert!(output.contains("20"), "should contain bonus_balance / daily_used");
        assert!(
            output.contains("2026-02-27T00:00:00Z"),
            "should contain reset_at"
        );
        assert!(output.contains("Credit Balance:"), "should have header");
        assert!(output.contains("Daily"), "should label daily credits");
        assert!(output.contains("Bonus"), "should label bonus credits");
        assert!(output.contains("Total"), "should label total credits");
        assert!(output.contains("Resets"), "should label reset time");
    }

    #[test]
    fn test_format_balance_zero_values() {
        let balance = CreditBalance {
            daily_balance: 0,
            bonus_balance: 0,
            daily_used: 0,
            reset_at: "2026-02-27T00:00:00Z".to_string(),
        };

        let output = format_balance(&balance);

        // Should still produce a valid, non-empty string with all labels
        assert!(!output.is_empty(), "output should not be empty");
        assert!(output.contains("Daily"), "should still have Daily label");
        assert!(output.contains("Bonus"), "should still have Bonus label");
        assert!(output.contains("Total  : 0"), "total should be 0");
        assert!(
            output.contains("2026-02-27T00:00:00Z"),
            "should contain reset_at even with zero values"
        );
    }

    #[tokio::test]
    async fn test_fetch_credits_unreachable_returns_error() {
        // Use an unreachable address — should return Err, not panic.
        let result = fetch_credits("http://127.0.0.1:1", "fake-token").await;
        assert!(
            result.is_err(),
            "fetch_credits to unreachable host should return Err"
        );
    }

    #[test]
    fn test_credit_balance_deserialization() {
        let json = r#"{
            "daily_balance": 50,
            "bonus_balance": 10,
            "daily_used": 30,
            "reset_at": "2026-02-28T00:00:00Z"
        }"#;

        let balance: CreditBalance =
            serde_json::from_str(json).expect("should deserialize CreditBalance from JSON");

        assert_eq!(balance.daily_balance, 50);
        assert_eq!(balance.bonus_balance, 10);
        assert_eq!(balance.daily_used, 30);
        assert_eq!(balance.reset_at, "2026-02-28T00:00:00Z");
    }
}
