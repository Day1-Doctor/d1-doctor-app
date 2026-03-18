import { create } from "zustand";

export type EventCategory = "agent" | "tool" | "task" | "cost" | "system";

export interface EventLogEntry {
  id: string;
  timestamp: string;
  type: string;
  category: EventCategory;
  agentId: string;
  agentName: string;
  payload: Record<string, unknown>;
}

export interface ToolTraceEntry {
  id: string;
  toolName: string;
  agentId: string;
  agentName: string;
  params: string;
  durationMs: number;
  status: "success" | "fail";
  timestamp: string;
}

export interface AgentTokenUsage {
  agentName: string;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  costDD: number;
}

interface EventLogState {
  events: EventLogEntry[];
  toolTraces: ToolTraceEntry[];
  agentTokenUsage: AgentTokenUsage[];
  addEvent: (event: EventLogEntry) => void;
  addToolTrace: (trace: ToolTraceEntry) => void;
  clearEvents: () => void;
}

function categorize(type: string): EventCategory {
  if (type.startsWith("agent.")) return "agent";
  if (type.startsWith("tool.")) return "tool";
  if (type.startsWith("task.")) return "task";
  if (type.startsWith("cost.")) return "cost";
  return "system";
}

const now = new Date();
function ts(offsetMs: number): string {
  return new Date(now.getTime() - offsetMs).toISOString();
}

const mockEvents: EventLogEntry[] = [
  {
    id: "evt-001",
    timestamp: ts(120000),
    type: "agent.state_changed",
    category: "agent",
    agentId: "1",
    agentName: "Dr. Bob",
    payload: { from: "idle", to: "working" },
  },
  {
    id: "evt-002",
    timestamp: ts(115000),
    type: "task.created",
    category: "task",
    agentId: "1",
    agentName: "Dr. Bob",
    payload: { task_id: "task-1", title: "Compare AI Frameworks" },
  },
  {
    id: "evt-003",
    timestamp: ts(110000),
    type: "task.decomposed",
    category: "task",
    agentId: "1",
    agentName: "Dr. Bob",
    payload: { task_id: "task-1", subtask_count: 4 },
  },
  {
    id: "evt-004",
    timestamp: ts(105000),
    type: "agent.state_changed",
    category: "agent",
    agentId: "2",
    agentName: "Scout",
    payload: { from: "idle", to: "working" },
  },
  {
    id: "evt-005",
    timestamp: ts(100000),
    type: "tool.started",
    category: "tool",
    agentId: "2",
    agentName: "Scout",
    payload: { tool: "web-search", params: { query: "AI frameworks comparison 2026" } },
  },
  {
    id: "evt-006",
    timestamp: ts(95000),
    type: "tool.finished",
    category: "tool",
    agentId: "2",
    agentName: "Scout",
    payload: { tool: "web-search", duration_ms: 2340, status: "success" },
  },
  {
    id: "evt-007",
    timestamp: ts(90000),
    type: "cost.updated",
    category: "cost",
    agentId: "2",
    agentName: "Scout",
    payload: { tokens: 1200, cost_dd: 3 },
  },
  {
    id: "evt-008",
    timestamp: ts(85000),
    type: "agent.state_changed",
    category: "agent",
    agentId: "2",
    agentName: "Scout",
    payload: { from: "executing", to: "thinking" },
  },
  {
    id: "evt-009",
    timestamp: ts(80000),
    type: "tool.started",
    category: "tool",
    agentId: "2",
    agentName: "Scout",
    payload: { tool: "web-fetch", params: { url: "https://example.com/ai-report" } },
  },
  {
    id: "evt-010",
    timestamp: ts(75000),
    type: "tool.finished",
    category: "tool",
    agentId: "2",
    agentName: "Scout",
    payload: { tool: "web-fetch", duration_ms: 1520, status: "success" },
  },
  {
    id: "evt-011",
    timestamp: ts(70000),
    type: "task.step_completed",
    category: "task",
    agentId: "2",
    agentName: "Scout",
    payload: { task_id: "task-1", step: "Research AI frameworks", status: "completed" },
  },
  {
    id: "evt-012",
    timestamp: ts(65000),
    type: "agent.state_changed",
    category: "agent",
    agentId: "3",
    agentName: "Sage",
    payload: { from: "idle", to: "thinking" },
  },
  {
    id: "evt-013",
    timestamp: ts(60000),
    type: "cost.updated",
    category: "cost",
    agentId: "3",
    agentName: "Sage",
    payload: { tokens: 2400, cost_dd: 5 },
  },
  {
    id: "evt-014",
    timestamp: ts(55000),
    type: "tool.started",
    category: "tool",
    agentId: "5",
    agentName: "Pixel",
    payload: { tool: "shell", params: { cmd: "npm test" } },
  },
  {
    id: "evt-015",
    timestamp: ts(50000),
    type: "tool.finished",
    category: "tool",
    agentId: "5",
    agentName: "Pixel",
    payload: { tool: "shell", duration_ms: 4200, status: "success" },
  },
  {
    id: "evt-016",
    timestamp: ts(45000),
    type: "agent.state_changed",
    category: "agent",
    agentId: "4",
    agentName: "Quill",
    payload: { from: "idle", to: "executing" },
  },
  {
    id: "evt-017",
    timestamp: ts(40000),
    type: "tool.started",
    category: "tool",
    agentId: "4",
    agentName: "Quill",
    payload: { tool: "filesystem", params: { action: "write", path: "/report.md" } },
  },
  {
    id: "evt-018",
    timestamp: ts(35000),
    type: "tool.finished",
    category: "tool",
    agentId: "4",
    agentName: "Quill",
    payload: { tool: "filesystem", duration_ms: 120, status: "success" },
  },
  {
    id: "evt-019",
    timestamp: ts(30000),
    type: "cost.updated",
    category: "cost",
    agentId: "4",
    agentName: "Quill",
    payload: { tokens: 3100, cost_dd: 4 },
  },
  {
    id: "evt-020",
    timestamp: ts(25000),
    type: "system.health_check",
    category: "system",
    agentId: "",
    agentName: "System",
    payload: { status: "ok", uptime_s: 300 },
  },
];

