import type { TaskStep } from "../../stores/taskStore";

interface StepConnectorProps {
  fromStatus: TaskStep["status"];
  toStatus: TaskStep["status"];
}

/**
 * StepConnector renders a horizontal line + arrow between two step badges.
 *
 * - Solid line when the "from" step is completed.
 * - Animated gradient when connecting completed -> running.
 * - Dashed line when both sides are pending/inactive.
 */
export function StepConnector({ fromStatus, toStatus }: StepConnectorProps) {
  const isCompleted = fromStatus === "completed";
  const isInProgress = fromStatus === "completed" && toStatus === "running";

  return (
    <div
      className="flex items-center shrink-0"
      style={{ width: 32, height: 24 }}
      aria-hidden="true"
    >
      <svg width="32" height="24" viewBox="0 0 32 24" fill="none">
        {/* Main line */}
        <line
          x1="0"
          y1="12"
          x2="24"
          y2="12"
          stroke={isCompleted ? "#22C55E" : "#6B7280"}
          strokeWidth="2"
          strokeDasharray={isCompleted ? "none" : "4 3"}
          className={isInProgress ? "connector-active" : ""}
        />
        {/* Arrow head */}
        <polyline
          points="20,8 26,12 20,16"
          stroke={isCompleted ? "#22C55E" : "#6B7280"}
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          fill="none"
        />
      </svg>
    </div>
  );
}
