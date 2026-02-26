<template>
  <div class="permission-badge" :class="state">{{ label }}</div>
</template>
<script setup lang="ts">
defineProps<{ state: 'allow' | 'ask' | 'deny'; label: string }>()
</script>
<style scoped>
.permission-badge { display: inline-flex; padding: 3px 8px; border-radius: var(--radius-sm); font: 11px var(--font-mono); }
.allow { background: var(--success-soft); color: var(--success); border: 1px solid rgba(34,197,94,0.25); }
.ask { background: rgba(245,158,11,0.1); color: var(--warning); border: 1px solid rgba(245,158,11,0.25); }
.deny { background: rgba(239,68,68,0.1); color: var(--error); border: 1px solid rgba(239,68,68,0.25); }
</style>
