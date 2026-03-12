# Design System Optimization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Elevate the d1-doctor-app design system from 3.8/5 to 4.5+/5 quality across all 26 Vue components, achieving WCAG AAA compliance, 100% token usage, and Raycast/Linear-level polish.

**Architecture:** 4 sequential PRs (Layer Cake): Foundation → Normalize → Accessibility → Polish. Each layer builds on the previous and is independently shippable.

**Tech Stack:** Vue 3 SFCs with scoped CSS, CSS custom properties in `tokens.css`, keyframe animations in `animations.css`. No external CSS frameworks.

**Base path:** All file paths relative to `crates/desktop/src/`

---

## Layer 1: Foundation — Token Expansion

### Task 1: Expand tokens.css with spacing, typography, transition, backdrop, and shadow tokens

**Files:**
- Modify: `shared/styles/tokens.css`

**Step 1: Add spacing tokens after line 38 (after `--traffic-maximize`)**

Add inside the existing `:root` block, after the traffic light tokens:

```css
  /* ── Spacing (4px grid) ── */
  --space-2xs: 2px;
  --space-xs: 4px;
  --space-sm: 8px;
  --space-md: 12px;
  --space-lg: 16px;
  --space-xl: 24px;
  --space-2xl: 32px;
  --space-3xl: 48px;

  /* ── Typography sizes ── */
  --font-size-2xs: 9px;
  --font-size-xs: 10px;
  --font-size-sm: 11px;
  --font-size-base: 12px;
  --font-size-md: 13px;
  --font-size-lg: 14px;
  --font-size-xl: 16px;
  --font-size-2xl: 18px;
  --font-size-3xl: 20px;

  /* ── Typography weights & line heights ── */
  --font-weight-light: 300;
  --font-weight-normal: 400;
  --font-weight-medium: 500;
  --font-weight-semibold: 600;
  --font-weight-bold: 700;
  --line-height-tight: 1.2;
  --line-height-base: 1.5;
  --line-height-relaxed: 1.6;

  /* ── Transition durations & easings ── */
  --duration-instant: 0.1s;
  --duration-fast: 0.15s;
  --duration-base: 0.2s;
  --duration-slow: 0.3s;
  --duration-slower: 0.5s;
  --easing-default: ease;
  --easing-out: cubic-bezier(0.16, 1, 0.3, 1);
  --easing-in: cubic-bezier(0.7, 0, 0.84, 0);

  /* ── Backdrop filters ── */
  --backdrop-sm: blur(24px) saturate(130%);
  --backdrop-md: blur(30px) saturate(140%);
  --backdrop-lg: blur(40px) saturate(160%);
  --backdrop-xl: blur(50px) saturate(180%);

  /* ── Shadows ── */
  --shadow-sm: 0 2px 10px rgba(0, 0, 0, 0.3);
  --shadow-md: 0 8px 32px rgba(0, 0, 0, 0.4);
  --shadow-lg: 0 24px 64px rgba(0, 0, 0, 0.6);
  --shadow-xl: 0 24px 64px rgba(0, 0, 0, 0.8);

  /* ── Semantic additions ── */
  --text-on-accent: #000;
  --text-contrast: #fff;
  --text-placeholder: #878787;
  --border-translucent: rgba(255, 255, 255, 0.08);
  --success-dark: #16a34a;
  --color-disabled-bg: var(--muted);
  --color-disabled-text: var(--text-disabled);
  --scrollbar-thumb: #666666;
  --gradient-accent: linear-gradient(135deg, var(--accent), var(--accent-hover));
```

**Step 2: Verify tokens.css is valid**

Run: `cd /Users/wenqingyu/Documents/workspace/day1-doctor/d1-doctor-app && npx vite build --mode development 2>&1 | head -20`
Expected: No CSS parse errors. Build may fail for other reasons but CSS should be clean.

**Step 3: Commit**

```bash
git add crates/desktop/src/shared/styles/tokens.css
git commit -m "feat: expand design tokens with spacing, typography, transitions, backdrop, shadows"
```

---

### Task 2: Expand animations.css with centralized and new animations

**Files:**
- Modify: `shared/styles/animations.css`

**Step 1: Add centralized and new animations**

Add before the `@media (prefers-reduced-motion)` block (before line 17):

```css
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@keyframes messageIn {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes scaleIn {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}

@keyframes shakeX {
  0%, 100% { transform: translateX(0); }
  20%, 60% { transform: translateX(-4px); }
  40%, 80% { transform: translateX(4px); }
}
```

**Step 2: Commit**

```bash
git add crates/desktop/src/shared/styles/animations.css
git commit -m "feat: centralize pulse/spin animations, add messageIn/scaleIn/shakeX"
```

---

## Layer 2: Normalize — Replace Hardcoded Values With Tokens

### Task 3: Normalize App.vue and FullMode.vue

**Files:**
- Modify: `App.vue`
- Modify: `modes/full/FullMode.vue`

**Step 1: Normalize App.vue hardcoded colors and spacing**

In `App.vue` `<style scoped>`, make these replacements:

| Find (exact) | Replace with |
|---|---|
| `background: #ef4444` | `background: var(--error)` |
| `color: #fff` (in `.daemon-error-banner`) | `color: var(--text-contrast)` |
| `rgba(0, 0, 0, 0.2)` (code bg) | `var(--color-disabled-bg)` |
| `rgba(255, 255, 255, 0.8)` (dismiss btn) | `var(--text-placeholder)` |
| `color: #fff` (in `.banner-dismiss:hover`) | `color: var(--text-contrast)` |
| `padding: 8px 16px` | `padding: var(--space-sm) var(--space-lg)` |
| `gap: 12px` | `gap: var(--space-md)` |
| `padding: 2px 6px` (code) | `padding: var(--space-2xs) var(--space-xs)` |
| `border-radius: 4px` | `border-radius: var(--space-xs)` |
| `padding: 0 4px` | `padding: 0 var(--space-xs)` |
| `font-size: 12px` (banner) | `font-size: var(--font-size-base)` |
| `font-size: 14px` (dismiss) | `font-size: var(--font-size-lg)` |
| `transition: color 0.1s` | `transition: color var(--duration-instant)` |
| `transition: opacity 0.3s ease` (mode-switch) | `transition: opacity var(--duration-fast) var(--easing-default)` |

