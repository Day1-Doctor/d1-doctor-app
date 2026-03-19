import { useCallback, useEffect, useRef, useState } from "react";
import { useBillingStore } from "../../stores/billingStore";
import { useOfficeStore } from "../../stores/officeStore";
import {
  drawCampus,
  hitTestParcel,
  FILL_ORDER,
  PARCEL_POSITIONS,
  type Parcel,
} from "./ValleyRenderer";

// ---------------------------------------------------------------------------
// Tier → max offices mapping
// ---------------------------------------------------------------------------

const TIER_MAX_OFFICES: Record<string, number> = {
  free_man: 1,
  mini_shop: 3,
  rocket_inc: 8,
};

interface ValleyCanvasProps {
  width: number;
  height: number;
  onParcelClick: (parcelIndex: number, state: "running" | "empty" | "locked") => void;
  onParcelDoubleClick: (parcelIndex: number) => void;
}

/**
 * Build the array of 11 Parcel objects from office store + billing state.
 *
 * Fill order: center slots P5,P6,P7 (indices 4,5,6) first,
 * then P2,P4,P8,P10 (indices 1,3,7,9), then corners.
 */
function buildParcels(
  offices: Array<{ id: string; name: string; agentCount: number; skillCount: number; fileCount: number; taskProgress: number }>,
  maxOffices: number,
  hoveredIndex: number | null,
): Parcel[] {
  const parcels: Parcel[] = Array.from({ length: 11 }, (_, i) => ({
    number: i + 1,
    col: 0,
    row: 0,
    state: "locked" as const,
    isHovered: hoveredIndex === i,
  }));

  // Assign running offices by fill order
  for (let slot = 0; slot < offices.length && slot < 11; slot++) {
    const parcelIdx = FILL_ORDER[slot];
    parcels[parcelIdx] = {
      ...parcels[parcelIdx],
      state: "running",
      office: offices[slot],
    };
  }

  // Assign empty slots (unlocked but no office yet)
  const emptyCount = Math.max(0, maxOffices - offices.length);
  for (let slot = offices.length; slot < offices.length + emptyCount && slot < 11; slot++) {
    const parcelIdx = FILL_ORDER[slot];
    parcels[parcelIdx] = {
      ...parcels[parcelIdx],
      state: "empty",
    };
  }

  // Remaining parcels stay locked (already initialized)
  return parcels;
}

/**
 * `ValleyCanvas` renders the Day1 Valley parcel grid using Canvas 2D.
 * Owns the requestAnimationFrame loop and delegates drawing to ValleyRenderer.
 */
