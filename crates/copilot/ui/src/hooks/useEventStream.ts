import { useEffect, useRef, useCallback } from "react";
import { useAgentStore, type AgentStatus } from "../stores/agentStore";
import { useTaskStore, type TaskStatus } from "../stores/taskStore";
import { useConnectionStore } from "../stores/connectionStore";

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
  const updateTaskStatus = useTaskStore((s) => s.updateTaskStatus);
  const setConnected = useConnectionStore((s) => s.setConnected);
  const setLastEvent = useConnectionStore((s) => s.setLastEvent);

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
          // Future: update token count in store
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
          // Future: update cost display
          break;
        }
        default:
          break;
      }
    },
    [updateAgentStatus, updateTaskStatus, setLastEvent],
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
