import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { useApprovalStore, type ApprovalRequest, type RiskLevel } from "../../stores/approvalStore";

// ---------------------------------------------------------------------------
// Risk level badge colors
// ---------------------------------------------------------------------------

const RISK_COLORS: Record<RiskLevel, { bg: string; border: string; text: string }> = {
  medium: { bg: "rgba(234,179,8,0.15)", border: "rgba(234,179,8,0.4)", text: "#EAB308" },
  high: { bg: "rgba(239,68,68,0.15)", border: "rgba(239,68,68,0.4)", text: "#EF4444" },
  critical: { bg: "rgba(168,85,247,0.15)", border: "rgba(168,85,247,0.4)", text: "#A855F7" },
};

// ---------------------------------------------------------------------------
// Single approval card
// ---------------------------------------------------------------------------

function ApprovalCard({ request }: { request: ApprovalRequest }) {
  const { t } = useTranslation();
  const respond = useApprovalStore((s) => s.respond);
  const riskStyle = RISK_COLORS[request.riskLevel];

  const handleApprove = async () => {
    respond(request.id, "allow_once");
    // TODO: `respond_approval` Tauri command not yet in lib.rs -- attempt call
    try {
      await invoke("respond_approval", { requestId: request.id, decision: "approve" });
    } catch {
      // Non-fatal: local store already updated
    }
  };
  const handleDeny = async () => {
    respond(request.id, "reject");
    try {
      await invoke("respond_approval", { requestId: request.id, decision: "deny" });
    } catch {
      // Non-fatal: local store already updated
    }
  };

  return (
    <div
      style={{
        background: "#0A0A0A",
        border: "1px solid #1A1A1A",
        borderRadius: 6,
        padding: "10px 12px",
        fontFamily: 'ui-monospace, "SF Mono", Menlo, monospace',
        display: "flex",
        flexDirection: "column",
        gap: 6,
      }}
    >
      {/* Header row: warning icon + agent name + risk badge */}
      <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
        <span style={{ color: "#F59E0B", fontSize: 14 }}>⚠</span>
        <span style={{ color: "#D4D4D4", fontSize: 11, fontWeight: "bold" }}>
          {request.agentName}
        </span>
        <span
          style={{
            fontSize: 9,
            fontWeight: "bold",
            letterSpacing: 0.5,
            color: riskStyle.text,
            background: riskStyle.bg,
            border: `1px solid ${riskStyle.border}`,
            borderRadius: 3,
            padding: "1px 5px",
            textTransform: "uppercase",
            marginLeft: "auto",
          }}
        >
          {t("taskPanel.riskLevel", { level: request.riskLevel.toUpperCase() })}
        </span>
      </div>

      {/* Action description */}
      <div style={{ color: "#888", fontSize: 11 }}>
        {t("taskPanel.wantsTo", { tool: request.toolName, context: request.context })}
      </div>

      {/* Buttons */}
      <div style={{ display: "flex", gap: 6, marginTop: 2 }}>
        <button
          onClick={() => void handleApprove()}
          style={{
            flex: 1,
            padding: "4px 8px",
            background: "rgba(34,197,94,0.12)",
            border: "1px solid rgba(34,197,94,0.3)",
            borderRadius: 4,
            color: "#22C55E",
            fontSize: 10,
            fontFamily: 'ui-monospace, "SF Mono", Menlo, monospace',
            cursor: "pointer",
            fontWeight: "bold",
            letterSpacing: 0.5,
            transition: "background 0.1s",
          }}
          onMouseEnter={(e) => {
            (e.currentTarget as HTMLButtonElement).style.background = "rgba(34,197,94,0.22)";
          }}
          onMouseLeave={(e) => {
            (e.currentTarget as HTMLButtonElement).style.background = "rgba(34,197,94,0.12)";
          }}
        >
          {t("taskPanel.approve")}
        </button>

        <button
          onClick={() => void handleDeny()}
          style={{
            flex: 1,
            padding: "4px 8px",
            background: "rgba(239,68,68,0.12)",
            border: "1px solid rgba(239,68,68,0.3)",
            borderRadius: 4,
            color: "#EF4444",
            fontSize: 10,
            fontFamily: 'ui-monospace, "SF Mono", Menlo, monospace',
            cursor: "pointer",
            fontWeight: "bold",
            letterSpacing: 0.5,
            transition: "background 0.1s",
          }}
          onMouseEnter={(e) => {
            (e.currentTarget as HTMLButtonElement).style.background = "rgba(239,68,68,0.22)";
          }}
          onMouseLeave={(e) => {
            (e.currentTarget as HTMLButtonElement).style.background = "rgba(239,68,68,0.12)";
          }}
        >
          {t("taskPanel.deny")}
        </button>

        <button
          style={{
            padding: "4px 8px",
            background: "transparent",
            border: "1px solid #2A2A2A",
            borderRadius: 4,
            color: "#666",
            fontSize: 10,
            fontFamily: 'ui-monospace, "SF Mono", Menlo, monospace',
            cursor: "pointer",
            letterSpacing: 0.5,
            transition: "border-color 0.1s, color 0.1s",
          }}
          onMouseEnter={(e) => {
            const el = e.currentTarget as HTMLButtonElement;
            el.style.borderColor = "#444";
            el.style.color = "#999";
          }}
          onMouseLeave={(e) => {
            const el = e.currentTarget as HTMLButtonElement;
            el.style.borderColor = "#2A2A2A";
            el.style.color = "#666";
          }}
        >
          {t("taskPanel.viewDetail")}
        </button>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// ApprovalQueue
// ---------------------------------------------------------------------------

export function ApprovalQueue() {
  const { t } = useTranslation();
  const pendingApprovals = useApprovalStore((s) => s.pendingApprovals);

  return (
    <div
      style={{
        background: "#070707",
        border: "1px solid #1A1A1A",
        borderRadius: 6,
        overflow: "hidden",
        fontFamily: 'ui-monospace, "SF Mono", Menlo, monospace',
      }}
    >
      {/* Section header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 8,
          padding: "8px 12px",
          borderBottom: "1px solid #141414",
          background: "#090909",
        }}
      >
        <span style={{ color: "#888", fontSize: 10, fontWeight: "bold", letterSpacing: 1, textTransform: "uppercase" }}>
          {t("taskPanel.pendingApprovals")}
        </span>
        {pendingApprovals.length > 0 && (
          <span
            style={{
              display: "inline-flex",
              alignItems: "center",
              justifyContent: "center",
              width: 18,
              height: 18,
              borderRadius: "50%",
              background: "rgba(239,68,68,0.2)",
              border: "1px solid rgba(239,68,68,0.4)",
              color: "#EF4444",
              fontSize: 9,
              fontWeight: "bold",
            }}
          >
            {pendingApprovals.length}
          </span>
        )}
      </div>

      {/* Content */}
      <div style={{ padding: "8px 10px", display: "flex", flexDirection: "column", gap: 8 }}>
        {pendingApprovals.length === 0 ? (
          <div style={{ color: "#444", fontSize: 11, textAlign: "center", padding: "8px 0" }}>
            {t("taskPanel.noApprovals")}
          </div>
        ) : (
          pendingApprovals.map((req) => (
            <ApprovalCard key={req.id} request={req} />
          ))
        )}
      </div>
    </div>
  );
}
