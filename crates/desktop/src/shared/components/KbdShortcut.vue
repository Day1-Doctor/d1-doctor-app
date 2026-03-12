<template>
  <span class="kbd-shortcut">
    <kbd v-for="key in keys" :key="key" class="kbd-key">{{ key }}</kbd>
  </span>
</template>

<script setup lang="ts">
defineProps<{ keys: string[] }>()
</script>

<style scoped>
.kbd-shortcut { display: inline-flex; align-items: center; gap: var(--space-2xs); }
.kbd-key {
  display: inline-flex;
  align-items: center;
  padding: var(--space-2xs) var(--space-xs);
  border-radius: 3px;
  background: var(--muted);
  border: 1px solid var(--border);
  box-shadow: inset 0 -1px 0 var(--border), 0 1px 2px rgba(0, 0, 0, 0.3);
  font: var(--font-size-xs) var(--font-mono);
  color: var(--text-secondary);
  line-height: 1.4;
}
</style>
