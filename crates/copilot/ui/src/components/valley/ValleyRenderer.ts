/**
 * ValleyRenderer — Pure Canvas 2D rendering functions for the Cowork Valley
 * landscape view showing 6 office buildings in an isometric layout.
 *
 * Each building represents one agent's office. Active buildings are colorful
 * with lit windows; locked buildings are dimmed with a lock overlay.
 */

import { AGENT_COLORS } from "../office/OfficeRenderer";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface OfficeBuilding {
  id: string;
  name: string;
  agentName: string;
  agentRole: string;
  gridX: number;
  gridY: number;
  isActive: boolean;
  isSelected: boolean;
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

export const VALLEY_LAYOUT: OfficeBuilding[] = [
  {
    id: "office-1",
    name: "Dr. Bob's Office",
    agentName: "Dr. Bob",
    agentRole: "orchestrator",
    gridX: 4,
    gridY: 1,
    isActive: true,
    isSelected: false,
  },
  {
    id: "office-2",
    name: "Scout's Lab",
    agentName: "Scout",
    agentRole: "researcher",
    gridX: 2,
    gridY: 3,
    isActive: false,
    isSelected: false,
  },
  {
    id: "office-3",
    name: "Sage's Studio",
    agentName: "Sage",
    agentRole: "analyst",
    gridX: 6,
    gridY: 3,
    isActive: false,
    isSelected: false,
  },
  {
    id: "office-4",
    name: "Quill's Den",
    agentName: "Quill",
    agentRole: "writer",
    gridX: 1,
    gridY: 5,
    isActive: false,
    isSelected: false,
  },
  {
    id: "office-5",
    name: "Pixel's Lab",
    agentName: "Pixel",
    agentRole: "coder",
    gridX: 5,
    gridY: 5,
    isActive: false,
    isSelected: false,
  },
  {
    id: "office-6",
    name: "Atlas Ops",
    agentName: "Atlas",
    agentRole: "operator",
    gridX: 3,
    gridY: 7,
    isActive: false,
    isSelected: false,
  },
];

/** Valley isometric tile sizes (larger than office tiles for the landscape). */
const V_TILE_W = 60;
const V_TILE_H = 30;

/** Building dimensions in pixels. */
const BUILDING_W = 80;
const BUILDING_H = 60;
const ROOF_H = 30;

/** Colors */
const GROUND_COLOR = "#0A1A0A";
const GROUND_LINE_COLOR = "#0F2A0F";
const PATH_COLOR = "#1A1A1A";
const PATH_LINE_COLOR = "#242424";
const LOCKED_FILL = "#1A1A1A";
const LOCK_COLOR = "#F59E0B";
const BUILDING_OUTLINE = "#242424";
const LABEL_ACTIVE = "#E5E5E5";
const LABEL_LOCKED = "#555555";

// ---------------------------------------------------------------------------
// Coordinate helpers
// ---------------------------------------------------------------------------

/** Number of grid columns / rows for the valley landscape. */
const V_GRID_COLS = 9;
const V_GRID_ROWS = 9;

function valleyGridToScreen(
  col: number,
  row: number,
  canvasW: number,
  canvasH: number,
): { x: number; y: number } {
  const offsetX = canvasW / 2;
  const offsetY = canvasH / 2 - (V_GRID_ROWS * V_TILE_H) / 2 + 20;

  const x = (col - row) * V_TILE_W + offsetX;
  const y = (col + row) * V_TILE_H + offsetY;
  return { x, y };
}

/** Get the screen position for a building (centered on its grid cell). */
function buildingScreenPos(
  building: OfficeBuilding,
  canvasW: number,
  canvasH: number,
): { x: number; y: number } {
  return valleyGridToScreen(building.gridX, building.gridY, canvasW, canvasH);
}

// ---------------------------------------------------------------------------
// Color helpers
// ---------------------------------------------------------------------------

function darken(hex: string, amount: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  const factor = 1 - amount;
  return `rgb(${Math.floor(r * factor)}, ${Math.floor(g * factor)}, ${Math.floor(b * factor)})`;
}

function withAlpha(hex: string, alpha: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

// ---------------------------------------------------------------------------
// Drawing primitives
// ---------------------------------------------------------------------------

/** Draw a ground tile (grass). */
function drawGrassTile(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
): void {
  ctx.beginPath();
  ctx.moveTo(cx, cy - V_TILE_H);
  ctx.lineTo(cx + V_TILE_W, cy);
  ctx.lineTo(cx, cy + V_TILE_H);
  ctx.lineTo(cx - V_TILE_W, cy);
  ctx.closePath();
  ctx.fillStyle = GROUND_COLOR;
  ctx.fill();
  ctx.strokeStyle = GROUND_LINE_COLOR;
  ctx.lineWidth = 0.5;
  ctx.stroke();
}

/** Draw the ground grid. */
export function drawGround(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
): void {
  for (let row = 0; row < V_GRID_ROWS; row++) {
    for (let col = 0; col < V_GRID_COLS; col++) {
      const { x, y } = valleyGridToScreen(col, row, w, h);
      drawGrassTile(ctx, x, y);
    }
  }
}

/** Path segment between two grid positions. */
function drawPathSegment(
  ctx: CanvasRenderingContext2D,
  x1: number,
  y1: number,
  x2: number,
  y2: number,
): void {
  ctx.beginPath();
  ctx.moveTo(x1, y1);
  ctx.lineTo(x2, y2);
  ctx.strokeStyle = PATH_LINE_COLOR;
  ctx.lineWidth = 6;
  ctx.stroke();

  ctx.beginPath();
  ctx.moveTo(x1, y1);
  ctx.lineTo(x2, y2);
  ctx.strokeStyle = PATH_COLOR;
  ctx.lineWidth = 4;
  ctx.stroke();
}

/** Draw connecting paths between buildings. */
export function drawPaths(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
): void {
  const positions = VALLEY_LAYOUT.map((b) => buildingScreenPos(b, w, h));

  // Connect: Dr. Bob (0) -> Scout (1), Dr. Bob (0) -> Sage (2)
  drawPathSegment(ctx, positions[0].x, positions[0].y, positions[1].x, positions[1].y);
  drawPathSegment(ctx, positions[0].x, positions[0].y, positions[2].x, positions[2].y);
  // Scout (1) -> Quill (3)
  drawPathSegment(ctx, positions[1].x, positions[1].y, positions[3].x, positions[3].y);
  // Sage (2) -> Pixel (4)
  drawPathSegment(ctx, positions[2].x, positions[2].y, positions[4].x, positions[4].y);
  // Quill (3) -> Atlas (5)
  drawPathSegment(ctx, positions[3].x, positions[3].y, positions[5].x, positions[5].y);
  // Pixel (4) -> Atlas (5)
  drawPathSegment(ctx, positions[4].x, positions[4].y, positions[5].x, positions[5].y);
}

/** Draw an isometric building. */
export function drawBuilding(
  ctx: CanvasRenderingContext2D,
  building: OfficeBuilding,
  w: number,
  h: number,
  frame: number,
): void {
  const { x, y } = buildingScreenPos(building, w, h);
  const roleColor = AGENT_COLORS[building.agentRole] ?? "#6B7280";

  const halfW = BUILDING_W / 2;
  const baseY = y;

  // Hover/selection glow
  if (building.isSelected) {
    ctx.save();
    ctx.shadowColor = withAlpha(roleColor, 0.6);
    ctx.shadowBlur = 20;
  }

  if (building.isActive) {
    // --- Active building ---
    const wallColor = darken(roleColor, 0.7);
    const sideColor = darken(roleColor, 0.8);
    const roofColor = darken(roleColor, 0.4);

    // Front wall
    ctx.beginPath();
    ctx.moveTo(x - halfW, baseY - BUILDING_H);
    ctx.lineTo(x, baseY - BUILDING_H + halfW * 0.4);
    ctx.lineTo(x, baseY + halfW * 0.4);
    ctx.lineTo(x - halfW, baseY);
    ctx.closePath();
    ctx.fillStyle = wallColor;
    ctx.fill();
    ctx.strokeStyle = BUILDING_OUTLINE;
    ctx.lineWidth = 1;
    ctx.stroke();

    // Side wall
    ctx.beginPath();
    ctx.moveTo(x, baseY - BUILDING_H + halfW * 0.4);
    ctx.lineTo(x + halfW, baseY - BUILDING_H);
    ctx.lineTo(x + halfW, baseY);
    ctx.lineTo(x, baseY + halfW * 0.4);
    ctx.closePath();
    ctx.fillStyle = sideColor;
    ctx.fill();
    ctx.strokeStyle = BUILDING_OUTLINE;
    ctx.lineWidth = 1;
    ctx.stroke();

    // Roof
    ctx.beginPath();
    ctx.moveTo(x, baseY - BUILDING_H - ROOF_H);
    ctx.lineTo(x + halfW, baseY - BUILDING_H);
    ctx.lineTo(x, baseY - BUILDING_H + halfW * 0.4);
    ctx.lineTo(x - halfW, baseY - BUILDING_H);
    ctx.closePath();
    ctx.fillStyle = roofColor;
    ctx.fill();
    ctx.strokeStyle = BUILDING_OUTLINE;
    ctx.lineWidth = 1;
    ctx.stroke();

    // Window (lit, on front wall)
    const winX = x - halfW * 0.55;
    const winY = baseY - BUILDING_H * 0.55;
    const winW = 14;
    const winH = 10;
    const windowGlow = 0.6 + 0.2 * Math.sin(frame * 0.04);

    ctx.fillStyle = withAlpha(roleColor, windowGlow);
    ctx.fillRect(winX, winY, winW, winH);
    ctx.strokeStyle = withAlpha(roleColor, 0.8);
    ctx.lineWidth = 0.5;
    ctx.strokeRect(winX, winY, winW, winH);

    // Cross-bar in window
    ctx.beginPath();
    ctx.moveTo(winX + winW / 2, winY);
    ctx.lineTo(winX + winW / 2, winY + winH);
    ctx.moveTo(winX, winY + winH / 2);
    ctx.lineTo(winX + winW, winY + winH / 2);
    ctx.strokeStyle = withAlpha(roleColor, 0.3);
    ctx.lineWidth = 0.5;
    ctx.stroke();

    // Mini agent sprite visible in window (small colored square)
    const spriteX = winX + winW / 2 - 3;
    const spriteY = winY + winH / 2 - 2;
    const bobOffset = Math.sin(frame * 0.06) * 1;
    ctx.fillStyle = roleColor;
    ctx.fillRect(spriteX, spriteY + bobOffset, 6, 5);
    // Head
    ctx.fillStyle = withAlpha(roleColor, 0.9);
    ctx.fillRect(spriteX + 1, spriteY - 3 + bobOffset, 4, 4);

    // Side window
    const swinX = x + halfW * 0.15;
    const swinY = baseY - BUILDING_H * 0.55;
    ctx.fillStyle = withAlpha(roleColor, windowGlow * 0.7);
    ctx.fillRect(swinX, swinY, winW - 2, winH);
    ctx.strokeStyle = withAlpha(roleColor, 0.5);
    ctx.lineWidth = 0.5;
    ctx.strokeRect(swinX, swinY, winW - 2, winH);
  } else {
    // --- Locked building ---
    ctx.save();
    ctx.globalAlpha = 0.5;

    // Front wall
    ctx.beginPath();
    ctx.moveTo(x - halfW, baseY - BUILDING_H);
    ctx.lineTo(x, baseY - BUILDING_H + halfW * 0.4);
    ctx.lineTo(x, baseY + halfW * 0.4);
    ctx.lineTo(x - halfW, baseY);
    ctx.closePath();
    ctx.fillStyle = LOCKED_FILL;
    ctx.fill();
    ctx.strokeStyle = BUILDING_OUTLINE;
    ctx.lineWidth = 1;
    ctx.stroke();

    // Side wall
    ctx.beginPath();
    ctx.moveTo(x, baseY - BUILDING_H + halfW * 0.4);
    ctx.lineTo(x + halfW, baseY - BUILDING_H);
    ctx.lineTo(x + halfW, baseY);
    ctx.lineTo(x, baseY + halfW * 0.4);
    ctx.closePath();
    ctx.fillStyle = "#111111";
    ctx.fill();
    ctx.strokeStyle = BUILDING_OUTLINE;
    ctx.lineWidth = 1;
    ctx.stroke();

    // Roof
    ctx.beginPath();
    ctx.moveTo(x, baseY - BUILDING_H - ROOF_H);
    ctx.lineTo(x + halfW, baseY - BUILDING_H);
    ctx.lineTo(x, baseY - BUILDING_H + halfW * 0.4);
    ctx.lineTo(x - halfW, baseY - BUILDING_H);
    ctx.closePath();
    ctx.fillStyle = "#151515";
    ctx.fill();
    ctx.strokeStyle = BUILDING_OUTLINE;
    ctx.lineWidth = 1;
    ctx.stroke();

    // Dark window
    const winX = x - halfW * 0.55;
    const winY = baseY - BUILDING_H * 0.55;
    ctx.fillStyle = "#0A0A0A";
    ctx.fillRect(winX, winY, 14, 10);
    ctx.strokeStyle = "#222222";
    ctx.lineWidth = 0.5;
    ctx.strokeRect(winX, winY, 14, 10);

    ctx.restore();

    // Lock icon overlay (drawn at full opacity)
    const lockX = x - 6;
    const lockY = baseY - BUILDING_H * 0.5 - 8;

    // Lock body
    ctx.fillStyle = LOCK_COLOR;
    ctx.fillRect(lockX - 1, lockY + 4, 14, 10);
    ctx.strokeStyle = darken("#F59E0B", 0.3);
    ctx.lineWidth = 1;
    ctx.strokeRect(lockX - 1, lockY + 4, 14, 10);

    // Lock shackle (arc)
    ctx.beginPath();
    ctx.arc(lockX + 6, lockY + 4, 5, Math.PI, 0);
    ctx.strokeStyle = LOCK_COLOR;
    ctx.lineWidth = 2;
    ctx.stroke();

    // Keyhole
    ctx.fillStyle = darken("#F59E0B", 0.5);
    ctx.beginPath();
    ctx.arc(lockX + 6, lockY + 10, 2, 0, Math.PI * 2);
    ctx.fill();
  }

  if (building.isSelected) {
    ctx.restore();
  }

  // Name label
  ctx.fillStyle = building.isActive ? LABEL_ACTIVE : LABEL_LOCKED;
  ctx.font = '11px ui-monospace, "SF Mono", Menlo, monospace';
  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  ctx.fillText(building.name, x, baseY + halfW * 0.4 + 6);

  // Role label (smaller, below name)
  ctx.fillStyle = building.isActive
    ? withAlpha(AGENT_COLORS[building.agentRole] ?? "#6B7280", 0.7)
    : "#333333";
  ctx.font = '9px ui-monospace, "SF Mono", Menlo, monospace';
  ctx.fillText(building.agentRole, x, baseY + halfW * 0.4 + 20);
}

/** Draw decorations: trees, lamp posts, benches, clouds. */
export function drawDecorations(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  frame: number,
): void {
  // Trees at various positions
  const treePositions = [
    { col: 0, row: 0 },
    { col: 8, row: 0 },
    { col: 0, row: 8 },
    { col: 8, row: 8 },
    { col: 3, row: 0 },
    { col: 7, row: 2 },
    { col: 0, row: 4 },
    { col: 8, row: 6 },
  ];

  for (const pos of treePositions) {
    const { x, y } = valleyGridToScreen(pos.col, pos.row, w, h);
    drawTree(ctx, x, y, frame);
  }

  // Lamp posts near paths
  const lampPositions = [
    { col: 3, row: 2 },
    { col: 5, row: 2 },
    { col: 2, row: 6 },
    { col: 4, row: 6 },
  ];

  for (const pos of lampPositions) {
    const { x, y } = valleyGridToScreen(pos.col, pos.row, w, h);
    drawLampPost(ctx, x, y, frame);
  }

  // Benches
  const benchPositions = [
    { col: 1, row: 2 },
    { col: 7, row: 4 },
  ];

  for (const pos of benchPositions) {
    const { x, y } = valleyGridToScreen(pos.col, pos.row, w, h);
    drawBench(ctx, x, y);
  }

  // Clouds (floating slowly)
  drawClouds(ctx, w, frame);
}

function drawTree(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  frame: number,
): void {
  const sway = Math.sin(frame * 0.02 + x * 0.1) * 1;

  // Trunk
  ctx.fillStyle = "#2A1A0A";
  ctx.fillRect(x - 2, y - 20, 4, 20);

  // Foliage layers (3 triangles stacked)
  const foliageColor = "#0A3A0A";
  const foliageLight = "#0F4F0F";

  for (let i = 0; i < 3; i++) {
    const layerY = y - 18 - i * 10;
    const layerW = 14 - i * 2;
    ctx.beginPath();
    ctx.moveTo(x + sway, layerY - 12);
    ctx.lineTo(x + layerW + sway, layerY);
    ctx.lineTo(x - layerW + sway, layerY);
    ctx.closePath();
    ctx.fillStyle = i % 2 === 0 ? foliageColor : foliageLight;
    ctx.fill();
    ctx.strokeStyle = "#0A2A0A";
    ctx.lineWidth = 0.5;
    ctx.stroke();
  }
}

function drawLampPost(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  frame: number,
): void {
  // Post
  ctx.fillStyle = "#333333";
  ctx.fillRect(x - 1, y - 28, 2, 28);

  // Lamp head
  ctx.fillStyle = "#444444";
  ctx.fillRect(x - 4, y - 30, 8, 3);

  // Glow
  const glowAlpha = 0.15 + 0.05 * Math.sin(frame * 0.03);
  ctx.beginPath();
  ctx.arc(x, y - 28, 12, 0, Math.PI * 2);
  ctx.fillStyle = `rgba(255, 230, 150, ${glowAlpha})`;
  ctx.fill();
}

function drawBench(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
): void {
  // Seat
  ctx.fillStyle = "#2A1A0A";
  ctx.fillRect(x - 10, y - 4, 20, 3);

  // Legs
  ctx.fillStyle = "#1A1A1A";
  ctx.fillRect(x - 8, y - 1, 2, 4);
  ctx.fillRect(x + 6, y - 1, 2, 4);

  // Back rest
  ctx.fillStyle = "#2A1A0A";
  ctx.fillRect(x - 10, y - 10, 2, 8);
  ctx.fillRect(x + 8, y - 10, 2, 8);
  ctx.fillRect(x - 10, y - 10, 20, 2);
}

function drawClouds(
  ctx: CanvasRenderingContext2D,
  w: number,
  frame: number,
): void {
  const clouds = [
    { baseX: 100, y: 30, size: 1.0 },
    { baseX: 300, y: 20, size: 0.7 },
    { baseX: 500, y: 40, size: 0.85 },
  ];

  for (const cloud of clouds) {
    const drift = (frame * 0.15 + cloud.baseX) % (w + 200) - 100;
    const scale = cloud.size;

    ctx.save();
    ctx.globalAlpha = 0.06;
    ctx.translate(drift, cloud.y);
    ctx.scale(scale, scale);

    // Cloud puffs
    ctx.fillStyle = "#FFFFFF";
    ctx.beginPath();
    ctx.arc(0, 0, 16, 0, Math.PI * 2);
    ctx.arc(18, -4, 12, 0, Math.PI * 2);
    ctx.arc(-14, -2, 10, 0, Math.PI * 2);
    ctx.arc(8, 6, 10, 0, Math.PI * 2);
    ctx.fill();

    ctx.restore();
  }
}

/** Draw "Cowork Valley" title at the top. */
export function drawValleyTitle(
  ctx: CanvasRenderingContext2D,
  w: number,
  _h: number,
): void {
  ctx.fillStyle = "#E5E5E5";
  ctx.font = 'bold 16px ui-monospace, "SF Mono", Menlo, monospace';
  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  ctx.fillText("Cowork Valley", w / 2, 14);

  // Subtle underline
  ctx.strokeStyle = "#333333";
  ctx.lineWidth = 0.5;
  ctx.beginPath();
  ctx.moveTo(w / 2 - 60, 34);
  ctx.lineTo(w / 2 + 60, 34);
  ctx.stroke();
}

// ---------------------------------------------------------------------------
// Hit testing
// ---------------------------------------------------------------------------

/**
 * Test if a click at (px, py) hits any building. Returns the building ID
 * or null if nothing was hit.
 */
export function hitTestBuilding(
  px: number,
  py: number,
  canvasW: number,
  canvasH: number,
): string | null {
  for (const building of VALLEY_LAYOUT) {
    const { x, y } = buildingScreenPos(building, canvasW, canvasH);
    const halfW = BUILDING_W / 2;

    // Simple rectangular hit test around the building body
    const left = x - halfW;
    const right = x + halfW;
    const top = y - BUILDING_H - ROOF_H;
    const bottom = y + halfW * 0.4 + 24; // include label area

    if (px >= left && px <= right && py >= top && py <= bottom) {
      return building.id;
    }
  }
  return null;
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/**
 * Render the full Cowork Valley scene.
 */
export function drawValley(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  buildings: OfficeBuilding[],
  frame: number,
): void {
  // Clear
  ctx.fillStyle = "#050505";
  ctx.fillRect(0, 0, w, h);

  // Ground tiles
  drawGround(ctx, w, h);

  // Paths between buildings
  drawPaths(ctx, w, h);

  // Decorations (behind buildings in draw order)
  drawDecorations(ctx, w, h, frame);

  // Buildings — sort by gridY for proper overlap (back to front)
  const sorted = [...buildings].sort((a, b) => a.gridY - b.gridY);
  for (const building of sorted) {
    drawBuilding(ctx, building, w, h, frame);
  }

  // Title overlay
  drawValleyTitle(ctx, w, h);
}
