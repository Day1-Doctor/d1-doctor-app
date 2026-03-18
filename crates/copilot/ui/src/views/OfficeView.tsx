import { useRef } from "react";
import { useAgentStore } from "../stores/agentStore";
import { useTaskStore } from "../stores/taskStore";
import { useCanvasSize } from "../hooks/useCanvasSize";
import { IsometricCanvas } from "../components/office/IsometricCanvas";
import { TaskTimeline } from "../components/task/TaskTimeline";

/**
 * OfficeView — Full-height canvas container that renders the 2.5D
 * isometric office. Reads agents from the Zustand store and pipes
 * them into the `IsometricCanvas` component.
 *
 * When a task is active, a glassmorphic TaskTimeline bar is shown
 * at the bottom of the canvas.
 */
export function OfficeView() {
  const containerRef = useRef<HTMLDivElement>(null);
  const { width, height } = useCanvasSize(containerRef);
  const agents = useAgentStore((s) => s.agents);

  const activeTask = useTaskStore((s) =>
    s.tasks.find((t) => t.id === s.activeTaskId),
  );

  return (
    <div
      ref={containerRef}
      className="flex-1 relative"
      style={{ minHeight: 0, overflow: "hidden" }}
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
    </div>
  );
}
