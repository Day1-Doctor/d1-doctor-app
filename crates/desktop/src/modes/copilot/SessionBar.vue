<template>
  <div class="session-bar">
    <span class="session-name">{{ sessionName }}</span>
    <div
      class="status-dot"
      :class="statusDot"
      :data-status="statusDot"
      :title="statusDot"
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
  padding: 0 14px;
  gap: 8px;
  flex-shrink: 0;
}

.session-name {
  flex: 1;
  font: 12px var(--font-mono);
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
.status-dot.connecting   { background: var(--warning); }
.status-dot.disconnected { background: var(--text-disabled); }

.credit-est {
  font: 11px var(--font-mono);
  color: var(--text-disabled);
  white-space: nowrap;
}
</style>
