import { create } from "zustand";

export type AgentStatus =
  | "idle"
  | "working"
  | "thinking"
  | "executing"
  | "paused"
  | "error";

export interface Agent {
  id: string;
  name: string;
  role: string;
  status: AgentStatus;
}

interface AgentState {
  agents: Agent[];
  selectedAgentId: string | null;
  setAgents: (agents: Agent[]) => void;
  selectAgent: (id: string | null) => void;
  updateAgentStatus: (id: string, status: AgentStatus) => void;
}

export const useAgentStore = create<AgentState>((set) => ({
  agents: [
    {
      id: "researcher-1",
      name: "Researcher",
      role: "Research & Analysis",
      status: "idle",
    },
  ],
  selectedAgentId: null,
  setAgents: (agents) => set({ agents }),
  selectAgent: (id) => set({ selectedAgentId: id }),
  updateAgentStatus: (id, status) =>
    set((state) => ({
      agents: state.agents.map((a) =>
        a.id === id ? { ...a, status } : a,
      ),
    })),
}));
