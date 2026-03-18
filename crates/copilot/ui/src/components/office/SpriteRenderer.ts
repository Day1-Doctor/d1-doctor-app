/**
 * SpriteRenderer — Procedural pixel-art character drawing for agents.
 *
 * Instead of loading external sprite sheets, characters are drawn
 * programmatically using Canvas 2D primitives to create a retro
 * pixel-art aesthetic.
 *
 * Each agent role gets a distinct color scheme and the character
 * animates based on the current agent state.
 */

import { AGENT_COLORS, STATUS_COLORS } from "./OfficeRenderer";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface SpriteSheet {
  image: HTMLImageElement;
  frameWidth: number;
  frameHeight: number;
  framesPerRow: number;
  loaded: boolean;
}

export interface SpriteAnimation {
  startFrame: number;
  frameCount: number;
  fps: number;
}

// Animation definitions per agent state
export const ANIMATIONS: Record<string, SpriteAnimation> = {
  idle: { startFrame: 0, frameCount: 2, fps: 0.5 },
  thinking: { startFrame: 2, frameCount: 2, fps: 1 },
  typing: { startFrame: 4, frameCount: 2, fps: 3 },
  executing: { startFrame: 6, frameCount: 2, fps: 2 },
  working: { startFrame: 4, frameCount: 2, fps: 2.5 },
  paused: { startFrame: 0, frameCount: 2, fps: 0.3 },
  error: { startFrame: 2, frameCount: 2, fps: 0.8 },
};

// ---------------------------------------------------------------------------
// Color helpers
// ---------------------------------------------------------------------------

function darken(hex: string, amount: number): string {
  // Handle rgb() format
  if (hex.startsWith("rgb")) {
    const match = hex.match(/(\d+)/g);
    if (match) {
      const factor = 1 - amount;
      return `rgb(${Math.floor(Number(match[0]) * factor)}, ${Math.floor(Number(match[1]) * factor)}, ${Math.floor(Number(match[2]) * factor)})`;
    }
  }
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  const factor = 1 - amount;
  return `rgb(${Math.floor(r * factor)}, ${Math.floor(g * factor)}, ${Math.floor(b * factor)})`;
}

function lighten(hex: string, amount: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  const factor = amount;
  return `rgb(${Math.min(255, Math.floor(r + (255 - r) * factor))}, ${Math.min(255, Math.floor(g + (255 - g) * factor))}, ${Math.min(255, Math.floor(b + (255 - b) * factor))})`;
}

// ---------------------------------------------------------------------------
// Thought bubble / status indicators
// ---------------------------------------------------------------------------