In `FullMode.vue`:
| Find | Replace |
|---|---|
| `top: 8px; right: 8px` | `top: var(--space-sm); right: var(--space-sm)` |

**Step 2: Verify build**

Run: `cd /Users/wenqingyu/Documents/workspace/day1-doctor/d1-doctor-app/crates/desktop && npx vite build 2>&1 | tail -5`
Expected: Build succeeds or only pre-existing errors.

**Step 3: Commit**

```bash
git add crates/desktop/src/App.vue crates/desktop/src/modes/full/FullMode.vue
git commit -m "refactor: normalize App.vue and FullMode.vue to use design tokens"
```

---

### Task 4: Normalize TitleBar.vue

**Files:**
- Modify: `modes/full/TitleBar.vue`

**Step 1: Replace all hardcoded values**

In `<style scoped>`:

| Find | Replace |
|---|---|
| `rgba(13, 13, 13, 0.82)` | `var(--surface-title-bar)` |
| `backdrop-filter: blur(30px) saturate(150%)` | `backdrop-filter: var(--backdrop-md)` |
| `-webkit-backdrop-filter: blur(30px) saturate(150%)` | `-webkit-backdrop-filter: var(--backdrop-md)` |
| `height: 38px` | `height: 38px` (keep — structural, not a spacing token) |
| `padding: 0 16px` | `padding: 0 var(--space-lg)` |
| `gap: 8px` (traffic lights) | `gap: var(--space-sm)` |
| `margin-right: 16px` | `margin-right: var(--space-lg)` |
| `width: 12px; height: 12px` (traffic dot) | `width: var(--space-md); height: var(--space-md)` |
| `#FF5F57` | `var(--traffic-close)` |
| `#FFBD2E` | `var(--traffic-minimize)` |
| `#28C840` | `var(--traffic-maximize)` |
| `font: 12px/1 var(--font-mono)` | `font: var(--font-size-base)/var(--line-height-tight) var(--font-mono)` |
| `gap: 4px` (title actions) | `gap: var(--space-xs)` |
| `margin-left: 16px` | `margin-left: var(--space-lg)` |
| `width: 24px; height: 24px` (icon btn) | `width: var(--space-xl); height: var(--space-xl)` |
| `font-size: 14px` (icon btn) | `font-size: var(--font-size-lg)` |
| `transition: color 0.15s, background 0.15s` | `transition: color var(--duration-fast), background var(--duration-fast)` |

**Step 2: Commit**

```bash
git add crates/desktop/src/modes/full/TitleBar.vue
git commit -m "refactor: normalize TitleBar.vue to use design tokens"
```

---

### Task 5: Normalize Sidebar.vue

**Files:**
- Modify: `modes/full/Sidebar.vue`

**Step 1: Replace all hardcoded values**

Key replacements in `<style scoped>`:

| Find | Replace |
|---|---|
| `rgba(10, 10, 10, 0.78)` | `var(--surface-sidebar)` |
| `backdrop-filter: blur(30px) saturate(140%)` | `backdrop-filter: var(--backdrop-md)` |
| `-webkit-backdrop-filter: blur(30px) saturate(140%)` | `-webkit-backdrop-filter: var(--backdrop-md)` |
| `color: #000` (logo circle) | `color: var(--text-on-accent)` |
| `border: 1.5px solid var(--border)` (avatar) | `border: 1px solid var(--border)` |
| `padding: 16px 0` | `padding: var(--space-lg) 0` |
| `gap: 10px` (logo) | `gap: var(--space-sm)` |
| `padding: 0 16px 16px` | `padding: 0 var(--space-lg) var(--space-lg)` |
| `margin-bottom: 8px` | `margin-bottom: var(--space-sm)` |
| `width: 34px; height: 34px` | `width: 34px; height: 34px` (keep — logo has fixed size) |
| `font: 700 13px` (logo) | `font: var(--font-weight-bold) var(--font-size-md)` |
| `font: 600 12px` (logo name) | `font: var(--font-weight-semibold) var(--font-size-base)` |
| `font-size: 10px` (version) | `font-size: var(--font-size-xs)` |
| `padding: 1px 5px` (badge) | `padding: 1px var(--space-xs)` |
| `padding: 4px 0` (nav) | `padding: var(--space-xs) 0` |
| `gap: 10px` (nav item) | `gap: var(--space-sm)` |
| `padding: 9px 16px` (nav item) | `padding: var(--space-sm) var(--space-lg)` |
| `font-size: 12px` (nav item) | `font-size: var(--font-size-base)` |
| `font-size: 14px` (nav icon) | `font-size: var(--font-size-lg)` |
| `transition: background 0.12s, color 0.12s` | `transition: background var(--duration-fast), color var(--duration-fast)` |
| `font: 700 10px` (section label) | `font: var(--font-weight-bold) var(--font-size-xs)` |
| `padding: 12px 16px 8px` | `padding: var(--space-md) var(--space-lg) var(--space-sm)` |
| `margin-bottom: 8px` (section label) | `margin-bottom: var(--space-sm)` |
| `gap: 6px` (task list) | `gap: var(--space-xs)` |
| `gap: 8px` (task item) | `gap: var(--space-sm)` |
| `width: 6px; height: 6px` (task dot) | `width: 6px; height: 6px` (keep — tiny dot, not a spacing token) |
| `font-size: 11px` (task) | `font-size: var(--font-size-sm)` |
| `padding: 4px 0` (no tasks) | `padding: var(--space-xs) 0` |
| `padding: 12px 16px` (credits) | `padding: var(--space-md) var(--space-lg)` |
| `padding: 12px 16px 0` (user section) | `padding: var(--space-md) var(--space-lg) 0` |
| `width: 30px; height: 30px` (avatar) | `width: 30px; height: 30px` (keep — avatar is fixed) |
| `font: 600 11px` (user name) | `font: var(--font-weight-semibold) var(--font-size-sm)` |
| `font-size: 10px` (user email) | `font-size: var(--font-size-xs)` |
| `font-size: 14px` (sign-out) | `font-size: var(--font-size-lg)` |
| `transition: color 0.15s, background 0.15s` (sign-out) | `transition: color var(--duration-fast), background var(--duration-fast)` |

**Step 2: Commit**

```bash
git add crates/desktop/src/modes/full/Sidebar.vue
git commit -m "refactor: normalize Sidebar.vue to use design tokens"
```

