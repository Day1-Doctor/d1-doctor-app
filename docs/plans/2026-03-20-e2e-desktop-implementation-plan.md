# E2E Desktop Testing — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a 3-tier E2E test suite (97 tests + 35 screenshots) for the Day1 Copilot desktop app, runnable locally via `npm run e2e:all`.

**Architecture:** Tier 1 = Playwright against Vite dev server with mocked Tauri invoke. Tier 2 = Node.js WebSocket client against real Tauri binary. Tier 3 = osascript + screencapture visual regression on macOS.

**Tech Stack:** Playwright (TS), Node.js test runner (ESM), osascript (AppleScript), Python (screenshot diff)

---

## Phase A: Infrastructure Setup (4 tasks)

### Task A1: Set up Playwright in Copilot UI

**Files:**
- Create: `crates/copilot/ui/playwright.config.ts`
- Create: `crates/copilot/ui/e2e/tauri-mock.ts`
- Create: `crates/copilot/ui/e2e/ws-mock.ts`
- Create: `crates/copilot/ui/e2e/fixtures/agents.json`
- Create: `crates/copilot/ui/e2e/fixtures/tasks.json`
- Create: `crates/copilot/ui/e2e/fixtures/events.json`
- Modify: `crates/copilot/ui/package.json`

**Step 1: Install Playwright**
```bash
cd crates/copilot/ui
npm install -D @playwright/test
npx playwright install chromium
```

**Step 2: Create playwright.config.ts**
```typescript
import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  timeout: 15000,
  retries: 0,
  use: {
    baseURL: "http://localhost:1420",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },
  webServer: {
    command: "npm run dev",
    url: "http://localhost:1420",
    reuseExistingServer: true,
    timeout: 10000,
  },
});
```

**Step 3: Create Tauri invoke mock shim**
```typescript
// e2e/tauri-mock.ts
// Intercepts window.__TAURI_INTERNALS__ to mock invoke calls
// Returns test fixtures for list_agents, create_task, etc.
```
This file must:
- Define mock responses for every Tauri command used by the app
- Be injectable via Playwright's `page.addInitScript()`
- Support overriding responses per-test

**Step 4: Create WebSocket mock**
```typescript
// e2e/ws-mock.ts
// Mock WebSocket that simulates EventBus events
// Allows tests to emit agent.state_changed, cost.updated, etc.
```

**Step 5: Create test fixtures**
JSON files with realistic data: 6 agents, sample tasks, event sequences.

**Step 6: Add npm scripts**
```json
{
  "scripts": {
    "e2e:tier1": "npx playwright test",
    "e2e:tier1:ui": "npx playwright test --ui"
  }
}
```

**Step 7: Write a smoke test to verify setup**
```typescript
// e2e/smoke.spec.ts
import { test, expect } from "@playwright/test";
test("app loads", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("body")).toBeVisible();
});
```

**Step 8: Run and verify**
```bash
npx playwright test e2e/smoke.spec.ts
```

---

### Task A2: Set up WebSocket E2E framework

**Files:**
- Create: `crates/copilot/e2e-ws/package.json`
- Create: `crates/copilot/e2e-ws/client.mjs`
- Create: `crates/copilot/e2e-ws/runner.mjs`
- Create: `crates/copilot/e2e-ws/assert.mjs`

**Step 1: Create package.json**
```json
{
  "name": "d1-copilot-e2e-ws",
  "type": "module",
  "scripts": {
    "test": "node runner.mjs"
  },
  "dependencies": {
    "ws": "^8.0.0"
  }
}
```

**Step 2: Create client.mjs** — WebSocket + HTTP helper
```javascript
// Connects to ws://127.0.0.1:14200/ws/events
// Subscribes to events, collects them in a queue
// HTTP methods: get(path), post(path, body)
// waitForEvent(type, timeoutMs) — returns matching event
// collectEvents(duration) — collects all events for N ms
```

**Step 3: Create runner.mjs** — Test runner
```javascript
// Discovers *.test.mjs files
// Launches Tauri binary if not running
// Waits for IPC server to be ready
// Runs each test file, reports pass/fail
// Kills binary on cleanup
```

**Step 4: Create assert.mjs** — Lightweight assertion helpers
```javascript
export function assertEqual(actual, expected, msg) { ... }
export function assertIncludes(arr, item, msg) { ... }
export function assertMatch(obj, pattern, msg) { ... }
```

