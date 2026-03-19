import { create } from "zustand";

export type SubscriptionTier = "free_man" | "mini_shop" | "rocket_inc";

export interface BillingState {
  /** Current subscription tier. */
  tier: SubscriptionTier;
  /** Maximum number of agents allowed by current tier. */
  maxAgents: number;
  /** Monthly DD credit allocation for current tier. */
  monthlyCredits: number;
  /** Monthly price (USD) for the current tier. */
  priceMonthly: number;
  /** Whether the upgrade prompt modal is visible. */
  showUpgradePrompt: boolean;
  /** Message displayed in the upgrade prompt. */
  upgradeMessage: string;

  setTier: (tier: SubscriptionTier) => void;
  openUpgradePrompt: (message: string) => void;
  closeUpgradePrompt: () => void;
}

const TIER_CONFIG: Record<
  SubscriptionTier,
  { maxAgents: number; monthlyDD: number; priceMonthly: number }
> = {
  free_man: { maxAgents: 1, monthlyDD: 0, priceMonthly: 0 },
  mini_shop: { maxAgents: 3, monthlyDD: 3_500, priceMonthly: 30 },
  rocket_inc: { maxAgents: 10, monthlyDD: 15_000, priceMonthly: 100 },
};

export const useBillingStore = create<BillingState>((set) => ({
  tier: "free_man",
  maxAgents: TIER_CONFIG.free_man.maxAgents,
  monthlyCredits: TIER_CONFIG.free_man.monthlyDD,
  priceMonthly: TIER_CONFIG.free_man.priceMonthly,
  showUpgradePrompt: false,
  upgradeMessage: "",

  setTier: (tier) =>
    set({
      tier,
      maxAgents: TIER_CONFIG[tier].maxAgents,
      monthlyCredits: TIER_CONFIG[tier].monthlyDD,
      priceMonthly: TIER_CONFIG[tier].priceMonthly,
    }),

  openUpgradePrompt: (message) =>
    set({ showUpgradePrompt: true, upgradeMessage: message }),

  closeUpgradePrompt: () =>
    set({ showUpgradePrompt: false, upgradeMessage: "" }),
}));
