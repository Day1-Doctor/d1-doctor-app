# Day1 Copilot Desktop App — E2E Testing Design

> **Date:** 2026-03-20
> **Goal:** Comprehensive E2E test suite for the Tauri desktop app covering all user journeys with high coverage of continuous logic and edge cases.
> **Approach:** Three-tier hybrid — Playwright (UI), WebSocket (full stack), Visual (osascript + screenshots)

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  Tier 1 — Playwright (local, ~30s)                              │
│  Tests React UI in browser with mocked Tauri invoke              │
│  Location: crates/copilot/ui/e2e/                               │
│  Runner: npm run e2e:tier1                                       │
├─────────────────────────────────────────────────────────────────┤
│  Tier 2 — WebSocket E2E (local, ~60s)                           │
│  Node.js client → launches binary → connects WS → sends cmds    │
│  Location: crates/copilot/e2e-ws/                               │
│  Runner: npm run e2e:tier2                                       │
├─────────────────────────────────────────────────────────────────┤
│  Tier 3 — Visual E2E (local, ~5min)                             │
│  osascript launches .app → clicks through flows → screenshots    │
│  Location: crates/copilot/e2e-visual/                           │
│  Runner: npm run e2e:tier3                                       │
│  Gate: P0 failures BLOCK release, P1/P2 advisory                │
└─────────────────────────────────────────────────────────────────┘

# Run everything
npm run e2e:all
```

**Total: 97 tests + 35 screenshots across 3 tiers**

---

## Tier 1 — Playwright Tests (55 tests)

**Setup:** Vite dev server at `localhost:1420` + Tauri invoke mock shim that returns realistic data. No real Tauri binary needed.

**Mock shim:** `crates/copilot/ui/e2e/tauri-mock.ts` intercepts `@tauri-apps/api/core` invoke calls and returns test fixtures.

### 1a. Auth Flow (8 tests)

| # | Test | Type |
|---|------|------|
| 1 | App loads in demo mode — office visible without auth | Happy path |
| 2 | Auth panel opens when clicking "Sign in" | Happy path |
| 3 | Auth panel has Google + Email buttons | Happy path |
| 4 | Auth panel close X dismisses panel, returns to demo | Happy path |
| 5 | Google OAuth button opens correct Supabase URL with redirect_uri | Integration |
| 6 | Deep link callback sets token + dismisses auth wall | Happy path |
| 7 | Invalid/expired token shows error, doesn't crash | Edge case |
| 8 | Logout clears token, returns to demo mode | Happy path |

### 1b. Valley / Office View (7 tests)

| # | Test | Type |
|---|------|------|
| 9 | Valley renders 11 parcels with correct states | Happy path |
| 10 | Office view shows 6 agents with correct names/roles | Happy path |
| 11 | Agent status badges update on state_changed event | Integration |
| 12 | Clicking an agent shows agent detail card | Interaction |
| 13 | Zoom control works (80%-150% range) | Interaction |
| 14 | Zoom persists across view switches | Edge case |
| 15 | Window resize doesn't break canvas layout | Edge case |

### 1c. Sidebar Navigation (6 tests)

| # | Test | Type |
|---|------|------|
| 16 | All 4 accordion sections render (Valley, Tasks, Workspace, Metrics) | Happy path |
| 17 | Accordion expand/collapse works | Interaction |
| 18 | View switches correctly (Valley → Office → Tasks) | Navigation |
| 19 | Active view highlights in sidebar | State |
| 20 | Developer section visible in debug mode | Conditional |
| 21 | Sidebar collapse/expand works | Interaction |

### 1d. Plan Mode Chat + Task Creation (10 tests)

| # | Test | Type |
|---|------|------|
| 22 | Chat panel renders in Plan Mode by default | Happy path |
| 23 | User can type and send message | Interaction |
| 24 | "Confirm Plan" creates task via Tauri invoke | Happy path |
| 25 | Task creation shows loading state | UX |
| 26 | Unauthenticated user sees "free mode" hint, can still submit | Edge case |
| 27 | Empty message cannot be submitted | Validation |
| 28 | Long message (>5000 chars) handled gracefully | Edge case |
| 29 | BTW mode toggle switches chat context | Interaction |
| 30 | Chat history preserved across mode switches | State |
| 31 | Network error during task creation shows error toast | Error |

### 1e. Task View + Progress (8 tests)

| # | Test | Type |
|---|------|------|
| 32 | Task list renders with parent/subtask hierarchy | Happy path |
| 33 | Task status updates from WS events | Integration |
| 34 | Task step_completed updates progress bar | Integration |
| 35 | Subtask expand shows agent assignment + output | Interaction |
| 36 | Cancel task button works | Interaction |
| 37 | Empty task list shows placeholder | Edge case |
| 38 | 50+ tasks render without lag (performance) | Performance |
| 39 | Failed task shows error state with message | Error |

### 1f. Approval Flow (6 tests)

| # | Test | Type |
|---|------|------|
| 40 | Approval dialog renders on approval.requested event | Integration |
| 41 | Approve button sends respond_approval invoke | Happy path |
| 42 | Deny button sends deny decision | Happy path |
| 43 | Queue navigation (prev/next) works with multiple approvals | Interaction |
| 44 | Approval dialog dismisses after decision | UX |
| 45 | Rapid approve/deny doesn't double-submit | Edge case |

### 1g. Metrics + Workspace + Cost (10 tests)

| # | Test | Type |
|---|------|------|
| 46 | MetricsView shows real data from stores (not placeholders) | Happy path |
| 47 | DD balance updates on cost.updated event | Integration |
| 48 | Per-agent cost breakdown renders correctly | Happy path |
| 49 | WorkspaceView lists artifacts from store | Happy path |
| 50 | Empty workspace shows "No artifacts" state | Edge case |
| 51 | Memory section shows entries or empty state | Edge case |
| 52 | Cost display formats DD correctly (2 decimal places) | Formatting |
| 53 | Balance reaching 0 shows warning/upgrade prompt | Edge case |
| 54 | i18n toggle EN↔CN updates all visible text | Integration |
| 55 | Display zoom setting persists in settings store | State |

---

## Tier 2 — WebSocket E2E Tests (30 tests)

**Setup:** Build release binary → launch it → wait for IPC server on `ws://127.0.0.1:14200` → run Node.js test client.

