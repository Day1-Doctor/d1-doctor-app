import { useTranslation } from "react-i18next";
import { useAgentStore } from "../../stores/agentStore";
import { useConnectionStore } from "../../stores/connectionStore";
import { ZoomControl } from "./ZoomControl";

export function StatusBar() {
  const { t } = useTranslation();
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
              {t("statusBar.agent")} {activeAgent.name}
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
              {t(`agents.status.${activeAgent.status}`)}
            </span>
          </>
        ) : (
          <span>{t("statusBar.noAgent")}</span>
        )}
      </div>

      {/* Center: Token count */}
      <div className="flex-1 text-center">
        <span>{t("statusBar.tokens")} 0</span>
      </div>

      {/* Right: Zoom + Latency + Connection */}
      <div className="flex items-center gap-3 flex-1 justify-end">
        <ZoomControl />
        <span className="text-border">|</span>
        <span>{t("statusBar.latency")} --</span>
        <div className="flex items-center gap-1.5">
          <span
            className={`inline-block w-1.5 h-1.5 rounded-full ${
              isConnected ? "bg-success" : "bg-error"
            }`}
            aria-label={isConnected ? t("common.connected") : t("common.disconnected")}
          />
          <span className="text-text-muted">
            {isConnected ? t("common.connected") : t("common.disconnected")}
          </span>
        </div>
      </div>
    </footer>
  );
}
