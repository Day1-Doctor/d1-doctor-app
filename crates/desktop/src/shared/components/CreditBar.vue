<template>
  <div class="credit-bar" :class="variant">
    <span v-if="variant === 'full'" class="label">Credits</span>
    <div class="bar-track"><div class="bar-fill" :style="{ width: pct + '%' }" /></div>
    <span class="value">{{ credits }}<span v-if="variant === 'full'">/{{ max }}</span></span>
  </div>
</template>
<script setup lang="ts">
import { computed } from 'vue'
const props = defineProps<{ credits: number; max: number; variant: 'full' | 'mini' | 'dropdown' }>()
const pct = computed(() => Math.min(100, props.max > 0 ? (props.credits / props.max) * 100 : 0))
</script>
<style scoped>
.credit-bar { display: flex; align-items: center; gap: 8px; font: 11px var(--font-mono); color: var(--text-muted); }
.bar-track { flex: 1; height: 3px; background: var(--border); border-radius: 2px; overflow: hidden; min-width: 40px; }
.bar-fill { height: 100%; background: var(--accent); border-radius: 2px; transition: width 0.3s; }
</style>
