<template>
  <div class="step-item" :class="state">
    <div class="step-dot"><span>{{ state === 'done' ? '✓' : state === 'error' ? '✕' : index + 1 }}</span></div>
    <span class="step-label">{{ label }}</span>
  </div>
</template>
<script setup lang="ts">
import type { Step } from '@/shared/types'
defineProps<{ state: Step['state']; label: string; index: number }>()
</script>
<style scoped>
.step-item { display: flex; align-items: center; gap: 12px; padding: 8px 0; }
.step-dot { width: 22px; height: 22px; border-radius: 50%; border: 1px solid var(--border); display: flex; align-items: center; justify-content: center; font-size: 11px; font-family: var(--font-mono); flex-shrink: 0; }
.done .step-dot { background: var(--success-soft); border-color: var(--success); color: var(--success); }
.active .step-dot { background: var(--accent-soft); border-color: var(--accent); color: var(--accent); animation: agentPulse 1.8s ease-in-out infinite; }
.pending .step-dot { color: var(--text-disabled); }
.error .step-dot { background: var(--error-soft); border-color: var(--error); color: var(--error); }
.step-label { font: 13px/1.4 var(--font-mono); color: var(--text-secondary); }
</style>
