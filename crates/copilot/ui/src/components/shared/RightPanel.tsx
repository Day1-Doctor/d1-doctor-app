import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useLayoutStore } from "../../stores/layoutStore";
import { useAgentStore } from "../../stores/agentStore";
import { useChatStore, generateMsgId } from "../../stores/chatStore";
import { useCostStore } from "../../stores/costStore";

type CommandCenterMode = "chat" | "agents";

/** Agent role to accent color mapping. */
const agentRoleColor: Record<string, string> = {
  orchestrator: "#F97316",
  researcher: "#22C55E",
  analyst: "#3B82F6",
  writer: "#A855F7",
  coder: "#EC4899",
  operator: "#F59E0B",
};

/** Status dot colors. */
const statusColor: Record<string, string> = {
  idle: "#6B7280",
  working: "#22C55E",
  thinking: "#3B82F6",
  executing: "#F97316",
  paused: "#F59E0B",
  error: "#EF4444",
};

function formatTimestamp(ts: string): string {
  const date = new Date(ts);
  return `${date.getHours().toString().padStart(2, "0")}:${date.getMinutes().toString().padStart(2, "0")}`;
}

// ── Chat Panel ─────────────────────────────────────────────────────────────

function ChatPanel() {
  const { t } = useTranslation();
  const agents = useAgentStore((s) => s.agents);
  const selectedAgentId = useChatStore((s) => s.selectedAgentId);
  const setSelectedAgent = useChatStore((s) => s.setSelectedAgent);
  const allMessages = useChatStore((s) => s.messages);
  const addMessage = useChatStore((s) => s.addMessage);

  const [inputValue, setInputValue] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const selectedAgent = agents.find((a) => a.id === selectedAgentId);
  const agentMessages = allMessages.filter((m) => m.agentId === selectedAgentId);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [agentMessages.length]);

  function handleSend() {
    const text = inputValue.trim();
    if (!text || !selectedAgent) return;
    addMessage({
      id: generateMsgId(),
      agentId: selectedAgentId,
      agentName: selectedAgent.name,
      role: "user",
      content: text,
      timestamp: new Date().toISOString(),
    });
    setInputValue("");
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }

  return (
    <div className="flex flex-col flex-1 min-h-0">
      {/* Agent selector */}
      <div className="shrink-0 flex items-center gap-2 px-3 py-2 border-b border-border">
        <select
          value={selectedAgentId}
          onChange={(e) => setSelectedAgent(e.target.value)}
          className="flex-1 bg-card border border-border rounded px-2 py-1 text-sm text-text-primary
            focus:outline-none focus:ring-2 focus:ring-accent/50 font-mono cursor-pointer"
          aria-label={t("chat.selectAgent")}
        >
          {agents.map((agent) => (
            <option key={agent.id} value={agent.id}>
              {agent.name} ({agent.role})
            </option>
          ))}
        </select>
        {selectedAgent && (
          <span
            className="w-2 h-2 rounded-full shrink-0"
            style={{ backgroundColor: statusColor[selectedAgent.status] ?? "#6B7280" }}
            title={selectedAgent.status}
            aria-hidden="true"
          />
        )}
      </div>

      {/* Message list */}
      <div className="flex-1 overflow-y-auto px-3 py-2 space-y-2 min-h-0">
        {agentMessages.length === 0 && (
          <div className="flex items-center justify-center h-full">
            <p className="text-text-disabled text-sm text-center px-4">
              {t("chat.noMessages", { agent: selectedAgent?.name ?? "agent" })}
            </p>
          </div>
        )}
        {agentMessages.map((msg) => {
          const isUser = msg.role === "user";
          const borderColor = isUser
            ? "#E5E5E5"
            : agentRoleColor[selectedAgent?.role ?? "orchestrator"] ?? "#F97316";

          return (
            <div key={msg.id} className={`flex ${isUser ? "justify-end" : "justify-start"}`}>
              <div
                className={`max-w-[85%] rounded-lg px-2.5 py-1.5 ${
                  isUser ? "bg-muted border border-border" : "bg-card border-l-[3px]"
                }`}
                style={!isUser ? { borderLeftColor: borderColor } : undefined}
              >
                {!isUser && (
                  <div className="flex items-center gap-1 mb-0.5">
                    <span className="text-[12px] font-medium" style={{ color: borderColor }}>
                      {msg.agentName}
                    </span>
                    <span className="text-text-disabled text-[13px]">{formatTimestamp(msg.timestamp)}</span>
                  </div>
                )}
                <p className="text-text-primary text-[13px] leading-relaxed whitespace-pre-wrap">
                  {msg.content}
                </p>
                {isUser && (
                  <div className="flex justify-end mt-0.5">
                    <span className="text-text-disabled text-[13px]">{formatTimestamp(msg.timestamp)}</span>
                  </div>
                )}
              </div>
            </div>
          );
        })}
        <div ref={messagesEndRef} />
      </div>

      {/* Input area */}
      <div className="shrink-0 border-t border-border px-3 py-2">
        <div className="flex items-end gap-2">
          <textarea
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t("commandCenter.commandPlaceholder")}
            rows={1}
            className="flex-1 bg-card border border-border rounded-lg px-2.5 py-2 text-[13px] text-text-primary
              placeholder:text-text-disabled resize-none font-mono
              focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-transparent
              max-h-[80px] overflow-y-auto"
            style={{ minHeight: "32px" }}
          />
          <button
            type="button"
            onClick={handleSend}
            disabled={!inputValue.trim()}
            className="shrink-0 flex items-center justify-center w-8 h-8 rounded-lg
              bg-accent hover:bg-accent-hover disabled:bg-muted disabled:cursor-not-allowed
              transition-colors duration-100 focus:outline-none focus-visible:ring-2
              focus-visible:ring-accent/50"
            aria-label={t("commandCenter.send")}
          >
            <svg
              width="13"
              height="13"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              className="text-background"
              aria-hidden="true"
            >
              <line x1="22" y1="2" x2="11" y2="13" />
              <polygon points="22 2 15 22 11 13 2 9 22 2" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
}

