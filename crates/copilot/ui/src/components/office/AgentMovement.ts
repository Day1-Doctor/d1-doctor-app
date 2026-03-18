/**
 * AgentMovement — Agent position state and movement logic.
 *
 * Agents walk between their desks and rest-zone items depending on their
 * status. When idle for ~5 seconds (150 frames at 30 fps) they randomly
 * choose a rest-zone destination. When their status becomes non-idle they
 * return to their desk.
 */

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface AgentPosition {
  x: number;
  y: number;
  targetX: number;
  targetY: number;
  isMoving: boolean;
  /** Frames elapsed since the agent last became idle. */
  idleTimer: number;
  currentLocation: "desk" | "sofa" | "arcade" | "pool" | "cooler" | "whiteboard" | "walking";
}

// ---------------------------------------------------------------------------
// Rest zone destinations (grid coordinates)
// ---------------------------------------------------------------------------

const REST_ZONE_POSITIONS = [
  { id: "sofa", gridX: 6, gridY: 2 },
  { id: "arcade", gridX: 5, gridY: 1 },
  { id: "pool", gridX: 6, gridY: 4 },
  { id: "cooler", gridX: 7, gridY: 3 },
  { id: "whiteboard", gridX: 4, gridY: 0 },
] as const;

type RestZoneId = (typeof REST_ZONE_POSITIONS)[number]["id"];

// ---------------------------------------------------------------------------
// Public helpers
// ---------------------------------------------------------------------------

/**
 * Create an initial AgentPosition anchored at the given screen coordinates.
 */
export function createAgentPosition(screenX: number, screenY: number): AgentPosition {
  return {
    x: screenX,
    y: screenY,
    targetX: screenX,
    targetY: screenY,
    isMoving: false,
    idleTimer: 0,
    currentLocation: "desk",
  };
}

/**
 * Advance an AgentPosition by one frame.
 *
 * @param pos           Current position (mutated in-place and returned).
 * @param status        Agent status string.
 * @param deskScreenX   Screen X of the agent's desk.
 * @param deskScreenY   Screen Y of the agent's desk.
 * @param frame         Global animation frame counter (used for deterministic rest picks).
 * @param gridToScreen  Function converting grid coords to screen coords.
 * @returns The updated position object (same reference).
 */
export function updateAgentPosition(
  pos: AgentPosition,
  status: string,
  deskScreenX: number,
  deskScreenY: number,
  frame: number,
  gridToScreen: (gx: number, gy: number) => { x: number; y: number },
): AgentPosition {
  const speed = 1.5; // pixels per frame

  if (status !== "idle") {
    // Working — snap target back to desk
    pos.targetX = deskScreenX;
    pos.targetY = deskScreenY;
    pos.idleTimer = 0;
    if (!pos.isMoving) {
      pos.currentLocation = "desk";
    }
  } else {
    pos.idleTimer++;

    // After 150 idle frames (~5 s), pick a random rest-zone destination
    if (pos.idleTimer === 150 && pos.currentLocation === "desk") {
      // Use frame as extra entropy so agents don't all pick the same spot
      const pick = REST_ZONE_POSITIONS[(pos.idleTimer + frame) % REST_ZONE_POSITIONS.length];
      const screenPos = gridToScreen(pick.gridX, pick.gridY);
      pos.targetX = screenPos.x;
      pos.targetY = screenPos.y;
      pos.currentLocation = "walking";
    }

    // After 600 idle frames (~20 s) at rest, walk back to desk
    if (
      pos.idleTimer > 600 &&
      pos.currentLocation !== "desk" &&
      pos.currentLocation !== "walking"
    ) {
      pos.targetX = deskScreenX;
      pos.targetY = deskScreenY;
      pos.currentLocation = "walking";
      pos.idleTimer = 0;
    }
  }

  // Move toward target
  const dx = pos.targetX - pos.x;
  const dy = pos.targetY - pos.y;
  const dist = Math.sqrt(dx * dx + dy * dy);

  if (dist > speed) {
    pos.x += (dx / dist) * speed;
    pos.y += (dy / dist) * speed;
    pos.isMoving = true;
  } else {
    pos.x = pos.targetX;
    pos.y = pos.targetY;
    pos.isMoving = false;

    // Resolve arrival location
    if (pos.currentLocation === "walking") {
      const atDesk =
        Math.abs(pos.x - deskScreenX) < 5 && Math.abs(pos.y - deskScreenY) < 5;

      if (atDesk) {
        pos.currentLocation = "desk";
      } else {
        // Find the nearest rest-zone item
        const nearest = REST_ZONE_POSITIONS.find((rz) => {
          const sp = gridToScreen(rz.gridX, rz.gridY);
          return Math.abs(sp.x - pos.x) < 20 && Math.abs(sp.y - pos.y) < 20;
        });
        pos.currentLocation = (nearest?.id as RestZoneId | undefined) ?? "desk";
      }
    }
  }

  return pos;
}
