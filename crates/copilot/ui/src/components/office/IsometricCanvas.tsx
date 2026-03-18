import { useCallback, useEffect, useRef } from "react";
import type { Agent } from "../../stores/agentStore";
import {
  drawOffice,
  OFFICE_LAYOUT,
  type AgentRenderState,
  type DeskPosition,
} from "./OfficeRenderer";

interface IsometricCanvasProps {
  agents: Agent[];
  width: number;
  height: number;
}

/** Name-to-desk index for fast lookup. */
const deskByName = new Map<string, number>(
  OFFICE_LAYOUT.map((d, i) => [d.label, i]),
);

/**
 * Assign agents to desks by matching agent.name to desk.label.
 * Returns a fresh layout copy with `agentId` populated.
 */
function assignDesks(agents: Agent[]): DeskPosition[] {
  const layout: DeskPosition[] = OFFICE_LAYOUT.map((d) => ({ ...d }));
  for (const agent of agents) {
    const idx = deskByName.get(agent.name);
    if (idx !== undefined) {
      layout[idx].agentId = agent.id;
    }
  }
  return layout;
}

/**
 * `IsometricCanvas` renders the 2.5D isometric office using the Canvas 2D
 * API. It owns the `requestAnimationFrame` loop and delegates all drawing
 * to pure functions in `OfficeRenderer`.
 */
export function IsometricCanvas({
  agents,
  width,
  height,
}: IsometricCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const frameRef = useRef(0);
  const rafRef = useRef(0);
  const lastTimeRef = useRef(0);

  // Stash latest props in refs so the render loop closure always reads
  // current values without restarting the loop.
  const agentsRef = useRef(agents);
  agentsRef.current = agents;

  const widthRef = useRef(width);
  widthRef.current = width;

  const heightRef = useRef(height);
  heightRef.current = height;

  /** Core render tick. Targets ~30 FPS (33 ms interval). */
  const tick = useCallback((time: number) => {
    const elapsed = time - lastTimeRef.current;

    // Throttle to ~30 FPS
    if (elapsed >= 33) {
      lastTimeRef.current = time;
      frameRef.current += 1;

      const canvas = canvasRef.current;
      if (!canvas) return;

      const w = widthRef.current;
      const h = heightRef.current;
      if (w === 0 || h === 0) return;

      // Handle DPR for sharp rendering
      const dpr = window.devicePixelRatio || 1;
      if (
        canvas.width !== Math.floor(w * dpr) ||
        canvas.height !== Math.floor(h * dpr)
      ) {
        canvas.width = Math.floor(w * dpr);
        canvas.height = Math.floor(h * dpr);
      }

      const ctx = canvas.getContext("2d");
      if (!ctx) return;

      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

      // Build agent render map
      const currentAgents = agentsRef.current;
      const desks = assignDesks(currentAgents);

      const agentMap = new Map<string, AgentRenderState>();
      for (const agent of currentAgents) {
        agentMap.set(agent.id, { agent, frame: frameRef.current });
      }

      drawOffice(ctx, w, h, desks, agentMap);
    }

    rafRef.current = requestAnimationFrame(tick);
  }, []);

  // Start / stop the render loop
  useEffect(() => {
    rafRef.current = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(rafRef.current);
  }, [tick]);

  return (
    <canvas
      ref={canvasRef}
      style={{ width, height, display: "block" }}
      aria-label="Isometric office view showing agent workstations"
    />
  );
}
