import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

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
  trust_score?: number;
  room?: string;
}

interface AgentState {
  agents: Agent[];
  selectedAgentId: string | null;
  loading: boolean;
  setAgents: (agents: Agent[]) => void;
  selectAgent: (id: string | null) => void;
  updateAgentStatus: (id: string, status: AgentStatus) => void;
  getAgentByRole: (role: string) => Agent | undefined;
  fetchAgents: () => Promise<void>;
}

/** Mock agents as fallback when not in Tauri context. */
const mockAgents: Agent[] = [
  { id: "1", name: "Dr. Bob", role: "orchestrator", status: "idle" },
  { id: "2", name: "Scout", role: "researcher", status: "idle" },
  { id: "3", name: "Sage", role: "analyst", status: "idle" },
  { id: "4", name: "Quill", role: "writer", status: "idle" },
  { id: "5", name: "Pixel", role: "coder", status: "idle" },
  { id: "6", name: "Atlas", role: "operator", status: "idle" },
];

export const useAgentStore = create<AgentState>((set, get) => ({
  agents: mockAgents,
  selectedAgentId: null,
  loading: false,
  setAgents: (agents: Agent[]) => set({ agents }),
  selectAgent: (id: string | null) => set({ selectedAgentId: id }),
  updateAgentStatus: (id: string, status: AgentStatus) =>
    set((state: AgentState) => ({
      agents: state.agents.map((a: Agent) =>
        a.id === id ? { ...a, status } : a,
      ),
    })),
  getAgentByRole: (role: string) => {
    return get().agents.find((a: Agent) => a.role === role);
  },
  fetchAgents: async () => {
    set({ loading: true });
    try {
      const agents = await invoke<Agent[]>("list_agents");
      if (agents && agents.length > 0) {
        set({ agents, loading: false });
      } else {
        set({ loading: false });
      }
    } catch {
      // Not in Tauri context or runtime not ready — keep mock data
      set({ loading: false });
    }
  },
}));
