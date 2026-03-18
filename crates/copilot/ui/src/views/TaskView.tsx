import { useTaskStore } from "../stores/taskStore";
import { TaskTimeline } from "../components/task/TaskTimeline";

const statusLabel: Record<string, string> = {
  pending: "Pending",
  running: "Running",
  paused: "Paused",
  completed: "Completed",
  failed: "Failed",
  cancelled: "Cancelled",
};

const statusColor: Record<string, string> = {
  pending: "text-text-muted",
  running: "text-accent",
  paused: "text-warning",
  completed: "text-success",
  failed: "text-error",
  cancelled: "text-text-muted",
};

function formatDuration(ms: number): string {
  const seconds = Math.floor(ms / 1000);
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const remaining = seconds % 60;
  return `${minutes}m ${remaining}s`;
}

export function TaskView() {
  const activeTask = useTaskStore((s) =>
    s.tasks.find((t) => t.id === s.activeTaskId),
  );

  if (!activeTask) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center gap-4">
        <div className="flex items-center gap-3">
          <svg
            width="32"
            height="32"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
            className="text-text-muted"
            aria-hidden="true"
          >
            <path d="M9 11l3 3L22 4" />
            <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
          </svg>
          <h2 className="text-xl font-semibold text-text-primary">
            Task View
          </h2>
        </div>
        <p className="text-text-muted text-sm">No active task</p>
        <div className="w-12 h-0.5 bg-border rounded-full" />
      </div>
    );
  }

  const completedSteps = activeTask.steps.filter(
    (s) => s.status === "completed",
  ).length;
  const totalSteps = activeTask.steps.length;

  return (
    <div className="flex-1 flex flex-col min-h-0 px-6 py-4 gap-6">
      {/* Header: Task title + status badge + duration */}
      <header className="flex items-center justify-between shrink-0">
        <div className="flex items-center gap-3 min-w-0">
          <svg
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
            className="text-accent shrink-0"
            aria-hidden="true"
          >
            <path d="M9 11l3 3L22 4" />
            <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
          </svg>
          <h2 className="text-lg font-semibold text-text-primary truncate">
            {activeTask.title}
          </h2>
          <span
            className={`text-xs font-medium px-2 py-0.5 rounded-full border border-border ${statusColor[activeTask.status] ?? "text-text-muted"}`}
          >
            {statusLabel[activeTask.status] ?? activeTask.status}
          </span>
        </div>

        <div className="flex items-center gap-4 text-xs text-text-secondary shrink-0">
          <span>
            {completedSteps}/{totalSteps} steps
          </span>
          {activeTask.totalDuration != null && (
            <span>{formatDuration(activeTask.totalDuration)}</span>
          )}
        </div>
      </header>

      {/* Timeline */}
      <section className="shrink-0">
        <TaskTimeline steps={activeTask.steps} />
      </section>

      {/* Artifacts placeholder */}
      <section className="flex-1 flex flex-col items-center justify-center min-h-0 border border-dashed border-border rounded-lg">
        <svg
          width="24"
          height="24"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
          className="text-text-disabled mb-2"
          aria-hidden="true"
        >
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
          <polyline points="14 2 14 8 20 8" />
        </svg>
        <p className="text-text-disabled text-xs">
          Artifacts will appear here
        </p>
      </section>
    </div>
  );
}