**Step 5: Write smoke test**
```javascript
// runtime-smoke.test.mjs
import { createClient } from "./client.mjs";
const client = await createClient();
const agents = await client.get("/api/v1/agents");
assert(agents.length === 6, "expected 6 agents");
client.close();
```

**Step 6: Run and verify**
```bash
cd crates/copilot/e2e-ws && npm test
```

---

### Task A3: Set up Visual E2E framework

**Files:**
- Create: `crates/copilot/e2e-visual/run.sh`
- Create: `crates/copilot/e2e-visual/lib/launch.applescript`
- Create: `crates/copilot/e2e-visual/lib/screenshot.sh`
- Create: `crates/copilot/e2e-visual/lib/diff.py`
- Create: `crates/copilot/e2e-visual/baselines/.gitkeep`

**Step 1: Create run.sh** — Main runner
```bash
#!/bin/bash
# Builds release binary if needed
# Runs each journey script in journeys/
# Collects results, reports P0/P1 pass/fail
# Exit code: 0 if all P0 pass, 1 if any P0 fail
```

**Step 2: Create launch.applescript**
```applescript
-- Launch the Tauri app and wait for window
tell application "Day1 Copilot" to activate
delay 3
tell application "System Events"
  set frontApp to first application process whose frontmost is true
  set appWindow to first window of frontApp
end tell
```

**Step 3: Create screenshot.sh**
```bash
#!/bin/bash
# Takes a screenshot of the app window
# Usage: screenshot.sh <output-path>
screencapture -l $(osascript -e 'tell app "System Events" to id of first window of (first process whose frontmost is true)') "$1"
```

**Step 4: Create diff.py**
```python
# Pixel-diff two screenshots with tolerance
# Usage: python diff.py baseline.png actual.png --tolerance 5
# Exit 0 if within tolerance, exit 1 if different
```

**Step 5: Write smoke journey**
```bash
# journeys/smoke.sh
osascript lib/launch.applescript
sleep 3
bash lib/screenshot.sh /tmp/e2e-visual-smoke.png
echo "PASS: App launched and screenshot taken"
```

---

### Task A4: Wire all runners into root package.json

**Files:**
- Modify: `crates/copilot/ui/package.json` — add e2e:tier1
- Create or modify: root-level script or Makefile

**Add to copilot ui/package.json:**
```json
{
  "scripts": {
    "e2e:tier1": "npx playwright test",
    "e2e:tier2": "cd ../e2e-ws && node runner.mjs",
    "e2e:tier3": "cd ../e2e-visual && bash run.sh",
    "e2e:all": "npm run e2e:tier1 && npm run e2e:tier2 && npm run e2e:tier3"
  }
}
```

---

## Phase B: Tier 1 — Playwright Tests (7 tasks, 55 tests)

### Task B1: Auth flow tests (8 tests)

**Files:**
- Create: `crates/copilot/ui/e2e/auth.spec.ts`

**Tests:**
1. `app loads in demo mode — office visible without auth`
2. `auth panel opens when clicking sign in`
3. `auth panel has Google + Email buttons`
4. `auth panel close X dismisses panel`
5. `Google OAuth button has correct Supabase URL`
6. `deep link callback sets token + dismisses auth wall`
7. `invalid token shows error, does not crash`
8. `logout clears token, returns to demo mode`

Each test uses the Tauri mock to simulate auth state. Test 6 injects a mock deep-link event.

---

### Task B2: Valley / Office view tests (7 tests)

**Files:**
- Create: `crates/copilot/ui/e2e/office-view.spec.ts`

**Tests:**
9-15 per design doc. Key: mock `list_agents` returns 6 agents, simulate `agent.state_changed` WS events for test 11, verify canvas or DOM rendering for agents.

---

### Task B3: Sidebar navigation tests (6 tests)

**Files:**
- Create: `crates/copilot/ui/e2e/sidebar.spec.ts`

**Tests:** 16-21 per design doc.

---

### Task B4: Plan Mode chat + task creation tests (10 tests)

**Files:**
- Create: `crates/copilot/ui/e2e/plan-mode.spec.ts`

