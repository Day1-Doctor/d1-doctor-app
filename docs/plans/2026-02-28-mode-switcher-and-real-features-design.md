# Mode Switcher & Real Feature Implementation Design

**Date:** 2026-02-28
**Status:** Approved
**Scope:** Day 1 Doctor macOS desktop app (Tauri 2.x + Vue 3 + Pinia)

---

## Problem Statement

Four issues exist in the current app after the v2.4.1 launch:

1. **Ninja bar always visible** — the NinjaBar WebviewWindow appears on startup even when mode is Full, and cannot be dismissed from the UI
2. **No mode switcher** — users cannot switch between Full, Copilot, and Ninja modes from the sidebar
3. **Chat not wired to daemon** — messages sent in ChatWorkspace.vue are appended locally only; nothing reaches Bob (the daemon agent) over WebSocket
4. **Recent tasks are placeholders** — the sidebar shows a hardcoded 3-item array instead of real task history

---

## Approach: Two Windows + CSS Layout Shift

One main architectural decision drives everything: **how many Tauri WebviewWindows**.

Chosen approach: **Two windows** — main window handles Full and Copilot via CSS layout + Tauri window resize; NinjaBar remains its own window. Rejected alternatives:

- *Three separate windows*: clean separation but requires Pinia state sync across windows via Tauri events — high complexity
- *Pure Vue router layouts*: zero Tauri changes for mode switch but window never resizes to a proper side-panel shape — doesn't match Copilot UX

---

## Section 1: Mode System Architecture

### Window Model

| Window | Visible In |
|---|---|
| Main window | Full mode, Copilot mode |
| NinjaBar window | Ninja mode only |

### Mode Transition Table

| From → To | Actions |
|---|---|
| Any → **Full** | `appWindow.show()`; restore saved geometry; remove `copilot-layout` CSS class; emit `ninja-bar-hide` event |
| Any → **Copilot** | Save current geometry to store; `appWindow.setSize(LogicalSize(380, 700))`; `appWindow.setPosition(right screen edge)`; add `copilot-layout` CSS class; keep NinjaBar hidden |
| Any → **Ninja** | `appWindow.hide()`; emit `ninja-bar-show` event; NinjaBar window calls `appWindow.show()` |
| **App startup** | Always emit `ninja-bar-hide` on `init()` regardless of saved mode — fixes the always-showing bug |

### Ninja Bar Fix

**Root cause:** `useAppStore.init()` resets saved `'ninja'` mode to `'full'` correctly, but never calls `hide()` on the NinjaBar WebviewWindow. If the window was visible at last quit, it re-appears on next launch.

**Fix:** `init()` always emits a Tauri event `ninja-bar-hide` on startup. `NinjaApp.vue` listens via `listen('ninja-bar-hide', () => appWindow.hide())` and hides unconditionally.

### Copilot Layout (CSS)

When `copilot-layout` class is applied to `#app`:

- Left sidebar collapses to **40px icon-only strip** — nav icons and mode switcher icons remain interactive, labels hide
- Chat panel expands to fill remaining width
- Right panel (`CurrentTask`, `ActiveAgents`) hidden entirely via `display: none`

### Mode Switcher Component

**File:** `crates/desktop/src/shared/components/ModeSwitcher.vue`
**Position:** Left sidebar, above Credits section
**Behavior:** Three icon+label buttons (Full / Copilot / Ninja), mutually exclusive radio-style. In `copilot-layout`, labels hide, icons remain.

Icons:
- Full → grid/layout icon
- Copilot → sidebar-collapse icon
- Ninja → terminal/ghost icon

Clicking a button calls `useAppStore.switchMode(mode)`.

---

## Section 2: Chat → Daemon Wiring

### Current State
`ChatWorkspace.vue` → `submitMessage()` → appends user message to `conversationStore` locally only. `useDaemonConnection.submitTask()` exists but is never called.

### Wire-up Flow

```
User hits Send
  → submitMessage() in ChatWorkspace.vue
  → useDaemonConnection.submitTask(message)
  → WebSocket: { v:1, id, ts, type:"task.submit", payload:{ content } }
  → Daemon processes → streams back events
  → useDaemonConnection receives incoming events:
      task.progress  → append streaming chunk to current assistant message
      task.plan      → render plan card with Approve/Skip buttons inline
      task.completed → close current message, set final status
      task.failed    → show error inline, set error status
```

### Connection-Gated Input

| Daemon State | Input | Send Button |
|---|---|---|
| Connected | Enabled | Orange (active) |
| Connecting / Reconnecting | Enabled | Spinner |
| Disconnected | Enabled | Disabled + tooltip "Daemon offline" |

A slim banner renders above the chat area when disconnected:
`● Daemon offline — trying to reconnect... (attempt N)`

### Plan Approval

