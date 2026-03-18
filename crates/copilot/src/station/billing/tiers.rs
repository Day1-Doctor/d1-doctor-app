use serde::{Deserialize, Serialize};

/// Subscription tiers for Dr. Bob's Office.
///
/// Each tier provides a set number of "office spots" (agent slots) and
/// a monthly DD credit allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionTier {
    /// Free tier — 1 agent (Dr. Bob only), 100 DD/month.
    FreMan,
    /// Mid tier — 3 agents, 1 000 DD/month.
    MiniShop,
    /// Enterprise tier — 8 agents (full campus), 5 000 DD/month.
    RocketInc,
}

/// Configuration for a subscription tier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TierConfig {
    pub name: String,
    pub price_monthly: f64,
    pub price_annual: f64,
    pub max_agents: u32,
    pub monthly_dd_credits: u64,
}

impl SubscriptionTier {
    /// Return the display name for this tier.
    pub fn display_name(&self) -> &str {
        match self {
            SubscriptionTier::FreMan => "Free Man",
            SubscriptionTier::MiniShop => "Mini Shop",
            SubscriptionTier::RocketInc => "Rocket Inc.",
        }
    }
}

/// Get the full configuration for a subscription tier.
pub fn get_tier_config(tier: SubscriptionTier) -> TierConfig {
    match tier {
        SubscriptionTier::FreMan => TierConfig {
            name: "Free Man".to_string(),
            price_monthly: 0.0,
            price_annual: 0.0,
            max_agents: 1,
            monthly_dd_credits: 100,
        },
        SubscriptionTier::MiniShop => TierConfig {
            name: "Mini Shop".to_string(),
            price_monthly: 19.0,
            price_annual: 190.0,
            max_agents: 3,
            monthly_dd_credits: 1_000,
        },
        SubscriptionTier::RocketInc => TierConfig {
            name: "Rocket Inc.".to_string(),
            price_monthly: 49.0,
            price_annual: 490.0,
            max_agents: 8,
            monthly_dd_credits: 5_000,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_tier_config() {
        let config = get_tier_config(SubscriptionTier::FreMan);
        assert_eq!(config.name, "Free Man");
        assert!((config.price_monthly - 0.0).abs() < f64::EPSILON);
        assert!((config.price_annual - 0.0).abs() < f64::EPSILON);
        assert_eq!(config.max_agents, 1);
        assert_eq!(config.monthly_dd_credits, 100);
    }

    #[test]
    fn test_mini_shop_config() {
        let config = get_tier_config(SubscriptionTier::MiniShop);
        assert_eq!(config.name, "Mini Shop");
        assert!((config.price_monthly - 19.0).abs() < f64::EPSILON);
        assert!((config.price_annual - 190.0).abs() < f64::EPSILON);
        assert_eq!(config.max_agents, 3);
        assert_eq!(config.monthly_dd_credits, 1_000);
    }

    #[test]
    fn test_rocket_inc_config() {
        let config = get_tier_config(SubscriptionTier::RocketInc);
        assert_eq!(config.name, "Rocket Inc.");
        assert!((config.price_monthly - 49.0).abs() < f64::EPSILON);
        assert!((config.price_annual - 490.0).abs() < f64::EPSILON);
        assert_eq!(config.max_agents, 8);
        assert_eq!(config.monthly_dd_credits, 5_000);
    }
}
