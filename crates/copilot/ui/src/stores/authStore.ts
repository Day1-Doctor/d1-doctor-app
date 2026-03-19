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
  token: string | null;
  /** Store JWT received from OAuth callback. */
  storeToken: (token: string, refreshToken?: string) => Promise<void>;
  /** Handle a deep link OAuth callback URL. */
  handleDeepLinkCallback: (url: string) => Promise<void>;
  logout: () => void;
  refreshBalance: () => Promise<void>;
  /** Refresh the JWT using the stored refresh token. */
  refreshToken: () => Promise<void>;
  checkStoredAuth: () => Promise<void>;
  /** Start the automatic token refresh interval. */
  startRefreshInterval: () => void;
  /** Stop the automatic token refresh interval. */
  stopRefreshInterval: () => void;
}

const TIER_AGENTS: Record<SubscriptionTier, number> = {
  free: 1,
  mini_shop: 3,
  rocket_inc: 10,
};

/** 50-minute refresh cycle (JWT typically expires in 60 min). */
const REFRESH_INTERVAL_MS = 50 * 60 * 1000;

let refreshInterval: ReturnType<typeof setInterval> | null = null;

export const useAuthStore = create<AuthState>((set, get) => ({
  isAuthenticated: false,
  isLoading: false,
  userId: null,
  ddBalance: 0,
  subscriptionTier: "free",
  tierMaxAgents: 1,
  error: null,
  token: null,

  /** Called by the OAuth callback handler once a JWT is obtained. */
  storeToken: async (token: string, refreshToken?: string): Promise<void> => {
    set({ isLoading: true, error: null });
    try {
      await invoke("store_auth_token", { token, refreshToken: refreshToken ?? null });
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
        token,
        error: null,
      });
      get().startRefreshInterval();
    } catch (e) {
      set({ isLoading: false, error: typeof e === "string" ? e : "Authentication failed" });
    }
  },

  /** Handle a deep link callback URL (day1copilot://auth/callback?token=...). */
  handleDeepLinkCallback: async (url: string): Promise<void> => {
    set({ isLoading: true, error: null });
    try {
      const token = await invoke<string>("handle_auth_callback", { url });
      // Fetch balance and update state
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
        token,
        error: null,
      });
      get().startRefreshInterval();
    } catch (e) {
      set({ isLoading: false, error: typeof e === "string" ? e : "OAuth callback failed" });
    }
  },

  logout: () => {
    get().stopRefreshInterval();
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
      token: null,
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

  /** Attempt to refresh the JWT via the stored refresh token. */
  refreshToken: async () => {
    const state = get();
    if (!state.isAuthenticated) return;
    try {
      const newToken = await invoke<string | null>("refresh_auth_token");
      if (newToken) {
        set({ token: newToken });
      }
    } catch (e) {
      // Token refresh is non-fatal; log but don't break the session
      if (typeof e === "string" && e.includes("401")) {
        // Refresh token expired — force re-auth
        get().logout();
      }
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
        token: storedToken,
        error: null,
      });
      get().startRefreshInterval();
    } catch {
      set({ isLoading: false });
    }
  },

  startRefreshInterval: () => {
    if (refreshInterval) clearInterval(refreshInterval);
    refreshInterval = setInterval(() => {
      get().refreshToken();
    }, REFRESH_INTERVAL_MS);
  },

  stopRefreshInterval: () => {
    if (refreshInterval) {
      clearInterval(refreshInterval);
      refreshInterval = null;
    }
  },
}));
