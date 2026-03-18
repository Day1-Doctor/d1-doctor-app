import { useAgentStore, type Agent } from "../stores/agentStore";

export function useAgentState(agentId: string): Agent | null {
  const agent = useAgentStore((s) => s.agents.find((a) => a.id === agentId));
  return agent ?? null;
}

export function useAllAgents(): Agent[] {
  return useAgentStore((s) => s.agents);
}

export function useSelectedAgent(): Agent | null {
  const selectedId = useAgentStore((s) => s.selectedAgentId);
  const agent = useAgentStore((s) =>
    s.agents.find((a) => a.id === selectedId),
  );
  return agent ?? null;
}
