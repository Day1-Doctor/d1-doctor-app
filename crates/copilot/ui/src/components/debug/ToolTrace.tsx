import { useTranslation } from "react-i18next";
import { useEventLogStore, type ToolTraceEntry } from "../../stores/eventLogStore";

function StatusBadge({ status }: { status: "success" | "fail" }) {
  const isSuccess = status === "success";
  return (
    <span
      className={`inline-flex items-center px-1.5 py-0.5 rounded text-[13px] font-medium uppercase tracking-wider ${
        isSuccess
          ? "bg-success/10 text-success border border-success/30"
          : "bg-error/10 text-error border border-error/30"
      }`}
    >
      {status}
    </span>
  );
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function truncateParams(params: string, maxLen = 50): string {
  return params.length > maxLen ? params.slice(0, maxLen) + "..." : params;
}

function TraceRow({ trace }: { trace: ToolTraceEntry }) {
  return (
    <tr className="border-b border-border/50 hover:bg-muted/20 text-[13px]">
      <td className="px-3 py-1.5 text-text-primary font-medium">{trace.toolName}</td>
      <td className="px-3 py-1.5 text-text-secondary">{trace.agentName}</td>
      <td className="px-3 py-1.5 text-text-disabled truncate max-w-[180px]" title={trace.params}>
        {truncateParams(trace.params)}
      </td>
      <td className="px-3 py-1.5 text-text-secondary tabular-nums text-right">
        {formatDuration(trace.durationMs)}
      </td>
      <td className="px-3 py-1.5">
        <StatusBadge status={trace.status} />
      </td>
    </tr>
  );
}

export function ToolTrace() {
  const { t } = useTranslation();
  const toolTraces = useEventLogStore((s) => s.toolTraces);
  const agentTokenUsage = useEventLogStore((s) => s.agentTokenUsage);

  return (
    <div className="flex flex-col h-full min-h-0 gap-4">
      {/* Tool trace table */}
      <div className="flex-1 overflow-y-auto min-h-0">
        <table className="w-full text-left">
          <thead className="sticky top-0 bg-card border-b border-border">
            <tr className="text-[12px] text-text-muted uppercase tracking-wider">
              <th className="px-3 py-2 font-medium">{t("debug.tool")}</th>
              <th className="px-3 py-2 font-medium">{t("debug.agent")}</th>
              <th className="px-3 py-2 font-medium">{t("debug.params")}</th>
              <th className="px-3 py-2 font-medium text-right">{t("debug.duration")}</th>
              <th className="px-3 py-2 font-medium">{t("debug.status")}</th>
            </tr>
          </thead>
          <tbody>
            {toolTraces.length === 0 ? (
              <tr>
                <td colSpan={5} className="text-center py-8 text-text-disabled text-sm">
                  {t("debug.noToolTraces")}
                </td>
              </tr>
            ) : (
              toolTraces.map((trace) => <TraceRow key={trace.id} trace={trace} />)
            )}
          </tbody>
        </table>
      </div>

      {/* Per-agent token usage summary */}
      <div className="shrink-0 border-t border-border pt-3 px-3 pb-3">
        <h3 className="text-[12px] text-text-muted uppercase tracking-wider mb-2 font-medium">
          {t("debug.perAgentTokenUsage")}
        </h3>
        <div className="grid grid-cols-3 gap-2">
          {agentTokenUsage.map((usage) => (
            <div
              key={usage.agentName}
              className="rounded-lg border border-border p-2"
              style={{ backgroundColor: "rgba(15, 15, 15, 0.6)" }}
            >
              <p className="text-[13px] text-text-primary font-medium mb-1">{usage.agentName}</p>
              <div className="flex items-baseline gap-2">
                <span className="text-[12px] text-text-muted">
                  {usage.totalTokens.toLocaleString()} tok
                </span>
                <span className="text-[12px] text-accent tabular-nums ml-auto">
                  {usage.costDD} DD
                </span>
              </div>
              <div className="flex gap-2 mt-0.5">
                <span className="text-[13px] text-text-disabled">
                  in: {usage.inputTokens.toLocaleString()}
                </span>
                <span className="text-[13px] text-text-disabled">
                  out: {usage.outputTokens.toLocaleString()}
                </span>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
