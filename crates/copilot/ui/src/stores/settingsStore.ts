import { create } from "zustand";

export interface ProviderConfig {
  id: string;
  name: string;
  apiKey: string;
  endpoint: string;
  enabled: boolean;
}

export interface AgentConfig {
  id: string;
  name: string;
  role: string;
  model: string;
  enabled: boolean;
}

export type RiskLevel = "low" | "medium" | "high" | "critical";

export type ZoomLevel = 80 | 90 | 100 | 110 | 120 | 130 | 150;

interface SettingsState {
  providers: ProviderConfig[];
  agents: AgentConfig[];
  autoApprove: Record<RiskLevel, boolean>;
  zoomLevel: ZoomLevel;
  updateProvider: (id: string, updates: Partial<ProviderConfig>) => void;
  updateAgent: (id: string, updates: Partial<AgentConfig>) => void;
  setAutoApprove: (level: RiskLevel, value: boolean) => void;
  setZoomLevel: (level: ZoomLevel) => void;
}

const defaultProviders: ProviderConfig[] = [
  { id: "anthropic", name: "Anthropic", apiKey: "", endpoint: "https://api.anthropic.com", enabled: true },
  { id: "openai", name: "OpenAI", apiKey: "", endpoint: "https://api.openai.com/v1", enabled: false },
  { id: "ollama", name: "Ollama", apiKey: "", endpoint: "http://localhost:11434", enabled: false },
  { id: "custom", name: "Custom", apiKey: "", endpoint: "", enabled: false },
];

const defaultAgents: AgentConfig[] = [
  { id: "1", name: "Dr. Bob", role: "orchestrator", model: "claude-sonnet-4-20250514", enabled: true },
  { id: "2", name: "Scout", role: "researcher", model: "claude-sonnet-4-20250514", enabled: true },
  { id: "3", name: "Sage", role: "analyst", model: "claude-sonnet-4-20250514", enabled: true },
  { id: "4", name: "Quill", role: "writer", model: "claude-sonnet-4-20250514", enabled: true },
  { id: "5", name: "Pixel", role: "coder", model: "claude-sonnet-4-20250514", enabled: true },
  { id: "6", name: "Atlas", role: "operator", model: "claude-sonnet-4-20250514", enabled: true },
];

const defaultAutoApprove: Record<RiskLevel, boolean> = {
  low: true,
  medium: false,
  high: false,
  critical: false,
};

// Read persisted zoom level
const savedZoom = typeof window !== 'undefined'
  ? parseInt(localStorage.getItem('day1-zoom') || '100', 10) as ZoomLevel
  : 100 as ZoomLevel;

export const useSettingsStore = create<SettingsState>((set) => ({
  providers: defaultProviders,
  agents: defaultAgents,
  autoApprove: defaultAutoApprove,
  zoomLevel: savedZoom,
  updateProvider: (id, updates) =>
    set((state) => ({
      providers: state.providers.map((p) =>
        p.id === id ? { ...p, ...updates } : p,
      ),
    })),
  updateAgent: (id, updates) =>
    set((state) => ({
      agents: state.agents.map((a) =>
        a.id === id ? { ...a, ...updates } : a,
      ),
    })),
  setAutoApprove: (level: RiskLevel, value: boolean) =>
    set((state: SettingsState) => ({
      autoApprove: { ...state.autoApprove, [level]: value },
    })),
  setZoomLevel: (level: ZoomLevel) => {
    // Apply zoom by setting root font-size (scales all rem-based UI)
    document.documentElement.style.fontSize = `${level}%`;
    localStorage.setItem('day1-zoom', String(level));
    set({ zoomLevel: level });
  },
}));
