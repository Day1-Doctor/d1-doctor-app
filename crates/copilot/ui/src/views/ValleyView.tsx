import { useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { useViewStore } from "../stores/viewStore";
import { useTaskStore } from "../stores/taskStore";
import { useBillingStore } from "../stores/billingStore";
import { useCanvasSize } from "../hooks/useCanvasSize";
import { ValleyCanvas } from "../components/valley/ValleyCanvas";
import { TaskTimeline } from "../components/task/TaskTimeline";
import { VALLEY_LAYOUT } from "../components/valley/ValleyRenderer";

/**
 * ValleyView — Full-height canvas container that renders the Cowork Valley
 * isometric landscape. Clicking an active building navigates to the office
 * view; clicking a locked building shows an upgrade prompt.
 */
export function ValleyView() {
  const { t } = useTranslation();
  const containerRef = useRef<HTMLDivElement>(null);
  const { width, height } = useCanvasSize(containerRef);
  const setActiveView = useViewStore((s) => s.setActiveView);
  const openUpgradePrompt = useBillingStore((s) => s.openUpgradePrompt);

  const activeTask = useTaskStore((s) =>
    s.tasks.find((task) => task.id === s.activeTaskId),
  );

  /** Handle building click — navigate or upgrade prompt. */
  const handleBuildingClick = useCallback(
    (buildingId: string, isActive: boolean) => {
      if (isActive) {
        // Navigate to the office view
        setActiveView("office");
      } else {
        // Find the building name for the prompt
        const building = VALLEY_LAYOUT.find((b) => b.id === buildingId);
        const name = building?.name ?? "this office";
        openUpgradePrompt(
          t("valley.upgradeToUnlock") + `: ${name}`,
        );
      }
    },
    [setActiveView, openUpgradePrompt, t],
  );

  return (
    <div
      ref={containerRef}
      className="flex-1 relative"
      style={{ minHeight: 0, overflow: "hidden" }}
    >
      {width > 0 && height > 0 && (
        <ValleyCanvas
          width={width}
          height={height}
          onBuildingClick={handleBuildingClick}
        />
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
              {t("nav.tasks")}
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
