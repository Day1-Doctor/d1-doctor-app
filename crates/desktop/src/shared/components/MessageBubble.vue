<template>
  <div class="message-bubble" :class="role"><span>{{ content }}</span></div>
</template>
<script setup lang="ts">
defineProps<{ role: 'agent' | 'user'; content: string; timestamp: number }>()
</script>
<style scoped>
.message-bubble { padding: 12px 16px; border-radius: var(--radius-md); font: 13px/1.6 var(--font-mono); color: var(--text-primary); max-width: 85%; }
.agent { background: var(--card); border-left: 2px solid var(--accent); align-self: flex-start; }
.user { background: var(--muted); align-self: flex-end; }
</style>
