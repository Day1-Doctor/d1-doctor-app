<template>
  <div class="copilot-header">
    <div class="traffic-lights">
      <div class="traffic-dot close" title="Close" @click="closeWindow" />
      <div class="traffic-dot minimize" title="Minimize" @click="minimizeWindow" />
      <div class="traffic-dot maximize" title="Maximize" @click="toggleMaximize" />
    </div>
    <div class="app-logo" aria-label="Day 1 Doctor">D1</div>
    <span class="title">Day 1 Doctor</span>
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
  backdrop-filter: blur(30px);
  -webkit-backdrop-filter: blur(30px);
  border-bottom: 1px solid var(--border);
  -webkit-app-region: drag;
  display: flex;
  align-items: center;
  padding: 0 14px;
  gap: 10px;
  flex-shrink: 0;
}

.traffic-lights {
  -webkit-app-region: no-drag;
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.traffic-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  cursor: pointer;
  flex-shrink: 0;
}

.close    { background: var(--traffic-close); }
.minimize { background: var(--traffic-minimize); }
.maximize { background: var(--traffic-maximize); }

.app-logo {
  -webkit-app-region: no-drag;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: var(--accent);
  color: #000;
  font: 700 9px var(--font-mono);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  user-select: none;
}

.title {
  -webkit-app-region: no-drag;
  flex: 1;
  font: 12px var(--font-mono);
  color: var(--text-primary);
  user-select: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.header-actions {
  -webkit-app-region: no-drag;
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.icon-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 13px;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  transition: color 0.15s, background 0.15s;
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
