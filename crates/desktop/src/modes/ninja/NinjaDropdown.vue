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
        <div class="progress-track">
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

const progressPct = computed((): number => {
  if (props.steps.length === 0) return 0
  const done = props.steps.filter(s => s.state === 'done').length
  return Math.round((done / props.steps.length) * 100)
})
</script>

<style scoped>
.ninja-dropdown {
  width: 680px;
  background: rgba(5, 5, 5, 0.88);
  backdrop-filter: blur(50px) saturate(160%);
  -webkit-backdrop-filter: blur(50px) saturate(160%);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.8);
  overflow: hidden;
  animation: slideDown 0.25s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  margin-top: 4px;
}

/* Header */
.dropdown-header {
  padding: 16px 20px 12px;
  border-bottom: 1px solid var(--border);
}

.query-echo {
  font: 13px var(--font-mono);
  color: var(--text-muted);
  margin-bottom: 4px;
}

.agent-label {
  font: 11px var(--font-mono);
  color: var(--text-secondary);
}

/* Result card wrapper */
.result-card-wrapper {
  padding: 0 20px 12px;
}

/* Footer */
.dropdown-footer {
  padding: 12px 20px;
  border-top: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.progress-row {
  width: 100%;
}

.progress-track {
  width: 100%;
  height: 3px;
  background: var(--border);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.footer-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.credit-cost {
  font: 11px var(--font-mono);
  color: var(--text-muted);
}

.footer-btns {
  display: flex;
  gap: 8px;
}

/* Buttons */
.btn-approve,
.btn-dismiss {
  background: none;
  border-radius: var(--radius-sm);
  padding: 5px 14px;
  font: 12px var(--font-mono);
  cursor: pointer;
  transition: background 0.15s;
}

.btn-approve {
  border: 1px solid var(--success);
  color: var(--success);
}

.btn-approve:hover {
  background: rgba(34, 197, 94, 0.1);
}

.btn-dismiss {
  border: 1px solid var(--border);
  color: var(--text-secondary);
}

.btn-dismiss:hover {
  background: var(--muted);
}
</style>
