import { create } from "zustand";

export type RiskLevel = "medium" | "high" | "critical";

export interface ApprovalRequest {
  id: string;
  agentName: string;
  agentRole: string;
  toolName: string;
  params: Record<string, unknown>;
  riskLevel: RiskLevel;
  context: string;
}

export type ApprovalDecision = "allow_once" | "allow_always" | "reject";

export interface ApprovalState {
  /** Queue of pending approval requests (FIFO order). */
  pendingApprovals: ApprovalRequest[];

  /** Per-agent trust overrides: `${agentName}:${riskLevel}` -> true. */
  trustedAgents: Record<string, boolean>;

  /** Add a new approval request to the pending queue. */
  addApproval: (req: ApprovalRequest) => void;

  /** Respond to a pending approval request and remove it from the queue. */
  respond: (id: string, decision: ApprovalDecision) => void;

  /** Check whether a given agent is trusted at a given risk level. */
  isTrusted: (agentName: string, riskLevel: RiskLevel) => boolean;
}

export const useApprovalStore = create<ApprovalState>((set, get) => ({
  pendingApprovals: [],
  trustedAgents: {},

  addApproval: (req) =>
    set((state) => ({
      pendingApprovals: [...state.pendingApprovals, req],
    })),

  respond: (id, decision) =>
    set((state) => {
      const request = state.pendingApprovals.find((r) => r.id === id);
      const newTrusted = { ...state.trustedAgents };

      // When "allow_always" is chosen, record the trust override.
      if (decision === "allow_always" && request) {
        const key = `${request.agentName}:${request.riskLevel}`;
        newTrusted[key] = true;
      }

      return {
        pendingApprovals: state.pendingApprovals.filter((r) => r.id !== id),
        trustedAgents: newTrusted,
      };
    }),

  isTrusted: (agentName, riskLevel) => {
    const key = `${agentName}:${riskLevel}`;
    return get().trustedAgents[key] === true;
  },
}));
