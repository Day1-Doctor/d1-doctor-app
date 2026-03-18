# Day1 Copilot — Canvas Polish Design

**Date:** 2026-03-18
**Status:** Approved
**Scope:** Office interior richness, Cowork Campus parcel grid, character upgrades

---

## 1. Cowork Campus — Parcel Grid Layout

### Layout
Diamond/cross pattern of 11 parcels. Center row of 3, surrounded by locked parcels radiating outward. Matches the conceptual draft.

### Three Parcel States

**Running Office:**
- Accent border (orange/red glow, animated pulse)
- Interior shows 4-6 mini pixel agents at tiny desks
- Agents animate: typing when busy, idle bobbing when not
- Screen glow intensity reflects busyness
- Progress bar below agents (task completion %)
- Office name (user-editable, click to rename)
- Stats line: "N agents · N skills · N files"
- Click → zoom into office interior view

**Empty (unlocked slot available):**
- Dotted accent border
- "Empty Office" label
- "Set up office" button/prompt
- Slightly brighter background than locked
- Click → assign agents + name the office

**Locked (upgrade needed):**
- Dark card (`#0D0D0D`), no border glow
- Padlock icon (centered, muted gray)
- "LOCKED" text
- "Upgrade plan" subtext (muted)
- "Office #N" label at bottom
- Click → upgrade prompt with tier pricing

### Tier Allocation (updated: Rocket = 8)

| Tier | Price | Max Offices | Monthly DD |
|------|-------|-------------|-----------|
| Free Man | $0 | 1 | 0 (pay-as-go) |
| Mini Shop | $30/mo | 3 | 3,500 DD |
| Rocket Inc | $100/mo | 8 | 15,000 DD |

Example: Mini Shop user with 2 running offices:
- 2 parcels = Running (animated, named)
- 1 parcel = Empty (available to set up)
- 8 parcels = Locked (upgrade needed)

### Active Office Dynamics
- Mini pixel agents at tiny desks (scaled-down SpriteRenderer)
- Typing arms + head bobbing when tasks active
- Idle agents occasionally stretch or look around
- Screen glow pulses: bright = LLM call in progress, dim = idle
- Progress bar: green fill, animates during task execution
- Border glow intensity correlates with activity level

### User-Editable Office Names
- Click the name label → inline text input
- Persisted to `~/.day1copilot/offices.json`
- Default names: "Dr. Bob's Office", "Office #2", etc.
- Max 24 characters

---

## 2. Office Interior — Rich Pixel Detail

### Floor
- Keep isometric diamond grid (current style)
- **Work zone (left 60%):** Dark wood grain tiles (`#2A2018` / `#342820`) with plank pattern via `createPattern()`
- **Rest zone (right 40%):** Cooler tiles (`#1A2830` / `#1E3038`) with checkered pattern
- **Zone divider:** Carpet runner strip (`#3A2828`)
- Subtle 1px borders between tiles (darker shade)

### Walls (top edge)
- Main wall: base (`#1e2a4a`) + trim at top (`#2a3a5a`) + baseboard (`#152040`)
- 3 windows: night sky background, twinkling star animation, moon on center window
- Wall-mounted bookshelves: wooden frame + colored book spines (5+ books each)
- Clock: animated hands (second hand ticks, minute/hour smooth)
- "DAY1" neon sign: accent color glow effect

### Work Zone Desks (per agent)
| Element | Visual Detail |
|---------|--------------|
| Desk surface | Wood top (`#9A7A60`) with edge shadow, 2 drawers with handles |
| Chair | Seat + back + wheels visible, role-colored cushion |
| Monitor | Bezel frame + screen content (code/charts/docs per role) + green glow when active |
| Keyboard | Dark rectangle with key grid pattern below monitor |
| Coffee mug | Brown (`#5A3A20`) with handle, animated steam when agent working |
| Notebook | Small colored rectangle beside keyboard |
| Desk lamp | Warm glow circle when agent active, off when idle |

