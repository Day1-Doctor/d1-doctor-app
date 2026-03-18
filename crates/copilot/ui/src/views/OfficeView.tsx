import { useRef, useCallback } from "react";
import { useAgentStore } from "../stores/agentStore";
import { useTaskStore } from "../stores/taskStore";
import { useCanvasSize } from "../hooks/useCanvasSize";
import { IsometricCanvas } from "../components/office/IsometricCanvas";
import { AgentDetailPanel } from "../components/office/AgentDetailPanel";
import { TaskTimeline } from "../components/task/TaskTimeline";
import { OFFICE_LAYOUT, hitTestAgent } from "../components/office/OfficeRenderer";

/**
 * OfficeView — Full-height canvas container that renders the 2.5D
 * isometric office. Reads agents from the Zustand store and pipes
 * them into the `IsometricCanvas` component.
 *
 * When a task is active, a glassmorphic TaskTimeline bar is shown
 * at the bottom of the canvas.
 *
 * Clicking on an agent in the canvas opens the AgentDetailPanel.
 */
export function OfficeView() {
  const containerRef = useRef<HTMLDivElement>(null);
  const { width, height } = useCanvasSize(containerRef);
  const agents = useAgentStore((s) => s.agents);
  const selectAgent = useAgentStore((s) => s.selectAgent);
  const selectedAgentId = useAgentStore((s) => s.selectedAgentId);

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
      {width > 0 && height > 0 && (
        <IsometricCanvas agents={agents} width={width} height={height} />
      )}

      {/* Task timeline bottom overlay */}
      {activeTask && activeTask.steps.length > 0 && (
        <div
          className="absolute bottom-0 left-0 right-0 border-t border-border/50"
          style={{
            background: "rgba(5, 5, 5, 0.75)",
            backdropFilter: "blur(12px)",
            WebkitBackdropFilter: "blur(12px)",
          }}
        >
          <div className="flex items-center gap-3 px-4 py-1">
            <span className="text-text-secondary text-[10px] font-medium shrink-0 uppercase tracking-wider">
              Task
            </span>
            <span className="text-text-primary text-xs truncate max-w-[160px]">
              {activeTask.title}
            </span>
            <div className="w-px h-4 bg-border shrink-0" />
            <TaskTimeline
              steps={activeTask.steps}
              className="flex-1 min-w-0"
            />
          </div>
        </div>
      )}

      {/* Agent detail slide-in panel */}
      {selectedAgentId && <AgentDetailPanel />}
    </div>
  );
}
