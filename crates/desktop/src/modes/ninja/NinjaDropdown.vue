<template>
  <div class="ninja-dropdown">
    <!-- Header section -->
    <div class="dropdown-header">
      <div class="query-echo">&gt; {{ query }}</div>
      <div class="agent-label">● Day1 Doctor &nbsp; [Planner Agent]</div>
    </div>

    <!-- Step timeline -->
    <StepTimeline :steps="steps" />

    <!-- Result card — only shown when resultTitle is provided -->
    <div v-if="resultTitle" class="result-card-wrapper">
      <ResultCard :title="resultTitle" :detail="resultDetail ?? ''" />
    </div>

    <!-- Footer -->
    <div class="dropdown-footer">
      <div class="progress-row">
        <div
          class="progress-track"
          role="progressbar"
          :aria-valuenow="completedSteps"
          :aria-valuemax="totalSteps"
          aria-label="Plan progress"
        >
          <div class="progress-fill" :style="{ width: progressPct + '%' }" />
        </div>
      </div>
      <div class="footer-actions">
        <span class="credit-cost">{{ creditEstimate }}</span>
        <div class="footer-btns">
          <button class="btn-approve" @click="emit('approve')">Approve</button>
          <button class="btn-dismiss" @click="emit('dismiss')">Dismiss</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Step } from '@/shared/types'
import StepTimeline from './StepTimeline.vue'
import ResultCard from '@/shared/components/ResultCard.vue'

const props = defineProps<{
  query: string
  steps: Step[]
  resultTitle?: string
  resultDetail?: string
  creditEstimate: string
}>()

const emit = defineEmits<{ approve: []; dismiss: [] }>()

const completedSteps = computed((): number =>
  props.steps.filter(s => s.state === 'done').length
)

const totalSteps = computed((): number => props.steps.length)

const progressPct = computed((): number => {
  if (totalSteps.value === 0) return 0
  return Math.round((completedSteps.value / totalSteps.value) * 100)
})
</script>

<style scoped>
.ninja-dropdown {
  width: 680px;
  background: var(--surface-ninja-dropdown);
  backdrop-filter: var(--backdrop-xl);
  -webkit-backdrop-filter: var(--backdrop-xl);
  border-radius: var(--space-lg);
  border: 1px solid var(--border-translucent);
  box-shadow: var(--shadow-xl);
  overflow: hidden;
  animation: slideDown 0.25s var(--easing-out) forwards;
  margin-top: var(--space-xs);
}

/* Header */
.dropdown-header {
  padding: var(--space-lg) var(--space-xl) var(--space-md);
  border-bottom: 1px solid var(--border);
}

.query-echo {
  font: var(--font-size-md) var(--font-mono);
  color: var(--text-primary);
  margin-bottom: var(--space-xs);
}

.agent-label {
  font: var(--font-size-sm) var(--font-mono);
  color: var(--text-secondary);
}

/* Result card wrapper */
.result-card-wrapper {
  padding: 0 var(--space-xl) var(--space-md);
}

/* Footer */
.dropdown-footer {
  padding: var(--space-md) var(--space-xl);
  border-top: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.progress-row {
  width: 100%;
}

.progress-track {
  width: 100%;
  height: 3px;
  background: var(--border);
  border-radius: var(--space-2xs);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: var(--space-2xs);
  transition: width var(--duration-slow) var(--easing-default);
}

.footer-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.credit-cost {
  font: var(--font-size-sm) var(--font-mono);
  color: var(--text-muted);
}

.footer-btns {
  display: flex;
  gap: var(--space-sm);
}

/* Buttons */
.btn-approve,
.btn-dismiss {
  background: none;
  border-radius: var(--radius-sm);
  padding: var(--space-xs) var(--space-lg);
  font: var(--font-size-base) var(--font-mono);
  cursor: pointer;
  transition: background var(--duration-fast);
}

.btn-approve {
  border: 1px solid var(--success);
  color: var(--success);
}

.btn-approve:hover {
  background: var(--success-soft);
}

.btn-dismiss {
  border: 1px solid var(--border);
  color: var(--text-secondary);
}

.btn-dismiss:hover {
  background: var(--muted);
}
</style>