---

### Task 6: Normalize ChatWorkspace.vue

**Files:**
- Modify: `modes/full/ChatWorkspace.vue`

**Step 1: Replace hardcoded values**

| Find | Replace |
|---|---|
| `rgba(5, 5, 5, 0.75)` | `var(--surface-chat)` |
| `backdrop-filter: blur(24px) saturate(130%)` | `backdrop-filter: var(--backdrop-sm)` |
| `-webkit-backdrop-filter: blur(24px) saturate(130%)` | `-webkit-backdrop-filter: var(--backdrop-sm)` |
| `color: #000` (badge) | `color: var(--text-on-accent)` |
| `rgba(239, 68, 68, 0.12)` | `var(--error-soft)` |
| `rgba(239, 68, 68, 0.3)` | `var(--error-border)` |
| `#ef4444` | `var(--error)` |
| `color: #000` (send btn) | `color: var(--text-on-accent)` |
| `padding: 24px` (message list) | `padding: var(--space-xl)` |
| `gap: 16px` (messages) | `gap: var(--space-lg)` |
| `gap: 8px` (empty state) | `gap: var(--space-sm)` |
| `padding: 48px 24px` | `padding: var(--space-3xl) var(--space-xl)` |
| `margin-bottom: 8px` | `margin-bottom: var(--space-sm)` |
| `font-size: 32px` (empty icon) | `font-size: 32px` (keep — emoji size) |
| `font: 600 14px` (empty title) | `font: var(--font-weight-semibold) var(--font-size-lg)` |
| `font-size: 12px` (empty sub) | `font-size: var(--font-size-base)` |
| `padding: 5px 14px` (badge) | `padding: var(--space-xs) var(--space-lg)` |
| `font: 700 11px` (badge) | `font: var(--font-weight-bold) var(--font-size-sm)` |
| `border-radius: 999px` | `border-radius: 999px` (keep — pill shape) |
| `padding: 6px 16px` (disconnect) | `padding: var(--space-xs) var(--space-lg)` |
| `font-size: 11px` (disconnect) | `font-size: var(--font-size-sm)` |
| `padding: 16px 24px` (input bar) | `padding: var(--space-lg) var(--space-xl)` |
| `padding: 12px 16px` (input pill) | `padding: var(--space-md) var(--space-lg)` |
| `gap: 12px` (input pill) | `gap: var(--space-md)` |
| `transition: border-color 0.15s, box-shadow 0.15s` | `transition: border-color var(--duration-fast), box-shadow var(--duration-fast)` |
| `font: 13px/1.6 var(--font-mono)` (textarea) | `font: var(--font-size-md)/var(--line-height-relaxed) var(--font-mono)` |
| `max-height: 160px` | `max-height: 160px` (keep — structural) |
| `width: 28px; height: 28px` (send btn) | `width: 28px; height: 28px` (keep — button size) |
| `font-size: 14px` (send btn) | `font-size: var(--font-size-lg)` |
| `transition: background 0.15s, opacity 0.15s` (send) | `transition: background var(--duration-fast), opacity var(--duration-fast)` |
| `animation: fadeIn 0.15s ease` (badge) | `animation: fadeIn var(--duration-fast) var(--easing-default)` |

**Step 2: Commit**

```bash
git add crates/desktop/src/modes/full/ChatWorkspace.vue
git commit -m "refactor: normalize ChatWorkspace.vue to use design tokens"
```

---

### Task 7: Normalize UtilityPanel.vue

**Files:**
- Modify: `modes/full/UtilityPanel.vue`

**Step 1: Replace hardcoded values**

UtilityPanel already uses color tokens well. Focus on spacing, typography, backdrop, and transitions:

| Find | Replace |
|---|---|
| `backdrop-filter: blur(30px) saturate(140%)` | `backdrop-filter: var(--backdrop-md)` |
| `-webkit-backdrop-filter: blur(30px) saturate(140%)` | `-webkit-backdrop-filter: var(--backdrop-md)` |
| `padding: 16px` (panel) | `padding: var(--space-lg)` |
| `gap: 4px` (panel) | `gap: var(--space-xs)` |
| `padding: 10px 12px` (section header) | `padding: var(--space-sm) var(--space-md)` |
| `font: 700 10px` (section header) | `font: var(--font-weight-bold) var(--font-size-xs)` |
| `letter-spacing: 0.08em` | `letter-spacing: 0.08em` (keep — design choice) |
| `transition: background 0.12s` | `transition: background var(--duration-fast)` |
| `font-size: 12px` (chevron) | `font-size: var(--font-size-base)` |
| `transition: transform 0.2s` (chevron) | `transition: transform var(--duration-fast)` |
| `padding: 8px 12px 12px` (section body) | `padding: var(--space-sm) var(--space-md) var(--space-md)` |
| `gap: 8px` (section body, info row, agent list, agent row) | `gap: var(--space-sm)` |
| `font: 11px var(--font-mono)` (info row) | `font: var(--font-size-sm) var(--font-mono)` |
| `min-width: 72px` (info label) | `min-width: 72px` (keep — layout specific) |
| `padding: 2px 7px` (status badge) | `padding: var(--space-2xs) var(--space-sm)` |
| `font-size: 10px` (badge, refresh, agent status) | `font-size: var(--font-size-xs)` |
| `font-size: 11px` (agent name, empty hint, conn label) | `font-size: var(--font-size-sm)` |
| `padding: 4px 0` (empty hint) | `padding: var(--space-xs) 0` |
| `width: 8px; height: 8px` (conn dot) | `width: var(--space-sm); height: var(--space-sm)` |

**Step 2: Commit**

```bash
git add crates/desktop/src/modes/full/UtilityPanel.vue
git commit -m "refactor: normalize UtilityPanel.vue to use design tokens"
```

---

### Task 8: Normalize Copilot mode components

**Files:**
- Modify: `modes/copilot/CopilotMode.vue`
- Modify: `modes/copilot/CopilotHeader.vue`
- Modify: `modes/copilot/SessionBar.vue`
- Modify: `modes/copilot/CopilotInput.vue`

**Step 1: CopilotMode.vue**

