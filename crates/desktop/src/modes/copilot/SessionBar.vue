<template>
  <div class="session-bar">
    <span class="session-name">{{ sessionName }}</span>
    <div
      class="status-dot"
      :class="statusDot"
      :data-status="statusDot"
      :title="statusDot"
      :aria-label="'Connection: ' + statusDot"
    />
    <span class="credit-est">{{ creditEstimate }}</span>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  sessionName: string
  statusDot: 'connected' | 'disconnected' | 'connecting'
  creditEstimate: string
}>()
</script>

<style scoped>
.session-bar {
  height: 42px;
  background: var(--muted);
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  padding: 0 var(--space-lg);
  gap: var(--space-sm);
  flex-shrink: 0;
}

.session-name {
  flex: 1;
  font: var(--font-size-base) var(--font-mono);
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-dot.connected    { background: var(--success); }
.status-dot.connecting   { background: var(--warning); animation: pulse 1s infinite; }
.status-dot.disconnected { background: var(--text-muted); }

.credit-est {
  font: var(--font-size-sm) var(--font-mono);
  color: var(--text-disabled);
  white-space: nowrap;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
</style>
