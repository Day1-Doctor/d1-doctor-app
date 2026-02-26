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
.result-card { background: var(--success-soft); border: 1px solid var(--success-border); border-radius: var(--radius-md); padding: 16px; }
.result-title { font: 700 13px var(--font-mono); color: var(--success); margin-bottom: 4px; }
.result-detail { font: 12px var(--font-mono); color: var(--text-secondary); }
.result-code { margin-top: 10px; background: var(--muted); padding: 10px; border-radius: var(--radius-sm); font: 11px var(--font-mono); color: var(--text-primary); overflow-x: auto; white-space: pre-wrap; }
</style>
