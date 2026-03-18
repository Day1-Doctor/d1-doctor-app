/**
 * ValleyRenderer — Pure Canvas 2D rendering functions for the Cowork Campus
 * parcel grid view showing 11 office parcels in a diamond pattern.
 *
 * Layout (11 parcels):
 *   Row 0:     [P1]  [P2]  [P3]         (top 3)
 *   Row 1: [P4] [P5] [P6] [P7] [P8]    (middle 5, widest)
 *   Row 2:     [P9]  [P10] [P11]        (bottom 3)
 *
 * Center parcels P5, P6, P7 are priority active slots.
 *
 * Each parcel has one of three states:
 *   - running: dark card with accent glow border + animated mini agents
 *   - empty:   unlocked slot with dotted accent border + "+" icon
 *   - locked:  dark card with padlock icon + "Upgrade plan" text
 */

import { drawMiniPixelCharacter } from "../office/SpriteRenderer";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type ParcelState = "running" | "empty" | "locked";

export interface Office {
  id: string;
  name: string;
  agentCount: number;
  skillCount: number;
  fileCount: number;
  taskProgress: number;
}

export interface Parcel {
  /** 1-based parcel number (P1..P11) */
  number: number;
  /** Screen column in the diamond grid (0-4) */
  col: number;
  /** Screen row in the diamond grid (0-2) */
  row: number;
  state: ParcelState;
  /** Populated when state === "running" */
  office?: Office;
  isHovered: boolean;
}

// ---------------------------------------------------------------------------
// Layout constants
// ---------------------------------------------------------------------------

/**
 * Diamond layout: [col, row] positions for P1..P11 within the 5x3 grid.
 * Row 0: cols 1,2,3  (P1,P2,P3)
 * Row 1: cols 0,1,2,3,4  (P4,P5,P6,P7,P8)
 * Row 2: cols 1,2,3  (P9,P10,P11)
 */
export const PARCEL_POSITIONS: Array<{ col: number; row: number }> = [
  { col: 1, row: 0 }, // P1
  { col: 2, row: 0 }, // P2
  { col: 3, row: 0 }, // P3
  { col: 0, row: 1 }, // P4
  { col: 1, row: 1 }, // P5  (priority)
  { col: 2, row: 1 }, // P6  (priority)
  { col: 3, row: 1 }, // P7  (priority)
  { col: 4, row: 1 }, // P8
  { col: 1, row: 2 }, // P9
  { col: 2, row: 2 }, // P10
  { col: 3, row: 2 }, // P11
];

/**
 * Priority fill order: center first, then ring.
 * Indices are 0-based into PARCEL_POSITIONS (P1=0, P5=4, P6=5, P7=6...).
 */
export const FILL_ORDER = [4, 5, 6, 1, 3, 7, 9, 0, 2, 8, 10];

/** Parcel card dimensions */
const PARCEL_W = 160;
const PARCEL_H = 120;
const PARCEL_GAP_X = 16;
const PARCEL_GAP_Y = 16;
const PARCEL_RADIUS = 8;

/** Colors */
const ACCENT = "#F97316";
const BG_RUNNING = "#0D0D0D";
const BG_EMPTY = "#111115";
const BG_LOCKED = "#0A0A0E";
const BORDER_LOCKED = "#1A1A1E";
const TEXT_WHITE = "#E5E5E5";
const TEXT_LOCKED = "#555555";
const TEXT_UPGRADE = "#444444";
const TEXT_PARCEL_NUM = "#333340";

// ---------------------------------------------------------------------------
// Coordinate helpers
// ---------------------------------------------------------------------------

/** Total grid width in columns (0..4), height in rows (0..2). */
const GRID_COLS = 5;
const GRID_ROWS = 3;

function gridToScreen(
  col: number,
  row: number,
  canvasW: number,
  canvasH: number,
): { x: number; y: number } {
  const totalW = GRID_COLS * (PARCEL_W + PARCEL_GAP_X) - PARCEL_GAP_X;
  const totalH = GRID_ROWS * (PARCEL_H + PARCEL_GAP_Y) - PARCEL_GAP_Y;
  const startX = (canvasW - totalW) / 2;
  const startY = (canvasH - totalH) / 2;
  return {
    x: startX + col * (PARCEL_W + PARCEL_GAP_X),
    y: startY + row * (PARCEL_H + PARCEL_GAP_Y),
  };
}

// ---------------------------------------------------------------------------
// Drawing primitives
// ---------------------------------------------------------------------------

