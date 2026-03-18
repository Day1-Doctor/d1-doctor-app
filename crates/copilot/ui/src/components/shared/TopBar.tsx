import { useTaskStore } from "../../stores/taskStore";
import { CreditMeter } from "./CreditMeter";

export function TopBar() {
  const tasks = useTaskStore((s) => s.tasks);
  const activeTaskId = useTaskStore((s) => s.activeTaskId);
  const activeTask = tasks.find((t) => t.id === activeTaskId);

  return (
    <header
      className="h-12 flex items-center justify-between px-4 border-b border-border
        backdrop-blur-xl saturate-150 bg-black/60 select-none shrink-0"
      data-tauri-drag-region
      role="banner"
    >
      {/* Left: Logo */}
      <div className="flex items-center gap-2 min-w-[180px]">
        <span
          className="text-accent font-semibold text-sm tracking-tight"
          aria-label="Day1 Copilot"
        >
          Day1 Copilot
        </span>
        <span className="text-text-muted text-xs">v0.1.0</span>
      </div>

      {/* Center: Active task */}
      <div className="flex-1 text-center truncate px-4">
        {activeTask ? (
          <span className="text-text-primary text-xs">
            {activeTask.title}
          </span>
        ) : (
          <span className="text-text-disabled text-xs">
            No active task
          </span>
        )}
      </div>

      {/* Right: Credits + Settings */}
      <div className="flex items-center gap-3 min-w-[180px] justify-end">
        <CreditMeter />
        <button
          className="p-1.5 rounded hover:bg-muted text-text-secondary hover:text-text-primary
            transition-colors duration-100 focus:outline-none focus-visible:ring-2
            focus-visible:ring-accent/50"
          aria-label="Settings"
          tabIndex={0}
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
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
        </button>
      </div>
    </header>
  );
}
