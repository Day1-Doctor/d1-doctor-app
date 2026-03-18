import { useCallback, useEffect, useRef } from "react";
import { useBillingStore } from "../../stores/billingStore";
import { useAgentStore } from "../../stores/agentStore";
import {
  drawValley,
  hitTestBuilding,
  VALLEY_LAYOUT,
  type OfficeBuilding,
} from "./ValleyRenderer";

interface ValleyCanvasProps {
  width: number;
  height: number;
  onBuildingClick: (buildingId: string, isActive: boolean) => void;
}

/**
 * Determine which buildings are active based on subscription tier.
 *
 * free_man: 1 agent  -> office-1 (Dr. Bob) only
 * mini_shop: 3 agents -> office-1, office-2, office-3
 * rocket_inc: 6 agents -> all
 */
function getActiveBuildings(maxAgents: number): Set<string> {
  const active = new Set<string>();
  const orderedIds = [
    "office-1",
    "office-2",
    "office-3",
    "office-4",
    "office-5",
    "office-6",
  ];
  for (let i = 0; i < Math.min(maxAgents, orderedIds.length); i++) {
    active.add(orderedIds[i]);
  }
  return active;
}

/**
 * `ValleyCanvas` renders the Cowork Valley landscape using Canvas 2D.
 * Owns the requestAnimationFrame loop and delegates drawing to ValleyRenderer.
 */
export function ValleyCanvas({
  width,
  height,
  onBuildingClick,
}: ValleyCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const frameRef = useRef(0);
  const rafRef = useRef(0);
  const lastTimeRef = useRef(0);
  const hoveredRef = useRef<string | null>(null);

  const maxAgents = useBillingStore((s) => s.maxAgents);
  const agents = useAgentStore((s) => s.agents);

  // Stash in refs for the render loop
  const widthRef = useRef(width);
  widthRef.current = width;
  const heightRef = useRef(height);
  heightRef.current = height;
  const maxAgentsRef = useRef(maxAgents);
  maxAgentsRef.current = maxAgents;
  const agentsRef = useRef(agents);
  agentsRef.current = agents;
  const onBuildingClickRef = useRef(onBuildingClick);
  onBuildingClickRef.current = onBuildingClick;

  /** Build the buildings array with active/selected state. */
  function buildBuildingState(): OfficeBuilding[] {
    const activeSet = getActiveBuildings(maxAgentsRef.current);
    return VALLEY_LAYOUT.map((b) => ({
      ...b,
      isActive: activeSet.has(b.id),
      isSelected: hoveredRef.current === b.id,
    }));
  }

  /** Core render tick at ~30 FPS. */
  const tick = useCallback((time: number) => {
    const elapsed = time - lastTimeRef.current;

    if (elapsed >= 33) {
      lastTimeRef.current = time;
      frameRef.current += 1;

      const canvas = canvasRef.current;
      if (!canvas) return;

      const w = widthRef.current;
      const h = heightRef.current;
      if (w === 0 || h === 0) return;

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

      const buildings = buildBuildingState();
      drawValley(ctx, w, h, buildings, frameRef.current);
    }

    rafRef.current = requestAnimationFrame(tick);
  }, []);

  // Start / stop the render loop
  useEffect(() => {
    rafRef.current = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(rafRef.current);
  }, [tick]);

  /** Handle mouse move for hover effects. */
  const handleMouseMove = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      const rect = e.currentTarget.getBoundingClientRect();
      const px = e.clientX - rect.left;
      const py = e.clientY - rect.top;
      const hit = hitTestBuilding(px, py, widthRef.current, heightRef.current);
      hoveredRef.current = hit;
      e.currentTarget.style.cursor = hit ? "pointer" : "default";
    },
    [],
  );

  /** Handle click on buildings. */
  const handleClick = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      const rect = e.currentTarget.getBoundingClientRect();
      const px = e.clientX - rect.left;
      const py = e.clientY - rect.top;
      const hit = hitTestBuilding(px, py, widthRef.current, heightRef.current);
      if (hit) {
        const activeSet = getActiveBuildings(maxAgentsRef.current);
        onBuildingClickRef.current(hit, activeSet.has(hit));
      }
    },
    [],
  );

  const handleMouseLeave = useCallback(() => {
    hoveredRef.current = null;
  }, []);

  return (
    <canvas
      ref={canvasRef}
      style={{ width, height, display: "block" }}
      onMouseMove={handleMouseMove}
      onClick={handleClick}
      onMouseLeave={handleMouseLeave}
      aria-label="Cowork Valley landscape showing agent office buildings"
    />
  );
}
