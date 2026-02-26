<template>
  <div class="permission-badge" :class="state">{{ label }}</div>
</template>
<script setup lang="ts">
defineProps<{ state: 'allow' | 'ask' | 'deny'; label: string }>()
</script>
<style scoped>
.permission-badge { display: inline-flex; padding: 3px 8px; border-radius: var(--radius-sm); font: 11px var(--font-mono); }
.allow { background: var(--success-soft); color: var(--success); border: 1px solid var(--success-border); }
.ask { background: var(--warning-soft); color: var(--warning); border: 1px solid var(--warning-border); }
.deny { background: var(--error-soft); color: var(--error); border: 1px solid var(--error-border); }
</style>
