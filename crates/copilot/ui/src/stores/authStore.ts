import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { useCostStore } from "./costStore";

export type SubscriptionTier = "free" | "mini_shop" | "rocket_inc";

export interface AuthState {
  isAuthenticated: boolean;
  isLoading: boolean;
  userId: string | null;
  ddBalance: number;
  subscriptionTier: SubscriptionTier;
  tierMaxAgents: number;
  error: string | null;
  /** Store JWT received from OAuth callback. */
  storeToken: (token: string) => Promise<void>;
  logout: () => void;
  refreshBalance: () => Promise<void>;
  checkStoredAuth: () => Promise<void>;
}

const TIER_AGENTS: Record<SubscriptionTier, number> = {
  free: 1,
  mini_shop: 3,
  rocket_inc: 10,
};

export const useAuthStore = create<AuthState>((set, get) => ({
  isAuthenticated: false,
  isLoading: false,
  userId: null,
  ddBalance: 0,
  subscriptionTier: "free",
  tierMaxAgents: 1,
  error: null,

  /** Called by the OAuth callback handler once a JWT is obtained. */
  storeToken: async (token: string): Promise<void> => {
    set({ isLoading: true, error: null });
    try {
      await invoke("store_auth_token", { token });
      const balanceData = await invoke<Record<string, unknown>>("fetch_balance", { apiKey: token });
      const balance = typeof balanceData.dd_balance === "number" ? balanceData.dd_balance : 0;
      const limit = typeof balanceData.dd_limit === "number" ? balanceData.dd_limit : 0;
      const tier = (balanceData.subscription_tier as SubscriptionTier) ?? "free";
      const userId = typeof balanceData.user_id === "string" ? balanceData.user_id : null;
      const costStore = useCostStore.getState();
      costStore.setBalance(balance);
      costStore.setLimit(limit);
      set({
        isAuthenticated: true,
        isLoading: false,
        ddBalance: balance,
        subscriptionTier: tier,
        tierMaxAgents: TIER_AGENTS[tier] ?? 1,
        userId,
        error: null,
      });
    } catch (e) {
      set({ isLoading: false, error: typeof e === "string" ? e : "Authentication failed" });
    }
  },

  logout: () => {
    invoke("clear_auth").catch(() => {});
    const costStore = useCostStore.getState();
    costStore.setBalance(0);
    costStore.setLimit(0);
    costStore.reset();
    set({
      isAuthenticated: false,
      isLoading: false,
      userId: null,
      ddBalance: 0,
      subscriptionTier: "free",
      tierMaxAgents: 1,
      error: null,
    });
  },

  refreshBalance: async () => {
    const state = get();
    if (!state.isAuthenticated) return;
    try {
      const storedToken = await invoke<string | null>("get_auth_token");
      if (!storedToken) return;
      const balanceData = await invoke<Record<string, unknown>>("fetch_balance", { apiKey: storedToken });
      const balance = typeof balanceData.dd_balance === "number" ? balanceData.dd_balance : 0;
      const limit = typeof balanceData.dd_limit === "number" ? balanceData.dd_limit : 0;
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
      const storedToken = await invoke<string | null>("get_auth_token");
      if (!storedToken) { set({ isLoading: false }); return; }
      const balanceData = await invoke<Record<string, unknown>>("fetch_balance", { apiKey: storedToken });
      const balance = typeof balanceData.dd_balance === "number" ? balanceData.dd_balance : 0;
      const limit = typeof balanceData.dd_limit === "number" ? balanceData.dd_limit : 0;
      const tier = (balanceData.subscription_tier as SubscriptionTier) ?? "free";
      const userId = typeof balanceData.user_id === "string" ? balanceData.user_id : null;
      const costStore = useCostStore.getState();
      costStore.setBalance(balance);
      costStore.setLimit(limit);
      set({
        isAuthenticated: true,
        isLoading: false,
        ddBalance: balance,
        subscriptionTier: tier,
        tierMaxAgents: TIER_AGENTS[tier] ?? 1,
        userId,
        error: null,
      });
    } catch {
      set({ isLoading: false });
    }
  },
}));
