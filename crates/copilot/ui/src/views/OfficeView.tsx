import { useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { useAgentStore } from "../stores/agentStore";
import { useTaskStore } from "../stores/taskStore";
import { useViewStore } from "../stores/viewStore";
import { useCanvasSize } from "../hooks/useCanvasSize";
import { IsometricCanvas } from "../components/office/IsometricCanvas";
import { AgentDetailPanel } from "../components/office/AgentDetailPanel";
import { OFFICE_LAYOUT, hitTestAgent } from "../components/office/OfficeRenderer";
import type { Task } from "../stores/taskStore";

/**
 * Thin horizontal segmented progress bar showing task steps at the bottom
 * of the office view. Each segment represents one step, color-coded by status.
 */
function TaskProgressBar({ activeTask }: { activeTask: Task }) {
  const steps = activeTask.steps;
  if (!steps.length) return null;
  const totalSteps = steps.length;
  const completedCount = steps.filter((s) => s.status === "completed").length;
  const pct = Math.round((completedCount / totalSteps) * 100);

  return (
    <div
      className="absolute bottom-0 left-0 right-0 flex items-center px-3 gap-1 border-t border-border/50"
      style={{
        height: 40,
        background: "rgba(5, 5, 8, 0.8)",
        backdropFilter: "blur(12px)",
        WebkitBackdropFilter: "blur(12px)",
      }}
    >
      {/* Task name */}
      <span
        className="text-sm shrink-0 mr-2 truncate max-w-[150px]"
        style={{ color: "#888888" }}
      >
        {activeTask.title}
      </span>

      {/* Segmented progress bar */}
      <div className="flex-1 flex gap-0.5 rounded-full overflow-hidden" style={{ height: 12 }}>
        {steps.map((step) => (
          <button
            key={step.id}
            className="flex-1 relative cursor-pointer transition-all hover:brightness-125"
            style={{
              backgroundColor:
                step.status === "completed"
                  ? "#22C55E"
                  : step.status === "running"
                    ? "#F97316"
                    : step.status === "failed"
                      ? "#EF4444"
                      : "#1F1F1F",
            }}
            title={`${step.title}${step.agentName ? ` (${step.agentName})` : ""} — ${step.status}`}
          >
            {step.status === "running" && (
              <div className="absolute inset-0 bg-white/20 animate-pulse rounded" />
            )}
          </button>
        ))}
      </div>

      {/* Percentage */}
      <span
        className="text-sm shrink-0 ml-2 tabular-nums"
        style={{ color: "#555555" }}
      >
        {pct}%
      </span>
    </div>
  );
}

/**
 * OfficeView — Full-height canvas container that renders the 2.5D
 * isometric office. Reads agents from the Zustand store and pipes
 * them into the `IsometricCanvas` component.
 *
 * When a task is active, a thin segmented TaskProgressBar is shown
 * at the bottom of the canvas.
 *
 * Clicking on an agent in the canvas opens the AgentDetailPanel.
 */
export function OfficeView() {
  const { t } = useTranslation();
  const containerRef = useRef<HTMLDivElement>(null);
  const { width, height } = useCanvasSize(containerRef);
  const agents = useAgentStore((s) => s.agents);
  const selectAgent = useAgentStore((s) => s.selectAgent);
  const selectedAgentId = useAgentStore((s) => s.selectedAgentId);
  const setActiveView = useViewStore((s) => s.setActiveView);

  const activeTask = useTaskStore((s) =>
    s.tasks.find((t) => t.id === s.activeTaskId),
  );

  /** Handle clicks on the canvas to select/deselect agents. */
  const handleCanvasClick = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      if (width === 0 || height === 0) return;

      const rect = (e.currentTarget as HTMLDivElement).getBoundingClientRect();
      const clickX = e.clientX - rect.left;
      const clickY = e.clientY - rect.top;

      // Check if any agent was clicked
      for (const desk of OFFICE_LAYOUT) {
        const agent = agents.find((a) => a.name === desk.label);
        if (!agent) continue;

        if (hitTestAgent(desk, clickX, clickY, width, height)) {
          selectAgent(agent.id);
          return;
        }
      }

      // Clicked on empty space — deselect
      selectAgent(null);
    },
    [agents, width, height, selectAgent],
  );

  return (
    <div
      ref={containerRef}
      className="flex-1 relative"
      style={{ minHeight: 0, overflow: "hidden" }}
      onClick={handleCanvasClick}
    >
      {/* Back to Valley button */}
      <button
        onClick={() => setActiveView("valley")}
        className="absolute top-3 left-3 z-10 flex items-center gap-1.5 px-3 py-1.5
          rounded-md text-sm text-text-secondary hover:text-text-primary
          bg-black/50 hover:bg-black/70 border border-border/30
          backdrop-blur-sm transition-colors duration-100
          focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50"
        aria-label={t("valley.title")}
      >
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          aria-hidden="true"
        >
          <polyline points="15 18 9 12 15 6" />
        </svg>
        {t("valley.title")}
      </button>

      {width > 0 && height > 0 && (
        <IsometricCanvas agents={agents} width={width} height={height} />
      )}

      {/* Task progress bar — thin 40px strip at bottom */}
      {activeTask && activeTask.steps.length > 0 && (
        <TaskProgressBar activeTask={activeTask} />
      )}

      {/* Agent detail slide-in panel */}
      {selectedAgentId && <AgentDetailPanel />}
    </div>
  );
}
