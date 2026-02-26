<template>
  <div class="plan-card">
    <div class="plan-header">Plan</div>
    <StepIndicator v-for="step in steps" :key="step.id" v-bind="step" />
    <div class="progress-track"><div class="progress-fill" :style="{ width: progressPct + '%' }" /></div>
    <div class="plan-actions">
      <button class="btn-approve" @click="onApprove">✓ Approve</button>
      <button class="btn-reject" @click="onReject">✕ Reject</button>
    </div>
  </div>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import type { Step } from '@/shared/types'
import StepIndicator from './StepIndicator.vue'
const props = defineProps<{ steps: Step[]; onApprove: () => void; onReject: () => void }>()
const progressPct = computed(() => {
  if (!props.steps.length) return 0
  const done = props.steps.filter(s => s.state === 'done').length
  return (done / props.steps.length) * 100
})
</script>
<style scoped>
.plan-card { background: var(--card); border: 1px solid var(--border); border-radius: var(--radius-md); padding: 16px; display: flex; flex-direction: column; gap: 4px; }
.plan-header { font: 700 11px var(--font-mono); color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.08em; margin-bottom: 8px; }
.progress-track { height: 3px; background: var(--border); border-radius: 2px; overflow: hidden; margin: 12px 0 8px; }
.progress-fill { height: 100%; background: var(--accent); border-radius: 2px; transition: width 0.3s; }
.plan-actions { display: flex; gap: 8px; margin-top: 4px; }
button { flex: 1; padding: 7px 14px; border-radius: var(--radius-sm); border: 1px solid var(--border); background: var(--muted); color: var(--text-secondary); cursor: pointer; font: 12px var(--font-mono); transition: all 0.15s; }
.btn-approve { border-color: var(--accent); color: var(--accent); }
.btn-approve:hover { background: var(--accent-soft); }
.btn-reject:hover { background: rgba(239,68,68,0.1); color: var(--error); border-color: var(--error); }
</style>
