<template>
  <div class="title-bar">
    <div class="traffic-lights">
      <div class="traffic-dot close" title="Close" @click="closeWindow" />
      <div class="traffic-dot minimize" title="Minimize" @click="minimizeWindow" />
      <div class="traffic-dot maximize" title="Maximize" @click="toggleMaximize" />
    </div>
    <span class="window-title">Day1 Doctor — Workspace</span>
    <div class="title-actions">
      <button class="icon-btn" title="Grid view" aria-label="Grid view">⊞</button>
      <button class="icon-btn" title="Search" aria-label="Search">🔍</button>
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
.title-bar {
  height: 38px;
  background: var(--surface-title-bar);
  backdrop-filter: var(--backdrop-md);
  -webkit-backdrop-filter: var(--backdrop-md);
  border-bottom: 1px solid var(--border);
  -webkit-app-region: drag;
  display: flex;
  align-items: center;
  padding: 0 var(--space-lg);
  flex-shrink: 0;
}

.traffic-lights {
  -webkit-app-region: no-drag;
  display: flex;
  gap: var(--space-sm);
  margin-right: var(--space-lg);
}

.traffic-dot {
  width: var(--space-md);
  height: var(--space-md);
  border-radius: 50%;
  cursor: pointer;
  flex-shrink: 0;
}

.traffic-dot:hover { opacity: 0.85; }
.close    { background: var(--traffic-close); }
.minimize { background: var(--traffic-minimize); }
.maximize { background: var(--traffic-maximize); }

.window-title {
  -webkit-app-region: no-drag;
  flex: 1;
  text-align: center;
  font: 12px/1 var(--font-mono);
  color: var(--text-muted);
  user-select: none;
}

.title-actions {
  -webkit-app-region: no-drag;
  display: flex;
  gap: var(--space-xs);
  margin-left: var(--space-lg);
}

.icon-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: var(--font-size-lg);
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
</style>
