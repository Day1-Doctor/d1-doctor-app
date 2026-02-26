<template>
  <span class="kbd-shortcut">
    <kbd v-for="key in keys" :key="key" class="kbd-key">{{ key }}</kbd>
  </span>
</template>

<script setup lang="ts">
defineProps<{ keys: string[] }>()
</script>

<style scoped>
.kbd-shortcut { display: inline-flex; align-items: center; gap: 2px; }
.kbd-key {
  display: inline-flex;
  align-items: center;
  padding: 2px 6px;
  border-radius: 3px;
  background: var(--muted);
  border: 1px solid var(--border);
  font: 10px var(--font-mono);
  color: var(--text-secondary);
  line-height: 1.4;
}
</style>