// ── Agents Panel ───────────────────────────────────────────────────────────

const agentSkills: Record<string, string[]> = {
  orchestrator: ["coordinate", "plan", "delegate"],
  researcher: ["web-search", "browse", "summarize"],
  analyst: ["data-analysis", "chart", "memory"],
  writer: ["file-write", "format", "edit"],
  coder: ["code-exec", "file-write", "debug"],
  operator: ["browser", "system", "monitor"],
};

function AgentsPanel({ onOpenChat }: { onOpenChat: () => void }) {
  const { t } = useTranslation();
  const agents = useAgentStore((s) => s.agents);
  const agentCosts = useCostStore((s) => s.agentCosts);

  const workingCount = agents.filter((a) => a.status !== "idle").length;

  return (
    <div className="flex flex-col flex-1 min-h-0">
      {/* Working count banner */}
      {workingCount > 0 && (
        <div className="shrink-0 px-3 py-1.5 border-b border-border bg-accent/5">
          <span className="text-[12px] text-accent font-medium">
            {t("commandCenter.working", { count: workingCount })}
          </span>
        </div>
      )}

      {/* Agent list */}
      <div className="flex-1 overflow-y-auto px-2 py-2 space-y-1.5 min-h-0">
        {agents.map((agent) => {
          const roleColor = agentRoleColor[agent.role] ?? "#6B7280";
          const dotColor = statusColor[agent.status] ?? "#6B7280";
          const isPulsing = agent.status === "working" || agent.status === "executing";
          const cost = agentCosts[agent.name] ?? 0;
          const skills = agentSkills[agent.role] ?? [];

          return (
            <div
              key={agent.id}
              className="bg-card/60 border border-border rounded-lg px-2.5 py-2 space-y-1.5"
            >
              {/* Agent header row */}
              <div className="flex items-center gap-2">
                <span
                  className={`w-2 h-2 rounded-full shrink-0 ${isPulsing ? "animate-pulse" : ""}`}
                  style={{ backgroundColor: dotColor }}
                  aria-hidden="true"
                />
                <span className="text-[13px] font-medium text-text-primary truncate flex-1">
                  {agent.name}
                </span>
                <span
                  className="text-[12px] font-medium shrink-0"
                  style={{ color: roleColor }}
                >
                  {agent.role}
                </span>
              </div>

              {/* Status / task */}
              <div className="text-[12px] text-text-muted truncate pl-4">
                {agent.status !== "idle"
                  ? t(`agents.status.${agent.status}`, agent.status)
                  : "Idle"}
              </div>

              {/* Cost */}
              {cost > 0 && (
                <div className="flex items-center gap-2 pl-4 text-[12px] text-text-muted">
                  <span>{cost} DD</span>
                </div>
              )}

              {/* Skills */}
              <div className="flex flex-wrap gap-1 pl-4">
                {skills.map((skill) => (
                  <span
                    key={skill}
                    className="text-[13px] px-1.5 py-0.5 rounded-full bg-muted text-text-secondary border border-border"
                  >
                    {skill}
                  </span>
                ))}
              </div>
            </div>
          );
        })}
      </div>

      {/* Skills legend + Open Chat */}
      <div className="shrink-0 border-t border-border px-3 py-2">
        <button
          onClick={onOpenChat}
          className="w-full py-1.5 rounded-lg bg-accent hover:bg-accent-hover text-background
            text-[13px] font-medium transition-colors duration-100
            focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50"
        >
          {t("commandCenter.openChat")}
        </button>
      </div>
    </div>
  );
}

