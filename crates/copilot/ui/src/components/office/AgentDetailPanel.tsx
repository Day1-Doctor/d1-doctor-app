import { useTranslation } from "react-i18next";
import { useAgentStore, type AgentStatus } from "../../stores/agentStore";
import { useCostStore } from "../../stores/costStore";
import { AGENT_COLORS, STATUS_COLORS } from "./OfficeRenderer";

/** Mock activity log events for the agent detail panel. */
const MOCK_ACTIVITY: { time: string; text: string }[] = [
  { time: "0:42", text: "Task assigned by Dr. Bob" },
  { time: "0:38", text: "Reading project context..." },
  { time: "0:34", text: "Analyzing codebase structure" },
  { time: "0:29", text: "Generating initial plan" },
  { time: "0:25", text: "Executing step 1 of 3" },
  { time: "0:21", text: "Waiting for API response" },
  { time: "0:18", text: "Processing results" },
  { time: "0:14", text: "Writing output artifact" },
  { time: "0:10", text: "Quality check passed" },
  { time: "0:06", text: "Step 2 started" },
];

/** Map agent status to a human-readable FSM label. */
function fsmLabel(status: AgentStatus): string {
  const labels: Record<AgentStatus, string> = {
    idle: "IDLE",
    working: "WORKING",
    thinking: "THINKING",
    executing: "EXECUTING",
    paused: "AWAITING_APPROVAL",
    error: "ERROR",
  };
  return labels[status];
}

/** Mock task names by role. */
const MOCK_TASKS: Record<string, string> = {
  orchestrator: "Coordinating pipeline",
  researcher: "Research AI frameworks",
  analyst: "Analyze findings",
  writer: "Write comparison report",
  coder: "Implement prototype",
  operator: "Save to workspace",
};

/** Mock token counts by role. */
const MOCK_TOKENS: Record<string, number> = {
  orchestrator: 2450,
  researcher: 8200,
  analyst: 5100,
  writer: 4300,
  coder: 6700,
  operator: 1200,
};

/** Mock trust scores by role (0-100). */
const MOCK_TRUST_SCORES: Record<string, number> = {
  orchestrator: 95,
  researcher: 78,
  analyst: 82,
  writer: 88,
  coder: 72,
  operator: 65,
};

function trustScoreColor(score: number): string {
  if (score >= 80) return "#22C55E";
  if (score >= 60) return "#F59E0B";
  if (score >= 40) return "#F97316";
  return "#EF4444";
}

/**
 * AgentDetailPanel — Slide-in panel from the right that shows detailed info
 * about the selected agent. Uses glassmorphic styling with backdrop-blur.
 */
