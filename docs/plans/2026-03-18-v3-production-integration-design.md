# Day1 Copilot v3.0 — Production Integration Design

**Date:** 2026-03-18
**Status:** Approved
**Scope:** LLM integration, auth, Cowork Valley, sprites, Stripe, PRD gap closure

---

## 1. Cowork Valley — Subscription Landscape View

The top-level view is a 2.5D isometric "valley" showing all offices as buildings in a campus. This replaces the single Office View as the default landing screen.

### Layout
- Isometric campus with 6 office buildings arranged along paths
- Each building represents one agent's office
- Active offices: lit up, agent visible through window, smoke/activity
- Locked offices: dark, padlock icon, "Upgrade" badge
- Paths connect offices visually

### Tier Mapping
| Tier | Active Offices | Visual |
|------|---------------|--------|
| Free Man | Dr. Bob's Office only | 1 lit, 5 dark |
| Mini Shop | Dr. Bob + Scout's Lab + Sage's Studio | 3 lit, 3 dark |
| Rocket Inc | All 6 offices | Full campus active |

### Interactions
- Click active office → zoom transition into that office's interior (existing OfficeView)
- Click locked office → upgrade prompt overlay with pricing
- Bottom bar: task timeline (persists across views)
- Top bar: DD balance, settings, language toggle

### Navigation
```
Cowork Valley (default)
  ├── Dr. Bob's Office (click) → Office Interior View
  ├── Scout's Lab (click) → Office Interior View
  ├── ... (other active offices)
  ├── Locked Office (click) → Upgrade Prompt
  └── Back button in Office Interior → returns to Valley
```

---

## 2. Auth Flow — "Preview Then Auth"

### Sequence
1. **First launch:** App opens to Cowork Valley with demo animations (no auth needed)
2. **User explores:** Can click Dr. Bob's office, see agents, switch views — all with mock data
3. **User creates a task:** Auth wall appears

### Auth Wall Component
```
┌────────────────────────────────────────────┐
│                                            │
│     Sign in to start working               │
│                                            │
│  ┌──────────────────────────────────────┐  │
│  │  Enter your Day1 API Key             │  │
│  │  d1d_sk_________________________________│  │
│  │  [Connect]                           │  │
│  └──────────────────────────────────────┘  │
│                                            │
│  ── or ──                                  │
│                                            │
│  [Sign in with Day1 Account]               │
│     Opens browser → day1.doctor/auth       │
│                                            │
│  New to Day1? [Create free account]        │
│     Opens browser → day1.doctor/auth/register │
│                                            │
└────────────────────────────────────────────┘
```

### Auth Methods
1. **API Key** (`d1d_sk_*`): Entered in-app, stored in OS keychain via Tauri secure storage. Validated against gateway `/v1/credits/balance`.
2. **Supabase OAuth**: Opens system browser to `day1.doctor/auth/callback?redirect=day1copilot://auth`. App registers a custom URL scheme (`day1copilot://`) to receive the JWT callback.

### Token Storage
- API key → macOS Keychain (via `tauri-plugin-store` with encryption)
- JWT → short-lived, refreshed via Supabase client
- User profile cached locally, refreshed on app launch

### Auth State Store
```typescript
interface AuthState {
  isAuthenticated: boolean;
  userId: string | null;
  apiKey: string | null;  // masked in UI
  ddBalance: number;
  subscriptionTier: 'free' | 'mini_shop' | 'rocket_inc';
  profile: UserProfile | null;
}
```

---

## 3. LLM Integration via Day1 Gateway

### New Gateway Endpoints (dr-agent- prefix)

Add to `d1-doctor-platform/gateway/api/dr_agent.py`:

| Method | Endpoint | Purpose |
|--------|----------|---------|
| POST | `/dr-agent/v1/chat/completions` | Agent LLM calls (tagged source=copilot) |
| POST | `/dr-agent/v1/decompose` | Task decomposition via Claude Sonnet |
| GET | `/dr-agent/v1/session` | Get/create Copilot session for user |
| GET | `/dr-agent/v1/balance` | Quick balance check (lightweight) |

### Chat Completions Flow
Same as existing `/v1/chat/completions` but:
- Extra metadata: `source: "copilot"`, `agent_name`, `agent_role`, `session_id`
- Logged in `api_usage_log` with `source=copilot` for analytics separation
- DD deduction uses same `credit_service.deduct()` — user's single balance

### Task Decomposition Endpoint
```
POST /dr-agent/v1/decompose
Authorization: Bearer d1d_sk_*
Content-Type: application/json

{
  "description": "Research AI agent frameworks and write a comparison report",
  "max_steps": 6
}

Response:
{
  "plan": {
    "steps": [
      {"title": "Research AI frameworks", "role": "researcher", "depends_on": []},
      {"title": "Analyze and compare", "role": "analyst", "depends_on": [0]},
      {"title": "Write comparison report", "role": "writer", "depends_on": [1]},
      {"title": "Save to workspace", "role": "operator", "depends_on": [2]}
    ]
  },
  "dd_cost": 0.05,
  "model_used": "claude-sonnet-4"
}
```

### Copilot Agent Execution Flow
```
User types task
  → Copilot calls POST /dr-agent/v1/decompose (uses Sonnet, ~0.01 DD)
  → For each step:
      → Copilot calls POST /dr-agent/v1/chat/completions
        with agent's system prompt + step context + tools
      → Gateway authenticates, checks balance, routes to LLM
      → Response streamed back to Copilot
      → DD deducted from user's balance
      → Usage logged with agent_name metadata
  → Task completes, artifacts collected
```

---