function roundedRect(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  r: number,
): void {
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.lineTo(x + w - r, y);
  ctx.quadraticCurveTo(x + w, y, x + w, y + r);
  ctx.lineTo(x + w, y + h - r);
  ctx.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
  ctx.lineTo(x + r, y + h);
  ctx.quadraticCurveTo(x, y + h, x, y + h - r);
  ctx.lineTo(x, y + r);
  ctx.quadraticCurveTo(x, y, x + r, y);
  ctx.closePath();
}

// ---------------------------------------------------------------------------
// Parcel renderers
// ---------------------------------------------------------------------------

function drawRunningParcel(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  office: Office,
  parcelNum: number,
  isHovered: boolean,
  frame: number,
): void {
  // Glow pulse intensity
  const glowPulse = 0.5 + 0.5 * Math.sin(frame * 0.05);
  const glowBlur = 8 + glowPulse * 8;

  ctx.save();

  // Background
  roundedRect(ctx, x, y, PARCEL_W, PARCEL_H, PARCEL_RADIUS);
  ctx.fillStyle = BG_RUNNING;
  ctx.fill();

  // Accent glow border
  ctx.save();
  ctx.shadowColor = ACCENT;
  ctx.shadowBlur = glowBlur;
  roundedRect(ctx, x, y, PARCEL_W, PARCEL_H, PARCEL_RADIUS);
  ctx.strokeStyle = ACCENT;
  ctx.lineWidth = isHovered ? 2 : 1.5;
  ctx.stroke();
  ctx.restore();

  // ---- Office name (title at top) ----
  ctx.fillStyle = TEXT_WHITE;
  ctx.font = `bold 11px ui-monospace, "SF Mono", Menlo, monospace`;
  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  ctx.fillText(office.name, x + PARCEL_W / 2, y + 8, PARCEL_W - 12);

  // ---- Interior: mini desks (3x2 grid) centered in middle area ----
  // Area for agents: starts at y+36 (after title), ends at y+h-50 (before progress bar)
  const agentAreaTop = y + 36;
  const agentAreaBottom = y + PARCEL_H - 50;
  const agentAreaH = agentAreaBottom - agentAreaTop;

  const deskW = 20;
  const deskH = 11;
  const deskGapX = 8;
  const deskGapY = 6;
  const numCols = 3;
  const numRows = 2;
  const totalDeskW = numCols * deskW + (numCols - 1) * deskGapX;
  const totalDeskH = numRows * (deskH + 14) + (numRows - 1) * deskGapY; // include agent height
  const deskStartX = x + (PARCEL_W - totalDeskW) / 2;
  const deskStartY = agentAreaTop + (agentAreaH - totalDeskH) / 2;

  const agentRoles = ["orchestrator", "researcher", "analyst", "writer", "coder", "operator"];

  for (let row = 0; row < numRows; row++) {
    for (let col = 0; col < numCols; col++) {
      const deskX = deskStartX + col * (deskW + deskGapX);
      const deskY = deskStartY + row * (deskH + 14 + deskGapY);

      // Desk surface (brown rect)
      ctx.fillStyle = "#3D2B1A";
      ctx.fillRect(deskX, deskY, deskW, deskH);
      ctx.strokeStyle = "#5A3E28";
      ctx.lineWidth = 0.5;
      ctx.strokeRect(deskX, deskY, deskW, deskH);

      // Mini monitor on desk
      ctx.fillStyle = "#1A1A1A";
      ctx.fillRect(deskX + 6, deskY - 6, 8, 6);
      ctx.strokeStyle = "#333333";
      ctx.lineWidth = 0.5;
      ctx.strokeRect(deskX + 6, deskY - 6, 8, 6);

      // Monitor screen glow
      const screenAlpha = 0.4 + 0.3 * Math.sin(frame * 0.04 + col + row);
      ctx.fillStyle = `rgba(249, 115, 22, ${screenAlpha})`;
      ctx.fillRect(deskX + 7, deskY - 5, 6, 4);

      // Mini pixel agent at desk (if within agentCount)
      const agentIdx = row * numCols + col;
      if (agentIdx < office.agentCount) {
        const agentX = deskX + deskW / 2;
        // Agent drawn ABOVE the desk, within the card bounds
        const agentY = deskY - 2;
        const roleIndex = agentIdx % agentRoles.length;
        const state = agentIdx === 0 ? "working" : (frame + agentIdx * 17) % 60 < 40 ? "typing" : "idle";
        ctx.save();
        ctx.beginPath();
        ctx.rect(x, y, PARCEL_W, PARCEL_H);
        ctx.clip();
        drawMiniPixelCharacter(ctx, agentX, agentY, agentRoles[roleIndex], state, frame + agentIdx * 13);
        ctx.restore();
      }
    }
  }

  // ---- Progress bar (near bottom) ----
  const barX = x + 10;
  const barY = y + PARCEL_H - 26;
  const barW = PARCEL_W - 20;
  const barH = 5;
  const progress = Math.max(0, Math.min(1, office.taskProgress));

  ctx.fillStyle = "#1A1A1A";
  ctx.beginPath();
  ctx.roundRect(barX, barY, barW, barH, 2);
  ctx.fill();

  if (progress > 0) {
    const grad = ctx.createLinearGradient(barX, 0, barX + barW, 0);
    grad.addColorStop(0, "#22C55E");
    grad.addColorStop(1, ACCENT);
    ctx.fillStyle = grad;
    ctx.beginPath();
    ctx.roundRect(barX, barY, barW * progress, barH, 2);
    ctx.fill();
  }

  // ---- Stats at very bottom ----
  ctx.fillStyle = "#666666";
  ctx.font = `9px ui-monospace, "SF Mono", Menlo, monospace`;
  ctx.textAlign = "center";
  ctx.textBaseline = "bottom";
  const statsText = `${office.agentCount} agents · ${office.skillCount} skills`;
  ctx.fillText(statsText, x + PARCEL_W / 2, y + PARCEL_H - 6, PARCEL_W - 12);

  ctx.restore();

  void parcelNum;
}

