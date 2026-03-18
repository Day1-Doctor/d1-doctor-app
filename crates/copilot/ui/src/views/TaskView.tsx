import { TaskPanel } from "../components/task/TaskPanel";
import { ApprovalQueue } from "../components/task/ApprovalQueue";

/**
 * TaskView — Linear/Asana-style task management view.
 *
 * Layout:
 *   - Top: TaskPanel (scrollable task list with expand/collapse, sub-tasks)
 *   - Bottom: ApprovalQueue (pending agent approval requests)
 */
export function TaskView() {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        flex: 1,
        minHeight: 0,
        padding: "12px 14px",
        gap: 12,
        background: "#050508",
        overflowY: "auto",
      }}
    >
      {/* Task list panel — flex to fill available space */}
      <TaskPanel />

      {/* Pending approvals section */}
      <ApprovalQueue />
    </div>
  );
}
