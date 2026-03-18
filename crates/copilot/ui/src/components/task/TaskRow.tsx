import type { Task, TaskStep } from "../../stores/taskStore";

// ---------------------------------------------------------------------------
// Role color badges
// ---------------------------------------------------------------------------

const ROLE_COLORS: Record<string, string> = {
  orchestrator: "#F97316",
  researcher: "#3B82F6",
  analyst: "#8B5CF6",
  writer: "#10B981",
  coder: "#EC4899",
  operator: "#F59E0B",
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatDuration(ms: number): string {
  const seconds = Math.floor(ms / 1000);
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const remaining = seconds % 60;
  return `${minutes}m ${remaining}s`;
}

function getTaskProgress(task: Task): number {
  if (task.steps.length === 0) return 0;
  const done = task.steps.filter((s) => s.status === "completed" || s.status === "cancelled").length;
  return Math.round((done / task.steps.length) * 100);
}

function getStatusIcon(status: TaskStep["status"]): string {
  switch (status) {
    case "completed": return "✓";
    case "running": return "⏳";
    case "failed": return "✗";
    case "cancelled": return "○";
    default: return "○";
  }
}

function getStatusColor(status: TaskStep["status"]): string {
  switch (status) {
    case "completed": return "#22C55E";
    case "running": return "#F97316";
    case "failed": return "#EF4444";
    case "cancelled": return "#6B7280";
    default: return "#6B7280";
  }
}

function getProgressColor(status: string): string {
  switch (status) {
    case "completed": return "#22C55E";
    case "running": return "#3B82F6";
    case "failed": return "#EF4444";
    case "paused": return "#F59E0B";
    default: return "#6B7280";
  }
}

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

export interface TaskRowProps {
  task: Task;
  isSubTask: boolean;
  isExpanded?: boolean;
  onToggleExpand?: () => void;
}

// ---------------------------------------------------------------------------
// SubTaskRow — a single step row (indented)
// ---------------------------------------------------------------------------

function SubTaskRow({
  step,
  isLast,
}: {
  step: TaskStep;
  isLast: boolean;
}) {
  const roleColor = ROLE_COLORS[step.agentRole ?? ""] ?? "#6B7280";
  const statusColor = getStatusColor(step.status);
  const icon = getStatusIcon(step.status);

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 6,
        paddingLeft: 32,
        paddingRight: 12,
        paddingTop: 3,
        paddingBottom: 3,
        fontFamily: 'ui-monospace, "SF Mono", Menlo, monospace',
        fontSize: 11,
      }}
    >
      {/* Tree connector */}
      <span style={{ color: "#444", flexShrink: 0, width: 16, textAlign: "right" }}>
        {isLast ? "└" : "├"}
      </span>

      {/* Status icon */}
      <span style={{ color: statusColor, flexShrink: 0, width: 14, textAlign: "center" }}>
        {icon}
      </span>

      {/* Step title */}
      <span
        style={{
          flex: 1,
          color: step.status === "completed" ? "#888" : step.status === "running" ? "#E5E5E5" : "#666",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
          textDecoration: step.status === "cancelled" ? "line-through" : "none",
        }}
      >
        {step.title}
      </span>

      {/* Agent badge */}
      {step.agentName && (
        <span
          style={{
            flexShrink: 0,
            fontSize: 9,
            fontWeight: "bold",
            letterSpacing: 0.5,
            color: roleColor,
            background: `${roleColor}18`,
            border: `1px solid ${roleColor}40`,
            borderRadius: 3,
            padding: "1px 4px",
            textTransform: "uppercase",
          }}
        >
          {step.agentName}
        </span>
      )}

      {/* Duration or status text */}
      <span style={{ flexShrink: 0, color: "#555", fontSize: 10, minWidth: 36, textAlign: "right" }}>
        {step.status === "completed" && step.duration != null
          ? formatDuration(step.duration)
          : step.status === "running"
          ? "run"
          : step.status === "failed"
          ? "fail"
          : "wait"}
      </span>
    </div>
  );
}

// ---------------------------------------------------------------------------
// TaskRow — parent task row (with expand/collapse)
// ---------------------------------------------------------------------------

export function TaskRow({ task, isSubTask, isExpanded, onToggleExpand }: TaskRowProps) {
  if (isSubTask) {
    // Render as a step row — caller should use SubTaskRow directly
    return null;
  }

  const progress = getTaskProgress(task);
  const progressColor = getProgressColor(task.status);
  const hasSteps = task.steps.length > 0;

  return (
    <div
      style={{
        fontFamily: 'ui-monospace, "SF Mono", Menlo, monospace',
        borderBottom: "1px solid #111",
      }}
    >
      {/* Parent row */}
      <div
        onClick={hasSteps ? onToggleExpand : undefined}
        style={{
          display: "flex",
          alignItems: "center",
          gap: 6,
          paddingLeft: 10,
          paddingRight: 12,
          paddingTop: 7,
          paddingBottom: 7,
          cursor: hasSteps ? "pointer" : "default",
          background: "transparent",
          transition: "background 0.1s",
        }}
        onMouseEnter={(e) => {
          (e.currentTarget as HTMLDivElement).style.background = "#0F0F0F";
        }}
        onMouseLeave={(e) => {
          (e.currentTarget as HTMLDivElement).style.background = "transparent";
        }}
      >
        {/* Expand arrow */}
        <span
          style={{
            flexShrink: 0,
            width: 14,
            textAlign: "center",
            color: hasSteps ? "#666" : "transparent",
            fontSize: 10,
            transition: "transform 0.1s",
            display: "inline-block",
            transform: isExpanded ? "rotate(90deg)" : "rotate(0deg)",
          }}
        >
          ▶
        </span>

        {/* Status dot */}
        <span
          style={{
            flexShrink: 0,
            width: 7,
            height: 7,
            borderRadius: "50%",
            background: progressColor,
            display: "inline-block",
          }}
        />

        {/* Task title */}
        <span
          style={{
            flex: 1,
            color: "#D4D4D4",
            fontSize: 12,
            fontWeight: "500",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          {task.title}
        </span>

        {/* Progress bar + percentage */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 6,
            flexShrink: 0,
          }}
        >
          <div
            style={{
              width: 60,
              height: 4,
              background: "#1A1A1A",
              borderRadius: 2,
              overflow: "hidden",
            }}
          >
            <div
              style={{
                width: `${progress}%`,
                height: "100%",
                background: progressColor,
                borderRadius: 2,
                transition: "width 0.3s ease",
              }}
            />
          </div>
          <span
            style={{
              fontSize: 10,
              color: "#666",
              minWidth: 28,
              textAlign: "right",
            }}
          >
            {progress}%
          </span>
        </div>
      </div>

      {/* Sub-tasks (steps) */}
      {isExpanded && hasSteps && (
        <div style={{ paddingBottom: 4 }}>
          {task.steps.map((step, idx) => (
            <SubTaskRow
              key={step.id}
              step={step}
              isLast={idx === task.steps.length - 1}
            />
          ))}
        </div>
      )}
    </div>
  );
}
