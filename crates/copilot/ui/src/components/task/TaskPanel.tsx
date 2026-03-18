import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useTaskStore } from "../../stores/taskStore";
import { TaskRow } from "./TaskRow";

// ---------------------------------------------------------------------------
// TaskPanel — Linear-style task list
// ---------------------------------------------------------------------------

export function TaskPanel() {
  const { t } = useTranslation();
  const tasks = useTaskStore((s) => s.tasks);
  const expandedTaskIds = useTaskStore((s) => s.expandedTaskIds);
  const toggleExpand = useTaskStore((s) => s.toggleExpand);
  const addTask = useTaskStore((s) => s.addTask);

  const [isAdding, setIsAdding] = useState(false);
  const [newTaskTitle, setNewTaskTitle] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isAdding && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isAdding]);

  const handleNewTaskClick = () => {
    setIsAdding(true);
    setNewTaskTitle("");
  };

  const handleAddConfirm = () => {
    const title = newTaskTitle.trim();
    if (title) {
      addTask(title);
    }
    setIsAdding(false);
    setNewTaskTitle("");
  };

  const handleInputKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      handleAddConfirm();
    } else if (e.key === "Escape") {
      setIsAdding(false);
      setNewTaskTitle("");
    }
  };

  return (
    <div
      style={{
        background: "#070707",
        border: "1px solid #1A1A1A",
        borderRadius: 6,
        overflow: "hidden",
        display: "flex",
        flexDirection: "column",
        fontFamily: 'ui-monospace, "SF Mono", Menlo, monospace',
        flex: 1,
        minHeight: 0,
      }}
    >
      {/* Header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "8px 12px",
          borderBottom: "1px solid #141414",
          background: "#090909",
          flexShrink: 0,
        }}
      >
        <span style={{ color: "#888", fontSize: 10, fontWeight: "bold", letterSpacing: 1, textTransform: "uppercase" }}>
          {t("taskPanel.title")}
        </span>
        <button
          onClick={handleNewTaskClick}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 4,
            padding: "3px 8px",
            background: "rgba(249,115,22,0.12)",
            border: "1px solid rgba(249,115,22,0.3)",
            borderRadius: 4,
            color: "#F97316",
            fontSize: 10,
            fontFamily: 'ui-monospace, "SF Mono", Menlo, monospace',
            cursor: "pointer",
            fontWeight: "bold",
            letterSpacing: 0.5,
            transition: "background 0.1s",
          }}
          onMouseEnter={(e) => {
            (e.currentTarget as HTMLButtonElement).style.background = "rgba(249,115,22,0.22)";
          }}
          onMouseLeave={(e) => {
            (e.currentTarget as HTMLButtonElement).style.background = "rgba(249,115,22,0.12)";
          }}
        >
          + {t("taskPanel.newTask")}
        </button>
      </div>

      {/* Task list */}
      <div
        style={{
          flex: 1,
          overflowY: "auto",
          minHeight: 0,
        }}
      >
        {/* Inline new task input */}
        {isAdding && (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 8,
              padding: "7px 12px",
              borderBottom: "1px solid #111",
              background: "#0D0D0D",
            }}
          >
            <span style={{ color: "#F97316", fontSize: 10, flexShrink: 0 }}>+</span>
            <input
              ref={inputRef}
              value={newTaskTitle}
              onChange={(e) => setNewTaskTitle(e.target.value)}
              onKeyDown={handleInputKeyDown}
              onBlur={handleAddConfirm}
              placeholder={t("tasks.inputPlaceholder")}
              style={{
                flex: 1,
                background: "transparent",
                border: "none",
                outline: "none",
                color: "#D4D4D4",
                fontSize: 12,
                fontFamily: 'ui-monospace, "SF Mono", Menlo, monospace',
              }}
            />
          </div>
        )}

        {tasks.length === 0 && !isAdding ? (
          <div
            style={{
              color: "#444",
              fontSize: 11,
              textAlign: "center",
              padding: "24px 12px",
            }}
          >
            {t("taskPanel.noTasks")}
          </div>
        ) : (
          tasks.map((task) => (
            <TaskRow
              key={task.id}
              task={task}
              isSubTask={false}
              isExpanded={expandedTaskIds.has(task.id)}
              onToggleExpand={() => toggleExpand(task.id)}
            />
          ))
        )}
      </div>
    </div>
  );
}
