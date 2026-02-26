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

// ─── CreditStatus (Sprint 5 B17) ─────────────────────────────────────────────

/// Simplified credit status returned by the platform REST API `/api/credits`.
#[derive(Debug, Deserialize)]
pub struct CreditStatus {
    pub balance: i32,
    pub monthly_used: i32,
    pub tier: String,
}

/// Formats credit status into a human-readable display string.
pub fn format_credit_display(status: &CreditStatus) -> String {
    let tier_label = match status.tier.as_str() {
        "pro" | "team" => format!("{} Tier", capitalize(&status.tier)),
        _ => "Free Tier".to_string(),
    };

    if status.tier == "pro" || status.tier == "team" {
        return format!("Day 1 Doctor v1.0 | {tier_label}\nCredits: Unlimited");
    }

    let daily_total = 5;
    let monthly_total = 50;
    let monthly_remaining = monthly_total - status.monthly_used;

    format!(
        "Day 1 Doctor v1.0 | {tier_label}\nCredits: {}/{} remaining today | {}/{} this month",
        status.balance, daily_total, monthly_remaining, monthly_total
    )
}

/// Fetches credit status from the platform REST API.
pub async fn fetch_credit_status(token: &str, base_url: &str) -> Result<CreditStatus> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/credits", base_url);

    let resp = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to reach {}: {}", url, e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Credits API returned {status}: {body}");
    }

    let credit_status: CreditStatus = resp
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse credit status response: {}", e))?;

    Ok(credit_status)
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
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

    #[test]
    fn test_parse_credit_status_from_json() {
        let json = r#"{"balance": 3, "monthly_used": 12, "tier": "free"}"#;
        let status: CreditStatus = serde_json::from_str(json).expect("parse failed");
        assert_eq!(status.balance, 3);
        assert_eq!(status.monthly_used, 12);
        assert_eq!(status.tier, "free");
    }

    #[test]
    fn test_format_credit_display_free_tier() {
        let status = CreditStatus {
            balance: 3,
            monthly_used: 12,
            tier: "free".to_string(),
        };
        let display = format_credit_display(&status);
        assert!(display.contains("Credits: 3"), "Expected 'Credits: 3' in: {display}");
        assert!(display.contains("Free Tier"), "Expected 'Free Tier' in: {display}");
    }

    #[test]
    fn test_format_credit_display_pro_tier() {
        let status = CreditStatus {
            balance: 0,
            monthly_used: 0,
            tier: "pro".to_string(),
        };
        let display = format_credit_display(&status);
        assert!(display.contains("Pro Tier"), "Expected 'Pro Tier' in: {display}");
        assert!(display.contains("Unlimited"), "Expected 'Unlimited' in: {display}");
    }
}