export function ValleyCanvas({
  width,
  height,
  onParcelClick,
  onParcelDoubleClick,
}: ValleyCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const frameRef = useRef(0);
  const rafRef = useRef(0);
  const lastTimeRef = useRef(0);
  const hoveredRef = useRef<number | null>(null);

  const tier = useBillingStore((s) => s.tier);
  const offices = useOfficeStore((s) => s.offices);

  const maxOffices = TIER_MAX_OFFICES[tier] ?? 1;

  // Rename overlay state
  const [renameOverlay, setRenameOverlay] = useState<{
    parcelIndex: number;
    officeId: string;
    value: string;
    x: number;
    y: number;
  } | null>(null);

  const renameOffice = useOfficeStore((s) => s.renameOffice);

  // Stash in refs for the render loop
  const widthRef = useRef(width);
  widthRef.current = width;
  const heightRef = useRef(height);
  heightRef.current = height;
  const tierRef = useRef(tier);
  tierRef.current = tier;
  const officesRef = useRef(offices);
  officesRef.current = offices;
  const maxOfficesRef = useRef(maxOffices);
  maxOfficesRef.current = maxOffices;
  const onParcelClickRef = useRef(onParcelClick);
  onParcelClickRef.current = onParcelClick;

  /** Core render tick at ~30 FPS. */
  const tick = useCallback((time: number) => {
    const elapsed = time - lastTimeRef.current;

    if (elapsed >= 33) {
      lastTimeRef.current = time;
      frameRef.current += 1;

      const canvas = canvasRef.current;
      if (!canvas) {
        rafRef.current = requestAnimationFrame(tick);
        return;
      }

      const w = widthRef.current;
      const h = heightRef.current;
      if (w === 0 || h === 0) {
        rafRef.current = requestAnimationFrame(tick);
        return;
      }

      const dpr = window.devicePixelRatio || 1;
      if (
        canvas.width !== Math.floor(w * dpr) ||
        canvas.height !== Math.floor(h * dpr)
      ) {
        canvas.width = Math.floor(w * dpr);
        canvas.height = Math.floor(h * dpr);
      }

      const ctx = canvas.getContext("2d");
      if (!ctx) {
        rafRef.current = requestAnimationFrame(tick);
        return;
      }

      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

      const maxOff = TIER_MAX_OFFICES[tierRef.current] ?? 1;
      const parcels = buildParcels(officesRef.current, maxOff, hoveredRef.current);
      drawCampus(ctx, w, h, parcels, frameRef.current);
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
      const hit = hitTestParcel(px, py, widthRef.current, heightRef.current);
      hoveredRef.current = hit;
      e.currentTarget.style.cursor = hit !== null ? "pointer" : "default";
    },
    [],
  );

  const handleMouseLeave = useCallback(() => {
    hoveredRef.current = null;
  }, []);

  /** Handle single click. */
  const handleClick = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      const rect = e.currentTarget.getBoundingClientRect();
      const px = e.clientX - rect.left;
      const py = e.clientY - rect.top;
      const hit = hitTestParcel(px, py, widthRef.current, heightRef.current);
      if (hit !== null) {
        const maxOff = TIER_MAX_OFFICES[tierRef.current] ?? 1;
        const parcels = buildParcels(officesRef.current, maxOff, null);
        onParcelClickRef.current(hit, parcels[hit].state);
      }
    },
    [],
  );

  /** Handle double-click: open inline rename for running parcels. */
  const handleDoubleClick = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      const rect = e.currentTarget.getBoundingClientRect();
      const px = e.clientX - rect.left;
      const py = e.clientY - rect.top;
      const hit = hitTestParcel(px, py, widthRef.current, heightRef.current);
      if (hit === null) return;

      const maxOff = TIER_MAX_OFFICES[tierRef.current] ?? 1;
      const parcels = buildParcels(officesRef.current, maxOff, null);
      const parcel = parcels[hit];

      if (parcel.state === "running" && parcel.office) {
        // Calculate overlay position using imported constants
        const { col, row } = PARCEL_POSITIONS[hit];
        const PARCEL_W = 160;
        const PARCEL_H = 120;
        const PARCEL_GAP_X = 16;
        const PARCEL_GAP_Y = 16;
        const GRID_COLS = 5;
        const GRID_ROWS = 3;
        const totalW = GRID_COLS * (PARCEL_W + PARCEL_GAP_X) - PARCEL_GAP_X;
        const totalH = GRID_ROWS * (PARCEL_H + PARCEL_GAP_Y) - PARCEL_GAP_Y;
        const startX = (widthRef.current - totalW) / 2;
        const startY = (heightRef.current - totalH) / 2;
        const cardX = rect.left + startX + col * (PARCEL_W + PARCEL_GAP_X);
        const cardY = rect.top + startY + row * (PARCEL_H + PARCEL_GAP_Y);

        setRenameOverlay({
          parcelIndex: hit,
          officeId: parcel.office.id,
          value: parcel.office.name,
          x: cardX,
          y: cardY + 4,
        });
        onParcelDoubleClick(hit);
      }
    },
    [onParcelDoubleClick],
  );

  const handleRenameSubmit = useCallback(() => {
    if (renameOverlay) {
      renameOffice(renameOverlay.officeId, renameOverlay.value.trim() || renameOverlay.value);
      setRenameOverlay(null);
    }
  }, [renameOverlay, renameOffice]);

  const handleRenameKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Enter") handleRenameSubmit();
      if (e.key === "Escape") setRenameOverlay(null);
    },
    [handleRenameSubmit],
  );

  return (
    <div style={{ position: "relative", width, height }}>
      <canvas
        ref={canvasRef}
        style={{ width, height, display: "block" }}
        onMouseMove={handleMouseMove}
        onClick={handleClick}
        onDoubleClick={handleDoubleClick}
        onMouseLeave={handleMouseLeave}
        aria-label="Day1 Valley parcel grid showing agent offices"
      />

      {/* Inline rename overlay */}
      {renameOverlay && (
        <input
          autoFocus
          value={renameOverlay.value}
          onChange={(e) =>
            setRenameOverlay((prev) => prev ? { ...prev, value: e.target.value } : prev)
          }
          onBlur={handleRenameSubmit}
          onKeyDown={handleRenameKeyDown}
          style={{
            position: "fixed",
            left: renameOverlay.x + 8,
            top: renameOverlay.y,
            width: 144,
            background: "#1A1A1A",
            border: "1px solid #F97316",
            borderRadius: 4,
            color: "#E5E5E5",
            font: `bold 11px ui-monospace, "SF Mono", Menlo, monospace`,
            padding: "2px 4px",
            outline: "none",
            zIndex: 1000,
          }}
        />
      )}
    </div>
  );
}