## 4. DD Credit Integration in Copilot

### Balance Display
- Fetched on auth via `GET /dr-agent/v1/balance`
- Updated after each agent LLM call (response includes remaining balance)
- Credit meter in TopBar shows real-time balance with color transitions
- Cost breakdown per agent available in agent detail panel

### Insufficient Balance
When DD balance < estimated task cost:
- Show warning before starting: "This task may cost ~X DD. Your balance is Y DD."
- If balance hits 0 mid-task: pause agents, show "Top up to continue" prompt
- "Top up" button opens `day1.doctor/dashboard/credits` in browser

### Usage Tracking
- Each LLM call returns `dd_consumed` in response metadata
- Copilot aggregates per-session, per-agent, per-task
- Displayed in Debug View token metrics panel

---

## 5. Sprite Art System

### Approach: AI-Generated Pixel Art
Generate 6 character sprites using consistent pixel art style:

| Character | Visual Description |
|-----------|-------------------|
| Dr. Bob | Manager in suit, glasses, clipboard |
| Scout | Explorer with magnifying glass, backpack |
| Sage | Scholar with charts, lab coat |
| Quill | Writer with notepad, pen |
| Pixel | Coder with laptop, headphones |
| Atlas | Operator with toolbelt, wrench |

### Spritesheet Format
- Each character: 64x64px per frame
- 8 animation states per character
- Spritesheet: 512x64px (8 frames horizontal)
- Saved as `assets/sprites/{agent_name}.png`

### Animation Frame Map
| State | Frames | Speed |
|-------|--------|-------|
| idle | 2 frames (breathing) | 0.5 FPS |
| thinking | 3 frames (thought bubble) | 1 FPS |
| typing | 4 frames (keyboard) | 3 FPS |
| browsing | 3 frames (screen glow) | 1 FPS |
| executing | 3 frames (gears) | 2 FPS |
| paused | 2 frames (wait) | 0.5 FPS |
| error | 2 frames (alarm) | 2 FPS |
| done | 3 frames (celebration) | 1.5 FPS |

### Renderer Update
- `OfficeRenderer.ts` loads sprite images on init
- `drawAgent()` replaced: circle → sprite frame selection based on status + frame counter
- Valley view uses 32x32px mini sprites for building windows

---

## 6. Stripe & Subscription via Web

### Desktop App (Copilot) — No Stripe Checkout
- Shows DD balance and tier status
- "Top Up" and "Upgrade" buttons open browser to web dashboard
- Tier info fetched from user profile (`subscription_tier` field)

### Web Dashboard — Handles All Billing
- Existing checkout flow at `/api/checkout` unchanged
- Add subscription management page (upgrade/downgrade tiers)
- Add new credit packs matching v3.0 pricing (Boost $10/1K DD, Power Pack $50/6K DD)
- Webhook processes payments and credits DD balance

### Profile Enhancement
Add to `profiles` table:
```sql
ALTER TABLE profiles ADD COLUMN subscription_tier TEXT DEFAULT 'free';
ALTER TABLE profiles ADD COLUMN tier_monthly_dd INTEGER DEFAULT 0;
ALTER TABLE profiles ADD COLUMN tier_max_agents INTEGER DEFAULT 1;
ALTER TABLE profiles ADD COLUMN tier_expires_at TIMESTAMPTZ;
```

### Office Spot Enforcement
Copilot reads `tier_max_agents` from profile. Before activating agents:
```rust
if active_agents > profile.tier_max_agents {
    return Err("Upgrade to unlock more offices");
}
```

---

## 7. PRD Gap Closure Checklist

| PRD Feature | Current | Action |
|-------------|---------|--------|
| Real LLM calls | Keyword stub | Gateway dr-agent endpoints |
| Auth system | None | API key + OAuth flow |
| DD balance | Mock | Real gateway balance API |
| DD deduction | None | Gateway metering on each call |
| Cowork Valley | N/A | New top-level view |
| Office Spot gating | Stub tiers | Real enforcement via profile |
| Sprite art | Circles | AI-generated spritesheets |
| Spritesheet renderer | None | Canvas drawImage with frame selection |
| Approval dialog | Backend only | Frontend modal component |
| Built-in Skills | None | 16 skill prompt templates (TOML) |
| Task decomposition (LLM) | Keyword | Claude Sonnet via gateway |
| Stripe/billing | Stub | Web dashboard (existing) + new tiers |
| Hot-update | Stub | Config check + download mechanism |
| Agent auto-discovery | Stub | Real filesystem scan |

---

## 8. Implementation Order

### Wave A: Gateway + Auth (platform + copilot)
1. Add dr-agent endpoints to gateway
2. Add source=copilot to usage logging
3. Implement auth wall in Copilot
4. API key storage in keychain
5. Balance fetch + real credit meter

### Wave B: LLM Integration (copilot)
6. Replace keyword decomposer with gateway /dr-agent/v1/decompose
7. Wire agent LLM calls through gateway /dr-agent/v1/chat/completions
8. Stream responses to Event Bus → UI animations
9. DD deduction tracking per agent

### Wave C: Cowork Valley + Sprites (copilot)
10. Cowork Valley canvas view
11. Office zoom transition
12. AI-generate sprite assets
13. Spritesheet renderer in Canvas 2D
14. Valley mini-sprites for building windows

### Wave D: Subscription + Polish (platform + web + copilot)
15. Profile schema: subscription_tier, tier_max_agents
16. Office Spot enforcement in Copilot
17. Web pricing page update for v3.0 tiers
18. Approval dialog UI
19. Built-in skill templates (16 skills)
20. E2E test full user journey
