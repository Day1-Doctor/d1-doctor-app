<template>
  <div class="step-timeline">
    <div v-for="step in steps" :key="step.id" class="step-item" :class="step.state">
      <div class="step-dot"><span>{{ step.state === 'done' ? '✓' : step.index + 1 }}</span></div>
      <span class="step-label">{{ step.label }}</span>
    </div>
  </div>
</template>
<script setup lang="ts">
import type { Step } from '@/shared/types'
defineProps<{ steps: Step[] }>()
</script>
<style scoped>
.step-timeline { padding: 12px 20px; }
.step-item { display: flex; align-items: flex-start; gap: 12px; padding: 8px 0; position: relative; }
.step-item:not(:last-child)::after { content: ''; position: absolute; left: 11px; top: 30px; bottom: -8px; width: 2px; background: var(--border); }
.step-item.done::after { background: var(--success); }
.step-item.active::after { background: var(--accent); }
.step-dot { width: 22px; height: 22px; border-radius: 50%; flex-shrink: 0; display: flex; align-items: center; justify-content: center; font: 11px var(--font-mono); border: 1px solid var(--border); }
.done .step-dot { background: var(--success-soft); border-color: var(--success); color: var(--success); }
.active .step-dot { background: var(--accent-soft); border-color: var(--accent); color: var(--accent); animation: agentPulse 1.8s ease-in-out infinite; }
.pending .step-dot { color: var(--text-disabled); }
.step-label { font: 13px var(--font-mono); color: var(--text-secondary); padding-top: 4px; }
</style>
