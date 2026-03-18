import { useState } from "react";
import { EventLog } from "../components/debug/EventLog";
import { ToolTrace } from "../components/debug/ToolTrace";
import { FpsCounter } from "../utils/performance";

type DebugTab = "events" | "tools";

export function DebugView() {
  const [activeTab, setActiveTab] = useState<DebugTab>("events");

  return (
    <div className="flex-1 flex flex-col min-h-0">
      {/* Tab bar + FPS counter */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-border shrink-0">
        <div className="flex items-center gap-1">
          <TabButton
            label="Event Log"
            active={activeTab === "events"}
            onClick={() => setActiveTab("events")}
          />
          <TabButton
            label="Tool Trace"
            active={activeTab === "tools"}
            onClick={() => setActiveTab("tools")}
          />
        </div>
        <FpsCounter />
      </div>

      {/* Tab content */}
      <div className="flex-1 min-h-0">
        {activeTab === "events" && <EventLog />}
        {activeTab === "tools" && <ToolTrace />}
      </div>
    </div>
  );
}

function TabButton({
  label,
  active,
  onClick,
}: {
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`px-3 py-1 rounded text-xs font-medium transition-colors duration-100
        focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50
        ${
          active
            ? "bg-muted text-text-primary"
            : "text-text-secondary hover:text-text-primary hover:bg-muted/50"
        }`}
    >
      {label}
    </button>
  );
}