### Rest Zone Furniture
| Element | Visual Detail |
|---------|--------------|
| Sofa | 3-cushion, throw pillow, role-colored accents |
| Arcade machine | Cabinet + screen content + joystick + glowing buttons |
| Water cooler | Blue bottle on stand, subtle bubble animation |
| Plant | Potted fern, 3-4 leaf layers with shadow |
| Snack table | Small table with donut/coffee pixel items |
| Whiteboard | Wall-mounted, scribble marks, "SPRINT v3.0" text |

### Room Labels
- Increase zone tint to 6-8% opacity for visibility
- Larger labels with subtle background backing
- "WORK ZONE" / "REST ZONE" divider text

---

## 3. Character Upgrades

### Per-Role Customization

| Agent | Hair Color | Hair Style | Accessory | Desk Item |
|-------|-----------|------------|-----------|-----------|
| Dr. Bob | `#4A3728` brown | Short, neat | Stethoscope (neck) | Clipboard |
| Scout | `#2A6A4A` green | Messy, wild | Magnifying glass (hand) | Globe |
| Sage | `#6A4A8A` purple | Neat, parted | Glasses on face | Charts stack |
| Quill | `#C4A040` blonde | Long, flowing | Pen behind ear | Ink bottle |
| Pixel | `#D04080` pink | Spiky | Headphones on head | Extra monitor |
| Atlas | `#D0A030` amber | Buzz cut | Wrench on belt | Toolbox |

### Animation States (Expanded)

| State | At Desk | In Rest Zone | Visual Detail |
|-------|---------|-------------|--------------|
| idle (brief) | Leaning back, occasional stretch | — | Yawn animation every 10s |
| idle (>5s) | — | Walks to rest zone: sofa/arcade/cooler | Mood bubble (pixel icons float up) |
| thinking | Hand on chin, head tilted | — | Thought bubble with "?", monitor shows "..." |
| typing | Rapid arm movement | — | Monitor scrolls code/text, key particles |
| executing | Standing, arms on hips | — | Loading bar on monitor, rotating gear icon |
| browsing | Leaning forward toward screen | — | Monitor shows web-like content |
| paused | Arms crossed | — | Yellow "!" bubble, monitor pause icon |
| error | Hands raised in alarm | — | Red "!!" flash, monitor red X, screen flicker |
| done | Arms raised celebration | — | Green checkmark on monitor, confetti burst (2s) |

### Mood Bubbles (Rest Zone Idle)
Pixel icons that float upward and fade over 3s:
- Coffee cup, music note, game controller, book, lightbulb, star, zzZ (sleeping)
- Random selection every 8-12 seconds
- Unique to each agent's position

### Agent Movement
- Idle agents drift to rest zone after 5s idle
- Walk animation: 4-frame leg cycle
- Path: desk → zone divider → nearest rest furniture
- When task assigned: walk back to desk (reverse path)
- Walking speed: ~2 tiles/second

---

## 4. Implementation Waves

### Wave 1: Office Interior Polish
1. Floor texture system (wood grain + cool tiles + zone divider)
2. Wall rendering (backdrop + windows + shelves + clock + sign)
3. Enhanced desk rendering (drawers, keyboard, lamp, mug, notebook)
4. Rest zone furniture (sofa, arcade, water cooler, plant, whiteboard, snack table)

### Wave 2: Character Upgrades
5. Per-role hair + accessories + desk items
6. Expanded animation states (all 9 states at desk)
7. Agent walking + rest zone idle with mood bubbles
8. Done celebration (confetti particles)

### Wave 3: Cowork Campus Parcel Grid
9. Replace valley isometric with parcel grid layout (11 parcels, diamond)
10. Three parcel states (running/empty/locked) rendering
11. Mini agent scene in running parcels (dynamic animation)
12. Office naming (editable, persisted)
13. Progress bar + stats line on running parcels

### Wave 4: Tier Update + Integration
14. Update Rocket Inc from 10 → 8 spots in billing/tiers
15. Wire parcel count to subscription tier
16. Upgrade prompt from locked parcels
17. Empty office setup flow
