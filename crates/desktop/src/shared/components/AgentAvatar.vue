<template>
  <div class="agent-avatar" :class="{ active }">{{ agent.charAt(0).toUpperCase() }}</div>
</template>
<script setup lang="ts">
defineProps<{ agent: string; active: boolean }>()
</script>
<style scoped>
.agent-avatar { width: 30px; height: 30px; border-radius: 50%; background: var(--muted); border: 1.5px solid var(--accent-border); display: flex; align-items: center; justify-content: center; font: 700 11px var(--font-mono); color: var(--accent); flex-shrink: 0; }
.active { animation: agentPulse 1.8s ease-in-out infinite; border-color: var(--accent); }
</style>