// ── Command Center (RightPanel) ────────────────────────────────────────────

export function RightPanel() {
  const { t } = useTranslation();
  const [mode, setMode] = useState<CommandCenterMode>("chat");
  const collapsed = useLayoutStore((s) => s.rightPanelCollapsed);
  const rightPanelWidth = useLayoutStore((s) => s.rightPanelWidth);
  const setRightPanelWidth = useLayoutStore((s) => s.setRightPanelWidth);
  const toggleRightPanel = useLayoutStore((s) => s.toggleRightPanel);

  const isExpanded = rightPanelWidth === 420;

  return (
    <aside
      className={`
        border-l border-border bg-card/40 shrink-0
        transition-[width] duration-150 ease-out overflow-hidden flex flex-col
        ${collapsed ? "w-0 border-l-0" : ""}
      `}
      style={collapsed ? undefined : { width: rightPanelWidth }}
      role="complementary"
      aria-label={t("commandCenter.title")}
    >
      {!collapsed && (
        <>
          {/* Header */}
          <div className="shrink-0 flex items-center gap-1 h-10 px-3 border-b border-border">
            {/* Title */}
            <span className="text-[12px] text-text-muted uppercase tracking-wider font-semibold mr-1">
              {t("commandCenter.title")}
            </span>

            {/* Mode toggle */}
            <div className="flex items-center gap-0.5 bg-muted rounded-md p-0.5">
              <button
                onClick={() => setMode("chat")}
                className={`px-2 py-0.5 rounded text-[12px] font-medium transition-colors duration-100
                  focus:outline-none focus-visible:ring-1 focus-visible:ring-accent/50
                  ${mode === "chat" ? "bg-card text-text-primary shadow-sm" : "text-text-muted hover:text-text-secondary"}`}
              >
                {t("commandCenter.chat")}
              </button>
              <button
                onClick={() => setMode("agents")}
                className={`px-2 py-0.5 rounded text-[12px] font-medium transition-colors duration-100
                  focus:outline-none focus-visible:ring-1 focus-visible:ring-accent/50
                  ${mode === "agents" ? "bg-card text-text-primary shadow-sm" : "text-text-muted hover:text-text-secondary"}`}
              >
                {t("commandCenter.agents")}
              </button>
            </div>

            {/* Expand/collapse width toggle */}
            <button
              onClick={() => setRightPanelWidth(isExpanded ? 280 : 420)}
              className="ml-auto p-1 rounded hover:bg-muted text-text-muted hover:text-text-secondary
                transition-colors duration-100 focus:outline-none focus-visible:ring-2
                focus-visible:ring-accent/50"
              aria-label={isExpanded ? "Narrow panel" : "Expand panel"}
              title={isExpanded ? "Narrow panel" : "Expand panel"}
            >
              <svg
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                aria-hidden="true"
              >
                {isExpanded ? (
                  <>
                    <polyline points="15 18 9 12 15 6" />
                  </>
                ) : (
                  <>
                    <polyline points="9 18 15 12 9 6" />
                  </>
                )}
              </svg>
            </button>

            {/* Close panel */}
            <button
              onClick={toggleRightPanel}
              className="p-1 rounded hover:bg-muted text-text-muted hover:text-text-secondary
                transition-colors duration-100 focus:outline-none focus-visible:ring-2
                focus-visible:ring-accent/50"
              aria-label="Close Command Center"
            >
              <svg
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                aria-hidden="true"
              >
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>

          {/* Content */}
          {mode === "chat" ? (
            <ChatPanel />
          ) : (
            <AgentsPanel onOpenChat={() => setMode("chat")} />
          )}
        </>
      )}
    </aside>
  );
}
