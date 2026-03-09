//! Credit balance display and upgrade prompts.

use serde::{Deserialize, Serialize};

/// A single credit usage entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEntry {
    pub date: String,
    pub model: String,
    pub tokens: u64,
    pub cost: f64,
}

/// Credit balance summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditBalance {
    pub available: f64,
    pub used_this_month: f64,
    pub plan: String,
    pub recent_usage: Vec<UsageEntry>,
}

/// Fetch the current credit balance.
pub fn fetch_balance() -> CreditBalance {
    CreditBalance {
        available: 0.0,
        used_this_month: 0.0,
        plan: "free".to_string(),
        recent_usage: vec![],
    }
}

/// Print credit balance and recent usage to stdout.
pub fn print_credits() {
    let balance = fetch_balance();

    println!("{}", crate::i18n::t("credits.title"));
    println!();
    println!(
        "{}",
        crate::i18n::t_args("credits.plan_label", &[("plan", &balance.plan)])
    );
    println!(
        "{}",
        crate::i18n::t_args(
            "credits.available_label",
            &[("amount", &format!("{:.2}", balance.available))]
        )
    );
    println!(
        "{}",
        crate::i18n::t_args(
            "credits.used_label",
            &[("amount", &format!("{:.2}", balance.used_this_month))]
        )
    );

    if balance.recent_usage.is_empty() {
        println!();
        println!("{}", crate::i18n::t("credits.no_usage"));
    } else {
        println!();
        println!(
            "  {:<12} {:<24} {:>10} {:>10}",
            crate::i18n::t("credits.date_col"),
            crate::i18n::t("credits.model_col"),
            crate::i18n::t("credits.tokens_col"),
            crate::i18n::t("credits.cost_col"),
        );
        println!("  {}", "-".repeat(60));
        for entry in &balance.recent_usage {
            println!(
                "  {:<12} {:<24} {:>10} {:>9.4}",
                entry.date, entry.model, entry.tokens, entry.cost
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_balance_defaults() {
        let balance = fetch_balance();
        assert_eq!(balance.plan, "free");
        assert_eq!(balance.available, 0.0);
        assert!(balance.recent_usage.is_empty());
    }

    #[test]
    fn test_print_credits_no_panic() {
        crate::i18n::init("en");
        print_credits();
    }

    #[test]
    fn test_credit_balance_serialization() {
        let balance = CreditBalance {
            available: 10.50,
            used_this_month: 3.25,
            plan: "pro".to_string(),
            recent_usage: vec![UsageEntry {
                date: "2026-03-07".to_string(),
                model: "gpt-4o".to_string(),
                tokens: 1500,
                cost: 0.0225,
            }],
        };
        let json = serde_json::to_string(&balance).unwrap();
        let parsed: CreditBalance = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.plan, "pro");
        assert_eq!(parsed.recent_usage.len(), 1);
    }
}
