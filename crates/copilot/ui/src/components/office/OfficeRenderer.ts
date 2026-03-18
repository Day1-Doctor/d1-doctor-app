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
 * Wall height constant: max 20% of canvas height, capped at 140px.
 * Computed at render time but stored here for coordinate calculations.
 * We use a module-level variable updated each frame.
 */
let _currentWallHeight = 100;

/**
 * Convert grid (col, row) to screen (x, y) given canvas dimensions.
 * The isometric floor is centered in the space BELOW the compact wall.
 */
function gridToScreen(
  col: number,
  row: number,
  canvasW: number,
  canvasH: number,
): { x: number; y: number } {
  const offsetX = canvasW / 2;
  // Wall is compact (20% or 140px max). Floor starts right below wall.
  const wallH = _currentWallHeight;
  const floorAreaTop = wallH + 20; // floor area starts just below wall
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
  const baseFill = isAlt ? "#2A2018" : "#302418";

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
  ctx.strokeStyle = isAlt ? "#352818" : "#3A2C1C";
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
  const baseFill = isAlt ? "#1A2830" : "#1E3038";

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
  ctx.strokeStyle = isAlt ? "#1C2E38" : "#223440";
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

function drawWallBackdrop(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  frame: number,
): void {
  // Compact wall: max 20% of canvas height, capped at 140px
  // Structure: baseboard (3px) + main wall (up to 100px) + trim (3px)
  const wallH = Math.min(h * 0.2, 140);
  _currentWallHeight = wallH;
  const wallTop = 0;
  const wallBottom = wallH;

  // Base wall color
  const wallGrad = ctx.createLinearGradient(0, wallTop, 0, wallBottom);
  wallGrad.addColorStop(0, "#162038");
  wallGrad.addColorStop(1, "#1E2A4A");
  ctx.fillStyle = wallGrad;
  ctx.fillRect(0, wallTop, w, wallH);

  // Top trim strip (3px)
  ctx.fillStyle = "#2A3A5A";
  ctx.fillRect(0, wallTop, w, 3);

  // Baseboard at bottom of wall (3px)
  ctx.fillStyle = "#152040";
  ctx.fillRect(0, wallBottom - 3, w, 3);

  // ---- Windows (smaller, scaled to compact wall) ----
  const winMargin = 16;
  const winY = wallTop + winMargin;
  const winW = Math.min(36, w * 0.06);
  const winH = wallH - winMargin * 2;
  const winPositions = [w * 0.2, w * 0.5, w * 0.8];

  for (let wi = 0; wi < winPositions.length; wi++) {
    const wx = winPositions[wi] - winW / 2;

    // Window frame outer
    ctx.fillStyle = "#243458";
    ctx.fillRect(wx - 3, winY - 3, winW + 6, winH + 6);

    // Dark glass
    ctx.fillStyle = "#0A1530";
    ctx.fillRect(wx, winY, winW, winH);

    // Window pane cross
    ctx.strokeStyle = "#243458";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(wx + winW / 2, winY);
    ctx.lineTo(wx + winW / 2, winY + winH);
    ctx.moveTo(wx, winY + winH / 2);
    ctx.lineTo(wx + winW, winY + winH / 2);
    ctx.stroke();

    // Twinkling stars (alpha oscillation per star, per window)
    const starPositions = [
      { sx: 0.15, sy: 0.2 },
      { sx: 0.7, sy: 0.15 },
      { sx: 0.35, sy: 0.55 },
      { sx: 0.8, sy: 0.6 },
      { sx: 0.5, sy: 0.35 },
      { sx: 0.2, sy: 0.75 },
      { sx: 0.65, sy: 0.8 },
    ];
    for (let si = 0; si < starPositions.length; si++) {
      const sp = starPositions[si];
      const twinkle = 0.3 + 0.7 * Math.abs(Math.sin(frame * 0.05 + wi * 2.1 + si * 1.3));
      ctx.fillStyle = `rgba(200, 220, 255, ${twinkle})`;
      const starX = wx + sp.sx * winW;
      const starY = winY + sp.sy * winH;
      const starSize = si % 3 === 0 ? 1.5 : 1;
      ctx.fillRect(starX - starSize / 2, starY - starSize / 2, starSize, starSize);
    }

    // Center moon in middle window
    if (wi === 1) {
      const moonX = wx + winW * 0.7;
      const moonY = winY + winH * 0.3;
      ctx.fillStyle = "#D4C880";
      ctx.beginPath();
      ctx.arc(moonX, moonY, 7, 0, Math.PI * 2);
      ctx.fill();
      // Moon shadow (crescent)
      ctx.fillStyle = "#0A1530";
      ctx.beginPath();
      ctx.arc(moonX + 3, moonY - 2, 6, 0, Math.PI * 2);
      ctx.fill();
    }

    // Window sill
    ctx.fillStyle = "#2E4060";
    ctx.fillRect(wx - 5, winY + winH, winW + 10, 4);
  }

  // ---- Bookshelves (scaled to compact wall) ----
  const shelfW = Math.min(24, w * 0.04);
  const shelfPositions = [w * 0.07, w * 0.93 - shelfW];
  for (let si = 0; si < shelfPositions.length; si++) {
    const sx = shelfPositions[si];
    const sy = wallTop + 4;
    const sw = shelfW;
    const sh = wallH - 8;

    // Shelf wood frame
    ctx.fillStyle = "#6A5030";
    ctx.fillRect(sx, sy, sw, sh);
    ctx.fillStyle = "#4A3018";
    ctx.fillRect(sx, sy, sw, 3); // top
    ctx.fillRect(sx, sy + sh - 3, sw, 3); // bottom
    ctx.fillRect(sx, sy, 3, sh); // left side
    ctx.fillRect(sx + sw - 3, sy, 3, sh); // right side

    // Shelf dividers
    const numShelves = 3;
    const shelfH = (sh - 6) / numShelves;
    for (let shelf = 0; shelf < numShelves; shelf++) {
      const shelfY = sy + 3 + shelf * shelfH;
      ctx.fillStyle = "#5A4020";
      ctx.fillRect(sx + 3, shelfY + shelfH - 3, sw - 6, 3);

      // Books on each shelf
      const bookColors = [
        ["#3B82F6", "#EC4899", "#10B981"],
        ["#F97316", "#8B5CF6", "#F59E0B"],
        ["#EF4444", "#22C55E", "#3B82F6"],
      ];
      const shelfBooks = bookColors[shelf % bookColors.length];
      const bookW = Math.floor((sw - 8) / shelfBooks.length);
      for (let bi = 0; bi < shelfBooks.length; bi++) {
        const bx = sx + 4 + bi * (bookW + 1);
        const bh = 8 + (bi % 2) * 3;
        const by2 = shelfY + shelfH - 6 - bh;
        ctx.fillStyle = shelfBooks[bi] + "CC";
        ctx.fillRect(bx, by2, bookW, bh);
        // Book spine line
        ctx.fillStyle = "#00000040";
        ctx.fillRect(bx, by2, 1, bh);
      }
    }
  }

  // ---- Clock (smaller, in compact wall) ----
  const clockX = w * 0.5 + (winW / 2) + 22;
  const clockY = wallTop + wallH * 0.45;
  const clockR = Math.min(10, wallH * 0.25);

  ctx.fillStyle = "#1A2848";
  ctx.beginPath();
  ctx.arc(clockX, clockY, clockR + 2, 0, Math.PI * 2);
  ctx.fill();
  ctx.strokeStyle = "#4A6090";
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.arc(clockX, clockY, clockR + 2, 0, Math.PI * 2);
  ctx.stroke();

  ctx.fillStyle = "#0E1830";
  ctx.beginPath();
  ctx.arc(clockX, clockY, clockR, 0, Math.PI * 2);
  ctx.fill();

  // Clock tick marks
  ctx.strokeStyle = "#3A5080";
  ctx.lineWidth = 1;
  for (let tick = 0; tick < 12; tick++) {
    const angle = (tick / 12) * Math.PI * 2 - Math.PI / 2;
    const isHour = tick % 3 === 0;
    const innerR = isHour ? clockR - 4 : clockR - 2;
    ctx.beginPath();
    ctx.moveTo(clockX + Math.cos(angle) * innerR, clockY + Math.sin(angle) * innerR);
    ctx.lineTo(clockX + Math.cos(angle) * (clockR - 1), clockY + Math.sin(angle) * (clockR - 1));
    ctx.stroke();
  }

  // Clock hands (animated)
  const totalSeconds = frame * 0.5; // simulate time passage
  const minuteAngle = (totalSeconds / 60) * Math.PI * 2 - Math.PI / 2;
  const hourAngle = (totalSeconds / 720) * Math.PI * 2 - Math.PI / 2;

  // Hour hand
  ctx.strokeStyle = "#8AA0C0";
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(clockX, clockY);
  ctx.lineTo(clockX + Math.cos(hourAngle) * (clockR * 0.55), clockY + Math.sin(hourAngle) * (clockR * 0.55));
  ctx.stroke();

  // Minute hand
  ctx.strokeStyle = "#C0D0E0";
  ctx.lineWidth = 1.5;
  ctx.beginPath();
  ctx.moveTo(clockX, clockY);
  ctx.lineTo(clockX + Math.cos(minuteAngle) * (clockR * 0.8), clockY + Math.sin(minuteAngle) * (clockR * 0.8));
  ctx.stroke();

  // Center dot
  ctx.fillStyle = "#E0E8F0";
  ctx.beginPath();
  ctx.arc(clockX, clockY, 1.5, 0, Math.PI * 2);
  ctx.fill();

  // ---- DAY1 Neon Text (centered in compact wall) ----
  const neonX = w * 0.5;
  const neonY = wallTop + wallH * 0.55;
  const neonGlow = 0.6 + 0.4 * Math.abs(Math.sin(frame * 0.04));
  const neonText = "DAY1";

  ctx.save();
  ctx.font = 'bold 14px ui-monospace, "SF Mono", Menlo, monospace';
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";

  // Outer glow layers
  ctx.shadowColor = "#22C55E";
  ctx.shadowBlur = 16 * neonGlow;
  ctx.fillStyle = `rgba(34, 197, 94, ${0.3 * neonGlow})`;
  ctx.fillText(neonText, neonX, neonY);

  ctx.shadowBlur = 8 * neonGlow;
  ctx.fillStyle = `rgba(34, 197, 94, ${0.6 * neonGlow})`;
  ctx.fillText(neonText, neonX, neonY);

  // Core neon text
  ctx.shadowBlur = 4;
  ctx.fillStyle = `rgba(134, 255, 134, ${0.8 + 0.2 * neonGlow})`;
  ctx.fillText(neonText, neonX, neonY);
  ctx.restore();

  // ---- Whiteboard on wall (work zone side, left of center) ----
  const wbX = w * 0.32;
  const wbW2 = Math.min(48, wallH * 1.5);
  const wbH2 = Math.min(wallH * 0.55, 60);
  const wbY = wallTop + (wallH - wbH2) / 2;

  // Frame
  ctx.fillStyle = "#404040";
  ctx.fillRect(wbX - wbW2 / 2 - 2, wbY - 2, wbW2 + 4, wbH2 + 4);
  // Surface
  ctx.fillStyle = "#E8E8E0";
  ctx.fillRect(wbX - wbW2 / 2, wbY, wbW2, wbH2);

  // Scribble lines
  ctx.strokeStyle = "#1A3A8060";
  ctx.lineWidth = 1;
  const wbScribbles = [
    { x1: 4, y1: 6, x2: wbW2 * 0.6, y2: 6 },
    { x1: 4, y1: 12, x2: wbW2 * 0.8, y2: 12 },
    { x1: 4, y1: 18, x2: wbW2 * 0.5, y2: 18 },
    { x1: 4, y1: 24, x2: wbW2 * 0.7, y2: 24 },
  ];
  for (const sc of wbScribbles) {
    if (sc.y1 < wbH2 - 4) {
      ctx.beginPath();
      ctx.moveTo(wbX - wbW2 / 2 + sc.x1, wbY + sc.y1);
      ctx.lineTo(wbX - wbW2 / 2 + sc.x2, wbY + sc.y2);
      ctx.stroke();
    }
  }

  // "v3.0" text on whiteboard
  ctx.fillStyle = "#1A3A80";
  ctx.font = 'bold 7px ui-monospace, "SF Mono", Menlo, monospace';
  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  ctx.fillText("v3.0", wbX + wbW2 * 0.2, wbY + wbH2 - 12);
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

  // ---- Sofa (grid 6, 2) — large, comfortable, with cushions and throw pillows ----
  const sofaPos = gridToScreen(6, 2, w, h);
  const sx = sofaPos.x;
  const sy = sofaPos.y;

  // Sofa back (taller, more comfortable)
  ctx.fillStyle = "#2A4060";
  ctx.fillRect(sx - 28, sy - 34, 56, 16);
  ctx.strokeStyle = "#1A3050";
  ctx.lineWidth = 1;
  ctx.strokeRect(sx - 28, sy - 34, 56, 16);

  // Sofa seat (wider)
  ctx.fillStyle = "#244068";
  ctx.fillRect(sx - 26, sy - 19, 52, 12);
  ctx.strokeStyle = "#1A3050";
  ctx.strokeRect(sx - 26, sy - 19, 52, 12);

  // Cushions on back (3 cushions)
  ctx.fillStyle = "#304878";
  ctx.fillRect(sx - 25, sy - 32, 15, 12);
  ctx.fillRect(sx - 8, sy - 32, 15, 12);
  ctx.fillRect(sx + 9, sy - 32, 15, 12);
  // Cushion seam lines
  ctx.strokeStyle = "#1E3860";
  ctx.lineWidth = 0.5;
  for (let ci = 0; ci < 3; ci++) {
    const cx2 = sx - 25 + ci * 17 + 7;
    ctx.beginPath();
    ctx.moveTo(cx2, sy - 32);
    ctx.lineTo(cx2, sy - 20);
    ctx.stroke();
  }

  // Throw pillows on seat
  ctx.fillStyle = "#5A6090";
  ctx.beginPath();
  ctx.ellipse(sx - 14, sy - 14, 7, 5, 0.2, 0, Math.PI * 2);
  ctx.fill();
  ctx.fillStyle = "#6A7090";
  ctx.beginPath();
  ctx.ellipse(sx + 14, sy - 14, 7, 5, -0.2, 0, Math.PI * 2);
  ctx.fill();

  // Armrests
  ctx.fillStyle = "#1E3858";
  ctx.fillRect(sx - 34, sy - 34, 7, 26);
  ctx.fillRect(sx + 27, sy - 34, 7, 26);

  // Sofa legs
  ctx.fillStyle = "#3A2A1A";
  ctx.fillRect(sx - 30, sy - 7, 4, 7);
  ctx.fillRect(sx + 26, sy - 7, 4, 7);

  // ---- Arcade machine (grid 5, 1) — game cabinet ----
  const arcPos = gridToScreen(5, 1, w, h);
  const ax = arcPos.x;
  const ay = arcPos.y;

  // Cabinet body
  ctx.fillStyle = "#1A1430";
  ctx.fillRect(ax - 14, ay - 48, 28, 48);
  ctx.strokeStyle = "#2A2040";
  ctx.lineWidth = 1;
  ctx.strokeRect(ax - 14, ay - 48, 28, 48);

  // Side panel accent stripes
  ctx.fillStyle = "#F97316";
  ctx.fillRect(ax - 14, ay - 48, 2, 48);
  ctx.fillRect(ax + 12, ay - 48, 2, 48);

  // Screen bezel
  ctx.fillStyle = "#0A0818";
  ctx.fillRect(ax - 11, ay - 47, 22, 20);

  // Screen
  const arcScreenX = ax - 10;
  const arcScreenY = ay - 46;
  const arcScreenW = 20;
  const arcScreenH = 14;
  ctx.fillStyle = "#050A18";
  ctx.fillRect(arcScreenX, arcScreenY, arcScreenW, arcScreenH);
  ctx.strokeStyle = "#3A3060";
  ctx.strokeRect(arcScreenX, arcScreenY, arcScreenW, arcScreenH);

  // Screen content (simple game graphics)
  ctx.fillStyle = "#22C55E";
  const spriteX = arcScreenX + 4 + ((frame * 0.2) % (arcScreenW - 8)) | 0;
  ctx.fillRect(spriteX, arcScreenY + 8, 4, 4);
  ctx.fillRect(spriteX + 1, arcScreenY + 6, 2, 2);
  // Bullets / score items
  ctx.fillStyle = "#F97316";
  for (let bi = 0; bi < 3; bi++) {
    const bx = arcScreenX + 2 + bi * 7;
    const bphase = (frame * 0.15 + bi * 2) % arcScreenH;
    ctx.fillRect(bx, arcScreenY + (bphase | 0), 2, 3);
  }

  // Control panel
  ctx.fillStyle = "#241C3C";
  ctx.fillRect(ax - 14, ay - 26, 28, 12);
  // Buttons
  const btnColors = ["#EF4444", "#22C55E", "#3B82F6", "#F59E0B"];
  for (let bi = 0; bi < 4; bi++) {
    ctx.fillStyle = btnColors[bi];
    ctx.beginPath();
    ctx.arc(ax - 7 + bi * 5, ay - 20, 2.5, 0, Math.PI * 2);
    ctx.fill();
    // Button glow
    ctx.fillStyle = btnColors[bi] + "40";
    ctx.beginPath();
    ctx.arc(ax - 7 + bi * 5, ay - 20, 4, 0, Math.PI * 2);
    ctx.fill();
  }
  // Joystick
  ctx.fillStyle = "#4A4060";
  ctx.fillRect(ax + 5, ay - 25, 5, 8);
  ctx.fillStyle = "#6A6080";
  ctx.beginPath();
  ctx.arc(ax + 7, ay - 25, 3.5, 0, Math.PI * 2);
  ctx.fill();

  // Coin slot
  ctx.fillStyle = "#333030";
  ctx.fillRect(ax - 4, ay - 14, 8, 2);

  // ---- Pool table (grid 6, 4) — green felt ----
  const poolPos = gridToScreen(6, 4, w, h);
  const ptx = poolPos.x;
  const pty = poolPos.y;

  // Table body (isometric-ish)
  // Table top
  ctx.fillStyle = "#1A6A1A";  // green felt
  ctx.beginPath();
  ctx.moveTo(ptx, pty - 22);
  ctx.lineTo(ptx + 32, pty - 6);
  ctx.lineTo(ptx, pty + 10);
  ctx.lineTo(ptx - 32, pty - 6);
  ctx.closePath();
  ctx.fill();

  // Felt texture — subtle lines
  ctx.strokeStyle = "#1E7A1E";
  ctx.lineWidth = 0.5;
  ctx.save();
  ctx.clip();
  for (let gi = -30; gi < 30; gi += 6) {
    ctx.beginPath();
    ctx.moveTo(ptx + gi, pty - 22);
    ctx.lineTo(ptx + gi + 12, pty + 10);
    ctx.stroke();
  }
  ctx.restore();

  // Table border (brown rail)
  ctx.strokeStyle = "#8A5020";
  ctx.lineWidth = 3;
  ctx.beginPath();
  ctx.moveTo(ptx, pty - 22);
  ctx.lineTo(ptx + 32, pty - 6);
  ctx.lineTo(ptx, pty + 10);
  ctx.lineTo(ptx - 32, pty - 6);
  ctx.closePath();
  ctx.stroke();

  // Pockets (corner circles)
  ctx.fillStyle = "#0A0A0A";
  const pocketPositions = [
    { px: ptx, py: pty - 22 },       // top
    { px: ptx + 32, py: pty - 6 },   // right
    { px: ptx, py: pty + 10 },        // bottom
    { px: ptx - 32, py: pty - 6 },   // left
    { px: ptx + 16, py: pty - 14 },  // top-right mid
    { px: ptx - 16, py: pty - 14 },  // top-left mid
  ];
  for (const pp of pocketPositions) {
    ctx.beginPath();
    ctx.arc(pp.px, pp.py, 3, 0, Math.PI * 2);
    ctx.fill();
  }

  // Billiard balls
  const ballColors = ["#F5F5F5", "#F97316", "#3B82F6", "#EF4444", "#8B5CF6"];
  const ballPositions = [
    { bx: ptx - 5, by: pty - 8 },
    { bx: ptx + 8, by: pty - 12 },
    { bx: ptx - 12, by: pty - 4 },
    { bx: ptx + 3, by: pty - 2 },
    { bx: ptx - 3, by: pty - 16 },
  ];
  for (let bi = 0; bi < ballPositions.length; bi++) {
    const bp = ballPositions[bi];
    ctx.fillStyle = ballColors[bi % ballColors.length];
    ctx.beginPath();
    ctx.arc(bp.bx, bp.by, 3, 0, Math.PI * 2);
    ctx.fill();
    // Shine dot
    ctx.fillStyle = "rgba(255, 255, 255, 0.5)";
    ctx.beginPath();
    ctx.arc(bp.bx - 0.8, bp.by - 0.8, 1, 0, Math.PI * 2);
    ctx.fill();
  }

  // Table legs (front two visible)
  ctx.fillStyle = "#6A3A10";
  ctx.fillRect(ptx - 30, pty - 4, 5, 12);
  ctx.fillRect(ptx + 25, pty - 4, 5, 12);

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

  // Draw pixel-art character (replaces circle-based agent)
  drawPixelCharacter(ctx, x, charY, 1.0, agent.role, agent.status, frame);

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

  // Background
  ctx.fillStyle = "#050508";
  ctx.fillRect(0, 0, w, h);

  // Wall backdrop with windows, bookshelves, clock, neon sign
  drawWallBackdrop(ctx, w, h, f);

  // Rich floor tiles (wood / divider carpet / cool)
  drawFloor(ctx, w, h);

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
