<template>
  <div class="connection-status">
    <div class="conn-row" data-row="daemon">
      <span
        class="status-dot"
        :data-status="daemonStatus"
        :class="daemonStatus"
      />
      <span class="conn-label">Daemon</span>
      <span class="conn-text" :class="daemonStatus">{{ daemonLabel }}</span>
    </div>
    <div class="conn-row" data-row="platform">
      <span
        class="status-dot"
        :data-status="platformStatus"
        :class="platformStatus"
      />
      <span class="conn-label">Platform</span>
      <span class="conn-text" :class="platformStatus">{{ platformLabel }}</span>
    </div>
    <button class="reconnect-btn" @click="onReconnect" title="Reconnect daemon">
      ↺ Reconnect
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useDaemonStore } from '@/shared/stores/daemon'

const daemonStore = useDaemonStore()

const daemonStatus = computed(() => {
  if (daemonStore.status === 'connected') return 'connected'
  if (daemonStore.status === 'connecting') return 'connecting'
  return 'disconnected'
})

const daemonLabel = computed(() => {
  if (daemonStore.status === 'connected') return 'Connected'
  if (daemonStore.status === 'connecting') return 'Connecting…'
  return 'Offline'
})

// Platform is only reachable if daemon is connected
const platformStatus = computed(() => {
  if (daemonStore.status !== 'connected') return 'disconnected'
  return daemonStore.orchestratorConnected ? 'connected' : 'disconnected'
})

const platformLabel = computed(() => {
  return platformStatus.value === 'connected' ? 'Connected' : 'Offline'
})

function onReconnect(): void {
  // Page reload is the simplest way to trigger a fresh reconnect
  window.location.reload()
}
</script>

<style scoped>
.connection-status {
  padding: 8px 16px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.conn-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font: 10px var(--font-mono, monospace);
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-dot.connected    { background: var(--success, #22c55e); }
.status-dot.connecting   { background: var(--accent, #6366f1); animation: pulse 1s infinite; }
.status-dot.disconnected { background: var(--error, #ef4444); }

.conn-label {
  color: var(--text-disabled);
  width: 50px;
}

.conn-text {
  font-weight: 600;
}
.conn-text.connected    { color: var(--success, #22c55e); }
.conn-text.connecting   { color: var(--accent, #6366f1); }
.conn-text.disconnected { color: var(--error, #ef4444); }

.reconnect-btn {
  margin-top: 4px;
  background: transparent;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm, 4px);
  color: var(--text-secondary);
  font: 10px var(--font-mono, monospace);
  padding: 3px 8px;
  cursor: pointer;
  align-self: flex-start;
  transition: background 0.12s, color 0.12s;
}

.reconnect-btn:hover {
  background: var(--muted);
  color: var(--text-primary);
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
</style>
