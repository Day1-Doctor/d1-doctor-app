import { useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
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
  const [viewIndex, setViewIndex] = useState(0);

  // Clamp viewIndex to valid range
  const clampedIndex = Math.min(viewIndex, Math.max(0, pendingApprovals.length - 1));
  const request = pendingApprovals[clampedIndex];
  if (!request) return null;

  const canGoPrev = clampedIndex > 0;
  const canGoNext = clampedIndex < pendingApprovals.length - 1;

  const handleDecision = async (decision: ApprovalDecision) => {
    // If trust checkbox is checked, upgrade "allow_once" to "allow_always".
    const finalDecision =
      trustChecked && decision === "allow_once" ? "allow_always" : decision;

    // Update local approval store immediately for responsive UI
    respond(request.id, finalDecision);
    setTrustChecked(false);
    // After resolving, move view back to stay in bounds
    setViewIndex((i) => Math.max(0, Math.min(i, pendingApprovals.length - 2)));

    // Send decision to the backend via Tauri command
    // TODO: Add `respond_approval` Tauri command to lib.rs when backend
    // PermissionEngine supports async approval responses. For now we
    // optimistically update the local store and attempt the invoke.
    try {
      await invoke("respond_approval", {
        requestId: request.id,
        decision: finalDecision === "allow_always" ? "approve" : finalDecision === "allow_once" ? "approve" : "deny",
      });
    } catch {
      // `respond_approval` Tauri command not yet registered -- approval
      // was already handled in the local store above, so this is non-fatal.
    }
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
          <span className="ml-auto rounded-full bg-muted px-2 py-0.5 text-[12px] font-medium text-text-secondary">
            {request.agentRole}
          </span>
        </div>

        {/* Details table */}
        <div className="mb-4 space-y-2 text-sm">
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
              className={`rounded-full border px-2 py-0.5 text-[12px] font-bold uppercase ${badgeClass}`}
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

        {/* Queue navigation */}
        {pendingApprovals.length > 1 && (
          <div className="mb-3 flex items-center justify-center gap-3">
            <button
              onClick={() => setViewIndex((i) => Math.max(0, i - 1))}
              disabled={!canGoPrev}
              className="text-[12px] text-text-muted hover:text-text-primary disabled:opacity-30
                disabled:cursor-not-allowed transition-colors focus:outline-none"
              aria-label="Previous approval"
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                <polyline points="15 18 9 12 15 6" />
              </svg>
            </button>
            <span className="text-[12px] text-text-muted tabular-nums">
              {clampedIndex + 1} / {pendingApprovals.length}
            </span>
            <button
              onClick={() => setViewIndex((i) => Math.min(pendingApprovals.length - 1, i + 1))}
              disabled={!canGoNext}
              className="text-[12px] text-text-muted hover:text-text-primary disabled:opacity-30
                disabled:cursor-not-allowed transition-colors focus:outline-none"
              aria-label="Next approval"
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                <polyline points="9 18 15 12 9 6" />
              </svg>
            </button>
          </div>
        )}

        {/* Action buttons */}
        <div className="flex gap-2">
          <button
            onClick={() => void handleDecision("allow_once")}
            className="flex-1 rounded-lg bg-accent px-3 py-2 text-sm font-medium
              text-white transition-colors hover:bg-accent/90
              focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50"
          >
            {t("approval.allowOnce")}
          </button>

          <button
            onClick={() => void handleDecision("allow_always")}
            className="flex-1 rounded-lg border border-accent/40 bg-transparent
              px-3 py-2 text-sm font-medium text-accent transition-colors
              hover:bg-accent/10
              focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50"
          >
            {t("approval.allowAlways")}
          </button>

          <button
            onClick={() => void handleDecision("reject")}
            className="flex-1 rounded-lg border border-border bg-transparent
              px-3 py-2 text-sm font-medium text-text-secondary
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
          <span className="text-[13px] text-text-secondary">
            {t("approval.trustCheckbox", {
              level: riskBadgeLabel[request.riskLevel],
            })}
          </span>
        </label>
      </div>
    </div>
  );
}