function drawEmptyParcel(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  parcelNum: number,
  isHovered: boolean,
  frame: number,
): void {
  ctx.save();

  // Background
  roundedRect(ctx, x, y, PARCEL_W, PARCEL_H, PARCEL_RADIUS);
  ctx.fillStyle = BG_EMPTY;
  ctx.fill();

  // Dotted accent border
  roundedRect(ctx, x, y, PARCEL_W, PARCEL_H, PARCEL_RADIUS);
  ctx.setLineDash([4, 4]);
  ctx.strokeStyle = isHovered ? ACCENT : `rgba(249, 115, 22, ${0.5 + 0.3 * Math.sin(frame * 0.04)})`;
  ctx.lineWidth = 1.5;
  ctx.stroke();
  ctx.setLineDash([]);

  // Large "+" icon
  const cx = x + PARCEL_W / 2;
  const cy = y + PARCEL_H / 2 - 10;
  ctx.strokeStyle = ACCENT;
  ctx.lineWidth = 2;
  ctx.lineCap = "round";
  ctx.beginPath();
  ctx.moveTo(cx - 12, cy);
  ctx.lineTo(cx + 12, cy);
  ctx.moveTo(cx, cy - 12);
  ctx.lineTo(cx, cy + 12);
  ctx.stroke();

  // "Empty Office" text
  ctx.fillStyle = "#AAAAAA";
  ctx.font = `11px ui-monospace, "SF Mono", Menlo, monospace`;
  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  ctx.fillText("Empty Office", cx, cy + 18);

  // "Set up" link text
  ctx.fillStyle = ACCENT;
  ctx.font = `10px ui-monospace, "SF Mono", Menlo, monospace`;
  ctx.fillText("Set up", cx, cy + 32);

  // "Office #N" bottom label
  ctx.fillStyle = "#444455";
  ctx.font = `9px ui-monospace, "SF Mono", Menlo, monospace`;
  ctx.textBaseline = "bottom";
  ctx.fillText(`Office #${parcelNum}`, cx, y + PARCEL_H - 6);

  ctx.restore();
}

function drawLockedParcel(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  parcelNum: number,
): void {
  ctx.save();

  // Background
  roundedRect(ctx, x, y, PARCEL_W, PARCEL_H, PARCEL_RADIUS);
  ctx.fillStyle = BG_LOCKED;
  ctx.fill();

  // Subtle border
  roundedRect(ctx, x, y, PARCEL_W, PARCEL_H, PARCEL_RADIUS);
  ctx.strokeStyle = BORDER_LOCKED;
  ctx.lineWidth = 1;
  ctx.stroke();

  // ---- Padlock icon ----
  const cx = x + PARCEL_W / 2;
  const cy = y + PARCEL_H / 2 - 14;

  // Lock body (rounded rect)
  ctx.fillStyle = "#2A2A2A";
  ctx.strokeStyle = "#3A3A3A";
  ctx.lineWidth = 1;
  const bodyX = cx - 9;
  const bodyY = cy + 6;
  const bodyW = 18;
  const bodyH = 14;
  ctx.beginPath();
  ctx.roundRect(bodyX, bodyY, bodyW, bodyH, 3);
  ctx.fill();
  ctx.stroke();

  // Shackle (U-shaped arc)
  ctx.beginPath();
  ctx.arc(cx, cy + 6, 7, Math.PI, 0);
  ctx.strokeStyle = "#3A3A3A";
  ctx.lineWidth = 3;
  ctx.lineCap = "round";
  ctx.stroke();

  // Keyhole (circle + notch)
  ctx.fillStyle = "#1A1A1A";
  ctx.beginPath();
  ctx.arc(cx, bodyY + 6, 2.5, 0, Math.PI * 2);
  ctx.fill();
  ctx.fillRect(cx - 1, bodyY + 8, 2, 4);

  // "LOCKED" text
  ctx.fillStyle = TEXT_LOCKED;
  ctx.font = `bold 10px ui-monospace, "SF Mono", Menlo, monospace`;
  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  ctx.fillText("LOCKED", cx, cy + 24);

  // "Upgrade plan" text
  ctx.fillStyle = TEXT_UPGRADE;
  ctx.font = `9px ui-monospace, "SF Mono", Menlo, monospace`;
  ctx.fillText("Upgrade plan", cx, cy + 37);

  // "Office #N" bottom label
  ctx.fillStyle = TEXT_PARCEL_NUM;
  ctx.font = `9px ui-monospace, "SF Mono", Menlo, monospace`;
  ctx.textBaseline = "bottom";
  ctx.fillText(`Office #${parcelNum}`, cx, y + PARCEL_H - 6);

  ctx.restore();
}

