import { useEffect, useRef, useCallback } from "react";
import { useAgentStore, type AgentStatus } from "../stores/agentStore";
import { useTaskStore, type TaskStatus } from "../stores/taskStore";
import { useConnectionStore } from "../stores/connectionStore";
import { useCostStore } from "../stores/costStore";
import { useApprovalStore, type RiskLevel } from "../stores/approvalStore";
import { useArtifactStore, type ArtifactType } from "../stores/artifactStore";
import { useEventLogStore, categorize } from "../stores/eventLogStore";

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
  const updateStepStatus = useTaskStore((s) => s.updateStepStatus);
  const setConnected = useConnectionStore((s) => s.setConnected);
  const setLastEvent = useConnectionStore((s) => s.setLastEvent);
  const updateFromEvent = useCostStore((s) => s.updateFromEvent);
  const addApproval = useApprovalStore((s) => s.addApproval);
  const addArtifact = useArtifactStore((s) => s.addArtifact);
  const addEvent = useEventLogStore((s) => s.addEvent);
  const addToolTrace = useEventLogStore((s) => s.addToolTrace);

  const handleEvent = useCallback(
    (event: EventMessage) => {
      setLastEvent(event.timestamp);

      // Resolve agent display name for event logging
      const resolveAgentName = (): string => {
        const agent = agents.find((a) => a.id === event.agent_id);
        return agent?.name ?? event.agent_id;
      };

      // Log every event to the event log store
      addEvent({
        id: event.id,
        timestamp: event.timestamp,
        type: event.type,
        category: categorize(event.type),
        agentId: event.agent_id,
        agentName: resolveAgentName(),
        payload: event.payload,
      });

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
          const { tool_name, params } = event.payload as {
            tool_name?: string;
            tool?: string;
            params?: unknown;
          };
          const toolName = tool_name ?? (event.payload.tool as string) ?? "unknown";
          updateAgentStatus(event.agent_id, "executing");
          addToolTrace({
            id: `tt-${event.id}`,
            toolName,
            agentId: event.agent_id,
            agentName: resolveAgentName(),
            params: params ? JSON.stringify(params) : "{}",
            durationMs: 0, // Will be updated on tool.finished
            status: "success",
            timestamp: event.timestamp,
          });
          break;
        }
        case "tool.finished": {
          const { tool_name, duration_ms, status } = event.payload as {
            tool_name?: string;
            tool?: string;
            result?: unknown;
            duration_ms?: number;
            status?: string;
          };
          const toolName = tool_name ?? (event.payload.tool as string) ?? "unknown";
          updateAgentStatus(event.agent_id, "idle");
          // Add a completed tool trace entry
          addToolTrace({
            id: `tt-${event.id}`,
            toolName,
            agentId: event.agent_id,
            agentName: resolveAgentName(),
            params: "{}",
            durationMs: duration_ms ?? 0,
            status: (status === "fail" ? "fail" : "success") as "success" | "fail",
            timestamp: event.timestamp,
          });
          break;
        }
        case "approval.requested": {
          const { action, risk_level, context, tool_name, params } = event.payload as {
            action?: string;
            risk_level: string;
            context: string;
            tool_name?: string;
            params?: Record<string, unknown>;
          };
          const agent = agents.find((a) => a.id === event.agent_id);
          addApproval({
            id: event.id,
            agentName: agent?.name ?? event.agent_id,
            agentRole: agent?.role ?? "unknown",
            toolName: action ?? tool_name ?? "unknown",
            params: params ?? {},
            riskLevel: risk_level as RiskLevel,
            context: context ?? "",
          });
          break;
        }
        case "artifact.created": {
          const { task_id, artifact_type, path, name, size } = event.payload as {
            task_id?: string;
            artifact_type: string;
            path?: string;
            name?: string;
            size?: number;
          };
          const agent = agents.find((a) => a.id === event.agent_id);
          const artifactName = name ?? path?.split("/").pop() ?? `artifact-${event.id}`;
          addArtifact({
            id: event.id,
            name: artifactName,
            type: (artifact_type as ArtifactType) ?? "document",
            agent: agent?.name ?? event.agent_id,
            size: size ?? 0,
            timestamp: event.timestamp,
          });
          void task_id; // task_id available for future task-artifact linking
          break;
        }
        case "task.step_completed": {
          const { task_id, step_index, step_id, status, duration_ms } = event.payload as {
            task_id: string;
            step_index?: number;
            step_id?: string;
            status?: string;
            result?: unknown;
            duration_ms?: number;
          };
          // Update the parent task status if provided
          if (task_id && status) {
            updateTaskStatus(task_id, status as TaskStatus);
          }
          // Update individual step status if step_id is available
          if (task_id && step_id) {
            updateStepStatus(
              task_id,
              step_id,
              "completed",
              duration_ms,
            );
          }
          // If only step_index is available, use it as a fallback identifier
          if (task_id && step_index !== undefined && !step_id) {
            updateStepStatus(
              task_id,
              String(step_index),
              "completed",
              duration_ms,
            );
          }
          break;
        }
        case "task.created": {
          // Task creation events are logged via addEvent above.
          // The task store is updated via the create_task Tauri command response.
          break;
        }
        case "task.status_changed": {
          const { task_id, to } = event.payload as {
            task_id: string;
            from?: string;
            to: string;
          };
          if (task_id && to) {
            updateTaskStatus(task_id, to as TaskStatus);
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
          const costAgentName = costAgent?.name ?? event.agent_id;
          updateFromEvent(session_tokens, session_cost_dd, costAgentName);
          break;
        }
        default:
          break;
      }
    },
    [
      updateAgentStatus,
      updateTaskStatus,
      updateStepStatus,
      setLastEvent,
      updateFromEvent,
      addApproval,
      addArtifact,
      addEvent,
      addToolTrace,
      agents,
    ],
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
