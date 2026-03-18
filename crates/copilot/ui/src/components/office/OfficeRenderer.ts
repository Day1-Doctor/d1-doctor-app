/**
 * OfficeRenderer — Pure Canvas 2D rendering functions for the isometric office.
 *
 * All drawing is done through the Canvas 2D API. No DOM manipulation,
 * no React dependencies. The module exports a single entry point
 * `drawOffice` plus supporting types and constants.
 */

import type { Agent } from "../../stores/agentStore";

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
const FALLBACK_AGENT_COLOR = "#6B7280";

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
// Drawing primitives
// ---------------------------------------------------------------------------

/** Draw one isometric diamond tile outline. */
function drawTile(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  fill: boolean,
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

  ctx.strokeStyle = FLOOR_LINE_COLOR;
  ctx.lineWidth = 1;
  ctx.stroke();
}

/** Draw the entire floor grid. */
function drawFloor(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
): void {
  for (let row = 0; row < GRID_ROWS; row++) {
    for (let col = 0; col < GRID_COLS; col++) {
      const { x, y } = gridToScreen(col, row, w, h);
      drawTile(ctx, x, y, true);
    }
  }
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

/** Draw an agent circle + status dot + label at a desk. */
function drawAgent(
  ctx: CanvasRenderingContext2D,
  desk: DeskPosition,
  state: AgentRenderState,
  w: number,
  h: number,
): void {
  const { x, y } = gridToScreen(desk.gridX, desk.gridY, w, h);
  const { agent, frame } = state;

  // Determine agent body colour from role
  const bodyColor = AGENT_COLORS[agent.role] ?? FALLBACK_AGENT_COLOR;

  // Simple idle bobbing — +-2 px sinusoidal
  let bobOffset = 0;
  if (agent.status === "idle") {
    bobOffset = Math.sin(frame * 0.05) * 2;
  }

  // Pulsing alpha for "thinking"
  let alpha = 1;
  if (agent.status === "thinking") {
    alpha = 0.5 + 0.5 * Math.abs(Math.sin(frame * 0.08));
  }

  // Agent body — circle above the desk
  const circleY = y - TILE_H * 1.5 + bobOffset;
  const radius = 10;

  ctx.save();
  ctx.globalAlpha = alpha;
  ctx.beginPath();
  ctx.arc(x, circleY, radius, 0, Math.PI * 2);
  ctx.fillStyle = bodyColor;
  ctx.fill();
  ctx.strokeStyle = "#ffffff22";
  ctx.lineWidth = 1.5;
  ctx.stroke();
  ctx.restore();

  // Executing glow ring
  if (agent.status === "executing") {
    const glowRadius = radius + 4 + Math.sin(frame * 0.1) * 2;
    ctx.beginPath();
    ctx.arc(x, circleY, glowRadius, 0, Math.PI * 2);
    ctx.strokeStyle = STATUS_COLORS.executing + "88";
    ctx.lineWidth = 2;
    ctx.stroke();
  }

  // Working glow ring
  if (agent.status === "working") {
    const glowRadius = radius + 3 + Math.sin(frame * 0.12) * 2;
    ctx.beginPath();
    ctx.arc(x, circleY, glowRadius, 0, Math.PI * 2);
    ctx.strokeStyle = STATUS_COLORS.working + "88";
    ctx.lineWidth = 2;
    ctx.stroke();
  }

  // Status indicator dot — above the agent circle
  const dotY = circleY - radius - 8;
  const statusColor = STATUS_COLORS[agent.status] ?? STATUS_COLORS.idle;
  ctx.beginPath();
  ctx.arc(x, dotY, 4, 0, Math.PI * 2);
  ctx.fillStyle = statusColor;
  ctx.fill();

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

  // Floor
  drawFloor(ctx, w, h);

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
