<template>
  <div class="copilot copilot-panel">
    <!-- Mode switcher — always visible top-right -->
    <div class="copilot-mode-bar">
      <ModeBar />
    </div>
    <CopilotHeader />
    <SessionBar
      :session-name="sessionName"
      :status-dot="agentStore.connectionStatus"
      :credit-estimate="creditEstimate"
    />
    <div class="copilot-messages" ref="listEl" role="log" aria-live="polite" @scroll="onScroll">
      <div
        v-if="conversationStore.messages.length === 0 && !conversationStore.currentPlan"
        class="empty-state"
      >
        <div class="empty-title">Ask me anything</div>
        <div class="empty-sub">Dr. Day1 is ready to help.</div>
      </div>
      <template v-else>
        <MessageBubble
          v-for="msg in conversationStore.messages"
          :key="msg.id"
          :role="msg.role"
          :content="msg.content"
          :timestamp="msg.timestamp"
        />
        <PlanCard
          v-if="conversationStore.currentPlan"
          :steps="conversationStore.currentPlan.steps"
          @approve="onApprove"
          @reject="onReject"
        />
      </template>
    </div>
    <div
      v-if="!conversationStore.scrollPinned"
      class="new-messages-badge"
      @click="scrollToBottom"
    >
      ↓ New messages
    </div>
    <CopilotInput />
    <div class="credit-footer">
      <CreditBar
        :credits="agentStore.credits.current"
        :max="agentStore.credits.max"
        variant="mini"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { useConversationStore } from '@/shared/stores/conversation'
import { useAgentStore } from '@/shared/stores/agent'
import { useDaemonConnection } from '@/shared/composables/useDaemonConnection'
import { useDaemonStore } from '@/shared/stores/daemon'
import ModeBar from '@/shared/components/ModeBar.vue'
import CopilotHeader from './CopilotHeader.vue'
import SessionBar from './SessionBar.vue'
import CopilotInput from './CopilotInput.vue'
import MessageBubble from '@/shared/components/MessageBubble.vue'
import PlanCard from '@/shared/components/PlanCard.vue'
import CreditBar from '@/shared/components/CreditBar.vue'

const conversationStore = useConversationStore()
const agentStore = useAgentStore()
const daemonStore = useDaemonStore()
const { approvePlan } = useDaemonConnection()

const listEl = ref<HTMLDivElement | null>(null)

// TODO: replace with conversationStore.sessionName when session naming is implemented
const sessionName = computed(() => 'Active Session')

const creditEstimate = computed(() => {
  const used = agentStore.credits.current
  return `~${used.toFixed(1)} credits`
})

watch(
  () => conversationStore.messages.length,
  () => {
    if (conversationStore.scrollPinned) {
      nextTick(scrollToBottom)
    }
  }
)

function scrollToBottom(): void {
  if (listEl.value) {
    listEl.value.scrollTop = listEl.value.scrollHeight
    conversationStore.setScrollPinned(true)
  }
}

function onScroll(): void {
  if (!listEl.value) return
  const el = listEl.value
  const distFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight
  conversationStore.setScrollPinned(distFromBottom <= 60)
}

function onApprove(): void {
  if (conversationStore.currentPlan) {
    conversationStore.approvePlan(true)
    const taskId = daemonStore.currentTaskId
    const planId = daemonStore.currentPlanId
    if (taskId && planId) {
      approvePlan(taskId, planId, 'APPROVE')
    }
  }
}

function onReject(): void {
  if (conversationStore.currentPlan) {
    conversationStore.approvePlan(false)
    const taskId = daemonStore.currentTaskId
    const planId = daemonStore.currentPlanId
    if (taskId && planId) {
      approvePlan(taskId, planId, 'REJECT')
    }
  }
}
</script>

<style scoped>
.copilot-mode-bar {
  position: absolute;
  top: var(--space-sm);
  right: var(--space-sm);
  z-index: 100;
  -webkit-app-region: no-drag;  /* Tauri: allow clicks in draggable window */
}

.copilot-panel {
  background: var(--surface-copilot);
  backdrop-filter: var(--backdrop-lg);
  -webkit-backdrop-filter: var(--backdrop-lg);
  border: 1px solid var(--border);
  border-radius: var(--space-md);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-lg);
  width: 420px;
  height: 100vh;
  position: relative;
}

.copilot-messages {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.copilot-messages::-webkit-scrollbar {
  width: 3px;
}

.copilot-messages::-webkit-scrollbar-track {
  background: transparent;
}

.copilot-messages::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 2px;
}

/* Condensed message bubble overrides for copilot mode */
.copilot :deep(.message-bubble) {
  font-size: 12px;
  padding: 10px 14px;
}

.copilot :deep(.agent-avatar) {
  width: 26px;
  height: 26px;
  font-size: 9px;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-xs);
  padding: var(--space-2xl) var(--space-lg);
  color: var(--text-disabled);
}

.empty-title {
  font: var(--font-weight-semibold) var(--font-size-md) var(--font-mono);
  color: var(--text-muted);
}

.empty-sub {
  font: var(--font-size-base) var(--font-mono);
  color: var(--text-secondary);
  text-align: center;
}

.new-messages-badge {
  position: absolute;
  bottom: 100px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--accent);
  color: var(--text-on-accent);
  font: var(--font-weight-bold) var(--font-size-xs) var(--font-mono);
  padding: var(--space-xs) var(--space-md);
  border-radius: 999px;
  cursor: pointer;
  user-select: none;
  z-index: 10;
  box-shadow: 0 2px 10px var(--accent-glow);
  animation: fadeIn 0.15s ease;
}

.credit-footer {
  height: 36px;
  padding: 0 var(--space-lg);
  border-top: 1px solid var(--border);
  display: flex;
  align-items: center;
  flex-shrink: 0;
  background: var(--card);
}
</style>
