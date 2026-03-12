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
      <span class="conn-label">{{ $t('connection.daemon') }}</span>
      <span class="conn-text" :class="daemonStatus">{{ daemonLabel }}</span>
    </div>
    <div class="conn-row" data-row="platform">
      <span
        class="status-dot"
        :data-status="platformStatus"
        :class="platformStatus"
      />
      <span class="conn-label">{{ $t('connection.platform') }}</span>
      <span class="conn-text" :class="platformStatus">{{ platformLabel }}</span>
    </div>

    <!-- Helpful hint when offline: show the fix command if available -->
    <div v-if="showOfflineHint" class="offline-hint" data-testid="offline-hint">
      <span class="hint-text">{{ offlineHint }}</span>
      <code v-if="showStartCommand" class="hint-cmd">d1 start</code>
    </div>

    <button class="reconnect-btn" @click="onReconnect" :title="$t('connection.reconnect')">
      &#x21ba; {{ $t('connection.reconnect') }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useDaemonStore } from '@/shared/stores/daemon'

export type OverallConnectionStatus = 'connected' | 'local-only' | 'offline'

const { t } = useI18n()
const daemonStore = useDaemonStore()

const daemonStatus = computed(() => {
  if (daemonStore.status === 'connected') return 'connected'
  if (daemonStore.status === 'connecting') return 'connecting'
  return 'disconnected'
})

const daemonLabel = computed(() => {
  if (daemonStore.status === 'connected') return t('connection.connected')
  if (daemonStore.status === 'connecting') return t('connection.connecting')
  return t('connection.offline')
})

// Platform is only reachable if daemon is connected
const platformStatus = computed(() => {
  if (daemonStore.status !== 'connected') return 'disconnected'
  return daemonStore.orchestratorConnected ? 'connected' : 'disconnected'
})

const platformLabel = computed(() => {
  return platformStatus.value === 'connected' ? t('connection.connected') : t('connection.offline')
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
    case 'connected':  return t('connection.connected')
    case 'local-only': return t('connection.localOnly')
    case 'offline':    return t('connection.offline')
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
  return msg.replace(/\.\s*Start it with:.*$/i, '.').trim() || t('connection.daemonNotReachable')
})

function onReconnect(): void {
  window.location.reload()
}
</script>

<style scoped>
.connection-status {
  padding: var(--space-sm) var(--space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.conn-summary {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  padding-bottom: 4px;
  margin-bottom: 2px;
  border-bottom: 1px solid var(--border);
}

.conn-summary-text {
  font: var(--font-weight-semibold) var(--font-size-sm) var(--font-mono, monospace);
}
.conn-summary-text.connected  { color: var(--success); }
.conn-summary-text.local-only { color: var(--warning); }
.conn-summary-text.offline    { color: var(--error); }

.conn-row {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  font: var(--font-size-xs) var(--font-mono, monospace);
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-dot.connected    { background: var(--success); }
.status-dot.connecting   { background: var(--accent); animation: pulse 1s infinite; }
.status-dot.local-only   { background: var(--warning); }
.status-dot.disconnected { background: var(--error); }
.status-dot.offline      { background: var(--error); }

.conn-label {
  color: var(--text-disabled);
  width: 50px;
}

.conn-text {
  font-weight: var(--font-weight-semibold);
}
.conn-text.connected    { color: var(--success); }
.conn-text.connecting   { color: var(--accent); }
.conn-text.disconnected { color: var(--error); }

.offline-hint {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  font: var(--font-size-xs) var(--font-mono, monospace);
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
  font: var(--font-size-xs) var(--font-mono, monospace);
  color: var(--text-primary);
  white-space: nowrap;
}

.reconnect-btn {
  margin-top: 4px;
  background: transparent;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm, 4px);
  color: var(--text-secondary);
  font: var(--font-size-xs) var(--font-mono, monospace);
  padding: 3px 8px;
  cursor: pointer;
  align-self: flex-start;
  transition: background var(--duration-fast), color var(--duration-fast);
}

.reconnect-btn:hover {
  background: var(--muted);
  color: var(--text-primary);
}

</style>