**Tests:** 22-31 per design doc. Key: mock `create_task` invoke, test loading states, test free mode hint for unauthenticated user.

---

### Task B5: Task view + progress tests (8 tests)

**Files:**
- Create: `crates/copilot/ui/e2e/task-view.spec.ts`

**Tests:** 32-39 per design doc. Key: inject WS events for status transitions, test 50+ task performance with fixture data.

---

### Task B6: Approval flow tests (6 tests)

**Files:**
- Create: `crates/copilot/ui/e2e/approval.spec.ts`

**Tests:** 40-45 per design doc. Key: inject `approval.requested` WS event, verify dialog, mock `respond_approval` invoke.

---

### Task B7: Metrics + Workspace + Cost tests (10 tests)

**Files:**
- Create: `crates/copilot/ui/e2e/metrics-workspace.spec.ts`

**Tests:** 46-55 per design doc. Key: pre-populate stores via mocks, verify real data rendering, test i18n toggle, test zoom persistence.

---

## Phase C: Tier 2 — WebSocket Tests (5 tasks, 30 tests)

### Task C1: Runtime initialization tests (5 tests)

**Files:**
- Create: `crates/copilot/e2e-ws/runtime.test.mjs`

**Tests:** 1-5 per design doc. Binary must be built first. Tests verify IPC server startup, agent registration, model assignments.

---

### Task C2: Task lifecycle tests (8 tests)

**Files:**
- Create: `crates/copilot/e2e-ws/task-lifecycle.test.mjs`

**Tests:** 6-13 per design doc. Creates tasks via HTTP API, verifies decomposition, status transitions, cancel/pause.

---

### Task C3: Event stream tests (8 tests)

**Files:**
- Create: `crates/copilot/e2e-ws/event-stream.test.mjs`

**Tests:** 14-21 per design doc. Subscribes to WS, creates tasks to trigger events, verifies event ordering and payloads.

---

### Task C4: Tool dispatch tests (5 tests)

**Files:**
- Create: `crates/copilot/e2e-ws/tool-dispatch.test.mjs`

**Tests:** 22-26 per design doc. Tests tool execution via the HTTP API, verifies sandbox security.

---

### Task C5: Persistence + recovery tests (4 tests)

**Files:**
- Create: `crates/copilot/e2e-ws/persistence.test.mjs`

**Tests:** 27-30 per design doc. Creates data → kills process → restarts → verifies data survived. Most complex tests in Tier 2.

---

## Phase D: Tier 3 — Visual E2E (2 tasks, 12 journeys)

### Task D1: P0 visual journeys (6 journeys, 20 screenshots)

**Files:**
- Create: `crates/copilot/e2e-visual/journeys/p0-cold-start.sh`
- Create: `crates/copilot/e2e-visual/journeys/p0-demo-browse.sh`
- Create: `crates/copilot/e2e-visual/journeys/p0-auth-flow.sh`
- Create: `crates/copilot/e2e-visual/journeys/p0-create-task.sh`
- Create: `crates/copilot/e2e-visual/journeys/p0-task-execution.sh`
- Create: `crates/copilot/e2e-visual/journeys/p0-approval.sh`

Each script: launches app → performs journey steps via osascript → takes screenshots → diffs against baselines.

---

### Task D2: P1 visual journeys (6 journeys, 15 screenshots)

**Files:**
- Create: `crates/copilot/e2e-visual/journeys/p1-i18n.sh`
- Create: `crates/copilot/e2e-visual/journeys/p1-zoom.sh`
- Create: `crates/copilot/e2e-visual/journeys/p1-metrics.sh`
- Create: `crates/copilot/e2e-visual/journeys/p1-animations.sh`
- Create: `crates/copilot/e2e-visual/journeys/p1-workspace.sh`
- Create: `crates/copilot/e2e-visual/journeys/p1-multi-task.sh`

---

## Dispatch Strategy

**Wave 1 (parallel — infrastructure):** Tasks A1 + A2 + A3 + A4

**Wave 2 (parallel — Tier 1 tests):** Tasks B1 + B2 + B3 + B4 + B5 + B6 + B7
All touch different `.spec.ts` files, no conflicts.

**Wave 3 (parallel — Tier 2 + Tier 3):** Tasks C1-C5 + D1-D2
Different directories, no conflicts.

**Total: 18 tasks across 3 waves**