**Test client:** `crates/copilot/e2e-ws/client.mjs` — connects WebSocket, subscribes to events, calls HTTP API, asserts responses.

### 2a. Runtime Initialization (5 tests)

| # | Test | Type |
|---|------|------|
| 1 | IPC server starts on port 14200 within 5s | Smoke |
| 2 | GET /api/v1/agents returns 6 agents with correct roles | Happy path |
| 3 | All agents start in Idle status | State |
| 4 | Agent models match PRD (4× sonnet, 2× haiku) | Contract |
| 5 | Health endpoint responds | Smoke |

### 2b. Task Lifecycle via API (8 tests)

| # | Test | Type |
|---|------|------|
| 6 | POST /api/v1/tasks creates task and returns ID | Happy path |
| 7 | Task decomposes into subtasks (GET /api/v1/tasks/:id/steps) | Integration |
| 8 | Subtasks have agent_id assigned | Integration |
| 9 | Task status transitions: Pending → Running → Completed | Lifecycle |
| 10 | Cancel task mid-execution changes status to Cancelled | Lifecycle |
| 11 | Pause then resume task works | Lifecycle |
| 12 | Creating task with maxAgents=1 collapses to orchestrator | Edge case |
| 13 | POST /api/v1/tasks/:id/start triggers execution | API |

### 2c. Event Stream via WebSocket (8 tests)

| # | Test | Type |
|---|------|------|
| 14 | Subscribe to events receives agent.state_changed | Integration |
| 15 | Events arrive in correct FSM order (idle→working→thinking→working→idle) | Contract |
| 16 | cost.updated event has session_tokens + session_cost_dd | Contract |
| 17 | task.step_completed event has task_id + step_index + result | Contract |
| 18 | tool.started/finished events emitted during tool execution | Integration |
| 19 | Multiple concurrent subscribers all receive events | Concurrency |
| 20 | Late subscriber can replay recent events via history API | Edge case |
| 21 | Disconnected subscriber reconnects and resumes | Resilience |

### 2d. Tool Dispatch (5 tests)

| # | Test | Type |
|---|------|------|
| 22 | Filesystem read tool returns file content | Integration |
| 23 | Filesystem write tool creates file in workspace | Integration |
| 24 | Path traversal attack blocked by sandbox | Security |
| 25 | Web fetch returns extracted text | Integration |
| 26 | Unknown tool returns graceful error | Error |

### 2e. Persistence + Recovery (4 tests)

| # | Test | Type |
|---|------|------|
| 27 | Create task → kill process → restart → task still present | Persistence |
| 28 | Session costs survive restart | Persistence |
| 29 | Trust scores survive restart | Persistence |
| 30 | Audit trail entries written to SQLite | Persistence |

---

## Tier 3 — Visual E2E (12 journeys, 35 screenshots)

**Setup:** `osascript` launches the `.app` bundle → clicks via accessibility API → `screencapture` at key moments → pixel-diff against baseline images (5% tolerance).

**Baselines:** `crates/copilot/e2e-visual/baselines/` — PNG files captured during first run, committed to git.

### P0 Journeys (block release)

| # | Journey | Steps | Screenshots |
|---|---------|-------|-------------|
| 1 | **Cold start → Office view** | Launch app → wait for load → verify 6 agents visible | 2 |
| 2 | **Demo mode browse** | Navigate Valley → Office → Tasks → Workspace → Metrics without auth | 5 |
| 3 | **Auth flow** | Click Sign In → verify panel → click X to close → verify dismissed | 3 |
| 4 | **Create task (free mode)** | Type in Plan Mode → Confirm → verify task appears in sidebar | 3 |
| 5 | **Task execution** | Create task → watch agent state change → step completes → parent done | 4 |
| 6 | **Approval flow** | Trigger approval → dialog appears → approve → agent resumes | 3 |

