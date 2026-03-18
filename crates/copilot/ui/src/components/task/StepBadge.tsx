import type { TaskStep } from "../../stores/taskStore";

interface StepBadgeProps {
  step: TaskStep;
  isActive: boolean;
  onClick?: () => void;
}

const statusColors: Record<TaskStep["status"], string> = {
  pending: "#6B7280",
  running: "#F97316",
  completed: "#22C55E",
  failed: "#EF4444",
  cancelled: "#6B7280",
};

function StatusIcon({ status }: { status: TaskStep["status"] }) {
  switch (status) {
    case "completed":
      return (
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2.5"
          strokeLinecap="round"
          strokeLinejoin="round"
          className="text-success shrink-0"
          aria-hidden="true"
        >
          <polyline points="20 6 9 17 4 12" />
        </svg>
      );
    case "running":
      return (
        <span
          className="inline-block w-3 h-3 rounded-full bg-accent step-pulse shrink-0"
          aria-label="Running"
        />
      );
    case "failed":
      return (
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2.5"
          strokeLinecap="round"
          strokeLinejoin="round"
          className="text-error shrink-0"
          aria-hidden="true"
        >
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      );
    case "cancelled":
      return (
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          className="text-text-muted shrink-0"
          aria-hidden="true"
        >
          <circle cx="12" cy="12" r="10" />
          <line x1="4.93" y1="4.93" x2="19.07" y2="19.07" />
        </svg>
      );
    case "pending":
    default:
      return (
        <span
          className="inline-block w-3 h-3 rounded-full border-2 shrink-0"
          style={{ borderColor: "#6B7280" }}
          aria-label="Pending"
        />
      );
  }
}

function formatDuration(ms: number): string {
  const seconds = Math.floor(ms / 1000);
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const remaining = seconds % 60;
  return `${minutes}m ${remaining}s`;
}

export function StepBadge({ step, isActive, onClick }: StepBadgeProps) {
  const borderColor = statusColors[step.status];

  return (
    <button
      type="button"
      onClick={onClick}
      className={`
        flex flex-col items-start gap-1 px-3 py-2 rounded-lg
        border-l-[3px] min-w-[120px] max-w-[180px]
        transition-colors duration-150
        cursor-pointer select-none
        ${isActive ? "bg-muted/80" : "bg-card hover:bg-muted/50"}
      `}
      style={{ borderLeftColor: borderColor }}
      aria-current={isActive ? "step" : undefined}
    >
      <div className="flex items-center gap-2 w-full">
        <StatusIcon status={step.status} />
        <span className="text-text-primary text-xs font-medium truncate">
          {step.title}
        </span>
      </div>

      {step.agentName && (
        <span className="text-text-muted text-[10px] pl-5 truncate w-full">
          {step.agentName}
          {step.agentRole ? ` (${step.agentRole})` : ""}
        </span>
      )}

      {step.duration != null && step.status === "completed" && (
        <span className="text-text-disabled text-[10px] pl-5">
          {formatDuration(step.duration)}
        </span>
      )}
    </button>
  );
}