// ---------------------------------------------------------------------------
// Hit testing
// ---------------------------------------------------------------------------

/**
 * Test if (px, py) hits any parcel. Returns the 0-based parcel index
 * into PARCEL_POSITIONS (same as parcel.number - 1), or null.
 */
export function hitTestParcel(
  px: number,
  py: number,
  canvasW: number,
  canvasH: number,
): number | null {
  for (let i = 0; i < PARCEL_POSITIONS.length; i++) {
    const { col, row } = PARCEL_POSITIONS[i];
    const { x, y } = gridToScreen(col, row, canvasW, canvasH);
    if (px >= x && px <= x + PARCEL_W && py >= y && py <= y + PARCEL_H) {
      return i;
    }
  }
  return null;
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/**
 * Draw the atmospheric campus background: gradient sky, star dots, subtle grid lines.
 */
function drawCampusBackground(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  frame: number,
): void {
  // Gradient background: dark navy top → slightly warmer dark bottom
  const grad = ctx.createLinearGradient(0, 0, 0, h);
  grad.addColorStop(0, "#080812");
  grad.addColorStop(1, "#0A0A18");
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, w, h);

  // Subtle tech grid lines (very low opacity)
  ctx.strokeStyle = "rgba(100, 100, 180, 0.04)";
  ctx.lineWidth = 1;
  const gridSpacing = 40;
  for (let gx = 0; gx < w; gx += gridSpacing) {
    ctx.beginPath();
    ctx.moveTo(gx, 0);
    ctx.lineTo(gx, h);
    ctx.stroke();
  }
  for (let gy = 0; gy < h; gy += gridSpacing) {
    ctx.beginPath();
    ctx.moveTo(0, gy);
    ctx.lineTo(w, gy);
    ctx.stroke();
  }

  // Scattered tiny star dots (seeded positions, animated twinkle)
  // Use a deterministic pattern based on position
  const starCount = Math.floor((w * h) / 4000);
  for (let si = 0; si < starCount; si++) {
    // Pseudo-random positions using a simple LCG
    const sx = ((si * 7919 + 1337) % w);
    const sy = ((si * 6271 + 2749) % h);
    const twinkle = 0.1 + 0.1 * Math.abs(Math.sin(frame * 0.03 + si * 0.7));
    ctx.fillStyle = `rgba(255, 255, 255, ${twinkle})`;
    const size = si % 5 === 0 ? 1.5 : 1;
    ctx.fillRect(sx, sy, size, size);
  }
}

/**
 * Render the full Cowork Campus parcel grid.
 */
export function drawCampus(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  parcels: Parcel[],
  frame: number,
): void {
  // Atmospheric background (gradient + stars + grid)
  drawCampusBackground(ctx, w, h, frame);

  // Draw each parcel
  for (const parcel of parcels) {
    const { col, row } = PARCEL_POSITIONS[parcel.number - 1];
    const { x, y } = gridToScreen(col, row, w, h);

    switch (parcel.state) {
      case "running":
        if (parcel.office) {
          drawRunningParcel(ctx, x, y, parcel.office, parcel.number, parcel.isHovered, frame);
        }
        break;
      case "empty":
        drawEmptyParcel(ctx, x, y, parcel.number, parcel.isHovered, frame);
        break;
      case "locked":
        drawLockedParcel(ctx, x, y, parcel.number);
        break;
    }
  }
}
