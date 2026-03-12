<template>
  <div class="result-card">
    <div class="result-title">✓ {{ title }}</div>
    <div class="result-detail">{{ detail }}</div>
    <pre v-if="code" class="result-code">{{ code }}</pre>
  </div>
</template>

<script setup lang="ts">
defineProps<{ title: string; detail: string; code?: string }>()
</script>

<style scoped>
.result-card {
  background: var(--success-soft);
  border: 1px solid var(--success-border);
  border-radius: var(--radius-md);
  padding: var(--space-lg);
}
.result-title { font: var(--font-weight-bold) var(--font-size-md) var(--font-mono); color: var(--success); margin-bottom: var(--space-xs); }
.result-detail { font: var(--font-size-base)/1.5 var(--font-mono); color: var(--text-secondary); }
.result-code {
  margin-top: 10px;
  background: var(--muted);
  padding: var(--space-md);
  border-radius: var(--radius-sm);
  font: var(--font-size-sm)/1.5 var(--font-mono);
  color: var(--text-primary);
  overflow-x: auto;
  white-space: pre-wrap;
  border: 1px solid var(--border);
}
</style>