When `task.plan` arrives, a plan card renders inline in the chat:
- Checklist of steps with risk tier badges
- Progress bar showing current step
- **Approve** / **Skip** buttons

Clicking Approve calls `useDaemonConnection.approvePlan(planId)` (function already exists in the composable).

### Conversation Store

All events update `useConversationStore` — the same store Full and Copilot layouts share naturally, since both are rendered in the same main WebviewWindow (Approach B benefit; no cross-window sync needed).

---

## Section 3: Recent Tasks — Tauri Command → SQLite

### New Tauri Command: `list_recent_tasks`

**File:** `crates/desktop/src-tauri/src/commands/tasks.rs`

```sql
SELECT id, title, status, created_at
FROM tasks
ORDER BY created_at DESC
LIMIT 20
```

**Return type:** `Vec<TaskSummary>`

```rust
struct TaskSummary {
    id: String,
    title: String,
    status: String,   // "completed" | "failed" | "running"
    created_at: i64,  // Unix timestamp
}
```

DB path: `~/.d1doctor/d1doctor.db` (existing daemon SQLite, read-only open).

### Vue Side — Sidebar.vue

1. **On mount:** `invoke('list_recent_tasks')` → replaces hardcoded array
2. **Live prepend:** incoming `task.completed` / `task.failed` WebSocket events prepend a new `TaskSummary` to the top of the reactive list — no re-query
3. **Cap at 20:** if live list grows past 20, drop the last item

Status → dot color mapping:
- `completed` → green
- `failed` → red
- `running` → orange (animated pulse)

### Error Handling

| Condition | Behavior |
|---|---|
| DB file not found (fresh install) | Return `[]` — sidebar shows "No recent tasks" |
| DB locked (daemon writing) | Open read-only with 500ms busy timeout; on timeout return `[]` silently |

### Capabilities

Add `"core:path-all"` to `capabilities/default.json` so Rust can resolve the `~/.d1doctor/` home path.

---

## Section 4: Connection Status UI

### What We Surface

Two connection hops:

1. **Mac app → Daemon** (`ws://localhost:9876/ws`) — already tracked in `useDaemonConnection.isConnected`
2. **Daemon → Platform server** (`ws://localhost:8000/ws/connect`) — new: daemon sends `status.system` message

### Protocol Addition (daemon)

Daemon sends one new message type on connect and on change:

```json
{ "v": 1, "type": "status.system", "payload": { "platform_connected": true } }
```

Mac app stores `platformConnected` in `useDaemonStore`.

### UI Placement

Bottom of left sidebar, between mode switcher and credits:

```
┌─────────────────────────────┐
│  ● Daemon     Connected     │
│  ● Platform   Connected     │
└─────────────────────────────┘
```

Dot colors: 🟢 Connected · 🟡 Connecting · 🔴 Disconnected

**In Copilot layout** (icon-only strip): collapses to two stacked dots. Hover tooltip: "Daemon: Connected / Platform: Connected".

**On daemon disconnect:** both dots go red (platform is unreachable if daemon is down).

### Click Popover

Clicking opens a small popover:

```
Daemon    ● Connected    ws://localhost:9876
Platform  ● Connected    ws://localhost:8000
           [Reconnect]
```

"Reconnect" calls `useDaemonConnection.reconnect()` to force a new WebSocket attempt.

---

## Files Affected

### New Files
- `crates/desktop/src/shared/components/ModeSwitcher.vue`
- `crates/desktop/src/shared/components/ConnectionStatus.vue`
- `crates/desktop/src-tauri/src/commands/tasks.rs`
- `docs/plans/2026-02-28-mode-switcher-and-real-features-design.md`

### Modified Files
- `crates/desktop/src/shared/stores/app.ts` — window geometry save/restore, init ninja-bar hide
- `crates/desktop/src/shared/stores/daemon.ts` — add `platformConnected` field
- `crates/desktop/src/shared/composables/useDaemonConnection.ts` — handle `status.system` and `task.plan` events; call `submitTask` on send
- `crates/desktop/src/modes/full/Sidebar.vue` — add ModeSwitcher + ConnectionStatus, replace hardcoded tasks
- `crates/desktop/src/modes/full/ChatWorkspace.vue` — wire submitMessage to submitTask, add connection banner, plan card
- `crates/desktop/src/NinjaApp.vue` — listen for `ninja-bar-hide` event on startup
- `crates/desktop/src-tauri/src/lib.rs` — register `list_recent_tasks` command
- `crates/desktop/src-tauri/capabilities/default.json` — add `core:path-all`
- `crates/desktop/src-tauri/tauri.conf.json` — no changes needed (window config already correct)

---

## Out of Scope

- Apple Developer signing / notarization (separate release concern)
- Copilot mode screen-edge magnetic snapping (nice-to-have, not MVP)
- Multi-session conversation history UI
- Platform server health beyond connected/disconnected status
