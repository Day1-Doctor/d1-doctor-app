<template>
  <div class="app-window">
    <TitleBar />
    <div class="main-content">
      <Sidebar />
      <ChatWorkspace />
      <UtilityPanel />
    </div>
    <div class="status-bar">
      <p
        v-if="daemonStore.currentBobPhrase"
        class="bob-phrase"
        data-testid="bob-phrase"
        aria-live="polite"
      >{{ daemonStore.currentBobPhrase }}</p>
      <span
        class="connection-dot"
        :class="daemonStore.status"
        :title="`Daemon: ${daemonStore.status}`"
        data-testid="connection-dot"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useDaemonStore } from '@/shared/stores/daemon'
import TitleBar from './TitleBar.vue'
import Sidebar from './Sidebar.vue'
import ChatWorkspace from './ChatWorkspace.vue'
import UtilityPanel from './UtilityPanel.vue'

const daemonStore = useDaemonStore()
</script>

<style scoped>
.app-window {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--background);
  color: var(--text-primary);
  font-family: var(--font-mono);
  overflow: hidden;
}

.main-content {
  display: flex;
  flex-direction: row;
  flex: 1;
  overflow: hidden;
  min-height: 0;
}

.status-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 12px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
  background: var(--card);
  min-height: 24px;
}

.bob-phrase {
  font-family: 'Geist Mono', monospace;
  font-size: 11px;
  color: var(--text-secondary);
  margin: 0;
  animation: fadeIn 0.3s ease;
  flex: 1;
}

@media (prefers-reduced-motion: reduce) {
  .bob-phrase { animation: none; }
}

.connection-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.connection-dot.connected { background: var(--success, #22c55e); }
.connection-dot.connecting { background: var(--warning, #f59e0b); animation: agentPulse 1.5s infinite; }
.connection-dot.disconnected,
.connection-dot.error { background: var(--error, #ef4444); }
</style>
