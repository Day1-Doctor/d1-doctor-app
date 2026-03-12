<template>
  <div class="copilot-header" role="banner">
    <div class="traffic-lights">
      <div class="traffic-dot close" title="Close" @click="closeWindow" />
      <div class="traffic-dot minimize" title="Minimize" @click="minimizeWindow" />
      <div class="traffic-dot maximize" title="Maximize" @click="toggleMaximize" />
    </div>
    <div class="app-logo" aria-label="Day1 Doctor">dr. D1</div>
    <span class="title">Day1 Doctor</span>
    <div class="header-actions">
      <button class="icon-btn" disabled aria-label="Settings (coming soon)" title="Settings">⚙</button>
      <button class="icon-btn" disabled aria-label="Switch mode (coming soon)" title="Switch mode">⊞</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'

const appWindow = getCurrentWindow()

function closeWindow(): void { void appWindow.close() }
function minimizeWindow(): void { void appWindow.minimize() }
function toggleMaximize(): void { void appWindow.toggleMaximize() }
</script>

<style scoped>
.copilot-header {
  height: 50px;
  background: var(--surface-title-bar);
  backdrop-filter: var(--backdrop-md);
  -webkit-backdrop-filter: var(--backdrop-md);
  border-bottom: 1px solid var(--border);
  -webkit-app-region: drag;
  display: flex;
  align-items: center;
  padding: 0 var(--space-lg);
  gap: var(--space-sm);
  flex-shrink: 0;
}

.traffic-lights {
  -webkit-app-region: no-drag;
  display: flex;
  gap: var(--space-xs);
  flex-shrink: 0;
}

.traffic-dot {
  width: var(--space-sm);
  height: var(--space-sm);
  border-radius: 50%;
  cursor: pointer;
  flex-shrink: 0;
}

.close    { background: var(--traffic-close); }
.minimize { background: var(--traffic-minimize); }
.maximize { background: var(--traffic-maximize); }

.traffic-dot:hover { opacity: 0.85; }

.app-logo {
  -webkit-app-region: no-drag;
  width: var(--space-xl);
  height: var(--space-xl);
  border-radius: 50%;
  background: var(--accent);
  color: var(--text-on-accent);
  font: var(--font-weight-bold) var(--font-size-2xs) var(--font-mono);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  user-select: none;
}

.title {
  -webkit-app-region: no-drag;
  flex: 1;
  font: var(--font-size-base) var(--font-mono);
  color: var(--text-primary);
  user-select: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.header-actions {
  -webkit-app-region: no-drag;
  display: flex;
  gap: var(--space-xs);
  flex-shrink: 0;
}

.icon-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: var(--font-size-md);
  width: var(--space-xl);
  height: var(--space-xl);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  transition: color var(--duration-fast), background var(--duration-fast);
  padding: 0;
  line-height: 1;
}

.icon-btn:hover {
  color: var(--text-primary);
  background: var(--muted);
}

.icon-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.icon-btn:disabled:hover {
  color: var(--text-muted);
  background: transparent;
}
</style>
