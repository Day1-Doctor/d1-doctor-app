import { useRef } from "react";
import { useAgentStore } from "../stores/agentStore";
import { useCanvasSize } from "../hooks/useCanvasSize";
import { IsometricCanvas } from "../components/office/IsometricCanvas";

/**
 * OfficeView — Full-height canvas container that renders the 2.5D
 * isometric office. Reads agents from the Zustand store and pipes
 * them into the `IsometricCanvas` component.
 */
export function OfficeView() {
  const containerRef = useRef<HTMLDivElement>(null);
  const { width, height } = useCanvasSize(containerRef);
  const agents = useAgentStore((s) => s.agents);

  return (
    <div
      ref={containerRef}
      className="flex-1"
      style={{ minHeight: 0, overflow: "hidden" }}
    >
      {width > 0 && height > 0 && (
        <IsometricCanvas agents={agents} width={width} height={height} />
      )}
    </div>
  );
}
