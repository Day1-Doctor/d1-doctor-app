<template>
  <div class="step-timeline">
    <div
      v-for="step in steps"
      :key="step.id"
      class="step-item"
      :class="step.state"
    >
      <div class="step-dot" :class="step.state">
        <span>{{ dotContent(step) }}</span>
      </div>
      <span class="step-label">{{ step.label }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Step } from '@/shared/types'

defineProps<{ steps: Step[] }>()

function dotContent(step: Step): string {
  if (step.state === 'done') return '✓'
  if (step.state === 'error') return '✕'
  return String(step.index + 1)
}
</script>

<style scoped>
.step-timeline {
  padding: 12px 20px;
  position: relative;
}

.step-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 8px 0;
  position: relative;
}

/* Vertical connector line between steps */
.step-item:not(:last-child)::after {
  content: '';
  position: absolute;
  left: 11px; /* center of 22px dot */
  top: 30px;
  bottom: -8px;
  width: 2px;
  background: var(--border);
}

.step-item.done::after {
  background: var(--success);
}

.step-item.active::after {
  background: var(--accent);
}

/* Step dot base */
.step-dot {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font: 11px var(--font-mono);
}

.step-dot.done {
  background: rgba(34, 197, 94, 0.15);
  border: 1px solid var(--success);
  color: var(--success);
}

.step-dot.active {
  background: var(--accent-soft);
  border: 1px solid var(--accent);
  color: var(--accent);
  animation: agentPulse 1.8s ease-in-out infinite;
}

.step-dot.pending {
  background: transparent;
  border: 1px solid var(--text-disabled);
  color: var(--text-disabled);
}

.step-dot.error {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid var(--error);
  color: var(--error);
}

/* Step label */
.step-label {
  font: 13px/1.4 var(--font-mono);
  color: var(--text-secondary);
  padding-top: 4px;
}

.step-item.active .step-label {
  color: var(--text-primary);
  font-weight: 600;
}

.step-item.done .step-label {
  color: var(--text-muted);
}
</style>
