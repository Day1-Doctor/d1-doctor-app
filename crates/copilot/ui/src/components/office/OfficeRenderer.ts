/**
 * OfficeRenderer — Rich pixel-art Canvas 2D rendering for the isometric office.
 *
 * All drawing is done through the Canvas 2D API. No DOM manipulation,
 * no React dependencies. The module exports a single entry point
 * `drawOffice` plus supporting types and constants.
 *
 * Features:
 * - D1D-221: Error/escalation visual indicators (pulsing triangle + speech bubble)
 * - D1D-224: Themed room zones with rich pixel-art tiles, wall backdrop, furniture
 * - Rich floor zones: wood tiles (work) and cool tiles (rest) with detailed rendering
 * - Wall backdrop with windows, bookshelves, clock, and DAY1 neon sign
 * - Enhanced desks: wood surface, chair, monitor with state-based screen, accessories
 * - Rest zone furniture: sofa, arcade machine, water cooler, plant, whiteboard
 */

import type { Agent } from "../../stores/agentStore";
import { drawPixelCharacter } from "./SpriteRenderer";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface DeskPosition {
  gridX: number;
  gridY: number;
  agentId: string | null;
  label: string;
}

export interface AgentRenderState {
  agent: Agent;
  /** Animation frame counter — incremented each render tick. */
  frame: number;
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/** Number of columns in the floor grid. */
const GRID_COLS = 8;
/** Number of rows in the floor grid. */
const GRID_ROWS = 6;

/** Tile half-width (horizontal radius of the isometric diamond). */
const TILE_W = 48;
/** Tile half-height (vertical radius of the isometric diamond). */
const TILE_H = 24;

/**
 * Default desk layout — six agent stations.
 * All desks are in the WORK ZONE (gridX 0-4), 2-column layout:
 * - Column 1 (gridX=1): Dr. Bob, Quill, Atlas
 * - Column 2 (gridX=3): Scout, Sage, Pixel
 */
export const OFFICE_LAYOUT: DeskPosition[] = [
  { gridX: 1, gridY: 1, agentId: null, label: "Dr. Bob" },  // work zone col 1, row 1
  { gridX: 3, gridY: 1, agentId: null, label: "Scout" },    // work zone col 2, row 1
  { gridX: 1, gridY: 3, agentId: null, label: "Quill" },    // work zone col 1, row 2
  { gridX: 3, gridY: 3, agentId: null, label: "Sage" },     // work zone col 2, row 2
  { gridX: 1, gridY: 5, agentId: null, label: "Atlas" },    // work zone col 1, row 3
  { gridX: 3, gridY: 5, agentId: null, label: "Pixel" },    // work zone col 2, row 3
];

/** Role -> fill color mapping. */
export const AGENT_COLORS: Record<string, string> = {
  orchestrator: "#F97316",
  researcher: "#3B82F6",
  analyst: "#8B5CF6",
  writer: "#10B981",
  coder: "#EC4899",
  operator: "#F59E0B",
};

/** Status -> indicator dot color. */
export const STATUS_COLORS: Record<string, string> = {
  idle: "#6B7280",
  working: "#22C55E",
  thinking: "#3B82F6",
  executing: "#F97316",
  paused: "#F59E0B",
  error: "#EF4444",
};

// ---------------------------------------------------------------------------
// Coordinate helpers
// ---------------------------------------------------------------------------

/**
 * Wall height is now 0 — wall decoration removed entirely.
 * The office canvas is 100% floor + furniture + agents.
 */
let _currentWallHeight = 0;

/**
 * Convert grid (col, row) to screen (x, y) given canvas dimensions.
 * The isometric floor fills the entire canvas (wall height is 0).
 */
function gridToScreen(
  col: number,
  row: number,
  canvasW: number,
  canvasH: number,
): { x: number; y: number } {
  const offsetX = canvasW / 2;
  // No wall — floor area is the full canvas
  const wallH = _currentWallHeight; // always 0
  const floorAreaTop = wallH + 8;
  const floorAreaH = canvasH - floorAreaTop;

  // Total isometric grid height
  const isoGridH = (GRID_COLS + GRID_ROWS) * TILE_H;
  // Center the floor grid vertically in the floor area
  const floorCenterY = floorAreaTop + floorAreaH / 2;
  const offsetY = floorCenterY - isoGridH / 2;

  const x = (col - row) * TILE_W + offsetX;
  const y = (col + row) * TILE_H + offsetY;
  return { x, y };
}

// ---------------------------------------------------------------------------
// Hit testing (for click interaction)
// ---------------------------------------------------------------------------

/**
 * Test whether a screen coordinate (px, py) is within the clickable area
 * of an agent at the given desk position. The hit area covers the agent
 * circle and indicator icons above it.
 */
export function hitTestAgent(
  desk: DeskPosition,
  px: number,
  py: number,
  canvasW: number,
  canvasH: number,
): boolean {
  const { x, y } = gridToScreen(desk.gridX, desk.gridY, canvasW, canvasH);
  const circleY = y - TILE_H * 1.5;
  const hitRadius = 24; // generous click target

  const dx = px - x;
  const dy = py - circleY;
  return dx * dx + dy * dy <= hitRadius * hitRadius;
}

// ---------------------------------------------------------------------------
// Floor tile drawing — rich pixel-art tiles
// ---------------------------------------------------------------------------

/**
 * Draw a wood-patterned tile for the work zone (gridX < 5).
 * Alternates between two wood tones with grain lines.
 */
function drawWoodTile(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  col: number,
  row: number,
): void {
  const isAlt = (col + row) % 2 === 0;
  const baseFill = isAlt ? "#352A1E" : "#3A2F22";

  // Base diamond
  ctx.beginPath();
  ctx.moveTo(cx, cy - TILE_H);
  ctx.lineTo(cx + TILE_W, cy);
  ctx.lineTo(cx, cy + TILE_H);
  ctx.lineTo(cx - TILE_W, cy);
  ctx.closePath();
  ctx.fillStyle = baseFill;
  ctx.fill();

  // Wood grain lines (subtle horizontal stripes clipped to diamond)
  ctx.save();
  ctx.clip();
  ctx.strokeStyle = isAlt ? "#402E22" : "#453426";
  ctx.lineWidth = 0.5;
  for (let gy = cy - TILE_H; gy < cy + TILE_H; gy += 5) {
    ctx.beginPath();
    ctx.moveTo(cx - TILE_W, gy);
    ctx.lineTo(cx + TILE_W, gy);
    ctx.stroke();
  }
  // Occasional knot dot
  if ((col * 3 + row * 7) % 11 === 0) {
    ctx.fillStyle = "#1E1408";
    ctx.beginPath();
    ctx.ellipse(cx - 4 + (col % 3) * 4, cy + (row % 3) - 2, 3, 2, 0.3, 0, Math.PI * 2);
    ctx.fill();
  }
  ctx.restore();

  // Tile outline
  ctx.beginPath();
  ctx.moveTo(cx, cy - TILE_H);
  ctx.lineTo(cx + TILE_W, cy);
  ctx.lineTo(cx, cy + TILE_H);
  ctx.lineTo(cx - TILE_W, cy);
  ctx.closePath();
  ctx.strokeStyle = "#1A1008";
  ctx.lineWidth = 1;
  ctx.stroke();
}

/**
 * Draw a cool-tiled (rest zone) tile at gridX >= 5.
 * Checkered cool palette.
 */
function drawCoolTile(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  col: number,
  row: number,
): void {
  const isAlt = (col + row) % 2 === 0;
  const baseFill = isAlt ? "#1E3038" : "#223440";

  ctx.beginPath();
  ctx.moveTo(cx, cy - TILE_H);
  ctx.lineTo(cx + TILE_W, cy);
  ctx.lineTo(cx, cy + TILE_H);
  ctx.lineTo(cx - TILE_W, cy);
  ctx.closePath();
  ctx.fillStyle = baseFill;
  ctx.fill();

  // Subtle grid crosshatch
  ctx.save();
  ctx.clip();
  ctx.strokeStyle = isAlt ? "#223A48" : "#284050";
  ctx.lineWidth = 0.4;
  // Diagonal lines
  for (let gi = -TILE_W * 2; gi < TILE_W * 2; gi += 7) {
    ctx.beginPath();
    ctx.moveTo(cx + gi, cy - TILE_H);
    ctx.lineTo(cx + gi + TILE_H, cy + TILE_H);
    ctx.stroke();
  }
  ctx.restore();

  ctx.beginPath();
  ctx.moveTo(cx, cy - TILE_H);
  ctx.lineTo(cx + TILE_W, cy);
  ctx.lineTo(cx, cy + TILE_H);
  ctx.lineTo(cx - TILE_W, cy);
  ctx.closePath();
  ctx.strokeStyle = "#0F1E28";
  ctx.lineWidth = 1;
  ctx.stroke();
}

/**
 * Draw the divider carpet strip at gridX = 5 boundary.
 * A slightly raised-looking strip between work and rest zones.
 */
function drawDividerCarpet(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
): void {
  ctx.beginPath();
  ctx.moveTo(cx, cy - TILE_H);
  ctx.lineTo(cx + TILE_W, cy);
  ctx.lineTo(cx, cy + TILE_H);
  ctx.lineTo(cx - TILE_W, cy);
  ctx.closePath();

  // Purple-ish carpet
  ctx.fillStyle = "#2A1E38";
  ctx.fill();

  ctx.save();
  ctx.clip();
  // Carpet texture — small dots
  ctx.fillStyle = "#332244";
  for (let dx2 = -TILE_W + 4; dx2 < TILE_W; dx2 += 6) {
    for (let dy2 = -TILE_H + 4; dy2 < TILE_H; dy2 += 6) {
      ctx.fillRect(cx + dx2 - 1, cy + dy2 - 1, 2, 2);
    }
  }
  ctx.restore();

  ctx.beginPath();
  ctx.moveTo(cx, cy - TILE_H);
  ctx.lineTo(cx + TILE_W, cy);
  ctx.lineTo(cx, cy + TILE_H);
  ctx.lineTo(cx - TILE_W, cy);
  ctx.closePath();
  ctx.strokeStyle = "#1A0E28";
  ctx.lineWidth = 1.5;
  ctx.stroke();
}

/** Draw the entire floor grid with zone-appropriate tiles. */
function drawFloor(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
): void {
  for (let row = 0; row < GRID_ROWS; row++) {
    for (let col = 0; col < GRID_COLS; col++) {
      const { x, y } = gridToScreen(col, row, w, h);
      if (col === 5) {
        drawDividerCarpet(ctx, x, y);
      } else if (col < 5) {
        drawWoodTile(ctx, x, y, col, row);
      } else {
        drawCoolTile(ctx, x, y, col, row);
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Wall backdrop (~100px at top of canvas)
// ---------------------------------------------------------------------------

/**
 * Removed: wall decoration (windows, bookshelves, clock, DAY1 neon).
 * wallHeight is now 0 — office is 100% floor + furniture + agents.
 * This function is kept as a no-op to preserve the call site in drawOffice.
 */
/**
 * Draw 3D isometric room walls — back wall (top-right) and left wall (top-left)
 * forming an L-shape that gives depth to the office view.
 */
function drawWallBackdrop(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  frame: number,
): void {
  // The isometric floor's top-left corner and top-right corner define where walls go.
  // We draw two wall planes behind the floor to create a 3D room effect.

  const offsetX = w / 2;
  const offsetY = h * 0.12; // top area for walls
  const floorTopY = offsetY + TILE_H; // where the floor grid starts

  // Wall height in pixels
  const wallH = h * 0.28;

  // ── BACK WALL (runs along the top-right edge of the isometric floor) ──
  // This wall faces the viewer from the top-right
  const backWallColor = "#1A1E2E";
  const backWallDark = "#141828";
  const backWallLight = "#202640";

  // Back wall polygon: spans the top edge of the floor grid
  // From the top-left corner of grid to the top-right corner, then up by wallH
  const gridTopLeftX = offsetX - (GRID_COLS / 2) * TILE_W;
  const gridTopLeftY = floorTopY;
  const gridTopRightX = offsetX + (GRID_COLS / 2) * TILE_W;
  const gridTopRightY = floorTopY;
  const gridTopCenterX = offsetX;
  const gridTopCenterY = floorTopY - (GRID_COLS / 2) * TILE_H;

  // Right back wall (isometric plane going top-right)
  ctx.fillStyle = backWallColor;
  ctx.beginPath();
  ctx.moveTo(gridTopCenterX, gridTopCenterY);              // top of isometric grid
  ctx.lineTo(gridTopCenterX, gridTopCenterY - wallH);      // up
  ctx.lineTo(gridTopRightX, gridTopRightY - wallH);        // right along top
  ctx.lineTo(gridTopRightX, gridTopRightY);                // down to floor edge
  ctx.closePath();
  ctx.fill();

  // Left back wall (isometric plane going top-left) — slightly darker
  ctx.fillStyle = backWallDark;
  ctx.beginPath();
  ctx.moveTo(gridTopCenterX, gridTopCenterY);
  ctx.lineTo(gridTopCenterX, gridTopCenterY - wallH);
  ctx.lineTo(gridTopLeftX, gridTopLeftY - wallH);
  ctx.lineTo(gridTopLeftX, gridTopLeftY);
  ctx.closePath();
  ctx.fill();

  // ── Wall details ──

  // Baseboard strip on both walls (darker, 4px from floor edge)
  ctx.fillStyle = "#0E1220";
  // Right wall baseboard
  ctx.beginPath();
  ctx.moveTo(gridTopCenterX, gridTopCenterY);
  ctx.lineTo(gridTopCenterX, gridTopCenterY - 6);
  ctx.lineTo(gridTopRightX, gridTopRightY - 6);
  ctx.lineTo(gridTopRightX, gridTopRightY);
  ctx.closePath();
  ctx.fill();
  // Left wall baseboard
  ctx.beginPath();
  ctx.moveTo(gridTopCenterX, gridTopCenterY);
  ctx.lineTo(gridTopCenterX, gridTopCenterY - 6);
  ctx.lineTo(gridTopLeftX, gridTopLeftY - 6);
  ctx.lineTo(gridTopLeftX, gridTopLeftY);
  ctx.closePath();
  ctx.fill();

  // Top trim on both walls (lighter strip at top)
  ctx.fillStyle = backWallLight;
  // Right wall top trim
  ctx.beginPath();
  ctx.moveTo(gridTopCenterX, gridTopCenterY - wallH);
  ctx.lineTo(gridTopCenterX, gridTopCenterY - wallH - 4);
  ctx.lineTo(gridTopRightX, gridTopRightY - wallH - 4);
  ctx.lineTo(gridTopRightX, gridTopRightY - wallH);
  ctx.closePath();
  ctx.fill();
  // Left wall top trim
  ctx.beginPath();
  ctx.moveTo(gridTopCenterX, gridTopCenterY - wallH);
  ctx.lineTo(gridTopCenterX, gridTopCenterY - wallH - 4);
  ctx.lineTo(gridTopLeftX, gridTopLeftY - wallH - 4);
  ctx.lineTo(gridTopLeftX, gridTopLeftY - wallH);
  ctx.closePath();
  ctx.fill();

  // Corner edge line (where two walls meet — vertical line)
  ctx.strokeStyle = "#252A40";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(gridTopCenterX, gridTopCenterY);
  ctx.lineTo(gridTopCenterX, gridTopCenterY - wallH - 4);
  ctx.stroke();

  // ── Windows on the right back wall (2 windows) ──
  const winW = 30;
  const winH = wallH * 0.45;
  for (let i = 0; i < 2; i++) {
    const t = (i + 1) / 3; // position along wall
    const wx = gridTopCenterX + (gridTopRightX - gridTopCenterX) * t;
    const wy = gridTopCenterY + (gridTopRightY - gridTopCenterY) * t;
    const winTop = wy - wallH * 0.7;

    // Window frame
    ctx.fillStyle = "#0A1530";
    ctx.fillRect(wx - winW / 2, winTop, winW, winH);
    ctx.strokeStyle = "#2A3050";
    ctx.lineWidth = 1.5;
    ctx.strokeRect(wx - winW / 2, winTop, winW, winH);

    // Pane cross
    ctx.strokeStyle = "#2A3050";
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(wx, winTop);
    ctx.lineTo(wx, winTop + winH);
    ctx.moveTo(wx - winW / 2, winTop + winH / 2);
    ctx.lineTo(wx + winW / 2, winTop + winH / 2);
    ctx.stroke();

    // Stars in window
    const starSeed = i * 37;
    for (let s = 0; s < 4; s++) {
      const sx = wx - winW / 2 + 4 + ((starSeed + s * 17) % (winW - 8));
      const sy = winTop + 3 + ((starSeed + s * 23) % (winH * 0.4));
      const alpha = 0.3 + 0.4 * Math.sin(frame * 0.03 + s * 1.5);
      ctx.fillStyle = `rgba(255, 255, 255, ${alpha})`;
      ctx.fillRect(sx, sy, 1.5, 1.5);
    }
  }

  // ── "DAY1" text on left wall ──
  ctx.save();
  ctx.font = "bold 14px monospace";
  ctx.fillStyle = "#22C55E";
  ctx.shadowColor = "#22C55E";
  ctx.shadowBlur = 6 + Math.sin(frame * 0.04) * 3;
  const textX = gridTopCenterX + (gridTopLeftX - gridTopCenterX) * 0.5;
  const textY = gridTopCenterY + (gridTopLeftY - gridTopCenterY) * 0.5 - wallH * 0.5;
  ctx.fillText("DAY1", textX - 18, textY);
  ctx.shadowBlur = 0;
  ctx.restore();
}

// ---------------------------------------------------------------------------
// Enhanced desk rendering
// ---------------------------------------------------------------------------

/**
 * Determine the monitor screen state based on agent status.
 * Returns a screen drawing function.
 */
function drawMonitorScreen(
  ctx: CanvasRenderingContext2D,
  mx: number,
  my: number,
  mw: number,
  mh: number,
  status: string,
  frame: number,
): void {
  // Screen background
  let screenBg = "#0A1020";
  if (status === "working" || status === "executing") screenBg = "#071408";
  if (status === "error") screenBg = "#1A0808";
  if (status === "idle") screenBg = "#080810";

  ctx.fillStyle = screenBg;
  ctx.fillRect(mx, my, mw, mh);

  const innerW = mw - 4;
  const innerH = mh - 4;
  const ix = mx + 2;
  const iy = my + 2;

  if (status === "working" || status === "executing") {
    // Green code lines scrolling
    ctx.fillStyle = "#22C55E";
    const lineH = 3;
    const lineSpacing = 5;
    const scrollOffset = (frame * 0.5) % lineSpacing;
    const numLines = Math.floor(innerH / lineSpacing) + 1;
    for (let li = 0; li < numLines; li++) {
      const lineY = iy + li * lineSpacing - scrollOffset;
      if (lineY < iy || lineY > iy + innerH) continue;
      // Varying line lengths for code aesthetic
      const lineLen = (((li * 7 + frame) % 5) + 3) * (innerW / 9);
      ctx.globalAlpha = 0.5 + 0.3 * Math.abs(Math.sin(li * 0.8 + frame * 0.02));
      ctx.fillRect(ix, lineY, Math.min(lineLen, innerW), lineH - 1);
    }
    ctx.globalAlpha = 1;

    // Cursor blink
    if (Math.floor(frame * 0.1) % 2 === 0) {
      ctx.fillStyle = "#22C55E";
      ctx.fillRect(ix + 2, iy + innerH - 4, 3, 3);
    }

    // Green monitor glow
    ctx.shadowColor = "#22C55E";
    ctx.shadowBlur = 8;
    ctx.strokeStyle = "#22C55E40";
    ctx.lineWidth = 1;
    ctx.strokeRect(mx - 1, my - 1, mw + 2, mh + 2);
    ctx.shadowBlur = 0;

  } else if (status === "thinking") {
    // Loading bar
    const barW = innerW - 4;
    const barH = 4;
    const barX = ix + 2;
    const barY = iy + innerH / 2 - barH / 2;

    ctx.fillStyle = "#1A2040";
    ctx.fillRect(barX, barY, barW, barH);

    const progress = (Math.sin(frame * 0.04) * 0.5 + 0.5);
    ctx.fillStyle = "#3B82F6";
    ctx.fillRect(barX, barY, barW * progress, barH);

    // Thinking dots
    ctx.fillStyle = "#3B82F680";
    for (let di = 0; di < 3; di++) {
      const dotAlpha = 0.3 + 0.7 * Math.abs(Math.sin(frame * 0.08 + di * 1.2));
      ctx.fillStyle = `rgba(59, 130, 246, ${dotAlpha})`;
      ctx.fillRect(barX + di * 6 + 2, barY - 8, 3, 3);
    }

  } else if (status === "idle") {
    // Screensaver — bouncing pixel
    const ssX = ix + (Math.abs(Math.sin(frame * 0.03)) * innerW) | 0;
    const ssY = iy + (Math.abs(Math.sin(frame * 0.05 + 1)) * innerH) | 0;
    ctx.fillStyle = "#3B82F640";
    ctx.fillRect(ssX, ssY, 4, 4);
    // Faint grid
    ctx.strokeStyle = "#1A2A4020";
    ctx.lineWidth = 0.5;
    for (let gx2 = ix; gx2 < ix + innerW; gx2 += 5) {
      ctx.beginPath();
      ctx.moveTo(gx2, iy);
      ctx.lineTo(gx2, iy + innerH);
      ctx.stroke();
    }

  } else if (status === "error") {
    // Red X on screen
    ctx.strokeStyle = "#EF4444";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(ix + 2, iy + 2);
    ctx.lineTo(ix + innerW - 2, iy + innerH - 2);
    ctx.moveTo(ix + innerW - 2, iy + 2);
    ctx.lineTo(ix + 2, iy + innerH - 2);
    ctx.stroke();

  } else if (status === "paused") {
    // Green checkmark (waiting for input)
    ctx.strokeStyle = "#22C55E";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(ix + 2, iy + innerH / 2);
    ctx.lineTo(ix + innerW / 2 - 1, iy + innerH - 3);
    ctx.lineTo(ix + innerW - 2, iy + 2);
    ctx.stroke();
  }
}

/**
 * Draw an enhanced isometric desk with wood surface, monitor, chair, and accessories.
 */
function drawEnhancedDesk(
  ctx: CanvasRenderingContext2D,
  desk: DeskPosition,
  w: number,
  h: number,
  state: AgentRenderState | undefined,
  frame: number,
): void {
  const { x, y } = gridToScreen(desk.gridX, desk.gridY, w, h);
  const agentStatus = state?.agent.status ?? "idle";
  const agentRole = state?.agent.role ?? "operator";
  const roleColor = AGENT_COLORS[agentRole] ?? "#6B7280";

  const dw = TILE_W * 0.62;
  const dh = TILE_H * 0.62;

  // ---- Chair behind desk ----
  const chairY = y + dh * 0.5;
  const seatW = dw * 0.55;
  const seatH = dh * 0.4;

  // Chair wheels (small ellipses)
  ctx.fillStyle = "#1A1A1A";
  ctx.beginPath();
  ctx.ellipse(x - seatW * 0.35, chairY + seatH + 4, 3, 1.5, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.beginPath();
  ctx.ellipse(x + seatW * 0.35, chairY + seatH + 4, 3, 1.5, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.beginPath();
  ctx.ellipse(x, chairY + seatH + 5, 3, 1.5, 0, 0, Math.PI * 2);
  ctx.fill();

  // Chair seat
  ctx.beginPath();
  ctx.moveTo(x, chairY - seatH);
  ctx.lineTo(x + seatW, chairY);
  ctx.lineTo(x, chairY + seatH);
  ctx.lineTo(x - seatW, chairY);
  ctx.closePath();
  ctx.fillStyle = roleColor + "55";
  ctx.fill();
  ctx.strokeStyle = roleColor + "88";
  ctx.lineWidth = 0.8;
  ctx.stroke();

  // Chair back
  const backH = dh * 0.7;
  ctx.beginPath();
  ctx.moveTo(x - seatW * 0.6, chairY - seatH * 0.4);
  ctx.lineTo(x - seatW * 0.6, chairY - seatH * 0.4 - backH);
  ctx.lineTo(x + seatW * 0.6, chairY - seatH * 0.4 - backH);
  ctx.lineTo(x + seatW * 0.6, chairY - seatH * 0.4);
  ctx.fillStyle = roleColor + "44";
  ctx.fill();
  ctx.strokeStyle = roleColor + "66";
  ctx.lineWidth = 0.8;
  ctx.stroke();

  // ---- Desk front face (depth) ----
  const frontH = 8;
  ctx.beginPath();
  ctx.moveTo(x + dw, y);
  ctx.lineTo(x + dw, y + frontH);
  ctx.lineTo(x, y + dh + frontH);
  ctx.lineTo(x, y + dh);
  ctx.closePath();
  ctx.fillStyle = "#6A4A30";
  ctx.fill();
  ctx.strokeStyle = "#4A3020";
  ctx.lineWidth = 0.8;
  ctx.stroke();

  // Drawer handles on front face
  ctx.fillStyle = "#C0A060";
  ctx.fillRect(x + dw * 0.3 - 3, y + frontH * 0.3, 6, 2);
  ctx.fillRect(x + dw * 0.3 - 3, y + frontH * 0.6 + 1, 6, 2);

  // Left face for depth
  ctx.beginPath();
  ctx.moveTo(x - dw, y);
  ctx.lineTo(x - dw, y + frontH);
  ctx.lineTo(x, y + dh + frontH);
  ctx.lineTo(x, y + dh);
  ctx.closePath();
  ctx.fillStyle = "#5A3C22";
  ctx.fill();
  ctx.strokeStyle = "#4A3020";
  ctx.lineWidth = 0.8;
  ctx.stroke();

  // ---- Desk surface ----
  ctx.beginPath();
  ctx.moveTo(x, y - dh);
  ctx.lineTo(x + dw, y);
  ctx.lineTo(x, y + dh);
  ctx.lineTo(x - dw, y);
  ctx.closePath();
  ctx.fillStyle = "#9A7A60";
  ctx.fill();
  ctx.strokeStyle = "#7A5A40";
  ctx.lineWidth = 1;
  ctx.stroke();

  // Wood grain on surface
  ctx.save();
  ctx.beginPath();
  ctx.moveTo(x, y - dh);
  ctx.lineTo(x + dw, y);
  ctx.lineTo(x, y + dh);
  ctx.lineTo(x - dw, y);
  ctx.closePath();
  ctx.clip();
  ctx.strokeStyle = "#8A6A5020";
  ctx.lineWidth = 0.5;
  for (let grain = -dw; grain < dw; grain += 6) {
    ctx.beginPath();
    ctx.moveTo(x + grain, y - dh);
    ctx.lineTo(x + grain + dh, y + dh);
    ctx.stroke();
  }
  ctx.restore();

  // ---- Monitor on desk ----
  // Monitor stand
  ctx.fillStyle = "#2A2A2A";
  ctx.fillRect(x - 2, y - dh - 6, 4, 6);
  ctx.fillRect(x - 5, y - dh - 1, 10, 2);

  // Monitor body (isometric-ish rectangle)
  const monW = dw * 0.7;
  const monH = 18;
  const monX = x - monW / 2;
  const monY = y - dh - monH - 6;

  ctx.fillStyle = "#222222";
  ctx.fillRect(monX - 2, monY - 2, monW + 4, monH + 4);
  ctx.strokeStyle = "#333333";
  ctx.lineWidth = 1;
  ctx.strokeRect(monX - 2, monY - 2, monW + 4, monH + 4);

  // Screen content
  drawMonitorScreen(ctx, monX, monY, monW, monH, agentStatus, frame);

  // ---- Keyboard ----
  const kbW = dw * 0.55;
  const kbH = 6;
  const kbX = x - kbW / 2;
  const kbY = y - dh * 0.1 + 2;

  ctx.fillStyle = "#1A1A1A";
  ctx.fillRect(kbX, kbY, kbW, kbH);
  ctx.strokeStyle = "#2A2A2A";
  ctx.lineWidth = 0.5;
  ctx.strokeRect(kbX, kbY, kbW, kbH);

  // Key dots on keyboard
  ctx.fillStyle = "#303030";
  const keyCols = 8;
  const keyRows = 2;
  const keyW = (kbW - 4) / keyCols;
  const keyH = (kbH - 2) / keyRows;
  for (let kr = 0; kr < keyRows; kr++) {
    for (let kc = 0; kc < keyCols; kc++) {
      ctx.fillRect(
        kbX + 2 + kc * keyW,
        kbY + 1 + kr * keyH,
        keyW - 1,
        keyH - 1,
      );
    }
  }

  // ---- Coffee mug ----
  const mugX = x + dw * 0.55;
  const mugY = y - dh * 0.2;
  ctx.fillStyle = "#C0402020";
  ctx.strokeStyle = "#C04020";
  ctx.lineWidth = 0.8;
  // Mug body
  ctx.fillRect(mugX - 3, mugY - 7, 7, 7);
  ctx.strokeRect(mugX - 3, mugY - 7, 7, 7);
  // Handle
  ctx.beginPath();
  ctx.arc(mugX + 4, mugY - 3.5, 3, -Math.PI / 2, Math.PI / 2);
  ctx.stroke();

  // Steam when working
  if (agentStatus === "working" || agentStatus === "executing") {
    ctx.strokeStyle = "#FFFFFF40";
    ctx.lineWidth = 1;
    for (let si = 0; si < 2; si++) {
      const steamPhase = frame * 0.07 + si * 1.2;
      const steamX = mugX - 1 + si * 3;
      ctx.beginPath();
      ctx.moveTo(steamX, mugY - 7);
      ctx.quadraticCurveTo(
        steamX + Math.sin(steamPhase) * 2,
        mugY - 10,
        steamX + Math.sin(steamPhase + 1) * 2,
        mugY - 14,
      );
      ctx.stroke();
    }
  }

  // ---- Desk lamp ----
  const lampX = x - dw * 0.6;
  const lampY = y - dh * 0.15;
  ctx.strokeStyle = "#808080";
  ctx.lineWidth = 1.5;
  ctx.beginPath();
  ctx.moveTo(lampX, lampY);
  ctx.lineTo(lampX, lampY - 10);
  ctx.lineTo(lampX + 6, lampY - 14);
  ctx.stroke();
  // Lamp shade
  ctx.fillStyle = "#D4B060";
  ctx.beginPath();
  ctx.moveTo(lampX + 3, lampY - 14);
  ctx.lineTo(lampX + 10, lampY - 10);
  ctx.lineTo(lampX + 8, lampY - 18);
  ctx.lineTo(lampX + 2, lampY - 18);
  ctx.closePath();
  ctx.fill();

  // Warm lamp glow
  const lampGlow = ctx.createRadialGradient(
    lampX + 7, lampY - 10, 0,
    lampX + 7, lampY - 10, 20,
  );
  lampGlow.addColorStop(0, "rgba(255, 200, 80, 0.12)");
  lampGlow.addColorStop(1, "rgba(255, 200, 80, 0)");
  ctx.fillStyle = lampGlow;
  ctx.fillRect(lampX - 10, lampY - 25, 35, 25);

  // ---- Role-specific item ----
  drawRoleItem(ctx, desk.label, x, y, dw, dh, frame);
}

/**
 * Draw a role-specific decorative item on the desk.
 */
function drawRoleItem(
  ctx: CanvasRenderingContext2D,
  label: string,
  x: number,
  y: number,
  dw: number,
  dh: number,
  _frame: number,
): void {
  const ix = x - dw * 0.2;
  const iy = y - dh * 0.5;

  switch (label) {
    case "Dr. Bob": {
      // Clipboard — orchestrator
      ctx.fillStyle = "#D4C080";
      ctx.fillRect(ix - 6, iy - 12, 10, 13);
      ctx.fillStyle = "#8A6030";
      ctx.fillRect(ix - 4, iy - 14, 6, 4);
      ctx.strokeStyle = "#A08040";
      ctx.lineWidth = 0.4;
      for (let li = 0; li < 3; li++) {
        ctx.beginPath();
        ctx.moveTo(ix - 5, iy - 10 + li * 3);
        ctx.lineTo(ix + 3, iy - 10 + li * 3);
        ctx.stroke();
      }
      break;
    }
    case "Scout": {
      // Globe — researcher
      ctx.strokeStyle = "#3B82F6";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.arc(ix, iy - 6, 6, 0, Math.PI * 2);
      ctx.stroke();
      ctx.beginPath();
      ctx.ellipse(ix, iy - 6, 3, 6, 0, 0, Math.PI * 2);
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(ix - 6, iy - 6);
      ctx.lineTo(ix + 6, iy - 6);
      ctx.stroke();
      break;
    }
    case "Sage": {
      // Bar chart — analyst
      ctx.fillStyle = "#8B5CF6";
      const barHeights2 = [5, 8, 6, 9];
      for (let bi = 0; bi < barHeights2.length; bi++) {
        ctx.fillStyle = `rgba(139, 92, 246, ${0.5 + bi * 0.1})`;
        ctx.fillRect(ix - 7 + bi * 4, iy - barHeights2[bi], 3, barHeights2[bi]);
      }
      ctx.strokeStyle = "#6A3A8A";
      ctx.lineWidth = 0.5;
      ctx.strokeRect(ix - 8, iy - 10, 18, 11);
      break;
    }
    case "Quill": {
      // Ink pen + scroll — writer
      ctx.strokeStyle = "#10B981";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(ix - 4, iy - 2);
      ctx.lineTo(ix + 5, iy - 11);
      ctx.stroke();
      ctx.fillStyle = "#10B98160";
      ctx.beginPath();
      ctx.moveTo(ix + 5, iy - 11);
      ctx.lineTo(ix + 3, iy - 9);
      ctx.lineTo(ix + 7, iy - 9);
      ctx.closePath();
      ctx.fill();
      // Scroll
      ctx.fillStyle = "#D4C080";
      ctx.fillRect(ix - 8, iy - 8, 8, 8);
      ctx.strokeStyle = "#A08040";
      ctx.lineWidth = 0.4;
      ctx.beginPath();
      ctx.moveTo(ix - 7, iy - 6);
      ctx.lineTo(ix - 1, iy - 6);
      ctx.moveTo(ix - 7, iy - 4);
      ctx.lineTo(ix - 1, iy - 4);
      ctx.stroke();
      break;
    }
    case "Pixel": {
      // Second monitor — coder
      const m2X = ix + 10;
      const m2Y = iy - 14;
      ctx.fillStyle = "#111111";
      ctx.fillRect(m2X, m2Y, 14, 10);
      ctx.strokeStyle = "#333333";
      ctx.lineWidth = 0.8;
      ctx.strokeRect(m2X, m2Y, 14, 10);
      ctx.fillStyle = "#EC489920";
      ctx.fillRect(m2X + 1, m2Y + 1, 12, 8);
      // Code lines
      ctx.fillStyle = "#EC4899";
      for (let li = 0; li < 3; li++) {
        ctx.fillRect(m2X + 2, m2Y + 2 + li * 2.5, 4 + (li % 2) * 4, 1);
      }
      ctx.fillStyle = "#555555";
      ctx.fillRect(m2X + 4, m2Y + 10, 6, 2);
      break;
    }
    case "Atlas": {
      // Toolbox — operator
      ctx.fillStyle = "#F59E0B";
      ctx.fillRect(ix - 7, iy - 8, 12, 8);
      ctx.fillStyle = "#D48000";
      ctx.fillRect(ix - 7, iy - 10, 12, 3);
      ctx.fillRect(ix - 4, iy - 11, 6, 2);
      ctx.fillStyle = "#1A1A1A";
      ctx.fillRect(ix - 2, iy - 8, 3, 2);
      break;
    }
    default:
      break;
  }
}

// ---------------------------------------------------------------------------
// Rest zone furniture
// ---------------------------------------------------------------------------

function drawRestZoneFurniture(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  frame: number,
): void {
  // REST ZONE furniture: gridX 5-7 only. No desks or work items.
  // Slightly warmer/darker tint for cozy feel — draw subtle warm overlay
  const restOverlayPositions = [6, 7];
  for (const col of restOverlayPositions) {
    for (let row = 0; row < GRID_ROWS; row++) {
      const { x, y } = gridToScreen(col, row, w, h);
      ctx.save();
      ctx.beginPath();
      ctx.moveTo(x, y - TILE_H);
      ctx.lineTo(x + TILE_W, y);
      ctx.lineTo(x, y + TILE_H);
      ctx.lineTo(x - TILE_W, y);
      ctx.closePath();
      ctx.fillStyle = "rgba(40, 20, 10, 0.15)";
      ctx.fill();
      ctx.restore();
    }
  }

  // ---- Sofa (grid 6, 2) — wider, prominent, with cushions + throw pillow ----
  const sofaPos = gridToScreen(6, 2, w, h);
  const sx = sofaPos.x;
  const sy = sofaPos.y;

  // Sofa back (taller, more prominent)
  ctx.fillStyle = "#2A4060";
  ctx.fillRect(sx - 40, sy - 40, 80, 18);
  ctx.strokeStyle = "#1A3050";
  ctx.lineWidth = 1;
  ctx.strokeRect(sx - 40, sy - 40, 80, 18);

  // Sofa back subtle fabric pattern
  ctx.strokeStyle = "#223658";
  ctx.lineWidth = 0.4;
  for (let gi = -38; gi < 38; gi += 10) {
    ctx.beginPath();
    ctx.moveTo(sx + gi, sy - 40);
    ctx.lineTo(sx + gi, sy - 22);
    ctx.stroke();
  }

  // Sofa seat (wider — 80px isometric width)
  ctx.fillStyle = "#244068";
  ctx.fillRect(sx - 38, sy - 23, 76, 15);
  ctx.strokeStyle = "#1A3050";
  ctx.lineWidth = 1;
  ctx.strokeRect(sx - 38, sy - 23, 76, 15);

  // Cushions on back (3 sections)
  ctx.fillStyle = "#304878";
  ctx.fillRect(sx - 37, sy - 38, 22, 14);
  ctx.fillRect(sx - 12, sy - 38, 22, 14);
  ctx.fillRect(sx + 13, sy - 38, 22, 14);
  // Cushion seam lines
  ctx.strokeStyle = "#1E3860";
  ctx.lineWidth = 0.5;
  for (let ci = 0; ci < 3; ci++) {
    const cx2 = sx - 37 + ci * 25 + 11;
    ctx.beginPath();
    ctx.moveTo(cx2, sy - 38);
    ctx.lineTo(cx2, sy - 24);
    ctx.stroke();
  }

  // Throw pillow (accent color) on seat
  ctx.fillStyle = "#C87040";
  ctx.beginPath();
  ctx.ellipse(sx - 18, sy - 16, 9, 6, 0.3, 0, Math.PI * 2);
  ctx.fill();
  ctx.strokeStyle = "#A05020";
  ctx.lineWidth = 0.5;
  ctx.beginPath();
  ctx.ellipse(sx - 18, sy - 16, 9, 6, 0.3, 0, Math.PI * 2);
  ctx.stroke();
  // Second throw pillow
  ctx.fillStyle = "#5A6090";
  ctx.beginPath();
  ctx.ellipse(sx + 18, sy - 16, 9, 6, -0.3, 0, Math.PI * 2);
  ctx.fill();

  // Armrests (visible)
  ctx.fillStyle = "#1E3858";
  ctx.fillRect(sx - 46, sy - 40, 8, 32);
  ctx.fillRect(sx + 38, sy - 40, 8, 32);

  // Sofa legs
  ctx.fillStyle = "#3A2A1A";
  ctx.fillRect(sx - 42, sy - 7, 5, 8);
  ctx.fillRect(sx + 37, sy - 7, 5, 8);

  // ---- Arcade machine (grid 5, 1) — tall game cabinet against wall ----
  const arcPos = gridToScreen(5, 1, w, h);
  const ax = arcPos.x;
  const ay = arcPos.y;

  // Cabinet body (taller)
  ctx.fillStyle = "#1A1430";
  ctx.fillRect(ax - 17, ay - 62, 34, 62);
  ctx.strokeStyle = "#2A2040";
  ctx.lineWidth = 1.5;
  ctx.strokeRect(ax - 17, ay - 62, 34, 62);

  // Side panel accent stripes
  ctx.fillStyle = "#F97316";
  ctx.fillRect(ax - 17, ay - 62, 3, 62);
  ctx.fillRect(ax + 14, ay - 62, 3, 62);

  // "GAME" marquee text at top with glow
  const marqueeGlow = 0.5 + 0.5 * Math.abs(Math.sin(frame * 0.06));
  ctx.save();
  ctx.shadowColor = "#F97316";
  ctx.shadowBlur = 6 * marqueeGlow;
  ctx.fillStyle = `rgba(249, 115, 22, ${0.8 + 0.2 * marqueeGlow})`;
  ctx.font = 'bold 7px ui-monospace, "SF Mono", Menlo, monospace';
  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  ctx.fillText("GAME", ax, ay - 60);
  ctx.restore();

  // Screen bezel
  ctx.fillStyle = "#0A0818";
  ctx.fillRect(ax - 13, ay - 52, 26, 22);

  // Screen (brighter, larger)
  const arcScreenX = ax - 11;
  const arcScreenY = ay - 50;
  const arcScreenW = 22;
  const arcScreenH = 16;
  ctx.fillStyle = "#050A18";
  ctx.fillRect(arcScreenX, arcScreenY, arcScreenW, arcScreenH);
  ctx.strokeStyle = "#3A3060";
  ctx.lineWidth = 1;
  ctx.strokeRect(arcScreenX, arcScreenY, arcScreenW, arcScreenH);

  // Screen glow
  ctx.save();
  ctx.shadowColor = "#22C55E";
  ctx.shadowBlur = 4;
  // Animated pixel sprite
  ctx.fillStyle = "#22C55E";
  const spriteX = arcScreenX + 3 + ((frame * 0.2) % (arcScreenW - 8)) | 0;
  ctx.fillRect(spriteX, arcScreenY + 9, 5, 5);
  ctx.fillRect(spriteX + 1, arcScreenY + 7, 3, 2);
  // Bullets
  ctx.fillStyle = "#F97316";
  for (let bi = 0; bi < 3; bi++) {
    const bx = arcScreenX + 2 + bi * 7;
    const bphase = (frame * 0.15 + bi * 2) % arcScreenH;
    ctx.fillRect(bx, arcScreenY + (bphase | 0), 2, 4);
  }
  ctx.restore();

  // Control panel (wider)
  ctx.fillStyle = "#241C3C";
  ctx.fillRect(ax - 16, ay - 28, 32, 14);
  // Joystick
  ctx.fillStyle = "#4A4060";
  ctx.fillRect(ax - 12, ay - 27, 6, 9);
  ctx.fillStyle = "#6A6080";
  ctx.beginPath();
  ctx.arc(ax - 9, ay - 27, 4, 0, Math.PI * 2);
  ctx.fill();
  // 3 colored buttons
  const btnColors = ["#EF4444", "#22C55E", "#3B82F6"];
  for (let bi = 0; bi < 3; bi++) {
    ctx.fillStyle = btnColors[bi];
    ctx.beginPath();
    ctx.arc(ax + 2 + bi * 6, ay - 21, 3, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = btnColors[bi] + "50";
    ctx.beginPath();
    ctx.arc(ax + 2 + bi * 6, ay - 21, 5, 0, Math.PI * 2);
    ctx.fill();
  }

  // Coin slot
  ctx.fillStyle = "#333030";
  ctx.fillRect(ax - 6, ay - 14, 12, 3);

  // ---- Pool table (grid 6, 4) — LARGE green felt, 4 grid tiles wide ----
  const poolPos = gridToScreen(6, 4, w, h);
  const ptx = poolPos.x;
  const pty = poolPos.y;

  // Table body outer (wooden rails — dark brown border)
  ctx.fillStyle = "#0A3A0A"; // dark green border
  ctx.beginPath();
  ctx.moveTo(ptx, pty - 34);
  ctx.lineTo(ptx + 50, pty - 9);
  ctx.lineTo(ptx, pty + 16);
  ctx.lineTo(ptx - 50, pty - 9);
  ctx.closePath();
  ctx.fill();

  // Wooden rails
  ctx.strokeStyle = "#6A4A30";
  ctx.lineWidth = 5;
  ctx.beginPath();
  ctx.moveTo(ptx, pty - 34);
  ctx.lineTo(ptx + 50, pty - 9);
  ctx.lineTo(ptx, pty + 16);
  ctx.lineTo(ptx - 50, pty - 9);
  ctx.closePath();
  ctx.stroke();

  // Green felt top
  ctx.fillStyle = "#1A5A1A";
  ctx.beginPath();
  ctx.moveTo(ptx, pty - 30);
  ctx.lineTo(ptx + 44, pty - 7);
  ctx.lineTo(ptx, pty + 12);
  ctx.lineTo(ptx - 44, pty - 7);
  ctx.closePath();
  ctx.fill();

  // Felt texture lines
  ctx.save();
  ctx.beginPath();
  ctx.moveTo(ptx, pty - 30);
  ctx.lineTo(ptx + 44, pty - 7);
  ctx.lineTo(ptx, pty + 12);
  ctx.lineTo(ptx - 44, pty - 7);
  ctx.closePath();
  ctx.clip();
  ctx.strokeStyle = "#1E6A1E";
  ctx.lineWidth = 0.5;
  for (let gi = -44; gi < 44; gi += 7) {
    ctx.beginPath();
    ctx.moveTo(ptx + gi, pty - 30);
    ctx.lineTo(ptx + gi + 18, pty + 12);
    ctx.stroke();
  }
  ctx.restore();

  // Pockets (6 holes: 4 corners + 2 midpoints on long sides)
  ctx.fillStyle = "#050505";
  const pocketPositions = [
    { px: ptx, py: pty - 30 },         // top apex
    { px: ptx + 44, py: pty - 7 },     // right apex
    { px: ptx, py: pty + 12 },          // bottom apex
    { px: ptx - 44, py: pty - 7 },     // left apex
    { px: ptx + 22, py: pty - 19 },    // top-right mid
    { px: ptx - 22, py: pty - 19 },    // top-left mid
  ];
  for (const pp of pocketPositions) {
    ctx.beginPath();
    ctx.arc(pp.px, pp.py, 4, 0, Math.PI * 2);
    ctx.fill();
    // Pocket rim
    ctx.strokeStyle = "#2A1A0A";
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.arc(pp.px, pp.py, 4.5, 0, Math.PI * 2);
    ctx.stroke();
  }

  // Billiard balls (3-4 colored balls)
  const ballColors = ["#F5F5F5", "#F97316", "#3B82F6", "#EF4444", "#8B5CF6"];
  const ballPositions = [
    { bx: ptx - 6, by: pty - 10 },
    { bx: ptx + 10, by: pty - 15 },
    { bx: ptx - 14, by: pty - 5 },
    { bx: ptx + 4, by: pty - 1 },
  ];
  for (let bi = 0; bi < ballPositions.length; bi++) {
    const bp = ballPositions[bi];
    ctx.fillStyle = ballColors[bi % ballColors.length];
    ctx.beginPath();
    ctx.arc(bp.bx, bp.by, 4, 0, Math.PI * 2);
    ctx.fill();
    // Shine dot
    ctx.fillStyle = "rgba(255, 255, 255, 0.55)";
    ctx.beginPath();
    ctx.arc(bp.bx - 1, bp.by - 1, 1.5, 0, Math.PI * 2);
    ctx.fill();
  }

  // Table legs (front two visible)
  ctx.fillStyle = "#6A3A10";
  ctx.fillRect(ptx - 46, pty - 5, 6, 14);
  ctx.fillRect(ptx + 40, pty - 5, 6, 14);

  // ---- Water cooler (grid 7, 3) ----
  const wcPos = gridToScreen(7, 3, w, h);
  const wx2 = wcPos.x;
  const wy2 = wcPos.y;

  // Stand
  ctx.fillStyle = "#303030";
  ctx.fillRect(wx2 - 5, wy2 - 20, 10, 20);
  ctx.fillRect(wx2 - 8, wy2 - 4, 16, 4);

  // Blue bottle
  ctx.fillStyle = "#2060A0";
  ctx.fillRect(wx2 - 7, wy2 - 44, 14, 24);
  ctx.strokeStyle = "#104080";
  ctx.strokeRect(wx2 - 7, wy2 - 44, 14, 24);
  // Bottle cap
  ctx.fillStyle = "#4080C0";
  ctx.fillRect(wx2 - 4, wy2 - 48, 8, 5);

  // Bubbles inside bottle (rising)
  ctx.fillStyle = "#80C0FF80";
  for (let bi = 0; bi < 3; bi++) {
    const bubblePhase = (frame * 0.05 + bi * 1.5) % 20;
    ctx.beginPath();
    ctx.arc(wx2 - 3 + bi * 3, wy2 - 24 - bubblePhase, 1.5, 0, Math.PI * 2);
    ctx.fill();
  }

  // Dispense buttons
  ctx.fillStyle = "#3B82F6";
  ctx.beginPath();
  ctx.arc(wx2 - 3, wy2 - 8, 2.5, 0, Math.PI * 2);
  ctx.fill();
  ctx.fillStyle = "#EF4444";
  ctx.beginPath();
  ctx.arc(wx2 + 3, wy2 - 8, 2.5, 0, Math.PI * 2);
  ctx.fill();

  // ---- Plant (grid 5, 5) ----
  const plantPos = gridToScreen(5, 5, w, h);
  const px2 = plantPos.x;
  const py2 = plantPos.y;

  // Pot
  ctx.fillStyle = "#8A4020";
  ctx.fillRect(px2 - 8, py2 - 14, 16, 12);
  ctx.fillRect(px2 - 10, py2 - 16, 20, 4);
  ctx.fillStyle = "#704010";
  ctx.fillRect(px2 - 8, py2 - 14, 16, 2);

  // Stems
  ctx.strokeStyle = "#1A6020";
  ctx.lineWidth = 1.5;
  const stemSway = Math.sin(frame * 0.02) * 1;
  ctx.beginPath();
  ctx.moveTo(px2, py2 - 14);
  ctx.quadraticCurveTo(px2 + stemSway * 2, py2 - 24, px2, py2 - 34);
  ctx.stroke();
  ctx.beginPath();
  ctx.moveTo(px2, py2 - 18);
  ctx.quadraticCurveTo(px2 - 6 + stemSway, py2 - 26, px2 - 10 + stemSway, py2 - 30);
  ctx.stroke();
  ctx.beginPath();
  ctx.moveTo(px2, py2 - 16);
  ctx.quadraticCurveTo(px2 + 6 + stemSway, py2 - 24, px2 + 10 + stemSway, py2 - 28);
  ctx.stroke();

  // Leaves
  ctx.fillStyle = "#22A040";
  const leafPositions = [
    { lx: px2, ly: py2 - 34, rx: 8, ry: 5, rot: -0.3 },
    { lx: px2 - 10 + stemSway, ly: py2 - 30, rx: 7, ry: 4, rot: -0.8 },
    { lx: px2 + 10 + stemSway, ly: py2 - 28, rx: 7, ry: 4, rot: 0.6 },
  ];
  for (const lp of leafPositions) {
    ctx.save();
    ctx.translate(lp.lx, lp.ly);
    ctx.rotate(lp.rot);
    ctx.beginPath();
    ctx.ellipse(0, -lp.ry, lp.rx, lp.ry, 0, 0, Math.PI * 2);
    ctx.fill();
    ctx.restore();
  }

  // ---- Second plant — small cactus (grid 7, 1) ----
  const cactusPos = gridToScreen(7, 1, w, h);
  const cpx = cactusPos.x;
  const cpy = cactusPos.y;

  // Cactus pot
  ctx.fillStyle = "#9A4020";
  ctx.fillRect(cpx - 6, cpy - 10, 12, 10);
  ctx.fillRect(cpx - 8, cpy - 12, 16, 4);
  ctx.fillStyle = "#6A2C10";
  ctx.fillRect(cpx - 6, cpy - 10, 12, 2);

  // Cactus body (tall trunk)
  ctx.fillStyle = "#2A6A20";
  ctx.fillRect(cpx - 4, cpy - 28, 8, 18);
  ctx.strokeStyle = "#1A5010";
  ctx.lineWidth = 0.5;
  ctx.strokeRect(cpx - 4, cpy - 28, 8, 18);

  // Left arm
  ctx.fillStyle = "#2A6A20";
  ctx.fillRect(cpx - 10, cpy - 24, 7, 5);
  ctx.fillRect(cpx - 10, cpy - 30, 5, 7);

  // Right arm
  ctx.fillRect(cpx + 3, cpy - 22, 7, 5);
  ctx.fillRect(cpx + 5, cpy - 28, 5, 7);

  // Cactus spines
  ctx.strokeStyle = "#80C060";
  ctx.lineWidth = 0.5;
  const spinePositions = [
    { x: cpx - 4, y: cpy - 26 }, { x: cpx + 4, y: cpy - 24 },
    { x: cpx - 4, y: cpy - 20 }, { x: cpx + 4, y: cpy - 18 },
    { x: cpx - 10, y: cpy - 29 }, { x: cpx + 10, y: cpy - 27 },
  ];
  for (const sp of spinePositions) {
    ctx.beginPath();
    ctx.moveTo(sp.x, sp.y);
    ctx.lineTo(sp.x + (sp.x < cpx ? -4 : 4), sp.y - 2);
    ctx.stroke();
  }

  // ---- Snack table (grid 7, 5) ----
  const snackPos = gridToScreen(7, 5, w, h);
  const stx = snackPos.x;
  const sty = snackPos.y;

  // Table surface (small round table)
  ctx.fillStyle = "#5A3A1A";
  ctx.beginPath();
  ctx.ellipse(stx, sty - 12, 16, 8, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.strokeStyle = "#3A2010";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.ellipse(stx, sty - 12, 16, 8, 0, 0, Math.PI * 2);
  ctx.stroke();

  // Table leg
  ctx.fillStyle = "#3A2010";
  ctx.fillRect(stx - 2, sty - 12, 4, 14);
  // Base
  ctx.fillRect(stx - 8, sty, 16, 3);

  // Snacks on table
  // Bowl of snacks
  ctx.fillStyle = "#8A6030";
  ctx.beginPath();
  ctx.ellipse(stx - 5, sty - 15, 6, 3, 0, 0, Math.PI * 2);
  ctx.fill();
  // Snack dots (colorful)
  const snackColors = ["#F97316", "#EF4444", "#22C55E", "#F59E0B"];
  for (let sni = 0; sni < 4; sni++) {
    ctx.fillStyle = snackColors[sni];
    ctx.beginPath();
    ctx.arc(stx - 8 + sni * 3, sty - 16, 1.5, 0, Math.PI * 2);
    ctx.fill();
  }
  // Cup/mug
  ctx.fillStyle = "#3B82F6";
  ctx.fillRect(stx + 4, sty - 19, 7, 7);
  ctx.strokeStyle = "#2060A0";
  ctx.lineWidth = 0.5;
  ctx.strokeRect(stx + 4, sty - 19, 7, 7);
}

// ---------------------------------------------------------------------------
// Room labels — WORK ZONE / REST ZONE
// ---------------------------------------------------------------------------

function drawZoneLabels(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
): void {
  const labelFont = '10px ui-monospace, "SF Mono", Menlo, monospace';
  ctx.font = labelFont;
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";

  const labels = [
    { text: "WORK ZONE", col: 2.5, row: 5.5 },
    { text: "REST ZONE", col: 6.5, row: 5.5 },
  ];

  for (const lbl of labels) {
    const { x, y } = gridToScreen(lbl.col, lbl.row, w, h);

    // Measure text for pill background
    const textW = ctx.measureText(lbl.text).width;
    const pillW = textW + 16;
    const pillH = 16;

    // Semi-transparent pill background
    ctx.fillStyle = "rgba(0, 0, 0, 0.55)";
    const rx = x - pillW / 2;
    const ry = y - pillH / 2;
    const rr = 4;
    ctx.beginPath();
    ctx.moveTo(rx + rr, ry);
    ctx.lineTo(rx + pillW - rr, ry);
    ctx.quadraticCurveTo(rx + pillW, ry, rx + pillW, ry + rr);
    ctx.lineTo(rx + pillW, ry + pillH - rr);
    ctx.quadraticCurveTo(rx + pillW, ry + pillH, rx + pillW - rr, ry + pillH);
    ctx.lineTo(rx + rr, ry + pillH);
    ctx.quadraticCurveTo(rx, ry + pillH, rx, ry + pillH - rr);
    ctx.lineTo(rx, ry + rr);
    ctx.quadraticCurveTo(rx, ry, rx + rr, ry);
    ctx.closePath();
    ctx.fill();

    // Label text
    ctx.fillStyle = lbl.text === "WORK ZONE" ? "#C8A060" : "#60A8C8";
    ctx.fillText(lbl.text, x, y);
  }
}

// ---------------------------------------------------------------------------
// D1D-221: Error/Escalation visual indicators
// ---------------------------------------------------------------------------

/**
 * Draw a red exclamation triangle above the agent for 'error' status.
 * Includes pulsing animation.
 */
function drawErrorIndicator(
  ctx: CanvasRenderingContext2D,
  x: number,
  circleY: number,
  radius: number,
  frame: number,
): void {
  const pulseScale = 0.85 + 0.15 * Math.abs(Math.sin(frame * 0.06));
  const iconY = circleY - radius - 18;

  ctx.save();
  ctx.translate(x, iconY);
  ctx.scale(pulseScale, pulseScale);

  // Triangle
  const triSize = 8;
  ctx.beginPath();
  ctx.moveTo(0, -triSize);
  ctx.lineTo(triSize * 0.87, triSize * 0.5);
  ctx.lineTo(-triSize * 0.87, triSize * 0.5);
  ctx.closePath();

  ctx.fillStyle = "#EF444420";
  ctx.fill();
  ctx.strokeStyle = "#EF4444";
  ctx.lineWidth = 1.5;
  ctx.stroke();

  // Exclamation mark
  ctx.fillStyle = "#EF4444";
  ctx.font = 'bold 8px ui-monospace, "SF Mono", Menlo, monospace';
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText("!", 0, 0);

  ctx.restore();
}

/**
 * Draw a yellow speech bubble icon above the agent for 'paused' (approval needed).
 * Includes pulsing animation.
 */
function drawPausedIndicator(
  ctx: CanvasRenderingContext2D,
  x: number,
  circleY: number,
  radius: number,
  frame: number,
): void {
  const pulseScale = 0.85 + 0.15 * Math.abs(Math.sin(frame * 0.06));
  const iconY = circleY - radius - 18;

  ctx.save();
  ctx.translate(x, iconY);
  ctx.scale(pulseScale, pulseScale);

  // Speech bubble body (rounded rect approximation)
  const bw = 12;
  const bh = 8;
  const br = 2;

  ctx.beginPath();
  // Rounded rectangle
  ctx.moveTo(-bw / 2 + br, -bh / 2);
  ctx.lineTo(bw / 2 - br, -bh / 2);
  ctx.quadraticCurveTo(bw / 2, -bh / 2, bw / 2, -bh / 2 + br);
  ctx.lineTo(bw / 2, bh / 2 - br);
  ctx.quadraticCurveTo(bw / 2, bh / 2, bw / 2 - br, bh / 2);
  // Tail
  ctx.lineTo(2, bh / 2);
  ctx.lineTo(0, bh / 2 + 4);
  ctx.lineTo(-1, bh / 2);
  ctx.lineTo(-bw / 2 + br, bh / 2);
  ctx.quadraticCurveTo(-bw / 2, bh / 2, -bw / 2, bh / 2 - br);
  ctx.lineTo(-bw / 2, -bh / 2 + br);
  ctx.quadraticCurveTo(-bw / 2, -bh / 2, -bw / 2 + br, -bh / 2);
  ctx.closePath();

  ctx.fillStyle = "#F59E0B20";
  ctx.fill();
  ctx.strokeStyle = "#F59E0B";
  ctx.lineWidth = 1;
  ctx.stroke();

  // Pause bars inside bubble
  ctx.fillStyle = "#F59E0B";
  ctx.fillRect(-3, -2.5, 2, 5);
  ctx.fillRect(1.5, -2.5, 2, 5);

  ctx.restore();
}

/** Draw a pixel-art agent character + status indicators + label at a desk. */
function drawAgent(
  ctx: CanvasRenderingContext2D,
  desk: DeskPosition,
  state: AgentRenderState,
  w: number,
  h: number,
): void {
  const { x, y } = gridToScreen(desk.gridX, desk.gridY, w, h);
  const { agent, frame } = state;

  // Pulsing alpha for "thinking"
  let alpha = 1;
  if (agent.status === "thinking") {
    alpha = 0.5 + 0.5 * Math.abs(Math.sin(frame * 0.08));
  }

  // Position the pixel character above the desk
  const charY = y - TILE_H * 0.6;

  ctx.save();
  ctx.globalAlpha = alpha;

  // Draw pixel-art character at 1.3x scale for improved visibility
  drawPixelCharacter(ctx, x, charY, 1.3, agent.role, agent.status, frame);

  ctx.restore();

  // D1D-221: Error indicator — red exclamation triangle (positioned above character)
  if (agent.status === "error") {
    drawErrorIndicator(ctx, x, charY - 28, 10, frame);
  }

  // D1D-221: Paused indicator — yellow speech bubble
  if (agent.status === "paused") {
    drawPausedIndicator(ctx, x, charY - 28, 10, frame);
  }

  // Name label — below desk
  const labelY = y + TILE_H * 0.8;
  const labelX = x;

  ctx.font = '11px ui-monospace, "SF Mono", Menlo, monospace';
  ctx.textAlign = "center";

  const labelW = ctx.measureText(desk.label).width + 8;
  ctx.fillStyle = "rgba(0,0,0,0.5)";
  ctx.fillRect(labelX - labelW / 2, labelY, labelW, 13);

  ctx.fillStyle = "#C0C0C0";
  ctx.textBaseline = "top";
  ctx.fillText(desk.label, labelX, labelY + 1);
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Render the full office scene in a single frame.
 *
 * @param ctx  - Canvas 2D rendering context.
 * @param w    - Canvas pixel width.
 * @param h    - Canvas pixel height.
 * @param desks - Desk layout (positions + label).
 * @param agents - Map of agentId -> AgentRenderState for occupied desks.
 * @param frame - Optional global animation frame counter (for wall animations).
 */
export function drawOffice(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  desks: DeskPosition[],
  agents: Map<string, AgentRenderState>,
  frame?: number,
): void {
  const f = frame ?? 0;

  // Atmospheric background: radial gradient from center to dark edges
  const cx = w / 2;
  const cy = h / 2;
  const bgGrad = ctx.createRadialGradient(cx, cy, 0, cx, cy, Math.max(w, h) * 0.75);
  bgGrad.addColorStop(0, "#151520");
  bgGrad.addColorStop(1, "#080810");
  ctx.fillStyle = bgGrad;
  ctx.fillRect(0, 0, w, h);

  // Wall backdrop — removed (no-op, wallHeight = 0)
  drawWallBackdrop(ctx, w, h, f);

  // Rich floor tiles (wood / divider carpet / cool)
  drawFloor(ctx, w, h);

  // Warm spotlight on the work zone — makes office feel lit and alive
  const spotX = w * 0.35;
  const spotY = h * 0.5;
  const spotGrad = ctx.createRadialGradient(spotX, spotY, 0, spotX, spotY, h * 0.6);
  spotGrad.addColorStop(0, "rgba(255, 200, 100, 0.08)");
  spotGrad.addColorStop(0.4, "rgba(255, 180, 80, 0.04)");
  spotGrad.addColorStop(1, "rgba(0, 0, 0, 0)");
  ctx.fillStyle = spotGrad;
  ctx.fillRect(0, 0, w, h);

  // Rest zone furniture (drawn before desks so desks overlay properly)
  drawRestZoneFurniture(ctx, w, h, f);

  // Enhanced desks with all accessories
  for (const desk of desks) {
    const state = desk.agentId ? agents.get(desk.agentId) : undefined;
    drawEnhancedDesk(ctx, desk, w, h, state, f);
  }

  // Zone labels
  drawZoneLabels(ctx, w, h);

  // Agents at desks (drawn on top of everything)
  for (const desk of desks) {
    if (desk.agentId) {
      const state = agents.get(desk.agentId);
      if (state) {
        drawAgent(ctx, desk, state, w, h);
      }
    }
  }
}
