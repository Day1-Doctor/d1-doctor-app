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
  getAgentByRole: (role: string) => Agent | undefined;
}

/** Mock agents for development — one per office desk. */
const mockAgents: Agent[] = [
  { id: "1", name: "Dr. Bob", role: "orchestrator", status: "idle" },
  { id: "2", name: "Scout", role: "researcher", status: "thinking" },
  { id: "3", name: "Sage", role: "analyst", status: "idle" },
  { id: "4", name: "Quill", role: "writer", status: "executing" },
  { id: "5", name: "Pixel", role: "coder", status: "working" },
  { id: "6", name: "Atlas", role: "operator", status: "idle" },
];

export const useAgentStore = create<AgentState>((set, get) => ({
  agents: mockAgents,
  selectedAgentId: null,
  setAgents: (agents) => set({ agents }),
  selectAgent: (id) => set({ selectedAgentId: id }),
  updateAgentStatus: (id, status) =>
    set((state) => ({
      agents: state.agents.map((a) =>
        a.id === id ? { ...a, status } : a,
      ),
    })),
  getAgentByRole: (role) => {
    return get().agents.find((a) => a.role === role);
  },
}));
