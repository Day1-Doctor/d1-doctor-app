<template>
  <div class="connection-status">
    <!-- High-level connection indicator -->
    <div class="conn-summary" data-testid="conn-summary">
      <span
        class="status-dot"
        :data-status="overallStatus"
        :class="overallStatus"
      />
      <span class="conn-summary-text" :class="overallStatus">{{ overallLabel }}</span>
    </div>

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

    <!-- Helpful hint when offline: show the fix command if available -->
    <div v-if="showOfflineHint" class="offline-hint" data-testid="offline-hint">
      <span class="hint-text">{{ offlineHint }}</span>
      <code v-if="showStartCommand" class="hint-cmd">d1 start</code>
    </div>

    <button class="reconnect-btn" @click="onReconnect" title="Reconnect daemon">
      ↺ Reconnect
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useDaemonStore } from '@/shared/stores/daemon'

export type OverallConnectionStatus = 'connected' | 'local-only' | 'offline'

const daemonStore = useDaemonStore()

const daemonStatus = computed(() => {
  if (daemonStore.status === 'connected') return 'connected'
  if (daemonStore.status === 'connecting') return 'connecting'
  return 'disconnected'
})

const daemonLabel = computed(() => {
  if (daemonStore.status === 'connected') return 'Connected'
  if (daemonStore.status === 'connecting') return 'Connecting...'
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

/**
 * High-level connection status:
 * - connected (green): daemon + platform both connected
 * - local-only (yellow): daemon connected but platform offline
 * - offline (red): daemon not connected
 */
const overallStatus = computed<OverallConnectionStatus>(() => {
  if (daemonStore.status !== 'connected') return 'offline'
  if (!daemonStore.orchestratorConnected) return 'local-only'
  return 'connected'
})

const overallLabel = computed(() => {
  switch (overallStatus.value) {
    case 'connected':  return 'Connected'
    case 'local-only': return 'Local only'
    case 'offline':    return 'Offline'
  }
})

/** Show the offline hint row only when disconnected/errored and there's info. */
const showOfflineHint = computed(() => {
  return daemonStatus.value === 'disconnected' && !!daemonStore.errorMessage
})

/** Whether the error message mentions starting the daemon. */
const showStartCommand = computed(() => {
  return !!daemonStore.errorMessage?.toLowerCase().includes('start')
})

/** A short human-friendly version of the error -- strip redundant prefixes. */
const offlineHint = computed(() => {
  const msg = daemonStore.errorMessage ?? ''
  return msg.replace(/\.\s*Start it with:.*$/i, '.').trim() || 'Daemon not reachable.'
})

function onReconnect(): void {
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

.conn-summary {
  display: flex;
  align-items: center;
  gap: 6px;
  padding-bottom: 4px;
  margin-bottom: 2px;
  border-bottom: 1px solid var(--border);
}

.conn-summary-text {
  font: 600 11px var(--font-mono, monospace);
}
.conn-summary-text.connected  { color: var(--success, #22c55e); }
.conn-summary-text.local-only { color: var(--warning, #f59e0b); }
.conn-summary-text.offline    { color: var(--error, #ef4444); }

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
.status-dot.local-only   { background: var(--warning, #f59e0b); }
.status-dot.disconnected { background: var(--error, #ef4444); }
.status-dot.offline      { background: var(--error, #ef4444); }

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

.offline-hint {
  display: flex;
  align-items: center;
  gap: 6px;
  font: 10px var(--font-mono, monospace);
  color: var(--text-secondary);
  padding-top: 2px;
}

.hint-text {
  opacity: 0.8;
}

.hint-cmd {
  background: var(--muted, rgba(0, 0, 0, 0.15));
  border: 1px solid var(--border);
  border-radius: 3px;
  padding: 1px 5px;
  font: 10px var(--font-mono, monospace);
  color: var(--text-primary);
  white-space: nowrap;
}

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