const mockToolTraces: ToolTraceEntry[] = [
  { id: "tt-1", toolName: "web-search", agentId: "2", agentName: "Scout", params: '{"query":"AI frameworks comparison 2026"}', durationMs: 2340, status: "success", timestamp: ts(100000) },
  { id: "tt-2", toolName: "web-fetch", agentId: "2", agentName: "Scout", params: '{"url":"https://example.com/ai-report"}', durationMs: 1520, status: "success", timestamp: ts(80000) },
  { id: "tt-3", toolName: "shell", agentId: "5", agentName: "Pixel", params: '{"cmd":"npm test"}', durationMs: 4200, status: "success", timestamp: ts(55000) },
  { id: "tt-4", toolName: "filesystem", agentId: "4", agentName: "Quill", params: '{"action":"write","path":"/report.md"}', durationMs: 120, status: "success", timestamp: ts(40000) },
  { id: "tt-5", toolName: "web-search", agentId: "2", agentName: "Scout", params: '{"query":"LLM benchmarks"}', durationMs: 3100, status: "fail", timestamp: ts(20000) },
  { id: "tt-6", toolName: "memory", agentId: "1", agentName: "Dr. Bob", params: '{"op":"read","key":"session"}', durationMs: 45, status: "success", timestamp: ts(15000) },
];

const mockAgentTokenUsage: AgentTokenUsage[] = [
  { agentName: "Dr. Bob", inputTokens: 1200, outputTokens: 1250, totalTokens: 2450, costDD: 3 },
  { agentName: "Scout", inputTokens: 4800, outputTokens: 3400, totalTokens: 8200, costDD: 8 },
  { agentName: "Sage", inputTokens: 2600, outputTokens: 2500, totalTokens: 5100, costDD: 5 },
  { agentName: "Quill", inputTokens: 1800, outputTokens: 2500, totalTokens: 4300, costDD: 4 },
  { agentName: "Pixel", inputTokens: 3200, outputTokens: 3500, totalTokens: 6700, costDD: 7 },
  { agentName: "Atlas", inputTokens: 600, outputTokens: 600, totalTokens: 1200, costDD: 1 },
];

export { categorize };

export const useEventLogStore = create<EventLogState>((set) => ({
  events: mockEvents,
  toolTraces: mockToolTraces,
  agentTokenUsage: mockAgentTokenUsage,
  addEvent: (event) =>
    set((state) => ({
      events: [...state.events, { ...event, category: categorize(event.type) }],
    })),
  addToolTrace: (trace) =>
    set((state) => ({
      toolTraces: [...state.toolTraces, trace],
    })),
  clearEvents: () => set({ events: [], toolTraces: [] }),
}));
