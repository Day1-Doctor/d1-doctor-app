use serde::{Deserialize, Serialize};

/// Top-up packs for purchasing additional DD credits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TopUpPack {
    /// Boost pack: 1 000 DD for $10.
    Boost,
    /// Power Pack: 6 000 DD for $50.
    PowerPack,
    /// Custom amount: user picks a USD price, DD is proportional.
    Custom(f64),
}

impl TopUpPack {
    /// The DD credit amount for this pack.
    pub fn dd_credits(&self) -> u64 {
        match self {
            TopUpPack::Boost => 1_000,
            TopUpPack::PowerPack => 6_000,
            TopUpPack::Custom(price_usd) => calculate_custom_dd(*price_usd),
        }
    }

    /// The USD price for this pack.
    pub fn price_usd(&self) -> f64 {
        match self {
            TopUpPack::Boost => 10.0,
            TopUpPack::PowerPack => 50.0,
            TopUpPack::Custom(price) => *price,
        }
    }

    /// Display name for the pack.
    pub fn display_name(&self) -> String {
        match self {
            TopUpPack::Boost => "Boost".to_string(),
            TopUpPack::PowerPack => "Power Pack".to_string(),
            TopUpPack::Custom(price) => format!("Custom (${:.2})", price),
        }
    }
}

/// Calculate the DD credits for a custom top-up price.
///
/// Uses the same rate as the Boost pack: 100 DD per $1 USD.
pub fn calculate_custom_dd(price_usd: f64) -> u64 {
    // 1 000 DD / $10 = 100 DD per $1.
    (price_usd * 100.0).round() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_packs() {
        let boost = TopUpPack::Boost;
        assert_eq!(boost.dd_credits(), 1_000);
        assert!((boost.price_usd() - 10.0).abs() < f64::EPSILON);

        let power = TopUpPack::PowerPack;
        assert_eq!(power.dd_credits(), 6_000);
        assert!((power.price_usd() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_custom_pack_proportional() {
        assert_eq!(calculate_custom_dd(10.0), 1_000);
        assert_eq!(calculate_custom_dd(50.0), 5_000);
        assert_eq!(calculate_custom_dd(1.0), 100);
        assert_eq!(calculate_custom_dd(25.0), 2_500);
        assert_eq!(calculate_custom_dd(0.0), 0);

        let custom = TopUpPack::Custom(25.0);
        assert_eq!(custom.dd_credits(), 2_500);
        assert!((custom.price_usd() - 25.0).abs() < f64::EPSILON);
    }
}
