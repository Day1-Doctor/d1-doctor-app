import { useLayoutStore } from "../../stores/layoutStore";

export function RightPanel() {
  const collapsed = useLayoutStore((s) => s.rightPanelCollapsed);
  const toggleRightPanel = useLayoutStore((s) => s.toggleRightPanel);

  return (
    <aside
      className={`
        border-l border-border bg-card/40 shrink-0
        transition-[width] duration-150 ease-out overflow-hidden
        ${collapsed ? "w-0 border-l-0" : "w-[280px]"}
      `}
      role="complementary"
      aria-label="Artifacts panel"
    >
      {!collapsed && (
        <div className="flex flex-col h-full">
          {/* Header */}
          <div className="flex items-center justify-between h-10 px-3 border-b border-border">
            <span className="text-xs text-text-secondary font-medium">
              Artifacts
            </span>
            <button
              onClick={toggleRightPanel}
              className="p-1 rounded hover:bg-muted text-text-muted hover:text-text-secondary
                transition-colors duration-100 focus:outline-none focus-visible:ring-2
                focus-visible:ring-accent/50"
              aria-label="Close artifacts panel"
              tabIndex={0}
            >
              <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                aria-hidden="true"
              >
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>

          {/* Empty state */}
          <div className="flex-1 flex items-center justify-center p-4">
            <p className="text-text-disabled text-xs text-center">
              No artifacts yet
            </p>
          </div>
        </div>
      )}
    </aside>
  );
}
