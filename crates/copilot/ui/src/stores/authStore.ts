import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { useCostStore } from "./costStore";

export type SubscriptionTier = "free" | "mini_shop" | "rocket_inc";

export interface AuthState {
  isAuthenticated: boolean;
  isLoading: boolean;
  userId: string | null;
  apiKeyPrefix: string | null;
  ddBalance: number;
  subscriptionTier: SubscriptionTier;
  tierMaxAgents: number;
  error: string | null;
  authenticateWithApiKey: (apiKey: string) => Promise<boolean>;
  logout: () => void;
  refreshBalance: () => Promise<void>;
  checkStoredAuth: () => Promise<void>;
}

const TIER_AGENTS: Record<SubscriptionTier, number> = {
  free: 1,
  mini_shop: 3,
  rocket_inc: 6,
};

export const useAuthStore = create<AuthState>((set, get) => ({
  isAuthenticated: false,
  isLoading: false,
  userId: null,
  apiKeyPrefix: null,
  ddBalance: 0,
  subscriptionTier: "free",
  tierMaxAgents: 1,
  error: null,

  authenticateWithApiKey: async (apiKey: string): Promise<boolean> => {
    set({ isLoading: true, error: null });
    try {
      const prefix = await invoke<string>("store_api_key", { key: apiKey });
      const balanceData = await invoke<Record<string, unknown>>("fetch_balance", { apiKey });
      const balance = typeof balanceData.dd_balance === "number" ? balanceData.dd_balance : 0;
      const limit = typeof balanceData.dd_limit === "number" ? balanceData.dd_limit : 100;
      const tier = (balanceData.subscription_tier as SubscriptionTier) ?? "free";
      const userId = typeof balanceData.user_id === "string" ? balanceData.user_id : null;
      const costStore = useCostStore.getState();
      costStore.setBalance(balance);
      costStore.setLimit(limit);
      set({ isAuthenticated: true, isLoading: false, apiKeyPrefix: prefix, ddBalance: balance, subscriptionTier: tier, tierMaxAgents: TIER_AGENTS[tier] ?? 1, userId, error: null });
      return true;
    } catch (e) {
      set({ isLoading: false, error: typeof e === "string" ? e : "Authentication failed" });
      return false;
    }
  },

  logout: () => {
    invoke("clear_auth").catch(() => {});
    const costStore = useCostStore.getState();
    costStore.setBalance(0);
    costStore.setLimit(100);
    costStore.reset();
    set({ isAuthenticated: false, isLoading: false, userId: null, apiKeyPrefix: null, ddBalance: 0, subscriptionTier: "free", tierMaxAgents: 1, error: null });
  },

  refreshBalance: async () => {
    const state = get();
    if (!state.isAuthenticated) return;
    try {
      const storedKey = await invoke<string | null>("get_stored_api_key");
      if (!storedKey) return;
      const balanceData = await invoke<Record<string, unknown>>("fetch_balance", { apiKey: storedKey });
      const balance = typeof balanceData.dd_balance === "number" ? balanceData.dd_balance : 0;
      const limit = typeof balanceData.dd_limit === "number" ? balanceData.dd_limit : 100;
      const costStore = useCostStore.getState();
      costStore.setBalance(balance);
      costStore.setLimit(limit);
      set({ ddBalance: balance });
    } catch {
      // Silently fail
    }
  },

  checkStoredAuth: async () => {
    set({ isLoading: true });
    try {
      const storedKey = await invoke<string | null>("get_stored_api_key");
      if (!storedKey) { set({ isLoading: false }); return; }
      const prefix = storedKey.slice(0, Math.min(12, storedKey.length));
      const balanceData = await invoke<Record<string, unknown>>("fetch_balance", { apiKey: storedKey });
      const balance = typeof balanceData.dd_balance === "number" ? balanceData.dd_balance : 0;
      const limit = typeof balanceData.dd_limit === "number" ? balanceData.dd_limit : 100;
      const tier = (balanceData.subscription_tier as SubscriptionTier) ?? "free";
      const userId = typeof balanceData.user_id === "string" ? balanceData.user_id : null;
      const costStore = useCostStore.getState();
      costStore.setBalance(balance);
      costStore.setLimit(limit);
      set({ isAuthenticated: true, isLoading: false, apiKeyPrefix: prefix, ddBalance: balance, subscriptionTier: tier, tierMaxAgents: TIER_AGENTS[tier] ?? 1, userId, error: null });
    } catch {
      set({ isLoading: false });
    }
  },
}));
