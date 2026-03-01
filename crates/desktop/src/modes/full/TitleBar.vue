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
  background: rgba(13, 13, 13, 0.82);
  backdrop-filter: blur(30px) saturate(150%);
  -webkit-backdrop-filter: blur(30px) saturate(150%);
  border-bottom: 1px solid var(--border);
  -webkit-app-region: drag;
  display: flex;
  align-items: center;
  padding: 0 16px;
  flex-shrink: 0;
}

.traffic-lights {
  -webkit-app-region: no-drag;
  display: flex;
  gap: 8px;
  margin-right: 16px;
}

.traffic-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  cursor: pointer;
  flex-shrink: 0;
}

.close    { background: #FF5F57; }
.minimize { background: #FFBD2E; }
.maximize { background: #28C840; }

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
  gap: 4px;
  margin-left: 16px;
}

.icon-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 14px;
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
</style>
