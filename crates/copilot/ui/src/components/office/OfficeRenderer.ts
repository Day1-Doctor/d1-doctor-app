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
 * - 3D isometric room walls: two colored planes (warm left + cool right) with decorations
 * - Wall decorations: whiteboard with scribbles, clock, picture frame, bookshelf, bulletin board
 * - Vibrant exterior gradient above walls (warm-to-cool gradient + bokeh glow)
 * - Enhanced desks: wood surface, chair, monitor with state-based screen, accessories
 * - Rest zone furniture: sofa, arcade machine, water cooler, plant, whiteboard
 * - Agent movement system: agents walk to rest zones when idle, return to desk when working
 */

import type { Agent } from "../../stores/agentStore";
import { drawPixelCharacter } from "./SpriteRenderer";
import {
  type AgentPosition,
  updateAgentPosition,
  createAgentPosition,
} from "./AgentMovement";

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
 * Wall height fraction of canvas height.
 * The walls occupy the top portion of the canvas.
 */
const WALL_HEIGHT_FRAC = 0.32;

/**
 * Convert grid (col, row) to screen (x, y) given canvas dimensions.
 * The isometric floor is placed below the 3D room walls.
 */
function gridToScreen(
  col: number,
  row: number,
  canvasW: number,
  canvasH: number,
): { x: number; y: number } {
  const offsetX = canvasW / 2;
  // Reserve top portion for walls; floor starts below that
  const wallH = canvasH * WALL_HEIGHT_FRAC;
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
// Wall backdrop — proper 3D isometric room walls
// ---------------------------------------------------------------------------

/**
 * Draw wall decorations placed on the left wall surface.
 * Coordinates are in screen space; decorations are positioned along the wall.
 */
function drawLeftWallDecorations(
  ctx: CanvasRenderingContext2D,
  cornerX: number,
  cornerY: number,
  leftEdgeX: number,
  leftEdgeY: number,
  wallHeight: number,
  frame: number,
): void {
  // Helper: interpolate position along the left wall at fraction t (0=corner, 1=left edge)
  // and height fraction hf (0=base, 1=top of wall)
  const wallPoint = (t: number, hf: number) => ({
    x: cornerX + (leftEdgeX - cornerX) * t,
    y: cornerY + (leftEdgeY - cornerY) * t - wallHeight * hf,
  });

  // ── Whiteboard ──
  // 40% along wall, 35% up
  const wbCenter = wallPoint(0.4, 0.35);
  const wbW = 64;
  const wbH = 40;
  // Board backing (subtle shadow)
  ctx.fillStyle = "#6A5030";
  ctx.fillRect(wbCenter.x - wbW / 2 + 2, wbCenter.y - wbH / 2 + 2, wbW, wbH);
  // Board surface
  ctx.fillStyle = "#E8E0D0";
  ctx.fillRect(wbCenter.x - wbW / 2, wbCenter.y - wbH / 2, wbW, wbH);
  // Subtle frame
  ctx.strokeStyle = "#A08060";
  ctx.lineWidth = 2;
  ctx.strokeRect(wbCenter.x - wbW / 2, wbCenter.y - wbH / 2, wbW, wbH);

  // Scribbles on whiteboard
  ctx.save();
  ctx.beginPath();
  ctx.rect(wbCenter.x - wbW / 2 + 2, wbCenter.y - wbH / 2 + 2, wbW - 4, wbH - 4);
  ctx.clip();
  // Red marker text scribble
  ctx.strokeStyle = "#CC3030";
  ctx.lineWidth = 1.2;
  ctx.beginPath();
  ctx.moveTo(wbCenter.x - 24, wbCenter.y - 12);
  ctx.lineTo(wbCenter.x - 16, wbCenter.y - 12);
  ctx.moveTo(wbCenter.x - 20, wbCenter.y - 12);
  ctx.lineTo(wbCenter.x - 20, wbCenter.y - 6);
  ctx.moveTo(wbCenter.x - 16, wbCenter.y - 6);
  ctx.lineTo(wbCenter.x - 12, wbCenter.y - 6);
  ctx.stroke();
  // Blue chart
  ctx.strokeStyle = "#2060CC";
  ctx.lineWidth = 1;
  const chartPts: [number, number][] = [
    [wbCenter.x - 4, wbCenter.y + 8],
    [wbCenter.x, wbCenter.y + 2],
    [wbCenter.x + 6, wbCenter.y + 5],
    [wbCenter.x + 12, wbCenter.y - 4],
    [wbCenter.x + 18, wbCenter.y - 10],
  ];
  ctx.beginPath();
  for (let i = 0; i < chartPts.length; i++) {
    if (i === 0) ctx.moveTo(chartPts[i][0], chartPts[i][1]);
    else ctx.lineTo(chartPts[i][0], chartPts[i][1]);
  }
  ctx.stroke();
  // Green text "v3.0"
  ctx.fillStyle = "#208040";
  ctx.font = 'bold 7px ui-monospace, "SF Mono", Menlo, monospace';
  ctx.textAlign = "left";
  ctx.textBaseline = "top";
  ctx.fillText("v3.0", wbCenter.x - 26, wbCenter.y - 18);
  ctx.restore();

  // Sticky notes (small colored squares at the bottom of whiteboard)
  const stickyColors = ["#FFEE44", "#FF9944", "#44AAFF"];
  for (let si = 0; si < 3; si++) {
    const sx = wbCenter.x - wbW / 2 + 4 + si * 16;
    const sy = wbCenter.y + wbH / 2 - 10;
    ctx.fillStyle = stickyColors[si];
    ctx.fillRect(sx, sy, 11, 9);
    ctx.strokeStyle = "rgba(0,0,0,0.15)";
    ctx.lineWidth = 0.5;
    ctx.strokeRect(sx, sy, 11, 9);
  }

  // ── Clock ──
  // 72% along wall, 65% up
  const clkCenter = wallPoint(0.72, 0.65);
  const clkR = 11;
  // Clock face
  ctx.fillStyle = "#D0C8B8";
  ctx.beginPath();
  ctx.arc(clkCenter.x, clkCenter.y, clkR, 0, Math.PI * 2);
  ctx.fill();
  ctx.strokeStyle = "#7A6040";
  ctx.lineWidth = 1.5;
  ctx.stroke();
  // Hour markers (4 dots at 12/3/6/9)
  for (let mi = 0; mi < 4; mi++) {
    const angle = (mi / 4) * Math.PI * 2 - Math.PI / 2;
    const mx2 = clkCenter.x + Math.cos(angle) * (clkR - 3);
    const my2 = clkCenter.y + Math.sin(angle) * (clkR - 3);
    ctx.fillStyle = "#5A4020";
    ctx.beginPath();
    ctx.arc(mx2, my2, 1.2, 0, Math.PI * 2);
    ctx.fill();
  }
  // Animated clock hands (rotate based on frame)
  const minuteAngle = (frame * 0.003) % (Math.PI * 2) - Math.PI / 2;
  const hourAngle = (frame * 0.00025) % (Math.PI * 2) - Math.PI / 2;
  // Minute hand
  ctx.strokeStyle = "#3A2810";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(clkCenter.x, clkCenter.y);
  ctx.lineTo(
    clkCenter.x + Math.cos(minuteAngle) * (clkR - 3),
    clkCenter.y + Math.sin(minuteAngle) * (clkR - 3),
  );
  ctx.stroke();
  // Hour hand
  ctx.lineWidth = 1.5;
  ctx.beginPath();
  ctx.moveTo(clkCenter.x, clkCenter.y);
  ctx.lineTo(
    clkCenter.x + Math.cos(hourAngle) * (clkR - 5),
    clkCenter.y + Math.sin(hourAngle) * (clkR - 5),
  );
  ctx.stroke();
  // Center dot
  ctx.fillStyle = "#3A2810";
  ctx.beginPath();
  ctx.arc(clkCenter.x, clkCenter.y, 1.5, 0, Math.PI * 2);
  ctx.fill();

  // ── Picture frame ──
  // 18% along wall, 55% up
  const picCenter = wallPoint(0.18, 0.55);
  const picW = 28;
  const picH = 20;
  // Frame border
  ctx.fillStyle = "#6A4820";
  ctx.fillRect(picCenter.x - picW / 2 - 3, picCenter.y - picH / 2 - 3, picW + 6, picH + 6);
  // Canvas
  ctx.fillStyle = "#B8D0E8";
  ctx.fillRect(picCenter.x - picW / 2, picCenter.y - picH / 2, picW, picH);
  // Simple landscape inside
  ctx.fillStyle = "#406080";
  ctx.fillRect(picCenter.x - picW / 2, picCenter.y - picH / 2, picW, picH * 0.6);
  // Hills
  ctx.fillStyle = "#3A7040";
  ctx.beginPath();
  ctx.arc(picCenter.x - 8, picCenter.y + 1, 8, -Math.PI, 0);
  ctx.fill();
  ctx.fillStyle = "#2A5030";
  ctx.beginPath();
  ctx.arc(picCenter.x + 5, picCenter.y + 2, 6, -Math.PI, 0);
  ctx.fill();
  // "Day1" mini text
  ctx.fillStyle = "rgba(255,255,255,0.7)";
  ctx.font = '5px ui-monospace, "SF Mono", Menlo, monospace';
  ctx.textAlign = "center";
  ctx.textBaseline = "bottom";
  ctx.fillText("Day1", picCenter.x, picCenter.y + picH / 2 - 1);
}

/**
 * Draw wall decorations placed on the right wall surface.
 */
function drawRightWallDecorations(
  ctx: CanvasRenderingContext2D,
  cornerX: number,
  cornerY: number,
  rightEdgeX: number,
  rightEdgeY: number,
  wallHeight: number,
  _frame: number,
): void {
  // Helper: interpolate position along the right wall at fraction t (0=corner, 1=right edge)
  // and height fraction hf (0=base, 1=top of wall)
  const wallPoint = (t: number, hf: number) => ({
    x: cornerX + (rightEdgeX - cornerX) * t,
    y: cornerY + (rightEdgeY - cornerY) * t - wallHeight * hf,
  });

  // ── Bookshelf ──
  // 28% along right wall
  const bsCenter = wallPoint(0.28, 0.3);
  const bsW = 50;
  const bsH = 52;
  // Shelf backing (wood)
  ctx.fillStyle = "#8A6A48";
  ctx.fillRect(bsCenter.x - bsW / 2, bsCenter.y - bsH / 2, bsW, bsH);
  ctx.strokeStyle = "#6A4A28";
  ctx.lineWidth = 1;
  ctx.strokeRect(bsCenter.x - bsW / 2, bsCenter.y - bsH / 2, bsW, bsH);
  // 3 shelves
  const shelfColors = [
    ["#CC3030", "#3060CC", "#30A040", "#E09020", "#802890"],
    ["#CC6020", "#204080", "#2A8060", "#CC3070", "#60A020"],
    ["#802040", "#4040A0", "#308050", "#C08020"],
  ];
  for (let shelf = 0; shelf < 3; shelf++) {
    const shelfY = bsCenter.y - bsH / 2 + 4 + shelf * 16;
    // Shelf plank
    ctx.fillStyle = "#A08060";
    ctx.fillRect(bsCenter.x - bsW / 2 + 1, shelfY + 13, bsW - 2, 3);
    // Books on this shelf
    const books = shelfColors[shelf];
    let bx = bsCenter.x - bsW / 2 + 3;
    for (let bi = 0; bi < books.length; bi++) {
      const bw = 6 + (bi % 2) * 2;
      const bh = 10 + (bi % 3) * 2;
      ctx.fillStyle = books[bi];
      ctx.fillRect(bx, shelfY + 13 - bh, bw, bh);
      ctx.strokeStyle = "rgba(0,0,0,0.2)";
      ctx.lineWidth = 0.4;
      ctx.strokeRect(bx, shelfY + 13 - bh, bw, bh);
      bx += bw + 1;
    }
  }
  // Small plant on top of bookshelf
  const plantTop = bsCenter.y - bsH / 2 - 10;
  ctx.fillStyle = "#7A4020";
  ctx.fillRect(bsCenter.x + bsW / 2 - 14, plantTop + 4, 10, 7);
  ctx.fillStyle = "#2A8040";
  ctx.beginPath();
  ctx.arc(bsCenter.x + bsW / 2 - 9, plantTop, 6, 0, Math.PI * 2);
  ctx.fill();
  ctx.beginPath();
  ctx.arc(bsCenter.x + bsW / 2 - 14, plantTop + 1, 4, 0, Math.PI * 2);
  ctx.fill();

  // ── Bulletin board ──
  // 62% along right wall, 55% up
  const bbCenter = wallPoint(0.62, 0.55);
  const bbW = 40;
  const bbH = 28;
  // Cork background
  ctx.fillStyle = "#C4A878";
  ctx.fillRect(bbCenter.x - bbW / 2, bbCenter.y - bbH / 2, bbW, bbH);
  ctx.strokeStyle = "#8A6840";
  ctx.lineWidth = 1.5;
  ctx.strokeRect(bbCenter.x - bbW / 2, bbCenter.y - bbH / 2, bbW, bbH);
  // Cork texture dots
  ctx.fillStyle = "rgba(150,100,60,0.3)";
  for (let ci = 0; ci < 8; ci++) {
    const cdx = ((ci * 47) % (bbW - 4)) + 2;
    const cdy = ((ci * 31) % (bbH - 4)) + 2;
    ctx.beginPath();
    ctx.arc(bbCenter.x - bbW / 2 + cdx, bbCenter.y - bbH / 2 + cdy, 1, 0, Math.PI * 2);
    ctx.fill();
  }
  // Pinned notes (small colored rectangles at slight angles)
  const pinNotes = [
    { dx: -14, dy: -8, w: 14, h: 10, color: "#FFEE88", angle: -0.08 },
    { dx: 4, dy: -10, w: 12, h: 9, color: "#AADDFF", angle: 0.06 },
    { dx: -16, dy: 4, w: 12, h: 10, color: "#FFBBAA", angle: 0.04 },
    { dx: 6, dy: 2, w: 10, h: 8, color: "#AAFFCC", angle: -0.05 },
  ];
  for (const note of pinNotes) {
    ctx.save();
    ctx.translate(bbCenter.x + note.dx + note.w / 2, bbCenter.y + note.dy + note.h / 2);
    ctx.rotate(note.angle);
    ctx.fillStyle = note.color;
    ctx.fillRect(-note.w / 2, -note.h / 2, note.w, note.h);
    ctx.strokeStyle = "rgba(0,0,0,0.1)";
    ctx.lineWidth = 0.5;
    ctx.strokeRect(-note.w / 2, -note.h / 2, note.w, note.h);
    // Pin dot
    ctx.fillStyle = "#EF4444";
    ctx.beginPath();
    ctx.arc(0, -note.h / 2 + 1, 1.5, 0, Math.PI * 2);
    ctx.fill();
    ctx.restore();
  }
  // Small photo on board
  ctx.fillStyle = "#608090";
  ctx.fillRect(bbCenter.x + 8, bbCenter.y - 4, 14, 10);
  ctx.fillStyle = "#405060";
  ctx.fillRect(bbCenter.x + 9, bbCenter.y - 3, 12, 6);

  // ── Wall cabinet / cupboard ──
  // 86% along right wall, 40% up
  const cabCenter = wallPoint(0.86, 0.4);
  const cabW = 28;
  const cabH = 40;
  // Cabinet body
  ctx.fillStyle = "#907868";
  ctx.fillRect(cabCenter.x - cabW / 2, cabCenter.y - cabH / 2, cabW, cabH);
  ctx.strokeStyle = "#705848";
  ctx.lineWidth = 1;
  ctx.strokeRect(cabCenter.x - cabW / 2, cabCenter.y - cabH / 2, cabW, cabH);
  // Door divider line
  ctx.strokeStyle = "#705848";
  ctx.lineWidth = 0.8;
  ctx.beginPath();
  ctx.moveTo(cabCenter.x, cabCenter.y - cabH / 2);
  ctx.lineTo(cabCenter.x, cabCenter.y + cabH / 2);
  ctx.stroke();
  ctx.beginPath();
  ctx.moveTo(cabCenter.x - cabW / 2, cabCenter.y);
  ctx.lineTo(cabCenter.x + cabW / 2, cabCenter.y);
  ctx.stroke();
  // Handles
  ctx.fillStyle = "#C0A060";
  ctx.fillRect(cabCenter.x - 5, cabCenter.y - 2, 4, 4);
  ctx.fillRect(cabCenter.x + 1, cabCenter.y - 2, 4, 4);
  ctx.fillRect(cabCenter.x - 5, cabCenter.y - cabH / 4 - 2, 4, 4);
  ctx.fillRect(cabCenter.x + 1, cabCenter.y - cabH / 4 - 2, 4, 4);
}

/**
 * Draw 3D isometric room walls — back-left wall (warm brown) and back-right wall
 * (cooler brown) forming an L-shape. Also draws vibrant exterior above the walls
 * and decorations on the wall surfaces.
 */
function drawWallBackdrop(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  frame: number,
): void {
  // Compute the isometric grid's top-most point.
  // gridToScreen(0,0) gives the top corner of the grid.
  const { x: gridTopX, y: gridTopY } = gridToScreen(0, 0, w, h);

  // The room corner is where the two walls meet — directly above the grid apex
  const cornerX = gridTopX;
  const cornerY = gridTopY;

  // Wall height
  const wallHeight = h * WALL_HEIGHT_FRAC;

  // Left edge of the floor grid (far left)
  const leftEdgeX = gridTopX - GRID_ROWS * TILE_W;
  const leftEdgeY = gridTopY + GRID_ROWS * TILE_H;

  // Right edge of the floor grid (far right)
  const rightEdgeX = gridTopX + GRID_COLS * TILE_W;
  const rightEdgeY = gridTopY + GRID_COLS * TILE_H;

  // ── VIBRANT EXTERIOR above the walls ──
  // Fill the entire upper region with a warm-to-cool horizontal gradient
  const extTopY = cornerY - wallHeight - 4;

  // Base exterior gradient (horizontal warm-left to cool-right)
  const extGrad = ctx.createLinearGradient(0, 0, w, 0);
  extGrad.addColorStop(0, "#2A1828");
  extGrad.addColorStop(0.3, "#1E1530");
  extGrad.addColorStop(0.7, "#152030");
  extGrad.addColorStop(1, "#1A2838");
  ctx.fillStyle = extGrad;
  ctx.fillRect(0, 0, w, cornerY + 10);

  // Warm glow from top-left
  const warmGlow = ctx.createRadialGradient(w * 0.2, 0, 0, w * 0.2, 0, h * 0.6);
  warmGlow.addColorStop(0, "rgba(180, 80, 40, 0.18)");
  warmGlow.addColorStop(0.5, "rgba(140, 60, 80, 0.08)");
  warmGlow.addColorStop(1, "rgba(0, 0, 0, 0)");
  ctx.fillStyle = warmGlow;
  ctx.fillRect(0, 0, w, cornerY + 10);

  // Cool glow from top-right
  const coolGlow = ctx.createRadialGradient(w * 0.8, 0, 0, w * 0.8, 0, h * 0.6);
  coolGlow.addColorStop(0, "rgba(40, 80, 180, 0.15)");
  coolGlow.addColorStop(0.5, "rgba(20, 60, 140, 0.06)");
  coolGlow.addColorStop(1, "rgba(0, 0, 0, 0)");
  ctx.fillStyle = coolGlow;
  ctx.fillRect(0, 0, w, cornerY + 10);

  // Subtle bokeh circles in the exterior
  const bokehSeeds = [
    { x: w * 0.12, y: h * 0.05, r: 18, color: "rgba(200, 100, 50, 0.04)" },
    { x: w * 0.28, y: h * 0.1, r: 12, color: "rgba(180, 80, 120, 0.03)" },
    { x: w * 0.55, y: h * 0.04, r: 22, color: "rgba(60, 80, 200, 0.04)" },
    { x: w * 0.75, y: h * 0.08, r: 16, color: "rgba(40, 100, 180, 0.03)" },
    { x: w * 0.88, y: h * 0.06, r: 14, color: "rgba(80, 120, 220, 0.04)" },
  ];
  for (const bk of bokehSeeds) {
    const br2 = ctx.createRadialGradient(bk.x, bk.y, 0, bk.x, bk.y, bk.r);
    br2.addColorStop(0, bk.color);
    br2.addColorStop(1, "rgba(0,0,0,0)");
    ctx.fillStyle = br2;
    ctx.beginPath();
    ctx.arc(bk.x, bk.y, bk.r, 0, Math.PI * 2);
    ctx.fill();
  }

  // Faint diagonal light rays at very low opacity
  ctx.save();
  ctx.globalAlpha = 0.025;
  ctx.strokeStyle = "rgba(200, 160, 100, 1)";
  ctx.lineWidth = 1;
  for (let ri = 0; ri < 8; ri++) {
    const rx = w * 0.1 + ri * w * 0.12;
    ctx.beginPath();
    ctx.moveTo(rx, 0);
    ctx.lineTo(rx + extTopY * 0.5, extTopY);
    ctx.stroke();
  }
  ctx.restore();

  // ── LEFT WALL (warm brown — faces right) ──
  ctx.fillStyle = "#8B6B50";
  ctx.beginPath();
  ctx.moveTo(cornerX, cornerY);
  ctx.lineTo(cornerX, cornerY - wallHeight);
  ctx.lineTo(leftEdgeX, leftEdgeY - wallHeight);
  ctx.lineTo(leftEdgeX, leftEdgeY);
  ctx.closePath();
  ctx.fill();

  // Left wall shadow gradient (darker at bottom, lighter at top)
  const leftShadow = ctx.createLinearGradient(0, cornerY, 0, cornerY - wallHeight);
  leftShadow.addColorStop(0, "rgba(0,0,0,0.22)");
  leftShadow.addColorStop(0.5, "rgba(0,0,0,0.06)");
  leftShadow.addColorStop(1, "rgba(0,0,0,0)");
  ctx.fillStyle = leftShadow;
  ctx.beginPath();
  ctx.moveTo(cornerX, cornerY);
  ctx.lineTo(cornerX, cornerY - wallHeight);
  ctx.lineTo(leftEdgeX, leftEdgeY - wallHeight);
  ctx.lineTo(leftEdgeX, leftEdgeY);
  ctx.closePath();
  ctx.fill();

  // ── RIGHT WALL (slightly cooler/darker brown — faces left) ──
  ctx.fillStyle = "#7A6048";
  ctx.beginPath();
  ctx.moveTo(cornerX, cornerY);
  ctx.lineTo(cornerX, cornerY - wallHeight);
  ctx.lineTo(rightEdgeX, rightEdgeY - wallHeight);
  ctx.lineTo(rightEdgeX, rightEdgeY);
  ctx.closePath();
  ctx.fill();

  // Right wall slightly darker (facing away from primary light source)
  ctx.fillStyle = "rgba(0,0,0,0.08)";
  ctx.beginPath();
  ctx.moveTo(cornerX, cornerY);
  ctx.lineTo(cornerX, cornerY - wallHeight);
  ctx.lineTo(rightEdgeX, rightEdgeY - wallHeight);
  ctx.lineTo(rightEdgeX, rightEdgeY);
  ctx.closePath();
  ctx.fill();

  // ── Wall top trim (lighter strip at the very top of each wall) ──
  ctx.fillStyle = "#A08060";
  // Left wall top trim
  ctx.beginPath();
  ctx.moveTo(cornerX, cornerY - wallHeight - 4);
  ctx.lineTo(cornerX, cornerY - wallHeight);
  ctx.lineTo(leftEdgeX, leftEdgeY - wallHeight);
  ctx.lineTo(leftEdgeX, leftEdgeY - wallHeight - 4);
  ctx.closePath();
  ctx.fill();
  // Right wall top trim
  ctx.fillStyle = "#957858";
  ctx.beginPath();
  ctx.moveTo(cornerX, cornerY - wallHeight - 4);
  ctx.lineTo(cornerX, cornerY - wallHeight);
  ctx.lineTo(rightEdgeX, rightEdgeY - wallHeight);
  ctx.lineTo(rightEdgeX, rightEdgeY - wallHeight - 4);
  ctx.closePath();
  ctx.fill();

  // ── Baseboard (darker strip at the floor-wall junction) ──
  ctx.fillStyle = "#5A4030";
  // Left wall baseboard
  ctx.beginPath();
  ctx.moveTo(cornerX, cornerY);
  ctx.lineTo(cornerX, cornerY - 6);
  ctx.lineTo(leftEdgeX, leftEdgeY - 6);
  ctx.lineTo(leftEdgeX, leftEdgeY);
  ctx.closePath();
  ctx.fill();
  // Right wall baseboard
  ctx.fillStyle = "#504030";
  ctx.beginPath();
  ctx.moveTo(cornerX, cornerY);
  ctx.lineTo(cornerX, cornerY - 6);
  ctx.lineTo(rightEdgeX, rightEdgeY - 6);
  ctx.lineTo(rightEdgeX, rightEdgeY);
  ctx.closePath();
  ctx.fill();

  // ── Corner edge (vertical line where the two walls meet) ──
  ctx.strokeStyle = "#6A5040";
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(cornerX, cornerY);
  ctx.lineTo(cornerX, cornerY - wallHeight - 4);
  ctx.stroke();

  // ── Wall decorations ──
  drawLeftWallDecorations(ctx, cornerX, cornerY, leftEdgeX, leftEdgeY, wallHeight, frame);
  drawRightWallDecorations(ctx, cornerX, cornerY, rightEdgeX, rightEdgeY, wallHeight, frame);
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

  // Upscaled desk: 1.4x width, 1.3x depth
  const dw = TILE_W * 0.62 * 1.4;
  const dh = TILE_H * 0.62 * 1.3;

  // ---- Shadow under desk ----
  ctx.save();
  ctx.globalAlpha = 0.18;
  ctx.fillStyle = "#000000";
  ctx.beginPath();
  ctx.ellipse(x, y + dh * 0.6, dw * 0.85, dh * 0.55, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();

  // ---- Chair behind desk ----
  const chairY = y + dh * 0.55;
  const seatW = dw * 0.52;
  const seatH = dh * 0.42;

  // Chair wheels (more visible, 5-wheel base)
  ctx.fillStyle = "#1A1A1A";
  for (let wi = 0; wi < 5; wi++) {
    const angle = (wi / 5) * Math.PI * 2;
    ctx.beginPath();
    ctx.ellipse(
      x + Math.cos(angle) * seatW * 0.38,
      chairY + seatH * 0.7 + Math.sin(angle) * seatH * 0.18,
      3.5, 2, 0, 0, Math.PI * 2,
    );
    ctx.fill();
  }
  // Wheel hub
  ctx.fillStyle = "#2A2A2A";
  ctx.beginPath();
  ctx.ellipse(x, chairY + seatH * 0.6, 5, 3, 0, 0, Math.PI * 2);
  ctx.fill();

  // Chair seat cushion with texture
  ctx.beginPath();
  ctx.moveTo(x, chairY - seatH);
  ctx.lineTo(x + seatW, chairY);
  ctx.lineTo(x, chairY + seatH);
  ctx.lineTo(x - seatW, chairY);
  ctx.closePath();
  ctx.fillStyle = roleColor + "66";
  ctx.fill();
  ctx.strokeStyle = roleColor + "99";
  ctx.lineWidth = 1;
  ctx.stroke();
  // Cushion centre seam
  ctx.save();
  ctx.beginPath();
  ctx.moveTo(x, chairY - seatH);
  ctx.lineTo(x + seatW, chairY);
  ctx.lineTo(x, chairY + seatH);
  ctx.lineTo(x - seatW, chairY);
  ctx.closePath();
  ctx.clip();
  ctx.strokeStyle = roleColor + "44";
  ctx.lineWidth = 0.8;
  ctx.beginPath();
  ctx.moveTo(x - seatW * 0.5, chairY - seatH * 0.5);
  ctx.lineTo(x + seatW * 0.5, chairY + seatH * 0.5);
  ctx.moveTo(x + seatW * 0.5, chairY - seatH * 0.5);
  ctx.lineTo(x - seatW * 0.5, chairY + seatH * 0.5);
  ctx.stroke();
  ctx.restore();

  // Chair back with padding detail
  const backH = dh * 0.9;
  const backW = seatW * 0.65;
  ctx.beginPath();
  ctx.moveTo(x - backW, chairY - seatH * 0.3);
  ctx.lineTo(x - backW, chairY - seatH * 0.3 - backH);
  ctx.lineTo(x + backW, chairY - seatH * 0.3 - backH);
  ctx.lineTo(x + backW, chairY - seatH * 0.3);
  ctx.fillStyle = roleColor + "55";
  ctx.fill();
  ctx.strokeStyle = roleColor + "77";
  ctx.lineWidth = 1;
  ctx.stroke();
  // Back padding lines
  ctx.strokeStyle = roleColor + "33";
  ctx.lineWidth = 0.7;
  for (let pi = 1; pi < 4; pi++) {
    const py3 = chairY - seatH * 0.3 - (backH * pi) / 4;
    ctx.beginPath();
    ctx.moveTo(x - backW + 2, py3);
    ctx.lineTo(x + backW - 2, py3);
    ctx.stroke();
  }
  // Headrest
  ctx.fillStyle = roleColor + "44";
  ctx.fillRect(x - backW * 0.55, chairY - seatH * 0.3 - backH - 6, backW * 1.1, 8);
  ctx.strokeStyle = roleColor + "66";
  ctx.lineWidth = 0.8;
  ctx.strokeRect(x - backW * 0.55, chairY - seatH * 0.3 - backH - 6, backW * 1.1, 8);

  // ---- Desk front face (depth) ----
  const frontH = 11;
  ctx.beginPath();
  ctx.moveTo(x + dw, y);
  ctx.lineTo(x + dw, y + frontH);
  ctx.lineTo(x, y + dh + frontH);
  ctx.lineTo(x, y + dh);
  ctx.closePath();
  ctx.fillStyle = "#7A5538";
  ctx.fill();
  ctx.strokeStyle = "#4A3020";
  ctx.lineWidth = 1;
  ctx.stroke();

  // Drawer handles on front face (2 drawers)
  ctx.fillStyle = "#D0B070";
  ctx.fillRect(x + dw * 0.25 - 5, y + frontH * 0.25, 10, 3);
  ctx.fillRect(x + dw * 0.25 - 5, y + frontH * 0.62, 10, 3);
  ctx.strokeStyle = "#B09050";
  ctx.lineWidth = 0.5;
  ctx.strokeRect(x + dw * 0.25 - 5, y + frontH * 0.25, 10, 3);
  ctx.strokeRect(x + dw * 0.25 - 5, y + frontH * 0.62, 10, 3);

  // Left face for depth
  ctx.beginPath();
  ctx.moveTo(x - dw, y);
  ctx.lineTo(x - dw, y + frontH);
  ctx.lineTo(x, y + dh + frontH);
  ctx.lineTo(x, y + dh);
  ctx.closePath();
  ctx.fillStyle = "#5E3C20";
  ctx.fill();
  ctx.strokeStyle = "#4A3020";
  ctx.lineWidth = 1;
  ctx.stroke();

  // ---- Desk surface ----
  ctx.beginPath();
  ctx.moveTo(x, y - dh);
  ctx.lineTo(x + dw, y);
  ctx.lineTo(x, y + dh);
  ctx.lineTo(x - dw, y);
  ctx.closePath();
  ctx.fillStyle = "#B08C6A";
  ctx.fill();
  ctx.strokeStyle = "#7A5A40";
  ctx.lineWidth = 1.2;
  ctx.stroke();

  // Wood grain on surface (richer)
  ctx.save();
  ctx.beginPath();
  ctx.moveTo(x, y - dh);
  ctx.lineTo(x + dw, y);
  ctx.lineTo(x, y + dh);
  ctx.lineTo(x - dw, y);
  ctx.closePath();
  ctx.clip();
  ctx.strokeStyle = "rgba(90, 60, 30, 0.15)";
  ctx.lineWidth = 0.6;
  for (let grain = -dw; grain < dw; grain += 5) {
    ctx.beginPath();
    ctx.moveTo(x + grain, y - dh);
    ctx.lineTo(x + grain + dh * 1.1, y + dh);
    ctx.stroke();
  }
  // Surface highlight (shine)
  const deskShine = ctx.createLinearGradient(x - dw * 0.4, y - dh * 0.6, x + dw * 0.2, y + dh * 0.2);
  deskShine.addColorStop(0, "rgba(255, 240, 200, 0.12)");
  deskShine.addColorStop(0.5, "rgba(255, 240, 200, 0.04)");
  deskShine.addColorStop(1, "rgba(0, 0, 0, 0)");
  ctx.fillStyle = deskShine;
  ctx.fill();
  ctx.restore();

  // ---- Monitor on desk (bigger, wider bezel, taller screen) ----
  const monW = dw * 0.82;
  const monH = 26;
  const monX = x - monW / 2;
  const monY = y - dh - monH - 10;

  // Monitor stand (wider base)
  ctx.fillStyle = "#282828";
  ctx.fillRect(x - 3, y - dh - 10, 6, 10);
  ctx.fillRect(x - 9, y - dh - 1, 18, 3);

  // Monitor outer bezel
  ctx.fillStyle = "#1C1C1C";
  ctx.fillRect(monX - 4, monY - 4, monW + 8, monH + 8);
  ctx.strokeStyle = "#3A3A3A";
  ctx.lineWidth = 1.5;
  ctx.strokeRect(monX - 4, monY - 4, monW + 8, monH + 8);
  // Inner bezel (thicker sides)
  ctx.fillStyle = "#121212";
  ctx.fillRect(monX - 2, monY - 2, monW + 4, monH + 4);

  // Screen content
  drawMonitorScreen(ctx, monX, monY, monW, monH, agentStatus, frame);
  // Screen shine
  ctx.save();
  ctx.beginPath();
  ctx.rect(monX, monY, monW, monH);
  ctx.clip();
  const screenShine = ctx.createLinearGradient(monX, monY, monX + monW * 0.5, monY + monH * 0.4);
  screenShine.addColorStop(0, "rgba(255,255,255,0.07)");
  screenShine.addColorStop(1, "rgba(255,255,255,0)");
  ctx.fillStyle = screenShine;
  ctx.fillRect(monX, monY, monW, monH);
  ctx.restore();

  // ---- Keyboard (wider, more key detail) ----
  const kbW = dw * 0.72;
  const kbH = 9;
  const kbX = x - kbW / 2;
  const kbY = y - dh * 0.08 + 2;

  ctx.fillStyle = "#181818";
  ctx.fillRect(kbX, kbY, kbW, kbH);
  ctx.strokeStyle = "#282828";
  ctx.lineWidth = 0.8;
  ctx.strokeRect(kbX, kbY, kbW, kbH);
  // Keyboard shine
  ctx.fillStyle = "rgba(255,255,255,0.04)";
  ctx.fillRect(kbX, kbY, kbW, kbH * 0.4);

  // Key grid (12 cols × 3 rows)
  const keyCols = 12;
  const keyRows = 3;
  const keyW = (kbW - 4) / keyCols;
  const keyH = (kbH - 2) / keyRows;
  for (let kr = 0; kr < keyRows; kr++) {
    for (let kc = 0; kc < keyCols; kc++) {
      const keyBrightness = (kc + kr) % 3 === 0 ? "#3A3A3A" : "#2C2C2C";
      ctx.fillStyle = keyBrightness;
      ctx.fillRect(
        kbX + 2 + kc * keyW,
        kbY + 1 + kr * keyH,
        keyW - 1,
        keyH - 1,
      );
    }
  }

  // ---- Coffee mug (larger) ----
  const mugX = x + dw * 0.6;
  const mugY = y - dh * 0.25;
  ctx.fillStyle = "#C04020";
  ctx.fillRect(mugX - 5, mugY - 11, 11, 11);
  ctx.strokeStyle = "#A03010";
  ctx.lineWidth = 1;
  ctx.strokeRect(mugX - 5, mugY - 11, 11, 11);
  // Mug rim highlight
  ctx.fillStyle = "#D06040";
  ctx.fillRect(mugX - 5, mugY - 11, 11, 2);
  // Handle
  ctx.beginPath();
  ctx.arc(mugX + 6, mugY - 5.5, 4, -Math.PI / 2, Math.PI / 2);
  ctx.strokeStyle = "#A03010";
  ctx.lineWidth = 1.5;
  ctx.stroke();
  // Mug interior (dark)
  ctx.fillStyle = "#1A0A0A";
  ctx.fillRect(mugX - 4, mugY - 10, 9, 3);

  // Steam when working
  if (agentStatus === "working" || agentStatus === "executing") {
    ctx.strokeStyle = "#FFFFFF50";
    ctx.lineWidth = 1.2;
    for (let si = 0; si < 3; si++) {
      const steamPhase = frame * 0.07 + si * 1.2;
      const steamX = mugX - 2 + si * 3.5;
      ctx.beginPath();
      ctx.moveTo(steamX, mugY - 11);
      ctx.quadraticCurveTo(
        steamX + Math.sin(steamPhase) * 2.5,
        mugY - 15,
        steamX + Math.sin(steamPhase + 1) * 2.5,
        mugY - 20,
      );
      ctx.stroke();
    }
  }

  // ---- Desk lamp (larger, better detail) ----
  const lampX = x - dw * 0.65;
  const lampY = y - dh * 0.2;
  ctx.strokeStyle = "#909090";
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(lampX, lampY);
  ctx.lineTo(lampX, lampY - 14);
  ctx.lineTo(lampX + 8, lampY - 20);
  ctx.stroke();
  // Lamp arm joint
  ctx.fillStyle = "#707070";
  ctx.beginPath();
  ctx.arc(lampX, lampY - 14, 2.5, 0, Math.PI * 2);
  ctx.fill();
  // Lamp shade (larger)
  ctx.fillStyle = "#E0C060";
  ctx.beginPath();
  ctx.moveTo(lampX + 4, lampY - 20);
  ctx.lineTo(lampX + 16, lampY - 14);
  ctx.lineTo(lampX + 13, lampY - 26);
  ctx.lineTo(lampX + 2, lampY - 26);
  ctx.closePath();
  ctx.fill();
  ctx.strokeStyle = "#C0A040";
  ctx.lineWidth = 0.8;
  ctx.beginPath();
  ctx.moveTo(lampX + 4, lampY - 20);
  ctx.lineTo(lampX + 16, lampY - 14);
  ctx.lineTo(lampX + 13, lampY - 26);
  ctx.lineTo(lampX + 2, lampY - 26);
  ctx.closePath();
  ctx.stroke();

  // Warm lamp glow (larger radius)
  const lampGlow = ctx.createRadialGradient(
    lampX + 10, lampY - 14, 0,
    lampX + 10, lampY - 14, 30,
  );
  lampGlow.addColorStop(0, "rgba(255, 200, 80, 0.15)");
  lampGlow.addColorStop(1, "rgba(255, 200, 80, 0)");
  ctx.fillStyle = lampGlow;
  ctx.fillRect(lampX - 12, lampY - 34, 50, 34);

  // ---- Role-specific item (1.5x larger) ----
  drawRoleItem(ctx, desk.label, x, y, dw, dh, frame);
}

/**
 * Draw a role-specific decorative item on the desk (1.5x larger than before).
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
  const ix = x - dw * 0.18;
  const iy = y - dh * 0.55;
  // Scale factor 1.5 applied to all dimensions below
  const S = 1.5;

  switch (label) {
    case "Dr. Bob": {
      // Clipboard — orchestrator (1.5x)
      ctx.fillStyle = "#D4C080";
      ctx.fillRect(ix - 9 * S, iy - 12 * S, 15 * S, 18 * S);
      ctx.fillStyle = "#8A6030";
      ctx.fillRect(ix - 6 * S, iy - 15 * S, 9 * S, 5 * S);
      // Clip circle
      ctx.fillStyle = "#A08040";
      ctx.beginPath();
      ctx.arc(ix - 1.5 * S, iy - 14.5 * S, 2 * S, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = "#A08040";
      ctx.lineWidth = 0.6;
      for (let li = 0; li < 4; li++) {
        ctx.beginPath();
        ctx.moveTo(ix - 7 * S, iy - 8 * S + li * 3.5 * S);
        ctx.lineTo(ix + 4 * S, iy - 8 * S + li * 3.5 * S);
        ctx.stroke();
      }
      break;
    }
    case "Scout": {
      // Globe — researcher (1.5x)
      const gr = 9 * S;
      ctx.strokeStyle = "#3B82F6";
      ctx.lineWidth = 1.2;
      ctx.beginPath();
      ctx.arc(ix, iy - gr, gr, 0, Math.PI * 2);
      ctx.stroke();
      ctx.beginPath();
      ctx.ellipse(ix, iy - gr, gr * 0.45, gr, 0, 0, Math.PI * 2);
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(ix - gr, iy - gr);
      ctx.lineTo(ix + gr, iy - gr);
      ctx.stroke();
      // Stand
      ctx.fillStyle = "#6A3A1A";
      ctx.fillRect(ix - 2, iy - 2, 4, 4);
      break;
    }
    case "Sage": {
      // Bar chart — analyst (1.5x)
      const barHeights2 = [7, 12, 9, 14, 10];
      for (let bi = 0; bi < barHeights2.length; bi++) {
        ctx.fillStyle = `rgba(139, 92, 246, ${0.45 + bi * 0.1})`;
        ctx.fillRect(ix - 10 + bi * 5, iy - barHeights2[bi], 4, barHeights2[bi]);
      }
      ctx.strokeStyle = "#6A3A8A";
      ctx.lineWidth = 0.7;
      ctx.strokeRect(ix - 12, iy - 16, 28, 17);
      // Axis lines
      ctx.strokeStyle = "#8B5CF680";
      ctx.lineWidth = 0.5;
      ctx.beginPath();
      ctx.moveTo(ix - 12, iy - 8);
      ctx.lineTo(ix + 16, iy - 8);
      ctx.stroke();
      break;
    }
    case "Quill": {
      // Ink pen + scroll — writer (1.5x)
      ctx.strokeStyle = "#10B981";
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.moveTo(ix - 6, iy - 3);
      ctx.lineTo(ix + 7, iy - 16);
      ctx.stroke();
      ctx.fillStyle = "#10B98180";
      ctx.beginPath();
      ctx.moveTo(ix + 7, iy - 16);
      ctx.lineTo(ix + 4, iy - 13);
      ctx.lineTo(ix + 10, iy - 13);
      ctx.closePath();
      ctx.fill();
      // Scroll (larger)
      ctx.fillStyle = "#D4C080";
      ctx.fillRect(ix - 12, iy - 12, 12, 12);
      ctx.strokeStyle = "#A08040";
      ctx.lineWidth = 0.6;
      for (let li = 0; li < 3; li++) {
        ctx.beginPath();
        ctx.moveTo(ix - 11, iy - 9 + li * 3.5);
        ctx.lineTo(ix - 1, iy - 9 + li * 3.5);
        ctx.stroke();
      }
      // Scroll rolled ends
      ctx.strokeStyle = "#C0A050";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.arc(ix - 12, iy - 6, 3, -Math.PI / 2, Math.PI / 2);
      ctx.stroke();
      ctx.beginPath();
      ctx.arc(ix, iy - 6, 3, Math.PI / 2, (3 * Math.PI) / 2);
      ctx.stroke();
      break;
    }
    case "Pixel": {
      // Second monitor — coder (1.5x)
      const m2X = ix + 8;
      const m2Y = iy - 21;
      ctx.fillStyle = "#111111";
      ctx.fillRect(m2X, m2Y, 21, 15);
      ctx.strokeStyle = "#333333";
      ctx.lineWidth = 1;
      ctx.strokeRect(m2X, m2Y, 21, 15);
      ctx.fillStyle = "#EC489918";
      ctx.fillRect(m2X + 1, m2Y + 1, 19, 13);
      // Code lines (more detail)
      const lineColors = ["#EC4899", "#3B82F6", "#22C55E", "#F59E0B"];
      for (let li = 0; li < 4; li++) {
        ctx.fillStyle = lineColors[li];
        ctx.fillRect(m2X + 2, m2Y + 2 + li * 3, 5 + (li % 3) * 5, 1.5);
      }
      // Stand
      ctx.fillStyle = "#555555";
      ctx.fillRect(m2X + 7, m2Y + 15, 8, 3);
      break;
    }
    case "Atlas": {
      // Toolbox — operator (1.5x)
      ctx.fillStyle = "#F59E0B";
      ctx.fillRect(ix - 10, iy - 12, 18, 12);
      ctx.strokeStyle = "#D48000";
      ctx.lineWidth = 0.8;
      ctx.strokeRect(ix - 10, iy - 12, 18, 12);
      // Box top
      ctx.fillStyle = "#D48000";
      ctx.fillRect(ix - 10, iy - 15, 18, 4);
      ctx.fillRect(ix - 6, iy - 17, 10, 3);
      // Handle
      ctx.strokeStyle = "#A06000";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.arc(ix - 1, iy - 16.5, 3.5, Math.PI, 0);
      ctx.stroke();
      // Latch
      ctx.fillStyle = "#1A1A1A";
      ctx.fillRect(ix - 3, iy - 11, 5, 3);
      // Tool handles peeking out
      ctx.fillStyle = "#FF4444";
      ctx.fillRect(ix - 7, iy - 14, 2, 7);
      ctx.fillStyle = "#4444FF";
      ctx.fillRect(ix + 5, iy - 14, 2, 5);
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
  // REST ZONE furniture: gridX 5-7 only.
  // Warm overlay tint for cozy feel
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

  // =========================================================================
  // SOFA — against the LEFT BACK WALL (grid 6, 0–1, pushed toward wall edge)
  // Width ~100px isometric, with cushions, armrests, throw pillows
  // =========================================================================
  const sofaPos = gridToScreen(6, 1, w, h);
  // Offset toward the top-left (wall direction) to make it look wall-hugging
  const sx = sofaPos.x - 14;
  const sy = sofaPos.y - 8;

  // Shadow under sofa
  ctx.save();
  ctx.globalAlpha = 0.2;
  ctx.fillStyle = "#000000";
  ctx.beginPath();
  ctx.ellipse(sx, sy + 4, 60, 12, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();

  // Sofa legs (draw first, behind everything)
  ctx.fillStyle = "#3A2A1A";
  ctx.fillRect(sx - 52, sy - 4, 6, 10);
  ctx.fillRect(sx + 46, sy - 4, 6, 10);
  ctx.fillRect(sx - 26, sy + 2, 5, 8);
  ctx.fillRect(sx + 21, sy + 2, 5, 8);

  // Armrests (drawn before seat/back so they appear as sides)
  ctx.fillStyle = "#1C3254";
  // Left armrest — 3 faces for 3D look
  ctx.beginPath();
  ctx.moveTo(sx - 52, sy - 44);
  ctx.lineTo(sx - 44, sy - 44);
  ctx.lineTo(sx - 44, sy + 6);
  ctx.lineTo(sx - 52, sy + 6);
  ctx.closePath();
  ctx.fill();
  ctx.strokeStyle = "#142840";
  ctx.lineWidth = 1;
  ctx.stroke();
  // Armrest top face
  ctx.fillStyle = "#263E5E";
  ctx.fillRect(sx - 52, sy - 46, 10, 4);
  ctx.strokeStyle = "#142840";
  ctx.lineWidth = 0.8;
  ctx.strokeRect(sx - 52, sy - 46, 10, 4);

  // Right armrest
  ctx.fillStyle = "#1C3254";
  ctx.beginPath();
  ctx.moveTo(sx + 44, sy - 44);
  ctx.lineTo(sx + 52, sy - 44);
  ctx.lineTo(sx + 52, sy + 6);
  ctx.lineTo(sx + 44, sy + 6);
  ctx.closePath();
  ctx.fill();
  ctx.strokeStyle = "#142840";
  ctx.lineWidth = 1;
  ctx.stroke();
  ctx.fillStyle = "#263E5E";
  ctx.fillRect(sx + 42, sy - 46, 10, 4);
  ctx.strokeStyle = "#142840";
  ctx.lineWidth = 0.8;
  ctx.strokeRect(sx + 42, sy - 46, 10, 4);

  // Sofa back (tall, prominent — represents wall-backing)
  ctx.fillStyle = "#243A60";
  ctx.fillRect(sx - 48, sy - 44, 96, 22);
  ctx.strokeStyle = "#142840";
  ctx.lineWidth = 1.2;
  ctx.strokeRect(sx - 48, sy - 44, 96, 22);
  // Back fabric texture (vertical channels)
  ctx.strokeStyle = "#1E3258";
  ctx.lineWidth = 0.5;
  for (let gi = -44; gi <= 44; gi += 14) {
    ctx.beginPath();
    ctx.moveTo(sx + gi, sy - 44);
    ctx.lineTo(sx + gi, sy - 22);
    ctx.stroke();
  }
  // Back top highlight
  ctx.fillStyle = "rgba(100, 160, 255, 0.06)";
  ctx.fillRect(sx - 48, sy - 44, 96, 5);

  // Sofa seat (wide, padded)
  ctx.fillStyle = "#1E3860";
  ctx.fillRect(sx - 46, sy - 22, 92, 20);
  ctx.strokeStyle = "#142840";
  ctx.lineWidth = 1;
  ctx.strokeRect(sx - 46, sy - 22, 92, 20);

  // 3 seat cushion sections
  const cushionColors = ["#2A4878", "#304878", "#2A4878"];
  for (let ci = 0; ci < 3; ci++) {
    ctx.fillStyle = cushionColors[ci];
    ctx.fillRect(sx - 44 + ci * 32, sy - 20, 30, 16);
    ctx.strokeStyle = "#1C3460";
    ctx.lineWidth = 0.8;
    ctx.strokeRect(sx - 44 + ci * 32, sy - 20, 30, 16);
    // Cushion centre seam
    ctx.strokeStyle = "#162C50";
    ctx.lineWidth = 0.5;
    ctx.beginPath();
    ctx.moveTo(sx - 29 + ci * 32, sy - 20);
    ctx.lineTo(sx - 29 + ci * 32, sy - 4);
    ctx.stroke();
  }

  // Back cushions (3 sections matching seat)
  for (let ci = 0; ci < 3; ci++) {
    ctx.fillStyle = "#2E4876";
    ctx.fillRect(sx - 44 + ci * 32, sy - 42, 30, 18);
    ctx.strokeStyle = "#1C3460";
    ctx.lineWidth = 0.8;
    ctx.strokeRect(sx - 44 + ci * 32, sy - 42, 30, 18);
    // Horizontal channel stitch
    ctx.strokeStyle = "#1A2E54";
    ctx.lineWidth = 0.5;
    ctx.beginPath();
    ctx.moveTo(sx - 44 + ci * 32, sy - 33);
    ctx.lineTo(sx - 14 + ci * 32, sy - 33);
    ctx.stroke();
  }

  // Throw pillow 1 (warm orange)
  ctx.save();
  ctx.shadowColor = "#C87040";
  ctx.shadowBlur = 3;
  ctx.fillStyle = "#D88040";
  ctx.beginPath();
  ctx.ellipse(sx - 22, sy - 14, 12, 8, 0.25, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();
  ctx.strokeStyle = "#A85020";
  ctx.lineWidth = 0.8;
  ctx.beginPath();
  ctx.ellipse(sx - 22, sy - 14, 12, 8, 0.25, 0, Math.PI * 2);
  ctx.stroke();
  // Pillow seam
  ctx.strokeStyle = "#B86030";
  ctx.lineWidth = 0.6;
  ctx.beginPath();
  ctx.ellipse(sx - 22, sy - 14, 7, 4, 0.25, 0, Math.PI * 2);
  ctx.stroke();

  // Throw pillow 2 (cool blue-gray)
  ctx.fillStyle = "#5A6090";
  ctx.beginPath();
  ctx.ellipse(sx + 22, sy - 13, 12, 8, -0.25, 0, Math.PI * 2);
  ctx.fill();
  ctx.strokeStyle = "#3A4070";
  ctx.lineWidth = 0.8;
  ctx.beginPath();
  ctx.ellipse(sx + 22, sy - 13, 12, 8, -0.25, 0, Math.PI * 2);
  ctx.stroke();

  // =========================================================================
  // GAME CONSOLE / ARCADE — against RIGHT BACK WALL (grid 7, 0–1)
  // Taller cabinet, bright screen, visible joystick + colored buttons
  // =========================================================================
  const arcPos = gridToScreen(7, 1, w, h);
  // Offset toward top-right wall
  const ax = arcPos.x + 10;
  const ay = arcPos.y - 4;

  // Shadow
  ctx.save();
  ctx.globalAlpha = 0.22;
  ctx.fillStyle = "#000000";
  ctx.beginPath();
  ctx.ellipse(ax, ay + 2, 22, 6, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();

  // Cabinet body (taller — 70px high)
  ctx.fillStyle = "#16102E";
  ctx.fillRect(ax - 20, ay - 70, 40, 70);
  ctx.strokeStyle = "#261C40";
  ctx.lineWidth = 1.5;
  ctx.strokeRect(ax - 20, ay - 70, 40, 70);

  // Side accent stripes (brighter)
  ctx.fillStyle = "#F97316";
  ctx.fillRect(ax - 20, ay - 70, 4, 70);
  ctx.fillRect(ax + 16, ay - 70, 4, 70);
  // Inner edge highlight
  ctx.fillStyle = "#FF9940";
  ctx.fillRect(ax - 16, ay - 70, 1, 70);
  ctx.fillRect(ax + 15, ay - 70, 1, 70);

  // "GAME" marquee panel with glow animation
  const marqueeGlow = 0.5 + 0.5 * Math.abs(Math.sin(frame * 0.06));
  ctx.fillStyle = "#0A0818";
  ctx.fillRect(ax - 18, ay - 70, 36, 14);
  ctx.save();
  ctx.shadowColor = "#F97316";
  ctx.shadowBlur = 8 * marqueeGlow;
  ctx.fillStyle = `rgba(249, 115, 22, ${0.85 + 0.15 * marqueeGlow})`;
  ctx.font = 'bold 8px ui-monospace, "SF Mono", Menlo, monospace';
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText("GAME", ax, ay - 63);
  ctx.restore();

  // Screen bezel (thick, dark)
  ctx.fillStyle = "#0A0620";
  ctx.fillRect(ax - 16, ay - 56, 32, 28);

  // Screen (bright, larger: 26×20)
  const arcScreenX = ax - 13;
  const arcScreenY = ay - 54;
  const arcScreenW = 26;
  const arcScreenH = 20;
  ctx.fillStyle = "#040810";
  ctx.fillRect(arcScreenX, arcScreenY, arcScreenW, arcScreenH);
  ctx.strokeStyle = "#3A2870";
  ctx.lineWidth = 1;
  ctx.strokeRect(arcScreenX, arcScreenY, arcScreenW, arcScreenH);
  // Screen ambient glow
  ctx.save();
  const screenGlow = ctx.createRadialGradient(
    arcScreenX + arcScreenW / 2, arcScreenY + arcScreenH / 2, 0,
    arcScreenX + arcScreenW / 2, arcScreenY + arcScreenH / 2, arcScreenW,
  );
  screenGlow.addColorStop(0, "rgba(34, 197, 94, 0.12)");
  screenGlow.addColorStop(1, "rgba(34, 197, 94, 0)");
  ctx.fillStyle = screenGlow;
  ctx.fillRect(arcScreenX, arcScreenY, arcScreenW, arcScreenH);
  ctx.restore();

  // Animated pixel sprite on screen
  ctx.save();
  ctx.shadowColor = "#22C55E";
  ctx.shadowBlur = 6;
  ctx.fillStyle = "#22C55E";
  const spriteX = arcScreenX + 2 + ((frame * 0.2) % (arcScreenW - 10)) | 0;
  // Sprite body
  ctx.fillRect(spriteX, arcScreenY + arcScreenH - 8, 7, 6);
  ctx.fillRect(spriteX + 1, arcScreenY + arcScreenH - 11, 5, 3);
  // Stars/score at top
  ctx.fillStyle = "#FFD700";
  ctx.font = '5px ui-monospace, "SF Mono", Menlo, monospace';
  ctx.textAlign = "left";
  ctx.textBaseline = "top";
  ctx.fillText("★ 99", arcScreenX + 2, arcScreenY + 2);
  // Enemy bullets
  ctx.fillStyle = "#EF4444";
  ctx.shadowColor = "#EF4444";
  for (let bi = 0; bi < 3; bi++) {
    const bx = arcScreenX + 3 + bi * 8;
    const bphase = (frame * 0.18 + bi * 2.5) % arcScreenH;
    ctx.fillRect(bx, arcScreenY + (bphase | 0), 2, 5);
  }
  ctx.restore();

  // Screen scanline overlay
  ctx.save();
  ctx.globalAlpha = 0.06;
  for (let sl = 0; sl < arcScreenH; sl += 3) {
    ctx.fillStyle = "#000000";
    ctx.fillRect(arcScreenX, arcScreenY + sl, arcScreenW, 1);
  }
  ctx.restore();

  // Control panel (angled)
  ctx.fillStyle = "#1E1838";
  ctx.fillRect(ax - 18, ay - 28, 36, 16);
  ctx.strokeStyle = "#2A2250";
  ctx.lineWidth = 1;
  ctx.strokeRect(ax - 18, ay - 28, 36, 16);

  // Joystick (left side of panel)
  ctx.fillStyle = "#3A3058";
  ctx.fillRect(ax - 14, ay - 27, 8, 11);
  ctx.fillStyle = "#201830";
  ctx.fillRect(ax - 14, ay - 27, 8, 3);
  ctx.fillStyle = "#7060A0";
  ctx.beginPath();
  ctx.arc(ax - 10, ay - 26, 5, 0, Math.PI * 2);
  ctx.fill();
  ctx.strokeStyle = "#4030708";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.arc(ax - 10, ay - 26, 5, 0, Math.PI * 2);
  ctx.stroke();
  // Joystick ball shine
  ctx.fillStyle = "rgba(180, 160, 255, 0.4)";
  ctx.beginPath();
  ctx.arc(ax - 11, ay - 27, 2, 0, Math.PI * 2);
  ctx.fill();

  // 4 colored buttons (right side)
  const btnColors = ["#EF4444", "#22C55E", "#F59E0B", "#3B82F6"];
  const btnPositions = [
    { bx: ax + 2, by: ay - 22 },
    { bx: ax + 9, by: ay - 25 },
    { bx: ax + 9, by: ay - 18 },
    { bx: ax + 16, by: ay - 22 },
  ];
  for (let bi = 0; bi < 4; bi++) {
    const { bx, by } = btnPositions[bi];
    // Glow halo
    ctx.fillStyle = btnColors[bi] + "40";
    ctx.beginPath();
    ctx.arc(bx, by, 5.5, 0, Math.PI * 2);
    ctx.fill();
    // Button body
    ctx.fillStyle = btnColors[bi];
    ctx.beginPath();
    ctx.arc(bx, by, 3.5, 0, Math.PI * 2);
    ctx.fill();
    // Button shine
    ctx.fillStyle = "rgba(255,255,255,0.35)";
    ctx.beginPath();
    ctx.arc(bx - 1, by - 1, 1.5, 0, Math.PI * 2);
    ctx.fill();
  }

  // Coin slot + speaker grille
  ctx.fillStyle = "#111028";
  ctx.fillRect(ax - 8, ay - 12, 16, 4);
  ctx.strokeStyle = "#2A2248";
  ctx.lineWidth = 0.5;
  ctx.strokeRect(ax - 8, ay - 12, 16, 4);
  // Speaker dots
  ctx.fillStyle = "#2A2248";
  for (let si = 0; si < 4; si++) {
    ctx.beginPath();
    ctx.arc(ax - 6 + si * 4, ay - 6, 1.2, 0, Math.PI * 2);
    ctx.fill();
  }

  // =========================================================================
  // POOL TABLE — CENTER of rest zone (grid 6, 3), LARGE (~120x80px isometric)
  // Dominant feature spanning ~4 grid tiles
  // =========================================================================
  const poolPos = gridToScreen(6, 3, w, h);
  const ptx = poolPos.x;
  const pty = poolPos.y;

  // Shadow under table
  ctx.save();
  ctx.globalAlpha = 0.25;
  ctx.fillStyle = "#000000";
  ctx.beginPath();
  ctx.moveTo(ptx, pty - 10);
  ctx.lineTo(ptx + 66, pty + 15);
  ctx.lineTo(ptx, pty + 40);
  ctx.lineTo(ptx - 66, pty + 15);
  ctx.closePath();
  ctx.fill();
  ctx.restore();

  // Table legs (visible front pair)
  ctx.fillStyle = "#5A3010";
  ctx.fillRect(ptx - 60, pty + 10, 8, 18);
  ctx.fillRect(ptx + 52, pty + 10, 8, 18);
  ctx.fillRect(ptx - 30, pty + 20, 6, 14);
  ctx.fillRect(ptx + 24, pty + 20, 6, 14);
  // Leg cross-brace
  ctx.strokeStyle = "#4A2808";
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(ptx - 56, pty + 22);
  ctx.lineTo(ptx + 56, pty + 22);
  ctx.stroke();

  // Outer body (dark wood — slightly larger than felt for rail effect)
  ctx.fillStyle = "#0A3A0A";
  ctx.beginPath();
  ctx.moveTo(ptx, pty - 42);
  ctx.lineTo(ptx + 62, pty - 11);
  ctx.lineTo(ptx, pty + 20);
  ctx.lineTo(ptx - 62, pty - 11);
  ctx.closePath();
  ctx.fill();

  // Wooden rails (rich brown, thick stroke)
  ctx.strokeStyle = "#7A5030";
  ctx.lineWidth = 7;
  ctx.lineJoin = "round";
  ctx.beginPath();
  ctx.moveTo(ptx, pty - 42);
  ctx.lineTo(ptx + 62, pty - 11);
  ctx.lineTo(ptx, pty + 20);
  ctx.lineTo(ptx - 62, pty - 11);
  ctx.closePath();
  ctx.stroke();
  // Rail inner highlight
  ctx.strokeStyle = "#9A6840";
  ctx.lineWidth = 1.5;
  ctx.beginPath();
  ctx.moveTo(ptx, pty - 36);
  ctx.lineTo(ptx + 56, pty - 7);
  ctx.lineTo(ptx, pty + 14);
  ctx.lineTo(ptx - 56, pty - 7);
  ctx.closePath();
  ctx.stroke();

  // Green felt top (richer saturated green #1A6A1A)
  ctx.fillStyle = "#1A6A1A";
  ctx.beginPath();
  ctx.moveTo(ptx, pty - 38);
  ctx.lineTo(ptx + 56, pty - 9);
  ctx.lineTo(ptx, pty + 16);
  ctx.lineTo(ptx - 56, pty - 9);
  ctx.closePath();
  ctx.fill();

  // Felt texture (diagonal grain lines)
  ctx.save();
  ctx.beginPath();
  ctx.moveTo(ptx, pty - 38);
  ctx.lineTo(ptx + 56, pty - 9);
  ctx.lineTo(ptx, pty + 16);
  ctx.lineTo(ptx - 56, pty - 9);
  ctx.closePath();
  ctx.clip();
  ctx.strokeStyle = "#1E7A1E";
  ctx.lineWidth = 0.6;
  for (let gi = -56; gi < 56; gi += 8) {
    ctx.beginPath();
    ctx.moveTo(ptx + gi, pty - 38);
    ctx.lineTo(ptx + gi + 24, pty + 16);
    ctx.stroke();
  }
  // Felt shine/highlight
  const feltShine = ctx.createLinearGradient(ptx - 20, pty - 35, ptx + 20, pty - 10);
  feltShine.addColorStop(0, "rgba(120, 255, 120, 0.08)");
  feltShine.addColorStop(0.5, "rgba(80, 200, 80, 0.04)");
  feltShine.addColorStop(1, "rgba(0, 0, 0, 0)");
  ctx.fillStyle = feltShine;
  ctx.fill();
  ctx.restore();

  // Center line (white stripe)
  ctx.strokeStyle = "rgba(255,255,255,0.18)";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(ptx + 28, pty - 23);
  ctx.lineTo(ptx - 28, pty + 3);
  ctx.stroke();
  // Center dot
  ctx.fillStyle = "rgba(255,255,255,0.25)";
  ctx.beginPath();
  ctx.arc(ptx, pty - 10, 3, 0, Math.PI * 2);
  ctx.fill();

  // Pockets (6 — dark circles with leather rim)
  ctx.fillStyle = "#050505";
  const pocketPositions = [
    { px: ptx, py: pty - 38 },           // top apex
    { px: ptx + 56, py: pty - 9 },       // right apex
    { px: ptx, py: pty + 16 },            // bottom apex
    { px: ptx - 56, py: pty - 9 },       // left apex
    { px: ptx + 28, py: pty - 24 },      // top-right mid
    { px: ptx - 28, py: pty - 24 },      // top-left mid
  ];
  for (const pp of pocketPositions) {
    // Leather pocket surround
    ctx.fillStyle = "#3A1A08";
    ctx.beginPath();
    ctx.arc(pp.px, pp.py, 6.5, 0, Math.PI * 2);
    ctx.fill();
    // Pocket hole
    ctx.fillStyle = "#020202";
    ctx.beginPath();
    ctx.arc(pp.px, pp.py, 5, 0, Math.PI * 2);
    ctx.fill();
  }

  // Billiard balls (6 colored balls in triangle formation)
  const ballColors = ["#F5F5F5", "#F97316", "#3B82F6", "#EF4444", "#8B5CF6", "#22C55E"];
  const ballPositions = [
    { bx: ptx - 8, by: pty - 18 },
    { bx: ptx + 4, by: pty - 22 },
    { bx: ptx - 18, by: pty - 14 },
    { bx: ptx + 14, by: pty - 14 },
    { bx: ptx - 4, by: pty - 9 },
    { bx: ptx + 10, by: pty - 4 },
  ];
  for (let bi = 0; bi < ballPositions.length; bi++) {
    const bp = ballPositions[bi];
    // Ball shadow
    ctx.fillStyle = "rgba(0,0,0,0.3)";
    ctx.beginPath();
    ctx.ellipse(bp.bx + 1, bp.by + 2, 5, 2.5, 0, 0, Math.PI * 2);
    ctx.fill();
    // Ball body
    ctx.fillStyle = ballColors[bi % ballColors.length];
    ctx.beginPath();
    ctx.arc(bp.bx, bp.by, 5, 0, Math.PI * 2);
    ctx.fill();
    // Ball stripe (for striped balls, alternating)
    if (bi % 2 === 1) {
      ctx.save();
      ctx.beginPath();
      ctx.arc(bp.bx, bp.by, 5, 0, Math.PI * 2);
      ctx.clip();
      ctx.fillStyle = "rgba(255,255,255,0.25)";
      ctx.fillRect(bp.bx - 5, bp.by - 1.5, 10, 3);
      ctx.restore();
    }
    // Shine dot
    ctx.fillStyle = "rgba(255, 255, 255, 0.65)";
    ctx.beginPath();
    ctx.arc(bp.bx - 1.5, bp.by - 1.5, 2, 0, Math.PI * 2);
    ctx.fill();
  }

  // =========================================================================
  // WATER COOLER — near right wall, between arcade and bookshelf (grid 7, 2)
  // Flush with wall area, standard size
  // =========================================================================
  const wcPos = gridToScreen(7, 2, w, h);
  const wx2 = wcPos.x + 6;
  const wy2 = wcPos.y - 2;

  // Shadow
  ctx.save();
  ctx.globalAlpha = 0.2;
  ctx.fillStyle = "#000000";
  ctx.beginPath();
  ctx.ellipse(wx2, wy2 + 2, 12, 4, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();

  // Stand body (solid, wider base)
  ctx.fillStyle = "#2C2C2C";
  ctx.fillRect(wx2 - 7, wy2 - 24, 14, 24);
  ctx.strokeStyle = "#1A1A1A";
  ctx.lineWidth = 1;
  ctx.strokeRect(wx2 - 7, wy2 - 24, 14, 24);
  // Stand base plate
  ctx.fillStyle = "#383838";
  ctx.fillRect(wx2 - 10, wy2 - 4, 20, 5);
  ctx.strokeStyle = "#1A1A1A";
  ctx.lineWidth = 0.8;
  ctx.strokeRect(wx2 - 10, wy2 - 4, 20, 5);

  // Blue water bottle (translucent look)
  ctx.fillStyle = "#1A58A8";
  ctx.fillRect(wx2 - 8, wy2 - 52, 16, 28);
  ctx.strokeStyle = "#0E3870";
  ctx.lineWidth = 1;
  ctx.strokeRect(wx2 - 8, wy2 - 52, 16, 28);
  // Bottle shine (left highlight)
  ctx.fillStyle = "rgba(100, 180, 255, 0.25)";
  ctx.fillRect(wx2 - 7, wy2 - 51, 4, 26);
  // Water level line
  ctx.strokeStyle = "rgba(100, 160, 255, 0.4)";
  ctx.lineWidth = 0.8;
  ctx.beginPath();
  ctx.moveTo(wx2 - 7, wy2 - 36);
  ctx.lineTo(wx2 + 7, wy2 - 36);
  ctx.stroke();
  // Bottle cap
  ctx.fillStyle = "#4A9ADA";
  ctx.fillRect(wx2 - 5, wy2 - 57, 10, 6);
  ctx.strokeStyle = "#2A6AB0";
  ctx.lineWidth = 0.8;
  ctx.strokeRect(wx2 - 5, wy2 - 57, 10, 6);

  // Bubbles inside bottle (rising animation)
  ctx.fillStyle = "rgba(160, 210, 255, 0.6)";
  for (let bi = 0; bi < 4; bi++) {
    const bubblePhase = (frame * 0.04 + bi * 1.8) % 24;
    ctx.beginPath();
    ctx.arc(wx2 - 4 + bi * 3, wy2 - 28 - bubblePhase, 1.8, 0, Math.PI * 2);
    ctx.fill();
  }

  // Dispense tap area (panel)
  ctx.fillStyle = "#1E1E1E";
  ctx.fillRect(wx2 - 6, wy2 - 22, 12, 10);
  // Dispense buttons (blue = cold, red = hot)
  ctx.fillStyle = "#2060E0";
  ctx.beginPath();
  ctx.arc(wx2 - 3, wy2 - 16, 3, 0, Math.PI * 2);
  ctx.fill();
  ctx.fillStyle = "rgba(32, 96, 224, 0.4)";
  ctx.beginPath();
  ctx.arc(wx2 - 3, wy2 - 16, 4.5, 0, Math.PI * 2);
  ctx.fill();
  ctx.fillStyle = "#E04020";
  ctx.beginPath();
  ctx.arc(wx2 + 3, wy2 - 16, 3, 0, Math.PI * 2);
  ctx.fill();
  ctx.fillStyle = "rgba(224, 64, 32, 0.4)";
  ctx.beginPath();
  ctx.arc(wx2 + 3, wy2 - 16, 4.5, 0, Math.PI * 2);
  ctx.fill();

  // =========================================================================
  // PLANT 1 — Corner where walls meet floor (grid 5, 0 — top corner)
  // Taller fern-style plant with multiple leaf clusters
  // =========================================================================
  const plantPos = gridToScreen(5, 0, w, h);
  const px2 = plantPos.x - 6;
  const py2 = plantPos.y + 4;

  // Shadow
  ctx.save();
  ctx.globalAlpha = 0.18;
  ctx.fillStyle = "#000000";
  ctx.beginPath();
  ctx.ellipse(px2, py2 + 2, 14, 5, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();

  // Pot (terracotta, wider rim)
  ctx.fillStyle = "#9A4822";
  ctx.beginPath();
  ctx.moveTo(px2 - 10, py2 - 4);
  ctx.lineTo(px2 + 10, py2 - 4);
  ctx.lineTo(px2 + 8, py2 + 10);
  ctx.lineTo(px2 - 8, py2 + 10);
  ctx.closePath();
  ctx.fill();
  ctx.strokeStyle = "#6A2E10";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(px2 - 10, py2 - 4);
  ctx.lineTo(px2 + 10, py2 - 4);
  ctx.lineTo(px2 + 8, py2 + 10);
  ctx.lineTo(px2 - 8, py2 + 10);
  ctx.closePath();
  ctx.stroke();
  // Pot rim (wider)
  ctx.fillStyle = "#B05830";
  ctx.fillRect(px2 - 12, py2 - 8, 24, 5);
  ctx.strokeStyle = "#6A2E10";
  ctx.lineWidth = 0.8;
  ctx.strokeRect(px2 - 12, py2 - 8, 24, 5);
  // Soil
  ctx.fillStyle = "#3A2010";
  ctx.fillRect(px2 - 9, py2 - 4, 18, 3);

  // Main stem (tall)
  const stemSway = Math.sin(frame * 0.018) * 1.5;
  ctx.strokeStyle = "#1A6022";
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(px2, py2 - 5);
  ctx.quadraticCurveTo(px2 + stemSway * 2, py2 - 25, px2 + stemSway, py2 - 46);
  ctx.stroke();

  // Secondary stems
  ctx.strokeStyle = "#1C6824";
  ctx.lineWidth = 1.5;
  ctx.beginPath();
  ctx.moveTo(px2, py2 - 12);
  ctx.quadraticCurveTo(px2 - 8 + stemSway, py2 - 28, px2 - 16 + stemSway, py2 - 38);
  ctx.stroke();
  ctx.beginPath();
  ctx.moveTo(px2, py2 - 10);
  ctx.quadraticCurveTo(px2 + 8 + stemSway, py2 - 26, px2 + 14 + stemSway, py2 - 36);
  ctx.stroke();
  ctx.lineWidth = 1.2;
  ctx.beginPath();
  ctx.moveTo(px2, py2 - 18);
  ctx.quadraticCurveTo(px2 - 5 + stemSway, py2 - 30, px2 - 10 + stemSway, py2 - 44);
  ctx.stroke();
  ctx.beginPath();
  ctx.moveTo(px2, py2 - 20);
  ctx.quadraticCurveTo(px2 + 5 + stemSway, py2 - 30, px2 + 10 + stemSway, py2 - 42);
  ctx.stroke();

  // Leaf clusters (larger ellipses, multiple colors for depth)
  const leafData = [
    { lx: px2 + stemSway, ly: py2 - 46, rx: 12, ry: 7, rot: -0.2, col: "#22A040" },
    { lx: px2 - 16 + stemSway, ly: py2 - 38, rx: 11, ry: 6, rot: -0.9, col: "#28B848" },
    { lx: px2 + 14 + stemSway, ly: py2 - 36, rx: 11, ry: 6, rot: 0.7, col: "#24A844" },
    { lx: px2 - 10 + stemSway, ly: py2 - 44, rx: 9, ry: 5, rot: -0.6, col: "#1E9038" },
    { lx: px2 + 10 + stemSway, ly: py2 - 42, rx: 9, ry: 5, rot: 0.5, col: "#1E9038" },
    { lx: px2, ly: py2 - 34, rx: 10, ry: 6, rot: -0.1, col: "#2AAC46" },
  ];
  for (const lf of leafData) {
    ctx.save();
    ctx.translate(lf.lx, lf.ly);
    ctx.rotate(lf.rot);
    ctx.fillStyle = lf.col;
    ctx.beginPath();
    ctx.ellipse(0, -lf.ry, lf.rx, lf.ry, 0, 0, Math.PI * 2);
    ctx.fill();
    // Leaf vein
    ctx.strokeStyle = "rgba(0, 80, 20, 0.4)";
    ctx.lineWidth = 0.5;
    ctx.beginPath();
    ctx.moveTo(0, 0);
    ctx.lineTo(0, -lf.ry * 1.8);
    ctx.stroke();
    ctx.restore();
  }

  // =========================================================================
  // PLANT 2 — Bottom-right corner of rest zone (grid 7, 5)
  // Larger cactus/fern variety
  // =========================================================================
  const cactusPos = gridToScreen(7, 5, w, h);
  const cpx = cactusPos.x + 4;
  const cpy = cactusPos.y;

  // Shadow
  ctx.save();
  ctx.globalAlpha = 0.18;
  ctx.fillStyle = "#000000";
  ctx.beginPath();
  ctx.ellipse(cpx, cpy + 2, 14, 5, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();

  // Cactus pot (terracotta, slightly bigger)
  ctx.fillStyle = "#A04A24";
  ctx.beginPath();
  ctx.moveTo(cpx - 10, cpy - 4);
  ctx.lineTo(cpx + 10, cpy - 4);
  ctx.lineTo(cpx + 8, cpy + 10);
  ctx.lineTo(cpx - 8, cpy + 10);
  ctx.closePath();
  ctx.fill();
  ctx.strokeStyle = "#6A2E10";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(cpx - 10, cpy - 4);
  ctx.lineTo(cpx + 10, cpy - 4);
  ctx.lineTo(cpx + 8, cpy + 10);
  ctx.lineTo(cpx - 8, cpy + 10);
  ctx.closePath();
  ctx.stroke();
  ctx.fillStyle = "#C06030";
  ctx.fillRect(cpx - 12, cpy - 8, 24, 5);
  ctx.strokeStyle = "#6A2E10";
  ctx.lineWidth = 0.8;
  ctx.strokeRect(cpx - 12, cpy - 8, 24, 5);

  // Cactus body (tall trunk, ribbed)
  ctx.fillStyle = "#2E7028";
  ctx.fillRect(cpx - 6, cpy - 36, 12, 28);
  ctx.strokeStyle = "#1A5018";
  ctx.lineWidth = 0.8;
  ctx.strokeRect(cpx - 6, cpy - 36, 12, 28);
  // Rib lines
  ctx.strokeStyle = "#367830";
  ctx.lineWidth = 0.5;
  for (let ri = 0; ri < 3; ri++) {
    ctx.beginPath();
    ctx.moveTo(cpx - 4 + ri * 4, cpy - 36);
    ctx.lineTo(cpx - 4 + ri * 4, cpy - 8);
    ctx.stroke();
  }
  // Top dome
  ctx.fillStyle = "#2E7028";
  ctx.beginPath();
  ctx.arc(cpx, cpy - 36, 6, Math.PI, 0);
  ctx.fill();

  // Left arm (larger)
  ctx.fillStyle = "#2E7028";
  ctx.fillRect(cpx - 16, cpy - 30, 11, 6);
  ctx.fillRect(cpx - 16, cpy - 38, 7, 9);
  ctx.strokeStyle = "#1A5018";
  ctx.lineWidth = 0.5;
  ctx.strokeRect(cpx - 16, cpy - 30, 11, 6);
  ctx.strokeRect(cpx - 16, cpy - 38, 7, 9);

  // Right arm (larger)
  ctx.fillStyle = "#2E7028";
  ctx.fillRect(cpx + 5, cpy - 28, 11, 6);
  ctx.fillRect(cpx + 9, cpy - 36, 7, 9);
  ctx.strokeStyle = "#1A5018";
  ctx.lineWidth = 0.5;
  ctx.strokeRect(cpx + 5, cpy - 28, 11, 6);
  ctx.strokeRect(cpx + 9, cpy - 36, 7, 9);

  // Cactus spines (longer, more visible)
  ctx.strokeStyle = "#A0CC80";
  ctx.lineWidth = 0.8;
  const spinePositions = [
    { x: cpx - 6, y: cpy - 32 }, { x: cpx + 6, y: cpy - 30 },
    { x: cpx - 6, y: cpy - 24 }, { x: cpx + 6, y: cpy - 22 },
    { x: cpx - 6, y: cpy - 16 }, { x: cpx + 6, y: cpy - 14 },
    { x: cpx - 16, y: cpy - 36 }, { x: cpx + 16, y: cpy - 34 },
    { x: cpx, y: cpy - 36 },
  ];
  for (const sp of spinePositions) {
    ctx.beginPath();
    ctx.moveTo(sp.x, sp.y);
    ctx.lineTo(sp.x + (sp.x < cpx ? -5 : 5), sp.y - 3);
    ctx.stroke();
  }

  // Flower on top (small pink bloom)
  ctx.fillStyle = "#FF88AA";
  for (let fi = 0; fi < 5; fi++) {
    const fangle = (fi / 5) * Math.PI * 2;
    ctx.beginPath();
    ctx.ellipse(
      cpx + Math.cos(fangle) * 4, cpy - 38 + Math.sin(fangle) * 3,
      3, 2, fangle, 0, Math.PI * 2,
    );
    ctx.fill();
  }
  ctx.fillStyle = "#FFE040";
  ctx.beginPath();
  ctx.arc(cpx, cpy - 38, 3, 0, Math.PI * 2);
  ctx.fill();
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

/**
 * Draw a pixel-art agent character + status indicators + label.
 * If agentPos is provided the character is rendered at the movement position
 * rather than fixed at the desk. The label always shows near the desk.
 */
function drawAgent(
  ctx: CanvasRenderingContext2D,
  desk: DeskPosition,
  state: AgentRenderState,
  w: number,
  h: number,
  agentPos?: AgentPosition,
): void {
  const { x: deskX, y: deskY } = gridToScreen(desk.gridX, desk.gridY, w, h);
  const { agent, frame } = state;

  // Use movement position if provided; otherwise fall back to desk position
  const charBaseX = agentPos ? agentPos.x : deskX;
  const charBaseY = agentPos ? agentPos.y : deskY;

  // Pulsing alpha for "thinking"
  let alpha = 1;
  if (agent.status === "thinking") {
    alpha = 0.5 + 0.5 * Math.abs(Math.sin(frame * 0.08));
  }

  // Choose animation state — use walking when moving
  const displayStatus = agentPos?.isMoving ? "executing" : agent.status;
  // Detect rest-zone idle for mood bubbles
  const inRestZone =
    agentPos !== undefined &&
    agentPos.currentLocation !== "desk" &&
    agentPos.currentLocation !== "walking" &&
    agent.status === "idle";

  // Position the pixel character above the floor position
  const charY = charBaseY - TILE_H * 0.6;

  ctx.save();
  ctx.globalAlpha = alpha;

  // Draw pixel-art character at 1.3x scale for improved visibility
  drawPixelCharacter(ctx, charBaseX, charY, 1.3, agent.role, displayStatus, frame, agent.id, inRestZone);

  ctx.restore();

  // D1D-221: Error indicator — red exclamation triangle (positioned above character)
  if (agent.status === "error") {
    drawErrorIndicator(ctx, charBaseX, charY - 28, 10, frame);
  }

  // D1D-221: Paused indicator — yellow speech bubble
  if (agent.status === "paused") {
    drawPausedIndicator(ctx, charBaseX, charY - 28, 10, frame);
  }

  // Name label — stays near the desk (not the moving character)
  const labelY = deskY + TILE_H * 0.8;
  const labelX = deskX;

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
 * @param agentPositions - Optional map of agentId -> AgentPosition for movement tracking.
 */
export function drawOffice(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  desks: DeskPosition[],
  agents: Map<string, AgentRenderState>,
  frame?: number,
  agentPositions?: Map<string, AgentPosition>,
): void {
  const f = frame ?? 0;

  // Atmospheric background: deep dark base for the floor area
  ctx.fillStyle = "#080810";
  ctx.fillRect(0, 0, w, h);

  // 3D isometric room walls — draws exterior + two colored wall planes + decorations
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

  // Agents (drawn on top of everything) — use movement positions when available
  for (const desk of desks) {
    if (desk.agentId) {
      const state = agents.get(desk.agentId);
      if (state) {
        const agentPos = agentPositions?.get(desk.agentId);
        drawAgent(ctx, desk, state, w, h, agentPos);
      }
    }
  }
}

// Re-export movement types + helpers so IsometricCanvas can use them
export { updateAgentPosition, createAgentPosition };
export type { AgentPosition };
