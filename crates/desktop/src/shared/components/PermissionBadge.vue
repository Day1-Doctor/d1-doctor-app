<template>
  <div class="permission-badge" :class="state">{{ label }}</div>
</template>

<script setup lang="ts">
defineProps<{ state: 'allow' | 'ask' | 'deny'; label: string }>()
</script>

<style scoped>
.permission-badge {
  display: inline-flex;
  align-items: center;
  padding: 3px 8px;
  border-radius: var(--radius-sm);
  font: 11px var(--font-mono);
  border: 1px solid transparent;
}
.allow { background: var(--success-soft); color: var(--success); border-color: var(--success-border); }
.ask   { background: var(--warning-soft); color: var(--warning); border-color: var(--warning-border); }
.deny  { background: var(--error-soft);   color: var(--error);   border-color: var(--error-border);   }
</style>
