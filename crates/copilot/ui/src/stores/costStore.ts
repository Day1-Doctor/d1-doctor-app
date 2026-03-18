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
  reset: () => void;
}

export const useCostStore = create<CostState>((set) => ({
  balance: 42,
  limit: 100,
  sessionCost: 12,
  agentCosts: { Scout: 8, Quill: 4 },
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
  reset: () =>
    set({ sessionCost: 0, agentCosts: {} }),
}));
