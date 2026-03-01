<template>
  <div class="message-bubble" :class="role">
    <div v-if="role === 'agent'" class="bubble-header">
      <span class="role-label">Dr. Day1</span>
      <span class="timestamp">{{ formattedTime }}</span>
    </div>
    <div class="bubble-content">{{ content }}</div>
    <div v-if="role === 'user'" class="bubble-footer">
      <span class="timestamp">{{ formattedTime }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
const props = defineProps<{ role: 'agent' | 'user'; content: string; timestamp: number }>()
const formattedTime = computed(() =>
  new Date(props.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
)
</script>

<style scoped>
.message-bubble {
  display: flex;
  flex-direction: column;
  max-width: 85%;
  padding: 12px 16px;
  border-radius: var(--radius-md);
  font: 13px/1.6 var(--font-mono);
  animation: fadeIn 0.15s ease;
}
.agent {
  align-self: flex-start;
  background: var(--card);
  border-left: 2px solid var(--accent);
  color: var(--text-primary);
}
.user {
  align-self: flex-end;
  background: var(--muted);
  color: var(--text-primary);
}
.bubble-header { display: flex; align-items: center; gap: 8px; margin-bottom: 4px; }
.role-label { font-size: 11px; font-weight: 700; color: var(--accent); }
.timestamp { font-size: 10px; color: var(--text-disabled); }
.bubble-content { white-space: pre-wrap; word-break: break-word; }
.bubble-footer { margin-top: 4px; display: flex; justify-content: flex-end; }
</style>