| Find | Replace |
|---|---|
| `backdrop-filter: blur(40px) saturate(160%)` | `backdrop-filter: var(--backdrop-lg)` |
| `-webkit-backdrop-filter: blur(40px) saturate(160%)` | `-webkit-backdrop-filter: var(--backdrop-lg)` |
| `border-radius: 12px` | `border-radius: var(--space-md)` |
| `box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6)` | `box-shadow: var(--shadow-lg)` |
| `color: #000` (badge) | `color: var(--text-on-accent)` |
| `padding: 12px` (messages) | `padding: var(--space-md)` |
| `gap: 8px` (messages) | `gap: var(--space-sm)` |
| `padding: 32px 16px` (empty) | `padding: var(--space-2xl) var(--space-lg)` |
| `gap: 6px` (empty) | `gap: var(--space-xs)` |
| `font: 600 13px` (empty title) | `font: var(--font-weight-semibold) var(--font-size-md)` |
| `font: 11px` (empty sub) | `font: var(--font-size-sm)` |
| `padding: 4px 12px` (badge) | `padding: var(--space-xs) var(--space-md)` |
| `font: 700 10px` (badge) | `font: var(--font-weight-bold) var(--font-size-xs)` |
| `height: 36px` (credit footer) | `height: 36px` (keep — structural) |
| `padding: 0 14px` (credit footer) | `padding: 0 var(--space-lg)` |
| `top: 8px` (mode bar) | `top: var(--space-sm)` |
| `right: 8px` (mode bar) | `right: var(--space-sm)` |

**Step 2: CopilotHeader.vue**

| Find | Replace |
|---|---|
| `backdrop-filter: blur(30px)` | `backdrop-filter: var(--backdrop-md)` |
| `-webkit-backdrop-filter: blur(30px)` | `-webkit-backdrop-filter: var(--backdrop-md)` |
| `color: #000` (logo) | `color: var(--text-on-accent)` |
| `height: 50px` | `height: 50px` (keep — structural) |
| `padding: 0 14px` | `padding: 0 var(--space-lg)` |
| `gap: 10px` | `gap: var(--space-sm)` |
| `gap: 6px` (traffic) | `gap: var(--space-xs)` |
| `width: 10px; height: 10px` (traffic dot) | `width: var(--space-sm); height: var(--space-sm)` |
| `width: 24px; height: 24px` (logo, icon btn) | `width: var(--space-xl); height: var(--space-xl)` |
| `font: 700 9px` (logo) | `font: var(--font-weight-bold) var(--font-size-2xs)` |
| `font: 12px` (title) | `font: var(--font-size-base)` |
| `gap: 4px` (actions) | `gap: var(--space-xs)` |
| `font-size: 13px` (icon btn) | `font-size: var(--font-size-md)` |
| `transition: color 0.15s, background 0.15s` | `transition: color var(--duration-fast), background var(--duration-fast)` |

**Step 3: SessionBar.vue**

| Find | Replace |
|---|---|
| `height: 42px` | `height: 42px` (keep — structural) |
| `padding: 0 14px` | `padding: 0 var(--space-lg)` |
| `gap: 8px` | `gap: var(--space-sm)` |
| `font: 12px var(--font-mono)` | `font: var(--font-size-base) var(--font-mono)` |
| `width: 7px; height: 7px` (status dot) | `width: 7px; height: 7px` (keep — tiny indicator) |
| `font: 11px var(--font-mono)` | `font: var(--font-size-sm) var(--font-mono)` |

**Step 4: CopilotInput.vue**

| Find | Replace |
|---|---|
| `color: #000` (send btn) | `color: var(--text-on-accent)` |
| `padding: 8px 12px` (container) | `padding: var(--space-sm) var(--space-md)` |
| `padding: 8px 12px` (pill) | `padding: var(--space-sm) var(--space-md)` |
| `gap: 8px` (pill) | `gap: var(--space-sm)` |
| `transition: border-color 0.15s, box-shadow 0.15s` | `transition: border-color var(--duration-fast), box-shadow var(--duration-fast)` |
| `font: 12px/1.5 var(--font-mono)` | `font: var(--font-size-base)/var(--line-height-base) var(--font-mono)` |
| `max-height: 50px` | `max-height: 50px` (keep — structural) |
| `width: 24px; height: 24px` (send btn) | `width: var(--space-xl); height: var(--space-xl)` |
| `font-size: 12px` (send btn) | `font-size: var(--font-size-base)` |
| `transition: background 0.15s, opacity 0.15s` | `transition: background var(--duration-fast), opacity var(--duration-fast)` |

**Step 5: Commit**

```bash
git add crates/desktop/src/modes/copilot/
git commit -m "refactor: normalize Copilot mode components to use design tokens"
```

---

### Task 9: Normalize Ninja mode components

**Files:**
- Modify: `modes/ninja/NinjaBar.vue`
- Modify: `modes/ninja/NinjaDropdown.vue`
- Modify: `modes/ninja/StepTimeline.vue`
- Modify: `NinjaApp.vue`

**Step 1: NinjaBar.vue**

| Find | Replace |
|---|---|
| `rgba(5, 5, 5, 0.82)` | `var(--surface-ninja-bar)` |
| `backdrop-filter: blur(50px) saturate(180%)` | `backdrop-filter: var(--backdrop-xl)` |
| `-webkit-backdrop-filter: blur(50px) saturate(180%)` | `-webkit-backdrop-filter: var(--backdrop-xl)` |
| `rgba(255, 255, 255, 0.08)` (border) | `var(--border-translucent)` |
| `0 24px 64px rgba(0, 0, 0, 0.7)` | `var(--shadow-lg)` |
| `rgba(249, 115, 22, 0.1)` (outer glow on box-shadow) | Keep as-is (compound shadow, append to var) |
| `linear-gradient(135deg, #F97316, #EA580C)` | `var(--gradient-accent)` |
| `rgba(249, 115, 22, 0.1)` (logo shadow) | `var(--accent-glow)` |
| `color: white` (logo text) | `color: var(--text-contrast)` |
| `color: white` (send btn) | `color: var(--text-on-accent)` |
| `width: 680px` | `width: 680px` (keep — structural) |
| `height: 64px` | `height: 64px` (keep — structural) |
| `border-radius: 20px` | `border-radius: 20px` (keep — unique pill shape) |
| `padding: 0 20px` | `padding: 0 var(--space-xl)` |
| `gap: 14px` | `gap: var(--space-lg)` |
| `width: 40px; height: 40px` (logo) | `width: 40px; height: 40px` (keep — logo size) |
| `border-radius: 12px` (logo) | `border-radius: var(--space-md)` |
| `font-size: 18px` (logo) | `font-size: var(--font-size-2xl)` |
| `font-weight: 700` (logo) | `font-weight: var(--font-weight-bold)` |
| `font: 16px/1 var(--font-mono)` (input) | `font: var(--font-size-xl)/var(--line-height-tight) var(--font-mono)` |
| `border-radius: 8px` (send) | `border-radius: var(--space-sm)` |
| `width: 32px; height: 32px` (send) | `width: var(--space-2xl); height: var(--space-2xl)` |
| `font-size: 16px` (send) | `font-size: var(--font-size-xl)` |
| `transition: background 0.15s` | `transition: background var(--duration-fast)` |
| `font: 11px var(--font-mono)` (hints) | `font: var(--font-size-sm) var(--font-mono)` |
| `transition: opacity 0.5s ease` (hints) | `transition: opacity var(--duration-slower) var(--easing-default)` |
| `gap: 20px` (hints) | `gap: var(--space-xl)` |