### P1 Journeys (advisory — failures logged but don't block)

| # | Journey | Steps | Screenshots |
|---|---------|-------|-------------|
| 7 | **i18n switch** | Settings → toggle CN → verify text changed → toggle back EN | 3 |
| 8 | **Zoom control** | Adjust to 120% → verify layout scales → reset 100% | 2 |
| 9 | **Metrics real data** | After task → open Metrics → verify non-zero values | 2 |
| 10 | **Agent animations** | Trigger task → observe idle→thinking→working states | 3 |
| 11 | **Workspace artifacts** | After task output → open Workspace → verify artifact listed | 2 |
| 12 | **Multi-task queue** | Create 3 tasks rapidly → verify all queued, process sequentially | 3 |

---

## Runner Commands

```bash
# Individual tiers
npm run e2e:tier1       # Playwright — ~30s
npm run e2e:tier2       # WebSocket full-stack — ~60s
npm run e2e:tier3       # Visual osascript — ~5min

# All tiers
npm run e2e:all         # Runs tier1 → tier2 → tier3 sequentially

# Update visual baselines (after intentional UI changes)
npm run e2e:tier3:update-baselines

# Run specific tier 1 suite
npx playwright test e2e/auth.spec.ts
npx playwright test e2e/task-creation.spec.ts
```

---

## Directory Structure

```
crates/copilot/
├── ui/
│   ├── e2e/                          # Tier 1 — Playwright
│   │   ├── tauri-mock.ts             # Mock @tauri-apps/api/core invoke
│   │   ├── ws-mock.ts                # Mock WebSocket events
│   │   ├── fixtures/                 # Test data (agents, tasks, events)
│   │   ├── auth.spec.ts              # 1a. Auth flow (8 tests)
│   │   ├── office-view.spec.ts       # 1b. Valley/Office (7 tests)
│   │   ├── sidebar.spec.ts           # 1c. Sidebar navigation (6 tests)
│   │   ├── plan-mode.spec.ts         # 1d. Chat + task creation (10 tests)
│   │   ├── task-view.spec.ts         # 1e. Task progress (8 tests)
│   │   ├── approval.spec.ts          # 1f. Approval flow (6 tests)
│   │   └── metrics-workspace.spec.ts # 1g. Metrics + workspace (10 tests)
│   └── playwright.config.ts
├── e2e-ws/                           # Tier 2 — WebSocket
│   ├── client.mjs                    # WS + HTTP test client helper
│   ├── run.mjs                       # Test runner (launches binary, runs tests)
│   ├── runtime.test.mjs              # 2a. Runtime init (5 tests)
│   ├── task-lifecycle.test.mjs       # 2b. Task lifecycle (8 tests)
│   ├── event-stream.test.mjs         # 2c. Event stream (8 tests)
│   ├── tool-dispatch.test.mjs        # 2d. Tool dispatch (5 tests)
│   └── persistence.test.mjs          # 2e. Persistence (4 tests)
├── e2e-visual/                       # Tier 3 — Visual
│   ├── run.sh                        # Main runner (osascript + screencapture)
│   ├── lib/                          # Helper scripts
│   │   ├── launch.applescript        # Launch and wait for app
│   │   ├── navigate.applescript      # Click sidebar items
│   │   ├── screenshot.sh             # Capture + diff
│   │   └── diff.py                   # Pixel comparison (5% tolerance)
│   ├── journeys/                     # Journey scripts
│   │   ├── p0-cold-start.sh
│   │   ├── p0-demo-browse.sh
│   │   ├── p0-auth-flow.sh
│   │   ├── p0-create-task.sh
│   │   ├── p0-task-execution.sh
│   │   ├── p0-approval.sh
│   │   ├── p1-i18n.sh
│   │   ├── p1-zoom.sh
│   │   ├── p1-metrics.sh
│   │   ├── p1-animations.sh
│   │   ├── p1-workspace.sh
│   │   └── p1-multi-task.sh
│   └── baselines/                    # Reference screenshots (committed)
│       ├── p0-cold-start-01.png
│       ├── p0-cold-start-02.png
│       └── ...
└── package.json                      # e2e:tier1, e2e:tier2, e2e:tier3, e2e:all scripts
```

---

## Release Gate Rules

| Tier | Trigger | Pass Criteria |
|------|---------|---------------|
| Tier 1 | Before every commit / local dev | 55/55 pass |
| Tier 2 | Before PR / release candidate | 30/30 pass |
| Tier 3 | Before tagging release | All P0 journeys pass (6/6). P1 failures are advisory. |

**Release is blocked if:** Any Tier 1 or Tier 2 test fails, OR any P0 Tier 3 journey fails.
**Release proceeds with advisory if:** Only P1 Tier 3 journeys have screenshot diffs.