function drawThoughtBubble(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  frame: number,
): void {
  const bobY = Math.sin(frame * 0.08) * 1;

  // Small dots leading to bubble
  ctx.fillStyle = "#FFFFFF40";
  ctx.beginPath();
  ctx.arc(x - 2, y + 6 + bobY, 1.5, 0, Math.PI * 2);
  ctx.fill();
  ctx.beginPath();
  ctx.arc(x + 1, y + 2 + bobY, 2, 0, Math.PI * 2);
  ctx.fill();

  // Main bubble
  ctx.fillStyle = "#FFFFFF20";
  ctx.strokeStyle = "#FFFFFF40";
  ctx.lineWidth = 0.5;
  ctx.beginPath();
  ctx.ellipse(x + 4, y - 4 + bobY, 7, 5, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.stroke();

  // Dots inside bubble (thinking animation)
  const dotPhase = frame * 0.15;
  for (let i = 0; i < 3; i++) {
    const dotAlpha = 0.3 + 0.5 * Math.abs(Math.sin(dotPhase + i * 1.2));
    ctx.fillStyle = `rgba(255, 255, 255, ${dotAlpha})`;
    ctx.beginPath();
    ctx.arc(x + 1 + i * 3, y - 4 + bobY, 1, 0, Math.PI * 2);
    ctx.fill();
  }
}

function drawTypingArms(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  frame: number,
  color: string,
): void {
  const armColor = darken(color, 0.2);

  // Left arm - typing motion
  const leftAngle = Math.sin(frame * 0.3) * 3;
  ctx.fillStyle = armColor;
  ctx.fillRect(x - 10, y - 2 + leftAngle, 4, 2);

  // Right arm - typing motion (offset phase)
  const rightAngle = Math.sin(frame * 0.3 + Math.PI) * 3;
  ctx.fillStyle = armColor;
  ctx.fillRect(x + 6, y - 2 + rightAngle, 4, 2);
}

function drawExecutingGears(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  frame: number,
): void {
  const rotation = frame * 0.1;

  ctx.save();
  ctx.translate(x + 10, y - 18);
  ctx.rotate(rotation);

  // Simple gear shape
  ctx.strokeStyle = "#F9731680";
  ctx.lineWidth = 1;
  ctx.beginPath();
  for (let i = 0; i < 6; i++) {
    const angle = (i / 6) * Math.PI * 2;
    const outerX = Math.cos(angle) * 4;
    const outerY = Math.sin(angle) * 4;
    if (i === 0) {
      ctx.moveTo(outerX, outerY);
    } else {
      ctx.lineTo(outerX, outerY);
    }
  }
  ctx.closePath();
  ctx.stroke();

  // Center dot
  ctx.fillStyle = "#F97316";
  ctx.beginPath();
  ctx.arc(0, 0, 1.5, 0, Math.PI * 2);
  ctx.fill();

  ctx.restore();
}

// ---------------------------------------------------------------------------
// Main pixel character drawing
// ---------------------------------------------------------------------------

/**
 * Draw a procedural pixel-art character at the given position.
 *
 * @param ctx     Canvas context
 * @param x       Center X position
 * @param y       Base Y position (feet level)
 * @param size    Scale factor (1.0 = normal office size, 0.5 = mini valley size)
 * @param agentRole  Role name for color lookup
 * @param state   Current agent state (idle, thinking, etc.)
 * @param frame   Global animation frame counter
 */
export function drawPixelCharacter(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  size: number,
  agentRole: string,
  state: string,
  frame: number,
): void {
  const color = AGENT_COLORS[agentRole] ?? "#6B7280";
  const bodyColor = darken(color, 0.3);
  const headColor = color;
  const eyeColor = "#FFFFFF";
  const skinColor = lighten(color, 0.5);

  // Frame-based animation
  const bobOffset = Math.sin(frame * 0.05) * 2 * size;
  const breathe = Math.sin(frame * 0.03) * 0.5 * size;

  ctx.save();
  ctx.translate(x, y);
  ctx.scale(size, size);

  // Shadow under character
  ctx.fillStyle = "#00000030";
  ctx.beginPath();
  ctx.ellipse(0, 0, 8, 3, 0, 0, Math.PI * 2);
  ctx.fill();

  const charY = bobOffset;

  // Legs
  if (state === "executing" || state === "working") {
    // Animated legs (running/moving)
    const legOffset = Math.sin(frame * 0.2) * 2;
    ctx.fillStyle = darken(bodyColor, 0.2);
    ctx.fillRect(-4, -4 + charY + legOffset, 3, 6);
    ctx.fillRect(1, -4 + charY - legOffset, 3, 6);
  } else {
    // Standing legs
    ctx.fillStyle = darken(bodyColor, 0.2);
    ctx.fillRect(-4, -4 + charY, 3, 6);
    ctx.fillRect(1, -4 + charY, 3, 6);
  }

  // Body
  ctx.fillStyle = bodyColor;
  ctx.fillRect(-6, -14 + charY + breathe, 12, 12);

  // Body highlight
  ctx.fillStyle = lighten(bodyColor, 0.1) + "40";
  ctx.fillRect(-6, -14 + charY + breathe, 3, 12);

  // Arms
  if (state === "typing" || state === "working" || state === "executing") {
    drawTypingArms(ctx, 0, -8 + charY + breathe, frame, color);
  } else {
    // Resting arms
    ctx.fillStyle = bodyColor;
    ctx.fillRect(-9, -12 + charY + breathe, 3, 8);
    ctx.fillRect(6, -12 + charY + breathe, 3, 8);

    // Hands (skin-colored)
    ctx.fillStyle = skinColor;
    ctx.fillRect(-9, -4 + charY + breathe, 3, 2);
    ctx.fillRect(6, -4 + charY + breathe, 3, 2);
  }

  // Head
  ctx.fillStyle = headColor;
  ctx.fillRect(-5, -22 + charY, 10, 9);

  // Head highlight
  ctx.fillStyle = lighten(headColor, 0.15) + "60";
  ctx.fillRect(-5, -22 + charY, 3, 9);

  // Face (skin color center)
  ctx.fillStyle = skinColor;
  ctx.fillRect(-3, -19 + charY, 6, 5);

  // Eyes
  if (state === "error") {
    // X eyes for error
    ctx.strokeStyle = "#EF4444";
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(-3, -18 + charY);
    ctx.lineTo(-1, -16 + charY);
    ctx.moveTo(-1, -18 + charY);
    ctx.lineTo(-3, -16 + charY);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(1, -18 + charY);
    ctx.lineTo(3, -16 + charY);
    ctx.moveTo(3, -18 + charY);
    ctx.lineTo(1, -16 + charY);
    ctx.stroke();
  } else if (state === "paused") {
    // Closed eyes (sleeping)
    ctx.strokeStyle = eyeColor;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(-3, -17 + charY);
    ctx.lineTo(-1, -17 + charY);
    ctx.moveTo(1, -17 + charY);
    ctx.lineTo(3, -17 + charY);
    ctx.stroke();
  } else {
    // Normal eyes
    const blinkPhase = frame % 120;
    const isBlinking = blinkPhase > 115;

    if (isBlinking) {
      ctx.strokeStyle = eyeColor;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(-3, -17 + charY);
      ctx.lineTo(-1, -17 + charY);
      ctx.moveTo(1, -17 + charY);
      ctx.lineTo(3, -17 + charY);
      ctx.stroke();
    } else {
      ctx.fillStyle = eyeColor;
      ctx.fillRect(-3, -18 + charY, 2, 2);
      ctx.fillRect(1, -18 + charY, 2, 2);

      // Pupils
      ctx.fillStyle = "#111111";
      ctx.fillRect(-2, -17 + charY, 1, 1);
      ctx.fillRect(2, -17 + charY, 1, 1);
    }
  }

  // Mouth
  if (state === "error") {
    // Frown
    ctx.fillStyle = "#EF4444";
    ctx.fillRect(-2, -15 + charY, 4, 1);
  } else if (state === "idle") {
    // Neutral
    ctx.fillStyle = "#00000060";
    ctx.fillRect(-1, -15 + charY, 2, 1);
  } else {
    // Smile
    ctx.fillStyle = "#00000060";
    ctx.fillRect(-2, -15 + charY, 4, 1);
    ctx.fillRect(-1, -14 + charY, 2, 1);
  }

  // Hair/hat (role-specific top)
  ctx.fillStyle = darken(headColor, 0.2);
  ctx.fillRect(-6, -23 + charY, 12, 2);

  // State-specific indicators
  if (state === "thinking") {
    drawThoughtBubble(ctx, 4, -26 + charY, frame);
  } else if (state === "executing") {
    drawExecutingGears(ctx, 0, -4 + charY, frame);
  }

  // Status dot above head
  const statusColor = STATUS_COLORS[state] ?? STATUS_COLORS.idle;
  ctx.fillStyle = statusColor;
  ctx.beginPath();
  ctx.arc(0, -27 + charY, 2.5, 0, Math.PI * 2);
  ctx.fill();

  // Glow for working/executing
  if (state === "working" || state === "executing") {
    ctx.strokeStyle = statusColor + "60";
    ctx.lineWidth = 1;
    const glowRadius = 3.5 + Math.sin(frame * 0.1) * 1;
    ctx.beginPath();
    ctx.arc(0, -27 + charY, glowRadius, 0, Math.PI * 2);
    ctx.stroke();
  }

  ctx.restore();
}

/**
 * Draw a mini pixel character (for valley building windows).
 * Just a smaller, simplified version.
 */
export function drawMiniPixelCharacter(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  agentRole: string,
  state: string,
  frame: number,
): void {
  drawPixelCharacter(ctx, x, y, 0.5, agentRole, state, frame);
}
