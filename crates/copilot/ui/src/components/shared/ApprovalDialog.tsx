import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  useApprovalStore,
  type RiskLevel,
  type ApprovalDecision,
} from "../../stores/approvalStore";

// ---------------------------------------------------------------------------
// Risk level badge colors
// ---------------------------------------------------------------------------

const riskBadgeClasses: Record<RiskLevel, string> = {
  medium: "bg-yellow-500/20 text-yellow-400 border-yellow-500/30",
  high: "bg-red-500/20 text-red-400 border-red-500/30",
  critical: "bg-purple-500/20 text-purple-400 border-purple-500/30",
};

const riskBadgeLabel: Record<RiskLevel, string> = {
  medium: "MEDIUM",
  high: "HIGH",
  critical: "CRITICAL",
};

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/**
 * ApprovalDialog -- glassmorphic modal shown when an agent requests permission
 * for a risky tool invocation. Renders the first pending approval from the queue.
 *
 * PRD section 10.4.
 */
export function ApprovalDialog() {
  const { t } = useTranslation();
  const pendingApprovals = useApprovalStore((s) => s.pendingApprovals);
  const respond = useApprovalStore((s) => s.respond);
  const [trustChecked, setTrustChecked] = useState(false);

  // Show the first pending approval (FIFO).
  const request = pendingApprovals[0];
  if (!request) return null;

  const handleDecision = (decision: ApprovalDecision) => {
    // If trust checkbox is checked, upgrade "allow_once" to "allow_always".
    const finalDecision =
      trustChecked && decision === "allow_once" ? "allow_always" : decision;
    respond(request.id, finalDecision);
    setTrustChecked(false);
  };

  const badgeClass = riskBadgeClasses[request.riskLevel];

  // Format params for display (first meaningful key-value or JSON excerpt).
  const commandDisplay =
    request.params && Object.keys(request.params).length > 0
      ? JSON.stringify(request.params, null, 2).slice(0, 200)
      : "--";

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      role="dialog"
      aria-modal="true"
      aria-label={t("approval.title")}
    >
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/60" aria-hidden="true" />

      {/* Modal */}
      <div
        className="relative z-10 w-full max-w-md rounded-xl border border-border
          bg-card/80 p-6 shadow-2xl shadow-black/40"
        style={{
          backdropFilter: "blur(24px)",
          WebkitBackdropFilter: "blur(24px)",
        }}
      >
        {/* Header: agent name + role */}
        <div className="mb-4 flex items-center gap-2">
          <span className="inline-block h-2 w-2 rounded-full bg-accent" />
          <h2 className="text-sm font-semibold text-text-primary">
            {t("approval.agentRequests", { agent: request.agentName })}
          </h2>
          <span className="ml-auto rounded-full bg-muted px-2 py-0.5 text-[10px] font-medium text-text-secondary">
            {request.agentRole}
          </span>
        </div>

        {/* Details table */}
        <div className="mb-4 space-y-2 text-xs">
          {/* Action */}
          <div className="flex justify-between">
            <span className="text-text-secondary">{t("approval.action")}</span>
            <span className="font-medium text-text-primary">
              {request.toolName}
            </span>
          </div>

          {/* Command / Params */}
          <div className="flex justify-between">
            <span className="text-text-secondary">
              {t("approval.command")}
            </span>
            <span className="max-w-[260px] truncate font-mono text-text-primary">
              {commandDisplay}
            </span>
          </div>

          {/* Risk Level */}
          <div className="flex justify-between items-center">
            <span className="text-text-secondary">
              {t("approval.riskLevel")}
            </span>
            <span
              className={`rounded-full border px-2 py-0.5 text-[10px] font-bold uppercase ${badgeClass}`}
            >
              {riskBadgeLabel[request.riskLevel]}
            </span>
          </div>

          {/* Context */}
          <div className="flex justify-between">
            <span className="text-text-secondary">
              {t("approval.context")}
            </span>
            <span className="text-text-primary">{request.context}</span>
          </div>
        </div>

        {/* Queue indicator */}
        {pendingApprovals.length > 1 && (
          <p className="mb-3 text-center text-[10px] text-text-muted">
            +{pendingApprovals.length - 1} more pending
          </p>
        )}

        {/* Action buttons */}
        <div className="flex gap-2">
          <button
            onClick={() => handleDecision("allow_once")}
            className="flex-1 rounded-lg bg-accent px-3 py-2 text-xs font-medium
              text-white transition-colors hover:bg-accent/90
              focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50"
          >
            {t("approval.allowOnce")}
          </button>

          <button
            onClick={() => handleDecision("allow_always")}
            className="flex-1 rounded-lg border border-accent/40 bg-transparent
              px-3 py-2 text-xs font-medium text-accent transition-colors
              hover:bg-accent/10
              focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50"
          >
            {t("approval.allowAlways")}
          </button>

          <button
            onClick={() => handleDecision("reject")}
            className="flex-1 rounded-lg border border-border bg-transparent
              px-3 py-2 text-xs font-medium text-text-secondary
              transition-colors hover:bg-muted hover:text-text-primary
              focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50"
          >
            {t("approval.reject")}
          </button>
        </div>

        {/* Trust checkbox */}
        <label className="mt-3 flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={trustChecked}
            onChange={(e) => setTrustChecked(e.target.checked)}
            className="h-3.5 w-3.5 rounded border-border bg-muted text-accent
              focus:ring-accent/50"
          />
          <span className="text-[11px] text-text-secondary">
            {t("approval.trustCheckbox", {
              level: riskBadgeLabel[request.riskLevel],
            })}
          </span>
        </label>
      </div>
    </div>
  );
}
