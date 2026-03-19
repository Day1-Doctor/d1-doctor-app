import { useEffect, useRef, useCallback } from "react";
import { useAgentStore, type AgentStatus } from "../stores/agentStore";
import { useTaskStore, type TaskStatus } from "../stores/taskStore";
import { useConnectionStore } from "../stores/connectionStore";
import { useCostStore } from "../stores/costStore";

interface EventMessage {
  id: string;
  agent_id: string;
  timestamp: string;
  type: string;
  payload: Record<string, unknown>;
}

interface UseEventStreamOptions {
  url?: string;
  autoConnect?: boolean;
  reconnectInterval?: number;
}

export function useEventStream(options: UseEventStreamOptions = {}) {
  const {
    url = "ws://127.0.0.1:14200/ws/events",
    autoConnect = true,
    reconnectInterval = 3000,
  } = options;

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimerRef = useRef<ReturnType<typeof setTimeout> | undefined>(
    undefined,
  );
  const mountedRef = useRef(true);

  const updateAgentStatus = useAgentStore((s) => s.updateAgentStatus);
  const agents = useAgentStore((s) => s.agents);
  const updateTaskStatus = useTaskStore((s) => s.updateTaskStatus);
  const setConnected = useConnectionStore((s) => s.setConnected);
  const setLastEvent = useConnectionStore((s) => s.setLastEvent);
  const updateFromEvent = useCostStore((s) => s.updateFromEvent);

  const handleEvent = useCallback(
    (event: EventMessage) => {
      setLastEvent(event.timestamp);

      switch (event.type) {
        case "agent.state_changed": {
          const { to } = event.payload as { from: string; to: string };
          updateAgentStatus(event.agent_id, to as AgentStatus);
          break;
        }
        case "token.stream": {
          // Token streaming events are handled by cost.updated for DD tracking.
          // Individual token deltas are not tracked in the cost store.
          break;
        }
        case "tool.started": {
          updateAgentStatus(event.agent_id, "executing");
          break;
        }
        case "tool.finished": {
          updateAgentStatus(event.agent_id, "idle");
          break;
        }
        case "approval.requested": {
          // Future: surface approval request in UI
          break;
        }
        case "task.step_completed": {
          const { task_id, status } = event.payload as {
            task_id: string;
            status: string;
          };
          if (task_id && status) {
            updateTaskStatus(task_id, status as TaskStatus);
          }
          break;
        }
        case "cost.updated": {
          const { session_tokens, session_cost_dd } = event.payload as {
            session_tokens: number;
            session_cost_dd: number;
          };
          // Resolve agent_id to display name for the cost breakdown
          const costAgent = agents.find((a) => a.id === event.agent_id);
          const agentName = costAgent?.name ?? event.agent_id;
          updateFromEvent(session_tokens, session_cost_dd, agentName);
          break;
        }
        default:
          break;
      }
    },
    [updateAgentStatus, updateTaskStatus, setLastEvent, updateFromEvent, agents],
  );

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;
    if (wsRef.current?.readyState === WebSocket.CONNECTING) return;

    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      setConnected(true);
      // Subscribe to all events
      ws.send(JSON.stringify({ type: "subscribe", filters: {} }));
    };

    ws.onmessage = (event: MessageEvent) => {
      try {
        const data: EventMessage = JSON.parse(event.data as string);
        handleEvent(data);
      } catch {
        // Skip non-JSON messages (e.g. pings)
      }
    };

    ws.onclose = () => {
      setConnected(false);
      wsRef.current = null;
      // Auto-reconnect if still mounted
      if (mountedRef.current) {
        reconnectTimerRef.current = setTimeout(connect, reconnectInterval);
      }
    };

    ws.onerror = () => {
      ws.close();
    };
  }, [url, reconnectInterval, handleEvent, setConnected]);

  const disconnect = useCallback(() => {
    mountedRef.current = false;
    clearTimeout(reconnectTimerRef.current);
    if (wsRef.current) {
      wsRef.current.onclose = null; // Prevent reconnect on intentional close
      wsRef.current.close();
      wsRef.current = null;
    }
    setConnected(false);
  }, [setConnected]);

  useEffect(() => {
    mountedRef.current = true;
    if (autoConnect) connect();
    return () => disconnect();
  }, [autoConnect, connect, disconnect]);

  return {
    connect,
    disconnect,
  };
}
