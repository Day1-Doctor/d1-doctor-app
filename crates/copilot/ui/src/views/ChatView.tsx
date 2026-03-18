import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useAgentStore } from "../stores/agentStore";
import { useChatStore, generateMsgId } from "../stores/chatStore";

/** Agent role to left-border color mapping. */
const agentBorderColor: Record<string, string> = {
  orchestrator: "#F97316",
  researcher: "#22C55E",
  analyst: "#3B82F6",
  writer: "#A855F7",
  coder: "#EC4899",
  operator: "#F59E0B",
};

function formatTimestamp(ts: string): string {
  const date = new Date(ts);
  const hours = date.getHours().toString().padStart(2, "0");
  const minutes = date.getMinutes().toString().padStart(2, "0");
  return `${hours}:${minutes}`;
}

export function ChatView() {
  const { t } = useTranslation();
  const agents = useAgentStore((s) => s.agents);
  const selectedAgentId = useChatStore((s) => s.selectedAgentId);
  const setSelectedAgent = useChatStore((s) => s.setSelectedAgent);
  const allMessages = useChatStore((s) => s.messages);
  const addMessage = useChatStore((s) => s.addMessage);

  const [inputValue, setInputValue] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const selectedAgent = agents.find((a) => a.id === selectedAgentId);
  const agentMessages = allMessages.filter(
    (m) => m.agentId === selectedAgentId,
  );

  // Auto-scroll to bottom when messages change
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
    <div className="flex-1 flex flex-col min-h-0">
      {/* Agent selector */}
      <div className="shrink-0 flex items-center gap-2 px-4 py-2 border-b border-border">
        <label
          htmlFor="agent-select"
          className="text-[10px] text-text-secondary uppercase tracking-wider font-medium"
        >
          {t("debug.agent")}
        </label>
        <select
          id="agent-select"
          value={selectedAgentId}
          onChange={(e) => setSelectedAgent(e.target.value)}
          className="bg-card border border-border rounded px-2 py-1 text-xs text-text-primary
            focus:outline-none focus:ring-2 focus:ring-accent/50 font-mono cursor-pointer"
        >
          {agents.map((agent) => (
            <option key={agent.id} value={agent.id}>
              {agent.name} ({agent.role})
            </option>
          ))}
        </select>
        {selectedAgent && (
          <span className="text-[10px] text-text-muted ml-auto">
            {t(`agents.status.${selectedAgent.status}`, selectedAgent.status)}
          </span>
        )}
      </div>

      {/* Message list */}
      <div className="flex-1 overflow-y-auto px-4 py-3 space-y-3 min-h-0">
        {agentMessages.length === 0 && (
          <div className="flex items-center justify-center h-full">
            <p className="text-text-disabled text-xs">
              {t("chat.noMessages", { agent: selectedAgent?.name ?? "agent" })}
            </p>
          </div>
        )}

        {agentMessages.map((msg) => {
          const isUser = msg.role === "user";
          const borderColor = isUser
            ? "#E5E5E5"
            : agentBorderColor[selectedAgent?.role ?? "orchestrator"] ?? "#F97316";

          return (
            <div
              key={msg.id}
              className={`flex ${isUser ? "justify-end" : "justify-start"}`}
            >
              <div
                className={`max-w-[75%] rounded-lg px-3 py-2 ${
                  isUser
                    ? "bg-muted border border-border"
                    : "bg-card border-l-[3px]"
                }`}
                style={!isUser ? { borderLeftColor: borderColor } : undefined}
              >
                {/* Agent label */}
                {!isUser && (
                  <div className="flex items-center gap-1.5 mb-1">
                    <span
                      className="text-[10px] font-medium"
                      style={{ color: borderColor }}
                    >
                      {msg.agentName}
                    </span>
                    <span className="text-text-disabled text-[9px]">
                      {formatTimestamp(msg.timestamp)}
                    </span>
                  </div>
                )}

                <p className="text-text-primary text-xs leading-relaxed whitespace-pre-wrap">
                  {msg.content}
                </p>

                {isUser && (
                  <div className="flex justify-end mt-1">
                    <span className="text-text-disabled text-[9px]">
                      {formatTimestamp(msg.timestamp)}
                    </span>
                  </div>
                )}
              </div>
            </div>
          );
        })}
        <div ref={messagesEndRef} />
      </div>

      {/* Input area */}
      <div className="shrink-0 border-t border-border px-4 py-3">
        <div className="flex items-end gap-2">
          <textarea
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t("chat.inputPlaceholder", { agent: selectedAgent?.name ?? "agent" })}
            rows={1}
            className="flex-1 bg-card border border-border rounded-lg px-3 py-2 text-xs text-text-primary
              placeholder:text-text-disabled resize-none font-mono
              focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-transparent
              max-h-[100px] overflow-y-auto"
            style={{ minHeight: "36px" }}
          />
          <button
            type="button"
            onClick={handleSend}
            disabled={!inputValue.trim()}
            className="shrink-0 flex items-center justify-center w-9 h-9 rounded-lg
              bg-accent hover:bg-accent-hover disabled:bg-muted disabled:cursor-not-allowed
              transition-colors duration-100 focus:outline-none focus-visible:ring-2
              focus-visible:ring-accent/50"
            aria-label={t("chat.send")}
          >
            <svg
              width="16"
              height="16"
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
        <p className="text-text-disabled text-[9px] mt-1.5">
          {t("chat.inputHint")}
        </p>
      </div>
    </div>
  );
}
