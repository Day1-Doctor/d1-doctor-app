import { create } from "zustand";

export type SubscriptionTier = "free_man" | "mini_shop" | "rocket_inc";

export interface BillingState {
  /** Current subscription tier. */
  tier: SubscriptionTier;
  /** Maximum number of agents allowed by current tier. */
  maxAgents: number;
  /** Monthly DD credit allocation for current tier. */
  monthlyCredits: number;
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
  { maxAgents: number; monthlyCredits: number }
> = {
  free_man: { maxAgents: 1, monthlyCredits: 100 },
  mini_shop: { maxAgents: 3, monthlyCredits: 1_000 },
  rocket_inc: { maxAgents: 6, monthlyCredits: 5_000 },
};

export const useBillingStore = create<BillingState>((set) => ({
  tier: "free_man",
  maxAgents: TIER_CONFIG.free_man.maxAgents,
  monthlyCredits: TIER_CONFIG.free_man.monthlyCredits,
  showUpgradePrompt: false,
  upgradeMessage: "",

  setTier: (tier) =>
    set({
      tier,
      maxAgents: TIER_CONFIG[tier].maxAgents,
      monthlyCredits: TIER_CONFIG[tier].monthlyCredits,
    }),

  openUpgradePrompt: (message) =>
    set({ showUpgradePrompt: true, upgradeMessage: message }),

  closeUpgradePrompt: () =>
    set({ showUpgradePrompt: false, upgradeMessage: "" }),
}));