**Step 2: NinjaDropdown.vue**

| Find | Replace |
|---|---|
| `rgba(5, 5, 5, 0.88)` | `var(--surface-ninja-dropdown)` |
| `backdrop-filter: blur(50px) saturate(160%)` | `backdrop-filter: var(--backdrop-xl)` |
| `-webkit-backdrop-filter: blur(50px) saturate(160%)` | `-webkit-backdrop-filter: var(--backdrop-xl)` |
| `rgba(255, 255, 255, 0.08)` | `var(--border-translucent)` |
| `0 24px 64px rgba(0, 0, 0, 0.8)` | `var(--shadow-xl)` |
| `rgba(34, 197, 94, 0.1)` (approve hover) | `var(--success-soft)` |
| `width: 680px` | `width: 680px` (keep) |
| `border-radius: 16px` | `border-radius: var(--space-lg)` |
| `margin-top: 4px` | `margin-top: var(--space-xs)` |
| `padding: 16px 20px 12px` | `padding: var(--space-lg) var(--space-xl) var(--space-md)` |
| `margin-bottom: 4px` | `margin-bottom: var(--space-xs)` |
| `font: 13px var(--font-mono)` | `font: var(--font-size-md) var(--font-mono)` |
| `font: 11px var(--font-mono)` | `font: var(--font-size-sm) var(--font-mono)` |
| `padding: 0 20px 12px` | `padding: 0 var(--space-xl) var(--space-md)` |
| `padding: 12px 20px` | `padding: var(--space-md) var(--space-xl)` |
| `gap: 10px` | `gap: var(--space-sm)` |
| `height: 3px` (progress bar) | `height: 3px` (keep — thin bar) |
| `border-radius: 2px` | `border-radius: var(--space-2xs)` |
| `transition: width 0.3s ease` | `transition: width var(--duration-slow) var(--easing-default)` |
| `gap: 8px` (buttons) | `gap: var(--space-sm)` |
| `padding: 5px 14px` (buttons) | `padding: var(--space-xs) var(--space-lg)` |
| `font: 12px var(--font-mono)` | `font: var(--font-size-base) var(--font-mono)` |
| `transition: background 0.15s` | `transition: background var(--duration-fast)` |
| `animation: slideDown 0.25s cubic-bezier(0.16, 1, 0.3, 1)` | `animation: slideDown 0.25s var(--easing-out)` |

**Step 3: StepTimeline.vue**

| Find | Replace |
|---|---|
| `rgba(34, 197, 94, 0.15)` (done bg) | `var(--success-soft)` |
| `rgba(239, 68, 68, 0.15)` (error bg) | `var(--error-soft)` |
| `padding: 12px 20px` | `padding: var(--space-md) var(--space-xl)` |
| `gap: 12px` (step item) | `gap: var(--space-md)` |
| `padding: 8px 0` | `padding: var(--space-sm) 0` |
| `width: 22px; height: 22px` (dot) | `width: 22px; height: 22px` (keep — specific indicator size) |
| `font: 11px var(--font-mono)` (dot) | `font: var(--font-size-sm) var(--font-mono)` |
| `font: 13px/1.4 var(--font-mono)` (label) | `font: var(--font-size-md)/1.4 var(--font-mono)` |
| `padding-top: 4px` (label) | `padding-top: var(--space-xs)` |
| `font-weight: 600` (active label) | `font-weight: var(--font-weight-semibold)` |
| Connector line `left: 11px`, `width: 2px`, `top: 30px`, `bottom: -8px` | Keep — precise positioning for visual connector |

**Step 4: NinjaApp.vue**

| Find | Replace |
|---|---|
| `font-family: 'Geist Mono', monospace` | `font-family: var(--font-mono)` |
| `var(--success, #22c55e)` | `var(--success)` |
| `var(--warning, #f59e0b)` | `var(--warning)` |
| `var(--error, #ef4444)` | `var(--error)` |
| `gap: 6px` | `gap: var(--space-xs)` |
| `padding: 2px 8px` | `padding: var(--space-2xs) var(--space-sm)` |
| `font-size: 10px` | `font-size: var(--font-size-xs)` |
| `width: 6px; height: 6px` (dot) | `width: 6px; height: 6px` (keep — tiny indicator) |
| `animation: fadeIn 0.3s ease` | `animation: fadeIn var(--duration-slow) var(--easing-default)` |

Also remove the duplicate `@keyframes agentPulse` block from NinjaApp.vue — it's already in `animations.css`.

**Step 5: Commit**

```bash
git add crates/desktop/src/modes/ninja/ crates/desktop/src/NinjaApp.vue
git commit -m "refactor: normalize Ninja mode components to use design tokens"
```

---

### Task 10: Normalize shared components

**Files:**
- Modify: `shared/components/MessageBubble.vue`
- Modify: `shared/components/StepIndicator.vue`
- Modify: `shared/components/PlanCard.vue`
- Modify: `shared/components/CreditBar.vue`
- Modify: `shared/components/AgentAvatar.vue`
- Modify: `shared/components/ModeSwitcher.vue`
- Modify: `shared/components/ModeBar.vue`
- Modify: `shared/components/ConnectionStatus.vue`
- Modify: `shared/components/UpdateBanner.vue`
- Modify: `shared/components/LoginScreen.vue`
- Modify: `shared/components/KbdShortcut.vue`
- Modify: `shared/components/PermissionBadge.vue`
- Modify: `shared/components/ResultCard.vue`

