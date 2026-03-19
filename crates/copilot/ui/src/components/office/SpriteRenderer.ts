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
  browsing: { startFrame: 4, frameCount: 2, fps: 1.5 },
  done: { startFrame: 8, frameCount: 4, fps: 3 },
};

// ---------------------------------------------------------------------------
// Role profiles
// ---------------------------------------------------------------------------

export const AGENT_PROFILES: Record<
  string,
  { hairColor: string; hairStyle: string; accessory: string; skinColor: string }
> = {
  orchestrator: {
    hairColor: "#4A3728",
    hairStyle: "short",
    accessory: "stethoscope",
    skinColor: "#E8C8A0",
  },
  researcher: {
    hairColor: "#2A6A4A",
    hairStyle: "messy",
    accessory: "magnifier",
    skinColor: "#E8C8A0",
  },
  analyst: {
    hairColor: "#6A4A8A",
    hairStyle: "neat",
    accessory: "glasses",
    skinColor: "#D8B890",
  },
  writer: {
    hairColor: "#C4A040",
    hairStyle: "long",
    accessory: "pen",
    skinColor: "#E8C8A0",
  },
  coder: {
    hairColor: "#D04080",
    hairStyle: "spiky",
    accessory: "headphones",
    skinColor: "#D8B890",
  },
  operator: {
    hairColor: "#D0A030",
    hairStyle: "buzz",
    accessory: "wrench",
    skinColor: "#E8C8A0",
  },
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
// Hair styles
// ---------------------------------------------------------------------------

function drawHair(
  ctx: CanvasRenderingContext2D,
  hairStyle: string,
  hairColor: string,
  charY: number,
): void {
  ctx.fillStyle = hairColor;
  ctx.strokeStyle = darken(hairColor, 0.3);
  ctx.lineWidth = 0.5;

  switch (hairStyle) {
    case "short":
      // Flat top rect, short sides
      ctx.fillRect(-6, -24 + charY, 12, 3);
      ctx.fillRect(-7, -23 + charY, 2, 2);
      ctx.fillRect(5, -23 + charY, 2, 2);
      break;

    case "messy":
      // Jagged top with 3 spikes (triangles)
      ctx.fillRect(-5, -23 + charY, 10, 2);
      ctx.beginPath();
      ctx.moveTo(-5, -23 + charY);
      ctx.lineTo(-3, -27 + charY);
      ctx.lineTo(-1, -23 + charY);
      ctx.closePath();
      ctx.fill();
      ctx.beginPath();
      ctx.moveTo(-1, -23 + charY);
      ctx.lineTo(1, -28 + charY);
      ctx.lineTo(3, -23 + charY);
      ctx.closePath();
      ctx.fill();
      ctx.beginPath();
      ctx.moveTo(3, -23 + charY);
      ctx.lineTo(5, -26 + charY);
      ctx.lineTo(7, -23 + charY);
      ctx.closePath();
      ctx.fill();
      break;

    case "neat":
      // Smooth arc over head + side parting line
      ctx.beginPath();
      ctx.ellipse(0, -22 + charY, 6, 3, 0, Math.PI, 0);
      ctx.fill();
      ctx.fillRect(-6, -23 + charY, 12, 2);
      // Side parting line (light highlight)
      ctx.strokeStyle = lighten(hairColor, 0.3);
      ctx.lineWidth = 0.5;
      ctx.beginPath();
      ctx.moveTo(-1, -25 + charY);
      ctx.lineTo(-1, -22 + charY);
      ctx.stroke();
      break;

    case "long":
      // Wider rect flowing past head sides
      ctx.fillRect(-7, -24 + charY, 14, 3);
      ctx.fillRect(-8, -23 + charY, 3, 8);
      ctx.fillRect(5, -23 + charY, 3, 8);
      break;

    case "spiky":
      // 3-4 pointed triangles upward
      ctx.fillRect(-5, -23 + charY, 10, 2);
      for (let i = 0; i < 4; i++) {
        const bx = -5 + i * 3;
        ctx.beginPath();
        ctx.moveTo(bx, -23 + charY);
        ctx.lineTo(bx + 1.5, -28 + charY);
        ctx.lineTo(bx + 3, -23 + charY);
        ctx.closePath();
        ctx.fill();
      }
      break;

    case "buzz":
      // Very short thin rect, barely visible
      ctx.fillStyle = darken(hairColor, 0.1);
      ctx.fillRect(-5, -23 + charY, 10, 1);
      break;

    default:
      // Fallback: simple flat top
      ctx.fillRect(-6, -23 + charY, 12, 2);
      break;
  }
}

// ---------------------------------------------------------------------------
// Accessories
// ---------------------------------------------------------------------------

function drawAccessory(
  ctx: CanvasRenderingContext2D,
  accessory: string,
  charY: number,
  bodyColor: string,
): void {
  switch (accessory) {
    case "stethoscope": {
      // U-arc below head at neck, gray
      ctx.strokeStyle = "#888888";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.arc(0, -14 + charY, 4, 0, Math.PI);
      ctx.stroke();
      // Ear tips
      ctx.fillStyle = "#888888";
      ctx.fillRect(-4, -14 + charY, 2, 2);
      ctx.fillRect(2, -14 + charY, 2, 2);
      // Chest piece
      ctx.beginPath();
      ctx.arc(0, -10 + charY, 1.5, 0, Math.PI * 2);
      ctx.fillStyle = "#AAAAAA";
      ctx.fill();
      break;
    }

    case "magnifier": {
      // Circle + handle near hand, gold
      ctx.strokeStyle = "#C8A000";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.arc(8, -6 + charY, 3, 0, Math.PI * 2);
      ctx.stroke();
      ctx.strokeStyle = "#A07800";
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.moveTo(10, -4 + charY);
      ctx.lineTo(12, -2 + charY);
      ctx.stroke();
      break;
    }

    case "glasses": {
      // Two circles on face + bridge, gray
      ctx.strokeStyle = "#888888";
      ctx.lineWidth = 0.8;
      ctx.beginPath();
      ctx.arc(-2, -17 + charY, 1.8, 0, Math.PI * 2);
      ctx.stroke();
      ctx.beginPath();
      ctx.arc(2, -17 + charY, 1.8, 0, Math.PI * 2);
      ctx.stroke();
      // Bridge
      ctx.beginPath();
      ctx.moveTo(-0.2, -17 + charY);
      ctx.lineTo(0.2, -17 + charY);
      ctx.stroke();
      // Arms
      ctx.beginPath();
      ctx.moveTo(-3.8, -17 + charY);
      ctx.lineTo(-5, -17 + charY);
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(3.8, -17 + charY);
      ctx.lineTo(5, -17 + charY);
      ctx.stroke();
      break;
    }

    case "pen": {
      // Diagonal line behind ear, blue
      ctx.strokeStyle = "#2060C0";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(4, -22 + charY);
      ctx.lineTo(6, -19 + charY);
      ctx.stroke();
      // Pen tip
      ctx.fillStyle = "#102040";
      ctx.beginPath();
      ctx.moveTo(5.5, -19.5 + charY);
      ctx.lineTo(6.5, -18.5 + charY);
      ctx.lineTo(5, -18 + charY);
      ctx.closePath();
      ctx.fill();
      break;
    }

    case "headphones": {
      // Arc over head + 2 ear pad rects, dark
      ctx.strokeStyle = "#222222";
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.arc(0, -22 + charY, 6, Math.PI, 0);
      ctx.stroke();
      ctx.fillStyle = "#333333";
      ctx.fillRect(-7, -23 + charY, 2, 3);
      ctx.fillRect(5, -23 + charY, 2, 3);
      break;
    }

    case "wrench": {
      // Small tool shape at waist, silver
      ctx.strokeStyle = "#AAAAAA";
      ctx.lineWidth = 1;
      ctx.fillStyle = "#CCCCCC";
      // Handle
      ctx.fillRect(6, -10 + charY, 1, 5);
      // Head of wrench
      ctx.beginPath();
      ctx.arc(6.5, -10 + charY, 2, 0, Math.PI * 2);
      ctx.stroke();
      // Use body color for the open part of wrench
      ctx.fillStyle = darken(bodyColor, 0.1);
      ctx.fillRect(5.5, -11 + charY, 2, 1);
      break;
    }

    default:
      break;
  }
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

  // "?" dots inside bubble (animated)
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

  // Left arm - fast typing motion
  const leftAngle = Math.sin(frame * 0.4) * 3;
  ctx.fillStyle = armColor;
  ctx.fillRect(x - 10, y - 2 + leftAngle, 4, 2);

  // Right arm - typing motion (offset phase)
  const rightAngle = Math.sin(frame * 0.4 + Math.PI) * 3;
  ctx.fillStyle = armColor;
  ctx.fillRect(x + 6, y - 2 + rightAngle, 4, 2);

  // Tiny dots flying up from keyboard area
  for (let i = 0; i < 3; i++) {
    const dotAge = (frame + i * 17) % 30;
    const dotAlpha = 1 - dotAge / 30;
    const dotX = x - 4 + i * 4 + Math.sin(i * 2.1) * 2;
    const dotY = y + 2 - dotAge * 0.3;
    ctx.fillStyle = `rgba(180, 220, 255, ${dotAlpha * 0.7})`;
    ctx.fillRect(dotX, dotY, 1, 1);
  }
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

  // Gear teeth
  ctx.strokeStyle = "#F9731680";
  ctx.lineWidth = 1;
  ctx.beginPath();
  for (let i = 0; i < 8; i++) {
    const angle = (i / 8) * Math.PI * 2;
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

function drawThinkingArm(
  ctx: CanvasRenderingContext2D,
  charY: number,
  breathe: number,
  bodyColor: string,
  skinColor: string,
): void {
  // Right arm bent to chin (thinking pose)
  ctx.fillStyle = bodyColor;
  ctx.fillRect(6, -12 + charY + breathe, 3, 5);
  ctx.fillRect(5, -9 + charY + breathe, 5, 2);
  // Hand at chin
  ctx.fillStyle = skinColor;
  ctx.fillRect(4, -7 + charY + breathe, 3, 2);
  // Left arm resting
  ctx.fillStyle = bodyColor;
  ctx.fillRect(-9, -12 + charY + breathe, 3, 8);
  ctx.fillStyle = skinColor;
  ctx.fillRect(-9, -4 + charY + breathe, 3, 2);
}

function drawCrossedArms(
  ctx: CanvasRenderingContext2D,
  charY: number,
  breathe: number,
  bodyColor: string,
): void {
  // X pattern arms crossed
  ctx.strokeStyle = darken(bodyColor, 0.1);
  ctx.lineWidth = 3;
  ctx.beginPath();
  ctx.moveTo(-9, -12 + charY + breathe);
  ctx.lineTo(6, -6 + charY + breathe);
  ctx.stroke();
  ctx.beginPath();
  ctx.moveTo(9, -12 + charY + breathe);
  ctx.lineTo(-6, -6 + charY + breathe);
  ctx.stroke();
}

function drawRaisedArms(
  ctx: CanvasRenderingContext2D,
  charY: number,
  breathe: number,
  bodyColor: string,
  skinColor: string,
): void {
  // Both arms up (error or celebration)
  ctx.fillStyle = bodyColor;
  ctx.fillRect(-10, -18 + charY + breathe, 3, 6);
  ctx.fillRect(7, -18 + charY + breathe, 3, 6);
  ctx.fillStyle = skinColor;
  ctx.fillRect(-10, -20 + charY + breathe, 3, 2);
  ctx.fillRect(7, -20 + charY + breathe, 3, 2);
}

function drawVArms(
  ctx: CanvasRenderingContext2D,
  charY: number,
  breathe: number,
  bodyColor: string,
  skinColor: string,
): void {
  // V-arms celebration
  ctx.fillStyle = bodyColor;
  ctx.fillRect(-12, -20 + charY + breathe, 3, 6);
  ctx.fillRect(9, -20 + charY + breathe, 3, 6);
  // Tilt outward for V shape
  ctx.save();
  ctx.translate(-9, -18 + charY + breathe);
  ctx.rotate(-0.4);
  ctx.fillStyle = bodyColor;
  ctx.fillRect(0, -3, 2, 5);
  ctx.restore();
  ctx.save();
  ctx.translate(9, -18 + charY + breathe);
  ctx.rotate(0.4);
  ctx.fillStyle = bodyColor;
  ctx.fillRect(-2, -3, 2, 5);
  ctx.restore();
  ctx.fillStyle = skinColor;
  ctx.fillRect(-12, -22 + charY + breathe, 3, 2);
  ctx.fillRect(9, -22 + charY + breathe, 3, 2);
}

function drawExclamationBubble(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  frame: number,
): void {
  const bobY = Math.sin(frame * 0.06) * 1;

  ctx.fillStyle = "#FFDD00AA";
  ctx.strokeStyle = "#CC9900";
  ctx.lineWidth = 0.5;
  ctx.beginPath();
  ctx.ellipse(x + 4, y - 4 + bobY, 5, 5, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.stroke();

  // "!" mark
  ctx.fillStyle = "#664400";
  ctx.fillRect(x + 3.5, y - 8 + bobY, 1, 4);
  ctx.fillRect(x + 3.5, y - 3 + bobY, 1, 1);
}

// ---------------------------------------------------------------------------
// Error bubble (red pulsing "!" for error state)
// ---------------------------------------------------------------------------

function drawErrorBubble(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  frame: number,
): void {
  const bobY = Math.sin(frame * 0.12) * 1.5;
  const pulseAlpha = 0.7 + 0.3 * Math.abs(Math.sin(frame * 0.1));

  // Red bubble background
  ctx.fillStyle = `rgba(239, 68, 68, ${pulseAlpha * 0.8})`;
  ctx.strokeStyle = `rgba(185, 28, 28, ${pulseAlpha})`;
  ctx.lineWidth = 0.8;
  ctx.beginPath();
  ctx.ellipse(x + 4, y - 4 + bobY, 5, 5, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.stroke();

  // "!" mark in white
  ctx.fillStyle = `rgba(255, 255, 255, ${pulseAlpha})`;
  ctx.fillRect(x + 3.5, y - 8 + bobY, 1.5, 4);
  ctx.fillRect(x + 3.5, y - 3 + bobY, 1.5, 1.5);
}

// ---------------------------------------------------------------------------
// Confetti for "done" state
// ---------------------------------------------------------------------------

interface ConfettiParticle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  color: string;
  age: number;
}

const CONFETTI_COLORS = [
  "#FF6B6B",
  "#FFD93D",
  "#6BCB77",
  "#4D96FF",
  "#FF6BFF",
  "#FF9F43",
];

// Deterministic confetti per agent — use a simple hash so each agent gets
// the same burst pattern every cycle
function getConfettiParticles(agentHash: number, frame: number): ConfettiParticle[] {
  const cycleFrame = frame % 60;
  const particles: ConfettiParticle[] = [];
  for (let i = 0; i < 8; i++) {
    const seed = (agentHash * 31 + i * 17) & 0xffffff;
    const angle = ((seed % 360) / 360) * Math.PI * 2;
    const speed = 0.8 + (seed % 10) * 0.15;
    const vx = Math.cos(angle) * speed;
    const vy = Math.sin(angle) * speed - 1.5;
    const age = cycleFrame;
    const x = vx * age;
    const y = vy * age + 0.05 * age * age; // gravity
    particles.push({
      x,
      y,
      vx,
      vy,
      color: CONFETTI_COLORS[i % CONFETTI_COLORS.length],
      age,
    });
  }
  return particles;
}

function drawConfetti(
  ctx: CanvasRenderingContext2D,
  agentHash: number,
  charY: number,
  frame: number,
): void {
  const particles = getConfettiParticles(agentHash, frame);
  for (const p of particles) {
    const alpha = Math.max(0, 1 - p.age / 60);
    ctx.globalAlpha = alpha;
    ctx.fillStyle = p.color;
    ctx.fillRect(p.x, p.y - 24 + charY, 2, 2);
  }
  ctx.globalAlpha = 1;
}

// ---------------------------------------------------------------------------
// Mood bubbles (rest zone idle)
// ---------------------------------------------------------------------------

const MOOD_ICONS = [
  "coffee",
  "music",
  "gamepad",
  "book",
  "lightbulb",
  "star",
  "zzz",
] as const;
type MoodIcon = (typeof MOOD_ICONS)[number];

function drawMoodIcon(
  ctx: CanvasRenderingContext2D,
  icon: MoodIcon,
  x: number,
  y: number,
  alpha: number,
): void {
  ctx.globalAlpha = alpha;
  ctx.fillStyle = "#FFFFFFCC";

  switch (icon) {
    case "coffee":
      // Cup body
      ctx.fillRect(x, y + 2, 5, 4);
      // Handle
      ctx.strokeStyle = "#FFFFFFCC";
      ctx.lineWidth = 0.8;
      ctx.beginPath();
      ctx.arc(x + 5.5, y + 4, 1.5, -Math.PI / 2, Math.PI / 2);
      ctx.stroke();
      // Steam
      ctx.lineWidth = 0.5;
      ctx.beginPath();
      ctx.moveTo(x + 1, y + 2);
      ctx.quadraticCurveTo(x, y, x + 1, y - 1);
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(x + 3, y + 2);
      ctx.quadraticCurveTo(x + 2, y, x + 3, y - 1);
      ctx.stroke();
      break;

    case "music":
      // Note stem
      ctx.fillRect(x + 3, y, 1, 5);
      // Note head
      ctx.beginPath();
      ctx.ellipse(x + 2, y + 5, 2, 1.5, -0.3, 0, Math.PI * 2);
      ctx.fill();
      // Flag
      ctx.strokeStyle = "#FFFFFFCC";
      ctx.lineWidth = 0.8;
      ctx.beginPath();
      ctx.moveTo(x + 4, y);
      ctx.quadraticCurveTo(x + 7, y + 1, x + 4, y + 3);
      ctx.stroke();
      break;

    case "gamepad":
      // Body
      ctx.fillRect(x, y + 1, 6, 4);
      // D-pad
      ctx.fillStyle = "#888888CC";
      ctx.fillRect(x + 1, y + 2, 1, 2);
      ctx.fillRect(x, y + 3, 3, 1);
      // Button
      ctx.fillStyle = "#FF6B6BCC";
      ctx.fillRect(x + 4, y + 2, 1, 1);
      ctx.fillStyle = "#4D96FFCC";
      ctx.fillRect(x + 4, y + 4, 1, 1);
      break;

    case "book":
      // Cover
      ctx.fillStyle = "#FFD93DCC";
      ctx.fillRect(x, y, 5, 6);
      // Spine
      ctx.fillStyle = "#CC9900CC";
      ctx.fillRect(x, y, 1, 6);
      // Pages lines
      ctx.fillStyle = "#FFFFFFCC";
      ctx.fillRect(x + 2, y + 1, 2, 1);
      ctx.fillRect(x + 2, y + 3, 2, 1);
      break;

    case "lightbulb":
      // Bulb
      ctx.fillStyle = "#FFFF88CC";
      ctx.beginPath();
      ctx.arc(x + 3, y + 2, 2.5, 0, Math.PI * 2);
      ctx.fill();
      // Base
      ctx.fillStyle = "#AAAAAACC";
      ctx.fillRect(x + 1, y + 4, 4, 2);
      break;

    case "star":
      ctx.fillStyle = "#FFD93DCC";
      ctx.beginPath();
      for (let i = 0; i < 5; i++) {
        const outerA = (i * 2 * Math.PI) / 5 - Math.PI / 2;
        const innerA = outerA + Math.PI / 5;
        const px = x + 3 + Math.cos(outerA) * 3;
        const py = y + 3 + Math.sin(outerA) * 3;
        const ipx = x + 3 + Math.cos(innerA) * 1.2;
        const ipy = y + 3 + Math.sin(innerA) * 1.2;
        if (i === 0) {
          ctx.moveTo(px, py);
        } else {
          ctx.lineTo(px, py);
        }
        ctx.lineTo(ipx, ipy);
      }
      ctx.closePath();
      ctx.fill();
      break;

    case "zzz":
      ctx.font = "bold 4px monospace";
      ctx.fillStyle = `rgba(200,200,255,${alpha})`;
      ctx.fillText("z", x, y + 2);
      ctx.fillText("Z", x + 2, y);
      break;
  }

  ctx.globalAlpha = 1;
}

/**
 * Draw floating mood bubble for rest zone idle.
 * Floats up 20px over 90 frames, fades out.
 */
function drawMoodBubble(
  ctx: CanvasRenderingContext2D,
  agentHash: number,
  x: number,
  y: number,
  frame: number,
): void {
  // Cycle every 300+ frames, offset by agent hash
  const offset = agentHash % 300;
  const cycleFrame = (frame + offset) % 360;

  // Only show during a 90-frame window
  if (cycleFrame >= 90) return;

  const t = cycleFrame / 90;
  // Fade in first 10%, full 80%, fade out last 10%
  let alpha: number;
  if (t < 0.1) {
    alpha = t / 0.1;
  } else if (t < 0.9) {
    alpha = 1;
  } else {
    alpha = (1 - t) / 0.1;
  }

  // Rise up 20px
  const floatY = y - t * 20;

  // Pick icon based on hash
  const iconIndex = (agentHash + Math.floor(frame / 360)) % MOOD_ICONS.length;
  const icon = MOOD_ICONS[iconIndex];

  drawMoodIcon(ctx, icon, x - 3, floatY - 10, alpha * 0.85);
}

// ---------------------------------------------------------------------------
// Simple string hash for agent identity
// ---------------------------------------------------------------------------

function hashString(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) {
    h = (Math.imul(31, h) + s.charCodeAt(i)) | 0;
  }
  return Math.abs(h);
}

// ---------------------------------------------------------------------------
// Main pixel character drawing
// ---------------------------------------------------------------------------

/**
 * Draw a procedural pixel-art character at the given position.
 *
 * @param ctx        Canvas context
 * @param x          Center X position
 * @param y          Base Y position (feet level)
 * @param size       Scale factor (1.0 = normal office size, 0.5 = mini valley size)
 * @param agentRole  Role name for color + profile lookup
 * @param state      Current agent state (idle, thinking, typing, executing, browsing, paused, error, done)
 * @param frame      Global animation frame counter
 * @param agentId    Optional agent identifier for deterministic variation (mood bubbles, confetti)
 * @param inRestZone Optional flag — when true and state is idle, show floating mood bubbles
 */
export function drawPixelCharacter(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  size: number,
  agentRole: string,
  state: string,
  frame: number,
  agentId?: string,
  inRestZone?: boolean,
): void {
  const color = AGENT_COLORS[agentRole] ?? "#6B7280";
  const profile = AGENT_PROFILES[agentRole];
  const bodyColor = darken(color, 0.3);
  const headColor = color;
  const eyeColor = "#FFFFFF";
  // Use profile skin color if available, otherwise derive from role color
  const skinColor = profile?.skinColor ?? lighten(color, 0.5);
  const hairColor = profile?.hairColor ?? darken(headColor, 0.2);
  const hairStyle = profile?.hairStyle ?? "short";
  const accessory = profile?.accessory ?? "none";

  const agentHash = hashString(agentId ?? agentRole);

  // Frame-based animation
  const bobOffset = Math.sin(frame * 0.05) * 2 * size;
  const breathe = Math.sin(frame * 0.03) * 0.5 * size;

  // Error jitter (shake animation)
  const jitter = state === "error" ? ((frame % 3) - 1) * 1.5 : 0;

  ctx.save();
  ctx.translate(x + jitter, y);
  ctx.scale(size, size);

  // Error state: pulsing red glow underneath the character
  if (state === "error") {
    const flashAlpha = 0.15 + 0.1 * Math.abs(Math.sin(frame * 0.15));
    ctx.fillStyle = `rgba(239, 68, 68, ${flashAlpha})`;
    ctx.beginPath();
    ctx.ellipse(0, -10, 14, 18, 0, 0, Math.PI * 2);
    ctx.fill();
  }

  // Shadow under character
  ctx.fillStyle = "#00000030";
  ctx.beginPath();
  ctx.ellipse(0, 0, 8, 3, 0, 0, Math.PI * 2);
  ctx.fill();

  // Idle at desk: gentle Y oscillation (distinct slow bobbing), stretch every 300 frames
  let charY = bobOffset;
  const isStretching = state === "idle" && frame % 300 > 290;

  if (state === "idle") {
    // Gentle breathing-like bobbing with slower frequency than working states
    charY = Math.sin(frame * 0.025) * 2.5 * size + 1;
  } else if (state === "browsing") {
    charY = bobOffset - 2; // lean forward
  }

  // -----------------------------------------------------------------------
  // Legs
  // -----------------------------------------------------------------------
  if (state === "executing") {
    // Wider stance
    const legOffset = Math.sin(frame * 0.2) * 2;
    ctx.fillStyle = darken(bodyColor, 0.2);
    ctx.fillRect(-5, -4 + charY + legOffset, 3, 6);
    ctx.fillRect(2, -4 + charY - legOffset, 3, 6);
  } else if (state === "working") {
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

  // -----------------------------------------------------------------------
  // Body
  // -----------------------------------------------------------------------
  ctx.fillStyle = bodyColor;
  ctx.fillRect(-6, -14 + charY + breathe, 12, 12);

  // Body highlight
  ctx.fillStyle = lighten(bodyColor, 0.1) + "40";
  ctx.fillRect(-6, -14 + charY + breathe, 3, 12);

  // -----------------------------------------------------------------------
  // Arms
  // -----------------------------------------------------------------------
  if (state === "typing" || state === "working") {
    drawTypingArms(ctx, 0, -8 + charY + breathe, frame, color);
  } else if (state === "thinking") {
    drawThinkingArm(ctx, charY, breathe, bodyColor, skinColor);
  } else if (state === "paused") {
    drawCrossedArms(ctx, charY, breathe, bodyColor);
  } else if (state === "error") {
    drawRaisedArms(ctx, charY, breathe, bodyColor, skinColor);
  } else if (state === "done") {
    drawVArms(ctx, charY, breathe, bodyColor, skinColor);
  } else if (state === "executing") {
    drawTypingArms(ctx, 0, -8 + charY + breathe, frame, color);
  } else if (isStretching) {
    // Stretch: arms up briefly
    drawRaisedArms(ctx, charY, breathe, bodyColor, skinColor);
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

  // -----------------------------------------------------------------------
  // Head
  // -----------------------------------------------------------------------
  ctx.fillStyle = headColor;
  ctx.fillRect(-5, -22 + charY, 10, 9);

  // Head highlight
  ctx.fillStyle = lighten(headColor, 0.15) + "60";
  ctx.fillRect(-5, -22 + charY, 3, 9);

  // Face (skin color center)
  ctx.fillStyle = skinColor;
  ctx.fillRect(-3, -19 + charY, 6, 5);

  // -----------------------------------------------------------------------
  // Eyes
  // -----------------------------------------------------------------------
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
    // Closed/narrow eyes
    ctx.strokeStyle = eyeColor;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(-3, -17 + charY);
    ctx.lineTo(-1, -17 + charY);
    ctx.moveTo(1, -17 + charY);
    ctx.lineTo(3, -17 + charY);
    ctx.stroke();
  } else if (state === "browsing") {
    // Wider eyes (lean-forward focused)
    ctx.fillStyle = eyeColor;
    ctx.fillRect(-3, -19 + charY, 3, 3);
    ctx.fillRect(1, -19 + charY, 3, 3);
    ctx.fillStyle = "#111111";
    ctx.fillRect(-2, -18 + charY, 1, 1);
    ctx.fillRect(2, -18 + charY, 1, 1);
  } else {
    // Normal eyes with blink
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

  // -----------------------------------------------------------------------
  // Mouth
  // -----------------------------------------------------------------------
  if (state === "error") {
    // Frown
    ctx.fillStyle = "#EF4444";
    ctx.fillRect(-2, -15 + charY, 4, 1);
  } else if (state === "idle" || state === "paused") {
    // Neutral
    ctx.fillStyle = "#00000060";
    ctx.fillRect(-1, -15 + charY, 2, 1);
  } else if (state === "done") {
    // Big smile
    ctx.fillStyle = "#00000060";
    ctx.fillRect(-2, -15 + charY, 4, 1);
    ctx.fillRect(-1, -14 + charY, 2, 1);
    ctx.fillRect(-3, -15 + charY, 1, 1);
    ctx.fillRect(2, -15 + charY, 1, 1);
  } else {
    // Smile
    ctx.fillStyle = "#00000060";
    ctx.fillRect(-2, -15 + charY, 4, 1);
    ctx.fillRect(-1, -14 + charY, 2, 1);
  }

  // -----------------------------------------------------------------------
  // Hair (role-specific style and color)
  // -----------------------------------------------------------------------
  drawHair(ctx, hairStyle, hairColor, charY);

  // -----------------------------------------------------------------------
  // Accessory
  // -----------------------------------------------------------------------
  drawAccessory(ctx, accessory, charY, bodyColor);

  // -----------------------------------------------------------------------
  // State-specific overlays
  // -----------------------------------------------------------------------
  if (state === "thinking") {
    drawThoughtBubble(ctx, 4, -26 + charY, frame);
  } else if (state === "executing") {
    drawExecutingGears(ctx, 0, -4 + charY, frame);
  } else if (state === "paused") {
    drawExclamationBubble(ctx, 4, -26 + charY, frame);
  } else if (state === "error") {
    drawErrorBubble(ctx, 4, -26 + charY, frame);
  } else if (state === "done") {
    drawConfetti(ctx, agentHash, charY, frame);
  }

  // -----------------------------------------------------------------------
  // Status dot above head
  // -----------------------------------------------------------------------
  const statusColor = STATUS_COLORS[state] ?? STATUS_COLORS.idle;
  ctx.fillStyle = statusColor;
  ctx.beginPath();
  ctx.arc(0, -29 + charY, 2.5, 0, Math.PI * 2);
  ctx.fill();

  // Glow for working/executing/done
  if (state === "working" || state === "executing" || state === "done") {
    ctx.strokeStyle = statusColor + "60";
    ctx.lineWidth = 1;
    const glowRadius = 3.5 + Math.sin(frame * 0.1) * 1;
    ctx.beginPath();
    ctx.arc(0, -29 + charY, glowRadius, 0, Math.PI * 2);
    ctx.stroke();
  }

  ctx.restore();

  // -----------------------------------------------------------------------
  // Mood bubbles (outside scale transform, drawn in world space)
  // -----------------------------------------------------------------------
  if (inRestZone && state === "idle") {
    drawMoodBubble(ctx, agentHash, x, y - 28 * size, frame);
  }
}

/**
 * Draw a mini pixel character (for valley building windows).
 * Uses the profile hair color for visual role distinction.
 */
export function drawMiniPixelCharacter(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  agentRole: string,
  state: string,
  frame: number,
  agentId?: string,
): void {
  drawPixelCharacter(ctx, x, y, 0.5, agentRole, state, frame, agentId, false);
}
