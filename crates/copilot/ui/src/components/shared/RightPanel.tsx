import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { useLayoutStore } from "../../stores/layoutStore";
import { useAgentStore } from "../../stores/agentStore";
import { useAuthStore } from "../../stores/authStore";
import { useChatStore, generateMsgId } from "../../stores/chatStore";
import { useCostStore } from "../../stores/costStore";
import { useTaskStore } from "../../stores/taskStore";
import { useBillingStore } from "../../stores/billingStore";

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

// ── Chat Panel (Two modes: Plan Mode + BTW Mode) ──────────────────────────

type ChatMode = "plan" | "btw";

function ChatPanel() {
  const { t } = useTranslation();
  const allMessages = useChatStore((s) => s.messages);
  const addMessage = useChatStore((s) => s.addMessage);
  const addTask = useTaskStore((s) => s.addTask);
  const maxAgents = useBillingStore((s) => s.maxAgents);
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);

  const [inputValue, setInputValue] = useState("");
  const [chatMode, setChatMode] = useState<ChatMode>("plan");
  const [planConfirmed, setPlanConfirmed] = useState(false);
  const [isCreatingTask, setIsCreatingTask] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Always talk to Dr. Bob (team leader / orchestrator)
  const drBobId = "orchestrator";
  const drBobMessages = allMessages.filter((m) => m.agentId === drBobId || m.agentId === "team");

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [drBobMessages.length]);

  function handleSend() {
    const text = inputValue.trim();
    if (!text) return;
    addMessage({
      id: generateMsgId(),
      agentId: chatMode === "plan" ? drBobId : "team",
      agentName: chatMode === "plan" ? "Dr. Bob" : "Team",
      role: "user",
      content: text,
      timestamp: new Date().toISOString(),
    });
    setInputValue("");

    // Simulate Dr. Bob response (in real integration, this calls the gateway)
    if (chatMode === "plan") {
      setTimeout(() => {
        addMessage({
          id: generateMsgId(),
          agentId: drBobId,
          agentName: "Dr. Bob",
          role: "agent",
          content: t("chat.drBobAck", {
            defaultValue: "Got it. Let me think about how to break this down for the team...",
          }),
          timestamp: new Date().toISOString(),
        });
      }, 800);
    }
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }

  async function handleConfirmPlan() {
    // Gather user messages from Plan Mode as the task description
    const userMessages = drBobMessages
      .filter((m) => m.role === "user")
      .map((m) => m.content);
    const description = userMessages.join("\n");

    if (!description.trim()) return;

    setIsCreatingTask(true);
    setPlanConfirmed(true);

    // Optimistically add a task to the UI store
    addTask(description.length > 80 ? description.slice(0, 80) + "..." : description);

    try {
      // Call the Tauri create_task command to kick off real backend execution.
      // When not authenticated the backend will automatically fall back to the
      // free model endpoint (no auth required).
      const result = await invoke<{ id: string; title: string; status: string }>(
        "create_task",
        { description, maxAgents },
      );

      addMessage({
        id: generateMsgId(),
        agentId: drBobId,
        agentName: "Dr. Bob",
        role: "agent",
        content: t("chat.planConfirmed", {
          defaultValue: `Plan confirmed! Task "${result.title}" is now ${result.status}. I'll coordinate the agents to complete all tasks.`,
        }),
        timestamp: new Date().toISOString(),
      });
    } catch (e) {
      // Fallback: show confirmation even if Tauri is not available (e.g. dev/browser mode)
      addMessage({
        id: generateMsgId(),
        agentId: drBobId,
        agentName: "Dr. Bob",
        role: "agent",
        content: t("chat.planConfirmed", {
          defaultValue: "Plan confirmed! Handing off to the team now. I'll coordinate the agents to complete all tasks.",
        }),
        timestamp: new Date().toISOString(),
      });
    } finally {
      setIsCreatingTask(false);
    }
  }

  const isPlanMode = chatMode === "plan";
  const modeColor = isPlanMode ? "#F97316" : "#3B82F6";
  const modeLabel = isPlanMode
    ? t("chat.planMode", { defaultValue: "Plan Mode" })
    : t("chat.btwMode", { defaultValue: "BTW Mode" });
  const modeDesc = isPlanMode
    ? t("chat.planModeDesc", { defaultValue: "Dr. Bob plans & orchestrates" })
    : t("chat.btwModeDesc", { defaultValue: "Side chat, won't interrupt work" });
  const placeholder = isPlanMode
    ? t("chat.planPlaceholder", { defaultValue: "Describe what you need..." })
    : t("chat.btwPlaceholder", { defaultValue: "Quick note (won't interrupt agents)..." });

  return (
    <div className="flex flex-col flex-1 min-h-0">
      {/* Team header + mode toggle */}
      <div className="shrink-0 px-3 py-2 border-b border-border space-y-2">
        {/* Team indicator */}
        <div className="flex items-center gap-2">
          <span
            className="w-2.5 h-2.5 rounded-full shrink-0"
            style={{ backgroundColor: "#F97316" }}
          />
          <span className="text-sm font-semibold text-text-primary">Dr. Bob</span>
          <span className="text-[12px] text-text-muted">{t("chat.teamLeader", { defaultValue: "Team Leader" })}</span>
        </div>

        {/* Mode toggle */}
        <div className="flex items-center gap-1 bg-muted rounded-md p-0.5">
          <button
            onClick={() => setChatMode("plan")}
            className={`flex-1 px-2 py-1 rounded text-[12px] font-medium transition-colors duration-100
              ${isPlanMode ? "bg-card text-text-primary shadow-sm" : "text-text-muted hover:text-text-secondary"}`}
          >
            <span className="inline-block w-1.5 h-1.5 rounded-full mr-1" style={{ backgroundColor: "#F97316" }} />
            {t("chat.planMode", { defaultValue: "Plan" })}
          </button>
          <button
            onClick={() => setChatMode("btw")}
            className={`flex-1 px-2 py-1 rounded text-[12px] font-medium transition-colors duration-100
              ${!isPlanMode ? "bg-card text-text-primary shadow-sm" : "text-text-muted hover:text-text-secondary"}`}
          >
            <span className="inline-block w-1.5 h-1.5 rounded-full mr-1" style={{ backgroundColor: "#3B82F6" }} />
            {t("chat.btwMode", { defaultValue: "BTW" })}
          </button>
        </div>

        {/* Mode description */}
        <p className="text-[12px] text-text-muted leading-tight">{modeDesc}</p>
      </div>

      {/* Message list */}
      <div className="flex-1 overflow-y-auto px-3 py-2 space-y-2 min-h-0">
        {drBobMessages.length === 0 && (
          <div className="flex items-center justify-center h-full">
            <div className="text-center px-4">
              <p className="text-text-muted text-sm mb-1">
                {isPlanMode
                  ? t("chat.planEmpty", { defaultValue: "Tell Dr. Bob what you need. He'll create a plan and coordinate the team." })
                  : t("chat.btwEmpty", { defaultValue: "Send a quick note. This won't interrupt ongoing work." })}
              </p>
            </div>
          </div>
        )}
        {drBobMessages.map((msg) => {
          const isUser = msg.role === "user";
          const borderColor = isUser ? "#E5E5E5" : modeColor;

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

      {/* Plan confirm button (only in Plan Mode when there are messages) */}
      {isPlanMode && drBobMessages.length > 0 && !planConfirmed && (
        <div className="shrink-0 px-3 py-2 border-t border-border">
          {!isAuthenticated && (
            <p className="text-[12px] text-text-muted text-center mb-1.5">
              {t("chat.freeModelHint", { defaultValue: "Running in free mode. Sign in for full access." })}
            </p>
          )}
          <button
            onClick={() => void handleConfirmPlan()}
            disabled={isCreatingTask}
            className="w-full py-2 rounded-lg bg-accent hover:bg-accent-hover text-background
              text-sm font-semibold transition-colors duration-100
              disabled:opacity-50 disabled:cursor-not-allowed
              focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50"
          >
            {isCreatingTask
              ? t("chat.creatingTask", { defaultValue: "Creating task..." })
              : t("chat.confirmPlan", { defaultValue: "Confirm Plan & Start Execution" })}
          </button>
        </div>
      )}

      {/* Input area */}
      <div className="shrink-0 border-t border-border px-3 py-2">
        {/* Mode indicator pill */}
        <div className="flex items-center gap-1 mb-1.5">
          <span
            className="inline-block w-1.5 h-1.5 rounded-full"
            style={{ backgroundColor: modeColor }}
          />
          <span className="text-[12px] text-text-muted">{modeLabel}</span>
        </div>
        <div className="flex items-end gap-2">
          <textarea
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={placeholder}
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

export function RightPanel(_props: { onAuthRequired?: () => void }) {
  const { t } = useTranslation();
  const [mode, setMode] = useState<CommandCenterMode>("chat");
  const collapsed = useLayoutStore((s) => s.rightPanelCollapsed);
  const rightPanelWidth = useLayoutStore((s) => s.rightPanelWidth);
  const setRightPanelWidth = useLayoutStore((s) => s.setRightPanelWidth);
  const toggleRightPanel = useLayoutStore((s) => s.toggleRightPanel);

  // Drag-resize state
  const isDragging = useRef(false);
  const dragStartX = useRef(0);
  const dragStartWidth = useRef(0);

  const handleDragStart = (e: React.MouseEvent) => {
    e.preventDefault();
    isDragging.current = true;
    dragStartX.current = e.clientX;
    dragStartWidth.current = rightPanelWidth;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";

    const handleDragMove = (moveEvent: MouseEvent) => {
      if (!isDragging.current) return;
      // Dragging LEFT edge: moving left = wider panel
      const delta = dragStartX.current - moveEvent.clientX;
      const newWidth = Math.min(600, Math.max(200, dragStartWidth.current + delta));
      setRightPanelWidth(newWidth);
    };

    const handleDragEnd = () => {
      isDragging.current = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
      document.removeEventListener("mousemove", handleDragMove);
      document.removeEventListener("mouseup", handleDragEnd);
    };

    document.addEventListener("mousemove", handleDragMove);
    document.addEventListener("mouseup", handleDragEnd);
  };

  const isExpanded = rightPanelWidth >= 400;

  return (
    <aside
      className={`
        border-l border-border bg-card/40 shrink-0
        overflow-hidden flex flex-col relative
        ${collapsed ? "w-0 border-l-0" : ""}
      `}
      style={collapsed ? undefined : { width: rightPanelWidth }}
      role="complementary"
      aria-label={t("commandCenter.title")}
    >
      {/* Drag handle — left edge */}
      {!collapsed && (
        <div
          onMouseDown={handleDragStart}
          className="absolute left-0 top-0 bottom-0 w-1 cursor-col-resize z-10
            hover:bg-accent/40 active:bg-accent/60 transition-colors"
          title="Drag to resize"
        />
      )}
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