export function AgentDetailPanel() {
  const { t } = useTranslation();
  const selectedAgentId = useAgentStore((s) => s.selectedAgentId);
  const agents = useAgentStore((s) => s.agents);
  const selectAgent = useAgentStore((s) => s.selectAgent);
  const agentCosts = useCostStore((s) => s.agentCosts);

  const agent = agents.find((a) => a.id === selectedAgentId);

  if (!agent) return null;

  const roleColor = AGENT_COLORS[agent.role] ?? "#6B7280";
  const statusColor = STATUS_COLORS[agent.status] ?? STATUS_COLORS.idle;
  const tokenCount = MOCK_TOKENS[agent.role] ?? 0;
  const cost = agentCosts[agent.name] ?? 0;
  const taskName = MOCK_TASKS[agent.role] ?? t("agentDetail.noActiveTask");
  const trustScore = MOCK_TRUST_SCORES[agent.role] ?? 50;
  const tsColor = trustScoreColor(trustScore);

  return (
    <div
      className="absolute top-0 right-0 bottom-0 w-[300px] z-40
        border-l border-border overflow-y-auto"
      style={{
        background: "rgba(13, 13, 13, 0.85)",
        backdropFilter: "blur(20px)",
        WebkitBackdropFilter: "blur(20px)",
        animation: "slideInRight 0.2s ease-out",
      }}
      role="dialog"
      aria-label={`${t("agentDetail.fsmState")}: ${agent.name}`}
    >
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-border">
        <h2 className="text-sm text-text-primary font-medium">{agent.name}</h2>
        <button
          onClick={() => selectAgent(null)}
          className="p-1 rounded hover:bg-muted text-text-muted hover:text-text-primary
            transition-colors duration-100 focus:outline-none focus-visible:ring-2
            focus-visible:ring-accent/50"
          aria-label={t("agentDetail.close")}
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
            aria-hidden="true"
          >
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>

      {/* Body */}
      <div className="px-4 py-3 space-y-4">
        {/* Role badge */}
        <div className="flex items-center gap-2">
          <span
            className="inline-flex items-center px-2 py-0.5 rounded text-[10px]
              font-medium uppercase tracking-wider"
            style={{
              backgroundColor: roleColor + "20",
              color: roleColor,
              border: `1px solid ${roleColor}40`,
            }}
          >
            {agent.role}
          </span>
        </div>

        {/* FSM Status */}
        <div>
          <p className="text-[10px] text-text-muted uppercase tracking-wider mb-1">
            {t("agentDetail.fsmState")}
          </p>
          <div className="flex items-center gap-2">
            <span
              className="inline-block w-2 h-2 rounded-full"
              style={{ backgroundColor: statusColor }}
            />
            <span className="text-xs text-text-primary font-medium">
              {fsmLabel(agent.status)}
            </span>
          </div>
        </div>

        {/* Trust Score (D1D-238) */}
        <div>
          <div className="flex items-center justify-between mb-1">
            <p className="text-[10px] text-text-muted uppercase tracking-wider">
              {t("agentDetail.trustScore")}
            </p>
            <span
              className="text-xs font-medium tabular-nums"
              style={{ color: tsColor }}
            >
              {trustScore}%
            </span>
          </div>
          <div className="w-full h-1.5 bg-muted rounded-full overflow-hidden">
            <div
              className="h-full rounded-full transition-all duration-300"
              style={{
                width: `${trustScore}%`,
                backgroundColor: tsColor,
              }}
            />
          </div>
        </div>

        {/* Current Task */}
        <div>
          <p className="text-[10px] text-text-muted uppercase tracking-wider mb-1">
            {t("agentDetail.currentTask")}
          </p>
          <p className="text-xs text-text-secondary">{taskName}</p>
        </div>

        {/* Session Stats */}
        <div className="grid grid-cols-2 gap-3">
          <div
            className="rounded-lg border border-border p-2"
            style={{ backgroundColor: "rgba(15, 15, 15, 0.6)" }}
          >
            <p className="text-[10px] text-text-muted uppercase tracking-wider mb-0.5">
              {t("agentDetail.tokens")}
            </p>
            <p className="text-sm text-text-primary font-medium tabular-nums">
              {tokenCount.toLocaleString()}
            </p>
          </div>
          <div
            className="rounded-lg border border-border p-2"
            style={{ backgroundColor: "rgba(15, 15, 15, 0.6)" }}
          >
            <p className="text-[10px] text-text-muted uppercase tracking-wider mb-0.5">
              {t("agentDetail.costDD")}
            </p>
            <p className="text-sm text-accent font-medium tabular-nums">
              {cost}
            </p>
          </div>
        </div>

        {/* Activity Log */}
        <div>
          <p className="text-[10px] text-text-muted uppercase tracking-wider mb-2">
            {t("agentDetail.activityLog")}
          </p>
          <div className="space-y-1.5 max-h-[240px] overflow-y-auto">
            {MOCK_ACTIVITY.map((event, i) => (
              <div key={i} className="flex gap-2 text-[11px]">
                <span className="text-text-disabled shrink-0 tabular-nums w-8">
                  {event.time}
                </span>
                <span className="text-text-secondary">{event.text}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
