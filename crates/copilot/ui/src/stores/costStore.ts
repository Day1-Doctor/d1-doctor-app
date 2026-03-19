import { create } from "zustand";

export interface CostState {
  /** Current DD credit balance. */
  balance: number;
  /** Maximum DD credit limit. */
  limit: number;
  /** Total session cost in DD credits. */
  sessionCost: number;
  /** Per-agent cost breakdown (agent name -> DD credits). */
  agentCosts: Record<string, number>;
  setBalance: (balance: number) => void;
  setLimit: (limit: number) => void;
  addCost: (agentName: string, amount: number) => void;
  /** Update session totals from a WebSocket cost.updated event. */
  updateFromEvent: (sessionTokens: number, sessionCostDd: number, agentName: string) => void;
  /** Set the initial balance and limit (e.g., from a /balance API call). */
  setInitialBalance: (balance: number, limit: number) => void;
  reset: () => void;
}

export const useCostStore = create<CostState>((set) => ({
  balance: 0,
  limit: 0,
  sessionCost: 0,
  agentCosts: {},
  setBalance: (balance) => set({ balance }),
  setLimit: (limit) => set({ limit }),
  addCost: (agentName, amount) =>
    set((state) => ({
      sessionCost: state.sessionCost + amount,
      balance: Math.max(0, state.balance - amount),
      agentCosts: {
        ...state.agentCosts,
        [agentName]: (state.agentCosts[agentName] ?? 0) + amount,
      },
    })),
  updateFromEvent: (_sessionTokens, sessionCostDd, agentName) =>
    set((state) => {
      const delta = sessionCostDd - state.sessionCost;
      return {
        sessionCost: sessionCostDd,
        balance: Math.max(0, state.balance - delta),
        agentCosts: {
          ...state.agentCosts,
          [agentName]: (state.agentCosts[agentName] ?? 0) + delta,
        },
      };
    }),
  setInitialBalance: (balance, limit) =>
    set({ balance, limit }),
  reset: () =>
    set({ sessionCost: 0, agentCosts: {} }),
}));
