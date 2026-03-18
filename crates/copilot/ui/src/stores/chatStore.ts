import { create } from "zustand";

export interface ChatMessage {
  id: string;
  agentId: string;
  agentName: string;
  role: "user" | "agent";
  content: string;
  timestamp: string;
}

interface ChatState {
  messages: ChatMessage[];
  selectedAgentId: string;
  addMessage: (message: ChatMessage) => void;
  setSelectedAgent: (agentId: string) => void;
  getMessagesForAgent: (agentId: string) => ChatMessage[];
}

const mockMessages: ChatMessage[] = [
  {
    id: "msg-1",
    agentId: "1",
    agentName: "Dr. Bob",
    role: "user",
    content: "Can you help me compare different AI frameworks for our project?",
    timestamp: "2026-03-18T10:00:00Z",
  },
  {
    id: "msg-2",
    agentId: "1",
    agentName: "Dr. Bob",
    role: "agent",
    content:
      "Of course! I can coordinate our team to analyze the top AI frameworks. I will have Scout research the options, Sage analyze the data, and Quill compile the report. Should I focus on any specific criteria like performance, ecosystem maturity, or ease of use?",
    timestamp: "2026-03-18T10:00:15Z",
  },
  {
    id: "msg-3",
    agentId: "1",
    agentName: "Dr. Bob",
    role: "user",
    content: "Focus on production readiness and community support. Also compare inference speed.",
    timestamp: "2026-03-18T10:01:00Z",
  },
];

let nextMsgId = 4;

export const useChatStore = create<ChatState>((set, get) => ({
  messages: mockMessages,
  selectedAgentId: "1",
  addMessage: (message) =>
    set((state) => ({ messages: [...state.messages, message] })),
  setSelectedAgent: (agentId) => set({ selectedAgentId: agentId }),
  getMessagesForAgent: (agentId) => {
    return get().messages.filter((m) => m.agentId === agentId);
  },
}));

/** Generate a unique message ID. */
export function generateMsgId(): string {
  return `msg-${nextMsgId++}`;
}