**Step 1: Normalize each component's `<style scoped>` block**

Apply the same pattern to all shared components — replace hardcoded values with tokens. Key changes per component:

**MessageBubble.vue:**
- `padding: 12px 16px` → `var(--space-md) var(--space-lg)`
- `font: 13px/1.6` → `var(--font-size-md)/var(--line-height-relaxed)`
- `gap: 8px` → `var(--space-sm)`
- `margin-bottom: 4px` → `margin-bottom: var(--space-xs)`
- `font-size: 11px` → `var(--font-size-sm)`, `font-weight: 700` → `var(--font-weight-bold)`
- `font-size: 10px` → `var(--font-size-xs)`
- `animation: fadeIn 0.15s ease` → `animation: fadeIn var(--duration-fast) var(--easing-default)`

**PlanCard.vue:**
- `padding: 16px` → `var(--space-lg)`
- `gap: 4px` → `var(--space-xs)`
- `margin-bottom: 8px` → `var(--space-sm)`
- `font: 700 11px` → `var(--font-weight-bold) var(--font-size-sm)`
- `font: 11px` → `var(--font-size-sm)`
- `font: 12px` → `var(--font-size-base)`
- `transition: all 0.15s` → `transition: all var(--duration-fast)`
- `transition: width 0.3s ease` → `transition: width var(--duration-slow) var(--easing-default)`

**CreditBar.vue:**
- Remove all color fallbacks: `var(--error, #ef4444)` → `var(--error)`, `var(--warning, #f59e0b)` → `var(--warning)`
- `gap: 8px` → `var(--space-sm)`, `gap: 6px` → `var(--space-xs)`
- `font: 11px` → `var(--font-size-sm)`, `font-size: 10px` → `var(--font-size-xs)`
- `transition: width 0.3s` → `transition: width var(--duration-slow)`

**ModeSwitcher.vue:**
- `padding: 8px 16px` → `var(--space-sm) var(--space-lg)`
- `font: 700 10px` → `var(--font-weight-bold) var(--font-size-xs)`
- `gap: 4px` → `var(--space-xs)`, `gap: 3px` → `3px` (keep — very tight)
- `font-size: 14px` → `var(--font-size-lg)`
- `font-size: 9px` → `var(--font-size-2xs)`
- `font-weight: 600` → `var(--font-weight-semibold)`
- `transition: background 0.12s, color 0.12s, border-color 0.12s` → `transition: background var(--duration-fast), color var(--duration-fast), border-color var(--duration-fast)`

**ModeBar.vue (critical fixes):**
- `var(--surface-2, rgba(0,0,0,0.3))` → `var(--muted)` (fix undefined token)
- `var(--accent-soft, rgba(99,102,241,0.15))` → `var(--accent-soft)` (remove wrong indigo fallback)
- `var(--accent, #6366f1)` → `var(--accent)` (remove wrong indigo fallback)
- `gap: 2px` → `var(--space-2xs)`
- `padding: 2px` → `var(--space-2xs)`
- `width: 22px; height: 22px` → `22px` (keep — icon size)
- `font-size: 12px` → `var(--font-size-base)`
- `transition: background 0.1s, color 0.1s` → `transition: background var(--duration-instant), color var(--duration-instant)`

**ConnectionStatus.vue:**
- Remove all color fallbacks: `var(--success, #22c55e)` → `var(--success)`, etc.
- Remove `var(--accent, #6366f1)` → `var(--accent)` (fix wrong indigo)
- Remove inline `@keyframes pulse` — now centralized in animations.css
- `padding: 8px 16px` → `var(--space-sm) var(--space-lg)`
- `gap: 4px` → `var(--space-xs)`, `gap: 6px` → `var(--space-xs)`
- `font: 600 11px` → `var(--font-weight-semibold) var(--font-size-sm)`
- `font: 10px` → `var(--font-size-xs)`
- `transition: background 0.12s, color 0.12s` → `transition: background var(--duration-fast), color var(--duration-fast)`

**UpdateBanner.vue:**
- `linear-gradient(135deg, var(--success, #22C55E) 0%, #16a34a 100%)` → `linear-gradient(135deg, var(--success) 0%, var(--success-dark) 100%)`
- `color: #fff` → `var(--text-contrast)` (all instances)
- `#16a34a` (button text) → `var(--success-dark)`
- `rgba(255, 255, 255, 0.2)` → `rgba(255, 255, 255, 0.2)` (keep — alpha on white is unique)
- `padding: 8px 16px` → `var(--space-sm) var(--space-lg)`
- `gap: 12px` → `var(--space-md)`, `gap: 8px` → `var(--space-sm)`
- `font: 12px/1.4` → `var(--font-size-base)/1.4`
- `font-size: 11px` → `var(--font-size-sm)`, `font-weight: 600` → `var(--font-weight-semibold)`
- `transition: opacity 0.15s ease, transform 0.1s ease` → `transition: opacity var(--duration-fast) var(--easing-default), transform var(--duration-instant) var(--easing-default)`
- Enter/leave transitions: Replace `cubic-bezier(0.16, 1, 0.3, 1)` with `var(--easing-out)` and `cubic-bezier(0.7, 0, 0.84, 0)` with `var(--easing-in)`

**LoginScreen.vue:**
- `color: #fff` → `var(--text-contrast)` (all instances)
- `rgba(255, 255, 255, 0.3)` (spinner) → `rgba(255, 255, 255, 0.3)` (keep — alpha white)
- `#16a34a` → `var(--success-dark)`
- Remove inline `@keyframes spin` — now centralized in animations.css
- Replace all font sizes/weights with tokens
- Replace all padding/gap with spacing tokens
- Replace all transition durations with duration tokens

**StepIndicator.vue, AgentAvatar.vue, KbdShortcut.vue, PermissionBadge.vue, ResultCard.vue:**
- Same pattern: replace `font-size: Npx` → `var(--font-size-*)`, `padding: X Y` → `var(--space-*) var(--space-*)`, transitions → `var(--duration-*)`

**Step 2: Verify build**

Run: `cd /Users/wenqingyu/Documents/workspace/day1-doctor/d1-doctor-app/crates/desktop && npx vite build 2>&1 | tail -5`

