import { useAgentStore } from "../../stores/agentStore";
import { useConnectionStore } from "../../stores/connectionStore";

export function StatusBar() {
  const agents = useAgentStore((s) => s.agents);
  const selectedAgentId = useAgentStore((s) => s.selectedAgentId);
  const isConnected = useConnectionStore((s) => s.isConnected);

  const activeAgent =
    agents.find((a) => a.id === selectedAgentId) ?? agents[0];

  return (
    <footer
      className="h-7 flex items-center justify-between px-4 border-t border-border
        bg-card/80 text-[11px] text-text-muted select-none shrink-0"
      role="contentinfo"
    >
      {/* Left: Agent status */}
      <div className="flex items-center gap-2 min-w-0 flex-1">
        {activeAgent ? (
          <>
            <span className="text-text-secondary truncate">
              Agent: {activeAgent.name}
            </span>
            <span aria-hidden="true">--</span>
            <span
              className={`capitalize ${
                activeAgent.status === "working" ||
                activeAgent.status === "thinking" ||
                activeAgent.status === "executing"
                  ? "text-accent"
                  : activeAgent.status === "error"
                    ? "text-error"
                    : "text-text-muted"
              }`}
            >
              {activeAgent.status}
            </span>
          </>
        ) : (
          <span>No agent active</span>
        )}
      </div>

      {/* Center: Token count */}
      <div className="flex-1 text-center">
        <span>Tokens: 0</span>
      </div>

      {/* Right: Latency + Connection */}
      <div className="flex items-center gap-3 flex-1 justify-end">
        <span>Latency: --</span>
        <div className="flex items-center gap-1.5">
          <span
            className={`inline-block w-1.5 h-1.5 rounded-full ${
              isConnected ? "bg-success" : "bg-error"
            }`}
            aria-label={isConnected ? "Connected" : "Disconnected"}
          />
          <span className="text-text-muted">
            {isConnected ? "Connected" : "Disconnected"}
          </span>
        </div>
      </div>
    </footer>
  );
}
