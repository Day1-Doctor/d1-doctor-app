use serde::{Deserialize, Serialize};

/// Describes the transition plan from v2.x credit packs to v3.0 subscriptions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationPlan {
    /// Short label for this migration.
    pub name: String,
    /// Detailed description shown in the migration notice.
    pub description: String,
    /// Whether v2.x credits can still be used during the transition window.
    pub credits_still_valid: bool,
    /// Number of days remaining in the transition window.
    pub transition_days_remaining: u32,
}

impl MigrationPlan {
    /// Create the default v2.x -> v3.0 migration plan.
    pub fn default_plan() -> Self {
        Self {
            name: "v2.x Credit Pack Sunset".to_string(),
            description: concat!(
                "Starting with v3.0, DD credit packs are replaced by subscription tiers. ",
                "Your remaining v2.x credits will be honored for 90 days after upgrade. ",
                "After the transition window, unused v2.x credits convert to your new ",
                "tier's monthly allocation at a 1:1 ratio (up to the tier cap)."
            )
            .to_string(),
            credits_still_valid: true,
            transition_days_remaining: 90,
        }
    }
}

/// Determine whether the in-app migration notice should be displayed.
///
/// Returns `true` if the user has v2.x credits that need to be migrated.
pub fn should_show_migration_notice(user_has_v2_credits: bool) -> bool {
    user_has_v2_credits
}

/// Message templates for the in-app migration notice.
pub struct MigrationMessages;

impl MigrationMessages {
    /// Title for the migration banner.
    pub fn banner_title() -> &'static str {
        "Your credit plan is changing"
    }

    /// Body text explaining the migration.
    pub fn banner_body() -> &'static str {
        "Day1 Doctor is moving to subscription tiers in v3.0. Your existing DD credits \
         will be honored for 90 days. Tap below to choose your new plan."
    }

    /// CTA button text.
    pub fn banner_cta() -> &'static str {
        "View Plans"
    }

    /// Dismissal confirmation text.
    pub fn dismiss_text() -> &'static str {
        "Remind me later"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_show_migration_notice() {
        assert!(should_show_migration_notice(true));
        assert!(!should_show_migration_notice(false));
    }

    #[test]
    fn test_default_migration_plan() {
        let plan = MigrationPlan::default_plan();
        assert_eq!(plan.name, "v2.x Credit Pack Sunset");
        assert!(plan.credits_still_valid);
        assert_eq!(plan.transition_days_remaining, 90);
        assert!(plan.description.contains("subscription tiers"));
    }
}