**Step 3: Commit**

```bash
git add crates/desktop/src/shared/components/
git commit -m "refactor: normalize all shared components to use design tokens"
```

---

## Layer 3: Accessibility — WCAG AAA

### Task 11: Add global focus ring and shared accessibility styles

**Files:**
- Create: `shared/styles/a11y.css`
- Modify: `main.ts` (to import the new stylesheet)

**Step 1: Create a11y.css**

Create `shared/styles/a11y.css`:

```css
/* Global focus ring for WCAG AAA compliance (2.4.7 Focus Visible) */
*:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

/* Override for input pills — use box-shadow ring instead of outline */
.input-pill *:focus-visible,
.copilot-input-pill *:focus-visible {
  outline: none;
  box-shadow: none; /* parent handles the ring */
}

/* Global button press feedback */
button:active:not(:disabled) {
  transform: scale(0.97);
}

/* Scrollbar contrast fix */
::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb) !important;
}
```

**Step 2: Import in main.ts**

Find the existing import of `tokens.css` and `animations.css` in `main.ts` and add:

```typescript
import './shared/styles/a11y.css'
```

Also import in `ninja.ts` if it has separate imports.

**Step 3: Commit**

```bash
git add crates/desktop/src/shared/styles/a11y.css crates/desktop/src/main.ts crates/desktop/src/ninja.ts
git commit -m "feat: add global focus ring and accessibility styles for WCAG AAA"
```

---

### Task 12: Fix contrast failures across components

**Files:**
- Modify: `modes/copilot/CopilotInput.vue` (placeholder)
- Modify: `modes/copilot/SessionBar.vue` (disconnected dot)
- Modify: `modes/ninja/NinjaBar.vue` (placeholder)
- Modify: `modes/ninja/StepTimeline.vue` (pending dot)
- Modify: `modes/full/Sidebar.vue` (section labels)
- Modify: `modes/full/ChatWorkspace.vue` (scrollbar)

**Step 1: Fix placeholder contrast**

In CopilotInput.vue textarea styles, find:
```css
color: var(--text-disabled);
```
(for the `::placeholder` pseudo-element) and change to:
```css
color: var(--text-placeholder);
```

Same change in ChatWorkspace.vue textarea `::placeholder` and NinjaBar.vue input `::placeholder`.

**Step 2: Fix SessionBar disconnected dot**

In SessionBar.vue, find the `.status-dot` disconnected state styling that uses `var(--text-disabled)` and change to `var(--text-muted)`.

**Step 3: Fix StepTimeline pending dot**

In StepTimeline.vue, find `.pending .step-dot` border/color that uses `var(--text-disabled)` and change to `var(--text-muted)`.

**Step 4: Fix Sidebar section labels**

In Sidebar.vue, find `.section-label` color that uses `var(--text-disabled)` and change to `var(--text-muted)`.

**Step 5: Commit**

```bash
git add crates/desktop/src/modes/copilot/CopilotInput.vue crates/desktop/src/modes/copilot/SessionBar.vue crates/desktop/src/modes/ninja/NinjaBar.vue crates/desktop/src/modes/ninja/StepTimeline.vue crates/desktop/src/modes/full/Sidebar.vue crates/desktop/src/modes/full/ChatWorkspace.vue
git commit -m "fix: improve contrast ratios to meet WCAG AAA (placeholder, dots, labels)"
```

---

### Task 13: Add ARIA attributes to all components

**Files:**
- Modify: `modes/full/UtilityPanel.vue`
- Modify: `modes/ninja/StepTimeline.vue`
- Modify: `modes/copilot/CopilotMode.vue`
- Modify: `modes/copilot/SessionBar.vue`
- Modify: `modes/ninja/NinjaDropdown.vue`
- Modify: `modes/ninja/NinjaBar.vue`
- Modify: `App.vue`
- Modify: `modes/copilot/CopilotHeader.vue`
- Modify: `shared/components/CreditBar.vue`

**Step 1: UtilityPanel.vue — add aria-expanded and aria-controls**

On each section header button, add:
```html
:aria-expanded="sections.taskInfo"
aria-controls="section-task-info"
```
And on the corresponding section body div:
```html
id="section-task-info"
```

Repeat for all 5 sections (taskInfo, agents, permissions, health, connection).

**Step 2: StepTimeline.vue — add aria-current**

On the step item div, add:
```html
:aria-current="step.state === 'active' ? 'step' : undefined"
```

**Step 3: CopilotMode.vue — add role and aria-live to messages**

On the messages container div:
```html
role="log"
aria-live="polite"
```

**Step 4: SessionBar.vue — add aria-label to status dot**

```html
:aria-label="`Connection: ${connectionStatus}`"
```

**Step 5: NinjaDropdown.vue — add progressbar role**

On the progress bar element:
```html
role="progressbar"
:aria-valuenow="completedSteps"
:aria-valuemax="totalSteps"
aria-label="Plan progress"
```

**Step 6: NinjaBar.vue — add aria-describedby to input**

```html
aria-describedby="ninja-hints"
```
And on the hints element:
```html
id="ninja-hints"
```

**Step 7: App.vue — add aria-live to error banner**

```html
aria-live="assertive"
aria-atomic="true"
```

**Step 8: CopilotHeader.vue — add role="banner"**

On the header container:
```html
role="banner"
```

**Step 9: CreditBar.vue — change buy link to button**

Change `<a class="buy-link">` to `<button class="buy-link">` and ensure styling is preserved.

**Step 10: Commit**

```bash
git add crates/desktop/src/modes/full/UtilityPanel.vue crates/desktop/src/modes/ninja/StepTimeline.vue crates/desktop/src/modes/copilot/CopilotMode.vue crates/desktop/src/modes/copilot/SessionBar.vue crates/desktop/src/modes/ninja/NinjaDropdown.vue crates/desktop/src/modes/ninja/NinjaBar.vue crates/desktop/src/App.vue crates/desktop/src/modes/copilot/CopilotHeader.vue crates/desktop/src/shared/components/CreditBar.vue
git commit -m "feat: add ARIA attributes for screen reader support (WCAG AAA)"
```

---

## Layer 4: Polish — Premium Quality

### Task 14: Add micro-interactions and animation polish

