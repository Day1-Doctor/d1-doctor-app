/**
 * OfficeRenderer — Pure Canvas 2D rendering functions for the isometric office.
 *
 * All drawing is done through the Canvas 2D API. No DOM manipulation,
 * no React dependencies. The module exports a single entry point
 * `drawOffice` plus supporting types and constants.
 *
 * Features:
 * - D1D-221: Error/escalation visual indicators (pulsing triangle + speech bubble)
 * - D1D-224: Themed room zones with tinted tiles, labels, and furniture decorations
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

/** Default desk layout — six agent stations. */
export const OFFICE_LAYOUT: DeskPosition[] = [
  { gridX: 3, gridY: 2, agentId: null, label: "Dr. Bob" },
  { gridX: 1, gridY: 1, agentId: null, label: "Scout" },
  { gridX: 5, gridY: 1, agentId: null, label: "Sage" },
  { gridX: 1, gridY: 4, agentId: null, label: "Quill" },
  { gridX: 5, gridY: 4, agentId: null, label: "Pixel" },
  { gridX: 3, gridY: 5, agentId: null, label: "Atlas" },
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

const FLOOR_LINE_COLOR = "#1A1A1A";
const FLOOR_FILL_COLOR = "#0A0A0A";
const DESK_FILL = "#1F1F1F";
const DESK_BORDER = "#242424";
const MONITOR_FILL = "#111111";
const MONITOR_BORDER = "#333333";
const LABEL_COLOR = "#A0A0A0";

// ---------------------------------------------------------------------------
// D1D-224: Room tileset definitions
// ---------------------------------------------------------------------------

interface RoomZone {
  name: string;
  /** Grid column range [start, end) */
  colRange: [number, number];
  /** Grid row range [start, end) */
  rowRange: [number, number];
  /** Subtle tint color (RGBA) */
  tint: string;
  /** Label position offset from room center. */
  labelOffset: { dx: number; dy: number };
}

const ROOM_ZONES: RoomZone[] = [
  {
    name: "Research Room",
    colRange: [0, 3],
    rowRange: [0, 3],
    tint: "rgba(59, 130, 246, 0.04)", // blue
    labelOffset: { dx: 0, dy: -8 },
  },
  {
    name: "Analysis Station",
    colRange: [5, 8],
    rowRange: [0, 3],
    tint: "rgba(139, 92, 246, 0.04)", // purple
    labelOffset: { dx: 0, dy: -8 },
  },
  {
    name: "Writing Desk",
    colRange: [0, 3],
    rowRange: [3, 6],
    tint: "rgba(16, 185, 129, 0.04)", // green
    labelOffset: { dx: 0, dy: -8 },
  },
  {
    name: "Coding Room",
    colRange: [5, 8],
    rowRange: [3, 6],
    tint: "rgba(236, 72, 153, 0.04)", // pink
    labelOffset: { dx: 0, dy: -8 },
  },
  {
    name: "Operations",
    colRange: [2, 6],
    rowRange: [4, 6],
    tint: "rgba(245, 158, 11, 0.03)", // amber
    labelOffset: { dx: 0, dy: -8 },
  },
  {
    name: "Manager",
    colRange: [2, 6],
    rowRange: [1, 4],
    tint: "rgba(249, 115, 22, 0.03)", // orange
    labelOffset: { dx: 0, dy: -8 },
  },
];

// ---------------------------------------------------------------------------
// Coordinate helpers
// ---------------------------------------------------------------------------

/**
 * Convert grid (col, row) to screen (x, y) given canvas dimensions.
 * The grid is centered in the canvas.
 */
function gridToScreen(
  col: number,
  row: number,
  canvasW: number,
  canvasH: number,
): { x: number; y: number } {
  const offsetX = canvasW / 2;
  // Vertical centre, shifted up slightly so the grid doesn't sit on the bottom
  const offsetY = canvasH / 2 - ((GRID_ROWS * TILE_H) / 2);

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
// Drawing primitives
// ---------------------------------------------------------------------------

/** Draw one isometric diamond tile outline. */
function drawTile(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  fill: boolean,
  tintColor?: string,
): void {
  ctx.beginPath();
  ctx.moveTo(cx, cy - TILE_H);
  ctx.lineTo(cx + TILE_W, cy);
  ctx.lineTo(cx, cy + TILE_H);
  ctx.lineTo(cx - TILE_W, cy);
  ctx.closePath();

  if (fill) {
    ctx.fillStyle = FLOOR_FILL_COLOR;
    ctx.fill();
  }

  // Apply room tint overlay
  if (tintColor) {
    ctx.fillStyle = tintColor;
    ctx.fill();
  }

  ctx.strokeStyle = FLOOR_LINE_COLOR;
  ctx.lineWidth = 1;
  ctx.stroke();
}

/** Get the room tint for a tile at (col, row). Returns the last matching zone's tint. */
function getRoomTint(col: number, row: number): string | undefined {
  let tint: string | undefined;
  for (const zone of ROOM_ZONES) {
    if (
      col >= zone.colRange[0] &&
      col < zone.colRange[1] &&
      row >= zone.rowRange[0] &&
      row < zone.rowRange[1]
    ) {
      tint = zone.tint;
    }
  }
  return tint;
}

/** Draw the entire floor grid with room tints. */
function drawFloor(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
): void {
  for (let row = 0; row < GRID_ROWS; row++) {
    for (let col = 0; col < GRID_COLS; col++) {
      const { x, y } = gridToScreen(col, row, w, h);
      const tint = getRoomTint(col, row);
      drawTile(ctx, x, y, true, tint);
    }
  }
}

/** Draw room labels at approximate center of each zone. */
function drawRoomLabels(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
): void {
  ctx.font = '9px ui-monospace, "SF Mono", Menlo, monospace';
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillStyle = "#ffffff18";

  for (const zone of ROOM_ZONES) {
    const centerCol = (zone.colRange[0] + zone.colRange[1]) / 2;
    const centerRow = (zone.rowRange[0] + zone.rowRange[1]) / 2;
    const { x, y } = gridToScreen(centerCol, centerRow, w, h);
    ctx.fillText(
      zone.name.toUpperCase(),
      x + zone.labelOffset.dx,
      y + zone.labelOffset.dy,
    );
  }
}

/** Draw a simple bookshelf decoration (Research Room). */
function drawBookshelf(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
): void {
  const { x, y } = gridToScreen(0, 0, w, h);
  // Small isometric bookshelf shape
  const bw = 14;
  const bh = 20;

  ctx.fillStyle = "#1a1510";
  ctx.fillRect(x - bw / 2, y - bh - 4, bw, bh);
  ctx.strokeStyle = "#2a2218";
  ctx.lineWidth = 0.5;
  ctx.strokeRect(x - bw / 2, y - bh - 4, bw, bh);

  // Book spines
  const colors = ["#3B82F640", "#8B5CF640", "#10B98140"];
  for (let i = 0; i < 3; i++) {
    ctx.fillStyle = colors[i];
    ctx.fillRect(x - bw / 2 + 2 + i * 4, y - bh - 2, 3, bh - 4);
  }
}

/** Draw a small chart decoration (Analysis Station). */
function drawChart(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
): void {
  const { x, y } = gridToScreen(7, 0, w, h);
  const cw = 16;
  const ch = 14;
  const bx = x - cw / 2;
  const by = y - ch - 6;

  // Background
  ctx.fillStyle = "#0f0f14";
  ctx.fillRect(bx, by, cw, ch);
  ctx.strokeStyle = "#24242a";
  ctx.lineWidth = 0.5;
  ctx.strokeRect(bx, by, cw, ch);

  // Bars
  const barColors = ["#8B5CF650", "#8B5CF680", "#8B5CF6A0"];
  const barHeights = [4, 8, 6];
  for (let i = 0; i < 3; i++) {
    ctx.fillStyle = barColors[i];
    ctx.fillRect(bx + 2 + i * 5, by + ch - barHeights[i] - 1, 3, barHeights[i]);
  }
}

/** Draw extra monitor decorations at selected desks. */
function drawFurnitureDecorations(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
): void {
  // Draw a second monitor at certain desks for visual variety
  // Scout desk (researcher) - extra book
  const scoutPos = gridToScreen(1, 1, w, h);
  ctx.fillStyle = "#3B82F615";
  ctx.fillRect(scoutPos.x + 18, scoutPos.y - 8, 6, 8);
  ctx.strokeStyle = "#3B82F630";
  ctx.lineWidth = 0.5;
  ctx.strokeRect(scoutPos.x + 18, scoutPos.y - 8, 6, 8);

  // Pixel desk (coder) - second monitor
  const pixelPos = gridToScreen(5, 4, w, h);
  const mw = TILE_W * 0.6 * 0.3;
  const mh = TILE_H * 0.6 * 0.25;
  const dh = TILE_H * 0.6;
  ctx.beginPath();
  ctx.moveTo(pixelPos.x + 16, pixelPos.y - dh - mh * 2 + 2);
  ctx.lineTo(pixelPos.x + 16 + mw, pixelPos.y - dh - mh + 2);
  ctx.lineTo(pixelPos.x + 16, pixelPos.y - dh + 2);
  ctx.lineTo(pixelPos.x + 16 - mw, pixelPos.y - dh - mh + 2);
  ctx.closePath();
  ctx.fillStyle = "#110811";
  ctx.fill();
  ctx.strokeStyle = "#331833";
  ctx.lineWidth = 0.5;
  ctx.stroke();
}

/** Draw an isometric desk at a grid position. */
function drawDesk(
  ctx: CanvasRenderingContext2D,
  desk: DeskPosition,
  w: number,
  h: number,
): void {
  const { x, y } = gridToScreen(desk.gridX, desk.gridY, w, h);

  // Desk surface — isometric rectangle (smaller than tile)
  const dw = TILE_W * 0.6;
  const dh = TILE_H * 0.6;

  ctx.beginPath();
  ctx.moveTo(x, y - dh);
  ctx.lineTo(x + dw, y);
  ctx.lineTo(x, y + dh);
  ctx.lineTo(x - dw, y);
  ctx.closePath();
  ctx.fillStyle = DESK_FILL;
  ctx.fill();
  ctx.strokeStyle = DESK_BORDER;
  ctx.lineWidth = 1;
  ctx.stroke();

  // Monitor — thin rectangle on top of desk
  const mw = dw * 0.5;
  const mh = dh * 0.35;

  ctx.beginPath();
  ctx.moveTo(x, y - dh - mh * 2);
  ctx.lineTo(x + mw, y - dh - mh);
  ctx.lineTo(x, y - dh);
  ctx.lineTo(x - mw, y - dh - mh);
  ctx.closePath();
  ctx.fillStyle = MONITOR_FILL;
  ctx.fill();
  ctx.strokeStyle = MONITOR_BORDER;
  ctx.lineWidth = 0.5;
  ctx.stroke();
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
  ctx.fillStyle = LABEL_COLOR;
  ctx.font = '11px ui-monospace, "SF Mono", Menlo, monospace';
  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  ctx.fillText(desk.label, x, y + TILE_H * 0.8);
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
 */
export function drawOffice(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  desks: DeskPosition[],
  agents: Map<string, AgentRenderState>,
): void {
  // Clear
  ctx.fillStyle = "#050505";
  ctx.fillRect(0, 0, w, h);

  // Floor with room tints (D1D-224)
  drawFloor(ctx, w, h);

  // Room labels (D1D-224)
  drawRoomLabels(ctx, w, h);

  // Furniture decorations (D1D-224)
  drawBookshelf(ctx, w, h);
  drawChart(ctx, w, h);
  drawFurnitureDecorations(ctx, w, h);

  // Desks (drawn before agents so agents overlay)
  for (const desk of desks) {
    drawDesk(ctx, desk, w, h);
  }

  // Agents at desks
  for (const desk of desks) {
    if (desk.agentId) {
      const state = agents.get(desk.agentId);
      if (state) {
        drawAgent(ctx, desk, state, w, h);
      }
    }
  }
}
