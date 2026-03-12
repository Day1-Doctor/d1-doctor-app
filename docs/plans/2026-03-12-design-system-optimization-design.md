# Design System Optimization — d1-doctor-app

**Date:** 2026-03-12
**Status:** COMPLETED — Merged via PR #37, released as v2.6.1
**Author:** Senior Design Audit
**Scope:** All 26 Vue components across Full, Copilot, and Ninja modes
**Goal:** Elevate the app from functional (3.8/5) to premium Raycast/Linear quality (4.5+/5)

---

## Current State Assessment

### Strengths
- Strong dark-first aesthetic with cohesive orange accent (#F97316)
- Monospace-only typography (Geist Mono) reinforces technical brand
- Glassmorphic surfaces with backdrop-filter create proper visual hierarchy
- Three distinct UI modes (Full/Copilot/Ninja) with clear purpose
- Reduced motion support via `@media (prefers-reduced-motion: reduce)`
- Semantic HTML foundations (role, aria-live in some places)

### Critical Issues Found
1. **Focus indicators missing** on ~60% of interactive elements (fails WCAG AAA 2.4.7)
2. **Hardcoded colors** despite tokens existing (surfaces, errors, traffic lights, text-on-accent)
3. **Placeholder contrast** fails WCAG AA (2.45:1 ratio, needs 4.5:1)
4. **Undefined token reference** (`--surface-2` in ModeBar)
5. **Wrong color fallbacks** (`#6366f1` indigo used as accent fallback instead of `#F97316` orange)
6. **No spacing tokens** — all 26 components hardcode padding/gap/margin
7. **No typography size tokens** — font sizes 9-20px with no system
8. **No transition tokens** — durations range 0.1s to 0.5s with no consistency
9. **Animations defined inline** in 3 components (not centralized)
10. **Scrollbar contrast** fails WCAG 1.4.11 (1.2:1 ratio)

### WCAG AAA Compliance: ~35% (target: 100%)

---

## Architecture: Layer Cake (4 PRs)

Each layer builds on the previous. Each is a standalone improvement.

```
Layer 1: Foundation ──→ Layer 2: Normalize ──→ Layer 3: Accessibility ──→ Layer 4: Polish
(tokens only)          (swap hardcoded→tokens)  (focus, ARIA, contrast)   (micro-interactions)
Zero visual diff       Zero visual diff          Visible improvements      Premium feel
```

---

## Layer 1: Foundation — Token Expansion

**PR scope:** `tokens.css` + `animations.css` only. No component changes.

### New Tokens in `tokens.css`

#### Spacing (4px grid)
```css
--space-2xs: 2px;
--space-xs: 4px;
--space-sm: 8px;
--space-md: 12px;
--space-lg: 16px;
--space-xl: 24px;
--space-2xl: 32px;
--space-3xl: 48px;
```

#### Typography Sizes
```css
--font-size-2xs: 9px;
--font-size-xs: 10px;
--font-size-sm: 11px;
--font-size-base: 12px;
--font-size-md: 13px;
--font-size-lg: 14px;
--font-size-xl: 16px;
--font-size-2xl: 18px;
--font-size-3xl: 20px;
```

#### Typography Weights & Line Heights
```css
--font-weight-light: 300;
--font-weight-normal: 400;
--font-weight-medium: 500;
--font-weight-semibold: 600;
--font-weight-bold: 700;
--line-height-tight: 1.2;
--line-height-base: 1.5;
--line-height-relaxed: 1.6;
```

#### Transition Durations & Easings
```css
--duration-instant: 0.1s;
--duration-fast: 0.15s;
--duration-base: 0.2s;
--duration-slow: 0.3s;
--duration-slower: 0.5s;
--easing-default: ease;
--easing-out: cubic-bezier(0.16, 1, 0.3, 1);
--easing-in: cubic-bezier(0.7, 0, 0.84, 0);
```

#### Backdrop Filters
```css
--backdrop-sm: blur(24px) saturate(130%);
--backdrop-md: blur(30px) saturate(140%);
--backdrop-lg: blur(40px) saturate(160%);
--backdrop-xl: blur(50px) saturate(180%);
```

#### Shadows
```css
--shadow-sm: 0 2px 10px rgba(0, 0, 0, 0.3);
--shadow-md: 0 8px 32px rgba(0, 0, 0, 0.4);
--shadow-lg: 0 24px 64px rgba(0, 0, 0, 0.6);
--shadow-xl: 0 24px 64px rgba(0, 0, 0, 0.8);
```

#### Missing Semantic Colors
```css
--text-on-accent: #000;
--text-contrast: #fff;
--text-placeholder: #878787;
--border-translucent: rgba(255, 255, 255, 0.08);
--success-dark: #16a34a;
--color-disabled-bg: var(--muted);
--color-disabled-text: var(--text-disabled);
--scrollbar-thumb: #666666;
```

### Animations Expansion (`animations.css`)

Centralize from inline definitions:
- `pulse` (from ConnectionStatus) — 1s infinite opacity cycle
- `spin` (from LoginScreen) — 0.6s linear infinite rotation

Add new:
- `scaleIn` — 0→1 scale with fade, 0.15s, for menu items and badges
- `shakeX` — horizontal shake, 0.4s, for error states

**Estimated diff:** ~120 lines added, 0 lines changed in components.

---

## Layer 2: Normalize — Hardcoded → Tokens

**PR scope:** All 26 component `<style>` blocks. Zero visual change.

### Surface Colors (6 files)
| File | Find | Replace |
|------|------|---------|
| TitleBar.vue | `rgba(13,13,13,0.82)` | `var(--surface-title-bar)` |
| Sidebar.vue | `rgba(10,10,10,0.78)` | `var(--surface-sidebar)` |
| ChatWorkspace.vue | `rgba(5,5,5,0.75)` | `var(--surface-chat)` |
| UtilityPanel.vue | `rgba(13,13,13,0.78)` | `var(--surface-utility)` |
| NinjaBar.vue | `rgba(5,5,5,0.82)` | `var(--surface-ninja-bar)` |
| NinjaDropdown.vue | `rgba(5,5,5,0.88)` | New token: `var(--surface-ninja-dropdown)` |

### Error/Semantic Colors (3 files)
| File | Find | Replace |
|------|------|---------|
| ChatWorkspace.vue | `rgba(239,68,68,0.12)` | `var(--error-soft)` |
| ChatWorkspace.vue | `rgba(239,68,68,0.3)` | `var(--error-border)` |
| ChatWorkspace.vue | `#ef4444` | `var(--error)` |
| App.vue | `#ef4444`, `#fff` | `var(--error)`, `var(--text-contrast)` |
| CreditBar.vue | Remove `#ef4444`, `#f59e0b` fallbacks | Just `var(--error)`, `var(--warning)` |

### Text on Accent (5 occurrences)
All `color: #000` on orange → `var(--text-on-accent)`
All `color: #fff` on colored → `var(--text-contrast)`
- TitleBar logo, Sidebar logo, ChatWorkspace send, CopilotHeader logo, CopilotInput send

### Ninja Send Button Consistency
Change NinjaBar send `color: white` → `var(--text-on-accent)` (black, matching Copilot)

### Traffic Lights (TitleBar.vue)
`#FF5F57` → `var(--traffic-close)`
`#FFBD2E` → `var(--traffic-minimize)`
`#28C840` → `var(--traffic-maximize)`

### Glass Borders (NinjaBar, NinjaDropdown)
`rgba(255,255,255,0.08)` → `var(--border-translucent)`

### Broken References
- ModeBar: `var(--surface-2, rgba(0,0,0,0.3))` → `var(--muted)`
- ModeBar/ConnectionStatus: `var(--accent, #6366f1)` → `var(--accent)`

### Backdrop Filters (4 files)
Inline `backdrop-filter: blur(X) saturate(Y)` → `var(--backdrop-sm/md/lg/xl)`

### Spacing (all 26 files)
Replace all hardcoded `padding`, `gap`, `margin` values with `var(--space-*)` tokens.

### Typography (all 26 files)
Replace all hardcoded `font-size`, `font-weight` with `var(--font-size-*)`, `var(--font-weight-*)`.

### Transitions (all files with transitions)
Replace hardcoded durations/easings with `var(--duration-*)`, `var(--easing-*)`.
Fix slow transitions: UtilityPanel chevron 0.2s → `var(--duration-fast)`, App mode-switch 0.3s → `var(--duration-fast)`.

### Misc Fixes
- NinjaApp.vue: `'Geist Mono'` → `var(--font-mono)`
- Sidebar avatar: `1.5px` → `1px`
- NinjaBar logo gradient: Extract to `--gradient-accent: linear-gradient(135deg, var(--accent), var(--accent-hover))`

**Estimated diff:** ~400 lines changed across 26 files, zero visual change.

---

## Layer 3: Accessibility — WCAG AAA

**PR scope:** Focus rings, ARIA, contrast fixes, semantic HTML.

### Global Focus Ring
Add to shared styles (or tokens.css `:root` scope):
```css
*:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}
```

### Per-Component Focus Overrides
- Input pills: `box-shadow: 0 0 0 3px var(--accent-soft)` with `transition: box-shadow var(--duration-fast)`
- Nav items: Left-border accent + background shift (already exists for active, extend to focus)
- Buttons inside colored contexts: Override outline color if needed

### Focus Ring Coverage (14 elements)
- TitleBar: icon buttons
- Sidebar: nav items, sign-out button
- ChatWorkspace: send button
- UtilityPanel: section headers
- CopilotHeader: traffic lights, icon buttons
- CopilotInput: send button
- NinjaBar: send button
- NinjaDropdown: approve button, dismiss button
- ModeSwitcher: mode buttons
- ModeBar: mode buttons
- ConnectionStatus: reconnect button
- UpdateBanner: primary button, secondary button
- CreditBar: buy link (change to `<button>`)
- PlanCard: approve button, reject button

### Contrast Fixes
| Element | Current | Fix | Result |
|---------|---------|-----|--------|
| Placeholder text | `--text-disabled` (#555555) | `var(--text-placeholder)` (#878787) | 5.1:1 ✓ |
| Scrollbar thumb | `var(--border)` (#242424) | `var(--scrollbar-thumb)` (#666666) | 3.2:1 ✓ |
| Sidebar section labels | `--text-disabled` | `var(--text-muted)` (#878787) | 5.1:1 ✓ |
| SessionBar disconnected dot | `--text-disabled` (#555555) | `var(--text-muted)` (#878787) | 5.1:1 ✓ |
| StepTimeline pending dot | `--text-disabled` (#555555) | `var(--text-muted)` (#878787) | 5.1:1 ✓ |

### ARIA Additions
| Component | Attribute | Value |
|-----------|-----------|-------|
| UtilityPanel headers | `aria-expanded` | `true`/`false` per section state |
| UtilityPanel headers | `aria-controls` | `section-{name}` linking to body |
| StepTimeline active | `aria-current` | `"step"` |
| CopilotMode messages | `role` | `"log"` |
| CopilotMode messages | `aria-live` | `"polite"` |
| SessionBar status dot | `aria-label` | Dynamic: "Connected"/"Connecting"/"Disconnected" |
| NinjaDropdown progress | `role` | `"progressbar"` |
| NinjaDropdown progress | `aria-valuenow` / `aria-valuemax` | Dynamic values |
| NinjaBar input | `aria-describedby` | Link to hints element |
| App error banner | `aria-live` | `"assertive"` |
| App error banner | `aria-atomic` | `"true"` |
| CopilotHeader | `role` | `"banner"` |

### Semantic HTML
- CreditBar: `<a class="buy-link">` → `<button class="buy-link">`

**Estimated diff:** ~150 lines changed across 18 files.

---

## Layer 4: Polish — Premium Quality

**PR scope:** Animations, micro-interactions, visual refinements.

### Button Press Feedback (global)
```css
button:active:not(:disabled) {
  transform: scale(0.97);
}
```

### Send Button Glow
```css
.send-btn:hover:not(:disabled) {
  box-shadow: 0 0 16px var(--accent-glow);
}
```

### Input Pill Focus Transition
Add `transition: border-color var(--duration-fast), box-shadow var(--duration-fast)` to all input pills.

### Message Entrance
Enhance from `fadeIn 0.15s` to:
```css
@keyframes messageIn {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}
```

### Plan Card Step Stagger
Each StepIndicator fades in 50ms after the previous via `animation-delay: calc(var(--step-index) * 50ms)`.

### Utility Panel Collapse
Smooth collapse/expand with `max-height` + `opacity` transition instead of instant toggle.

### Keyboard Hints Depth
```css
.kbd {
  box-shadow: inset 0 -1px 0 var(--border), 0 1px 2px rgba(0, 0, 0, 0.3);
}
```

### Traffic Light Hover
```css
.traffic-btn:hover { opacity: 0.85; }
```

### SessionBar Connecting Pulse
Add `animation: pulse 1s infinite` to `.status-dot.connecting`.

### Scrollbar Reveal on Hover
```css
::-webkit-scrollbar-thumb { background: transparent; }
*:hover > ::-webkit-scrollbar-thumb { background: var(--scrollbar-thumb); }
```

### Empty State Enrichment
- CopilotMode: subtitle 11px → `--font-size-base`, `--text-disabled` → `--text-secondary`
- Add subtle pulsing dot next to placeholder text

### NinjaDropdown Query Echo
Change echo text from `--text-muted` → `--text-primary` for prominence.

### UpdateBanner Gradient
`#16a34a` → `var(--success-dark)`

### Centralize Inline Animations
Move `pulse` from ConnectionStatus and `spin` from LoginScreen to `animations.css`.

**Estimated diff:** ~200 lines changed across 20 files.

---

## Success Criteria

| Metric | Before | After |
|--------|--------|-------|
| WCAG AAA compliance | ~35% | 100% |
| Focus ring coverage | ~40% | 100% |
| Token usage | ~60% | 100% |
| Design system score | 3.8/5 | 4.5+/5 |
| Hardcoded color count | 20+ | 0 |
| Hardcoded spacing count | 50+ | 0 |
| Inline animation defs | 3 | 0 |

---

## Files Affected

### Layer 1 (2 files)
- `shared/styles/tokens.css`
- `shared/styles/animations.css`

### Layer 2 (26 files)
- All component `.vue` files in `shared/components/`, `modes/full/`, `modes/copilot/`, `modes/ninja/`
- `App.vue`, `NinjaApp.vue`

### Layer 3 (18 files)
- Same component files + shared styles for global focus ring

### Layer 4 (20 files)
- Same component files + `animations.css`