**Files:**
- Modify: `shared/styles/animations.css` (messageIn already added in Task 2)
- Modify: `shared/components/MessageBubble.vue`
- Modify: `modes/full/ChatWorkspace.vue`
- Modify: `modes/copilot/CopilotInput.vue`
- Modify: `modes/ninja/NinjaBar.vue`

**Step 1: Upgrade message entrance animation**

In MessageBubble.vue, change:
```css
animation: fadeIn var(--duration-fast) var(--easing-default);
```
to:
```css
animation: messageIn var(--duration-fast) var(--easing-out);
```

**Step 2: Add send button glow on hover**

In ChatWorkspace.vue `.send-btn:hover`:
```css
box-shadow: 0 0 16px var(--accent-glow);
```

Same in CopilotInput.vue and NinjaBar.vue send buttons.

**Step 3: Add input pill focus transition**

In ChatWorkspace.vue `.input-pill`, ensure transition includes box-shadow:
```css
transition: border-color var(--duration-fast), box-shadow var(--duration-fast);
```
(This should already be there from Layer 2, verify it's correct.)

**Step 4: Commit**

```bash
git add crates/desktop/src/shared/components/MessageBubble.vue crates/desktop/src/modes/full/ChatWorkspace.vue crates/desktop/src/modes/copilot/CopilotInput.vue crates/desktop/src/modes/ninja/NinjaBar.vue
git commit -m "feat: add message slide-up animation and send button glow"
```

---

### Task 15: Polish traffic lights, connecting state, keyboard hints

**Files:**
- Modify: `modes/full/TitleBar.vue`
- Modify: `modes/copilot/CopilotHeader.vue`
- Modify: `modes/copilot/SessionBar.vue`
- Modify: `shared/components/KbdShortcut.vue`

**Step 1: Add traffic light hover**

In TitleBar.vue, add to the `.traffic-dot` styles:
```css
.traffic-dot:hover {
  opacity: 0.85;
}
```

Same in CopilotHeader.vue for its traffic dots.

**Step 2: Add connecting pulse to SessionBar**

In SessionBar.vue, add to `.status-dot.connecting`:
```css
animation: pulse 1s infinite;
```

**Step 3: Add depth to keyboard hints**

In KbdShortcut.vue `.kbd-key`, replace the current styling with:
```css
box-shadow: inset 0 -1px 0 var(--border), 0 1px 2px rgba(0, 0, 0, 0.3);
```

**Step 4: Commit**

```bash
git add crates/desktop/src/modes/full/TitleBar.vue crates/desktop/src/modes/copilot/CopilotHeader.vue crates/desktop/src/modes/copilot/SessionBar.vue crates/desktop/src/shared/components/KbdShortcut.vue
git commit -m "feat: polish traffic light hover, connecting pulse, keyboard hint depth"
```

---

### Task 16: Polish empty states, query echo, scrollbar reveal

**Files:**
- Modify: `modes/copilot/CopilotMode.vue`
- Modify: `modes/ninja/NinjaDropdown.vue`
- Modify: `modes/full/ChatWorkspace.vue`

**Step 1: Enrich CopilotMode empty state**

In CopilotMode.vue empty state subtitle, change:
```css
font: var(--font-size-sm) var(--font-mono);
color: var(--text-disabled);
```
to:
```css
font: var(--font-size-base) var(--font-mono);
color: var(--text-secondary);
```

**Step 2: Boost NinjaDropdown query echo**

In NinjaDropdown.vue `.query-echo`, change color from `var(--text-muted)` to `var(--text-primary)`.

**Step 3: Scrollbar reveal on hover**

In ChatWorkspace.vue, UtilityPanel.vue, and CopilotMode.vue scrollbar styles, update to:
```css
::-webkit-scrollbar-thumb {
  background: transparent;
  border-radius: var(--space-2xs);
  transition: background var(--duration-base);
}
&:hover::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
}
```

Note: Vue scoped CSS may need `:deep()` or the scrollbar styles in the global a11y.css. Test and adjust.

**Step 4: Commit**

```bash
git add crates/desktop/src/modes/copilot/CopilotMode.vue crates/desktop/src/modes/ninja/NinjaDropdown.vue crates/desktop/src/modes/full/ChatWorkspace.vue
git commit -m "feat: polish empty states, query echo prominence, scrollbar reveal"
```

---

### Task 17: Final build verification and cleanup

**Files:**
- All modified files

**Step 1: Run full build**

```bash
cd /Users/wenqingyu/Documents/workspace/day1-doctor/d1-doctor-app/crates/desktop && npx vite build
```
Expected: Build succeeds with no CSS errors.

**Step 2: Run tests**

```bash
cd /Users/wenqingyu/Documents/workspace/day1-doctor/d1-doctor-app && npm test
```
Expected: All tests pass.

**Step 3: Run lint**

```bash
cd /Users/wenqingyu/Documents/workspace/day1-doctor/d1-doctor-app && npm run lint
```
Expected: No new lint errors.

**Step 4: Visual smoke test checklist**

Manually verify (or describe to user):
- [ ] Full Mode: sidebar, chat, utility panel render correctly
- [ ] Copilot Mode: floating panel, input, credits visible
- [ ] Ninja Mode: bar appears, dropdown works, steps animate
- [ ] Tab through all buttons — orange focus ring visible
- [ ] Send button glows on hover
- [ ] Messages slide up on arrival
- [ ] Keyboard hints have depth/shadow
- [ ] Traffic lights dim on hover
- [ ] Scrollbars appear on hover, hide otherwise
- [ ] Mode switch transition is fast (≤150ms)

---

## PR Strategy

Each layer is one PR:

| PR | Branch | Title | Approx Diff |
|----|--------|-------|-------------|
| 1 | `feature/D1D-xxx-design-tokens-foundation` | `[D1D-xxx] Expand design token system` | +120 lines |
| 2 | `feature/D1D-xxx-normalize-hardcoded-values` | `[D1D-xxx] Normalize all components to use design tokens` | ~400 lines changed |
| 3 | `feature/D1D-xxx-wcag-aaa-accessibility` | `[D1D-xxx] WCAG AAA: focus rings, ARIA, contrast` | ~150 lines changed |
| 4 | `feature/D1D-xxx-design-polish` | `[D1D-xxx] Premium polish: animations, micro-interactions` | ~200 lines changed |

Each PR depends on the previous. Merge in order.
