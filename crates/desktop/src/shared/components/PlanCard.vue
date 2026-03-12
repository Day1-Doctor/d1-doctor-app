<template>
  <div class="plan-card">
    <div class="plan-header">
      <span class="plan-label">{{ $t('plan.label') }}</span>
      <span class="plan-count">{{ doneCount }}/{{ steps.length }}</span>
    </div>
    <div class="step-list">
      <StepIndicator v-for="step in steps" :key="step.id" v-bind="step" />
    </div>
    <div class="progress-track">
      <div class="progress-fill" :style="{ width: progressPct + '%' }" />
    </div>
    <div class="plan-actions">
      <button class="btn-approve" @click="emit('approve')">&#x2713; {{ $t('plan.approve') }}</button>
      <button class="btn-reject" @click="emit('reject')">&#x2715; {{ $t('plan.reject') }}</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Step } from '@/shared/types'
import StepIndicator from './StepIndicator.vue'

const props = defineProps<{ steps: Step[] }>()
const emit = defineEmits<{ approve: []; reject: [] }>()

const doneCount = computed(() => props.steps.filter(s => s.state === 'done').length)
const progressPct = computed(() =>
  props.steps.length ? (doneCount.value / props.steps.length) * 100 : 0
)
</script>

<style scoped>
.plan-card {
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: var(--space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}
.plan-header {
  display: flex; align-items: center; justify-content: space-between;
  margin-bottom: var(--space-sm);
}
.plan-label { font: var(--font-weight-bold) var(--font-size-sm) var(--font-mono); color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.08em; }
.plan-count { font: var(--font-size-sm) var(--font-mono); color: var(--text-disabled); }
.step-list { display: flex; flex-direction: column; }
.progress-track { height: 3px; background: var(--border); border-radius: 2px; overflow: hidden; margin: 12px 0 8px; }
.progress-fill { height: 100%; background: var(--accent); border-radius: 2px; transition: width var(--duration-slow) var(--easing-default); }
.plan-actions { display: flex; gap: var(--space-sm); margin-top: var(--space-xs); }
button { flex: 1; padding: 7px 14px; border-radius: var(--radius-sm); border: 1px solid var(--border); background: var(--muted); color: var(--text-secondary); cursor: pointer; font: var(--font-size-base) var(--font-mono); transition: all var(--duration-fast); }
.btn-approve { border-color: var(--accent); color: var(--accent); }
.btn-approve:hover { background: var(--accent-soft); }
.btn-reject:hover { background: var(--error-soft); color: var(--error); border-color: var(--error); }
</style>
