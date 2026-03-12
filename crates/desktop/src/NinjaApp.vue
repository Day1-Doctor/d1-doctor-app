<template>
  <div
    class="ninja-app"
    ref="appEl"
    tabindex="-1"
    @keydown.esc="onEsc"
  >
    <NinjaBar @submit="onSubmit" @dismiss="onDismiss" />
    <NinjaDropdown
      v-if="showDropdown"
      :query="query"
      :steps="steps"
      :creditEstimate="CREDIT_ESTIMATE_PLACEHOLDER"
      @approve="onApprove"
      @dismiss="onDismiss"
    />
    <div class="ninja-status">
      <p
        v-if="daemonStore.currentBobPhrase"
        class="bob-phrase"
        data-testid="bob-phrase"
        aria-live="polite"
      >{{ daemonStore.currentBobPhrase }}</p>
      <span
        class="connection-dot"
        :class="daemonStore.status"
        :title="`${$t('connection.daemon')}: ${daemonStore.status}`"
        data-testid="connection-dot"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { emit } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import NinjaBar from '@/modes/ninja/NinjaBar.vue'
import NinjaDropdown from '@/modes/ninja/NinjaDropdown.vue'
import { useConversationStore } from '@/shared/stores/conversation'
import { useDaemonStore } from '@/shared/stores/daemon'
import { useDaemonConnection } from '@/shared/composables/useDaemonConnection'
import type { Step } from '@/shared/types'

// TODO: wire to agentStore.credits when credit tracking is implemented
const CREDIT_ESTIMATE_PLACEHOLDER = '~0.5 credits'

const conversationStore = useConversationStore()
const daemonStore = useDaemonStore()
const { approvePlan } = useDaemonConnection()

const appEl = ref<HTMLDivElement | null>(null)
const showDropdown = ref(false)
const query = ref('')

const steps = computed((): Step[] => conversationStore.currentPlan?.steps ?? [])

function onSubmit(q: string): void {
  query.value = q
  showDropdown.value = true
  conversationStore.appendMessage({
    id: crypto.randomUUID(),
    role: 'user',
    content: q,
    timestamp: Date.now(),
  })
}

function onDismiss(): void {
  showDropdown.value = false
}

async function onApprove(): Promise<void> {
  if (daemonStore.currentTaskId && daemonStore.currentPlanId) {
    approvePlan(daemonStore.currentTaskId, daemonStore.currentPlanId, 'APPROVE')
  }
  showDropdown.value = false
}

async function dismissNinjaWindow(): Promise<void> {
  await emit('ninja_dismissed')
  await getCurrentWindow().hide()
}

function onEsc(): void {
  if (showDropdown.value) {
    showDropdown.value = false
  } else if (query.value !== '') {
    query.value = ''
  } else {
    // Bar is clear and dropdown is hidden — full dismiss
    void dismissNinjaWindow()
  }
}

onMounted(() => {
  appEl.value?.focus()
})
</script>

<style scoped>
.ninja-app {
  display: flex;
  flex-direction: column;
  align-items: center;
  outline: none;
}

.ninja-status {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-2xs) var(--space-sm);
}

.bob-phrase {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-secondary);
  margin: 0;
  animation: fadeIn var(--duration-slow) var(--easing-default);
}

@media (prefers-reduced-motion: reduce) {
  .bob-phrase { animation: none; }
}

.connection-dot {
  display: inline-block;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.connection-dot.connected { background: var(--success); }
.connection-dot.connecting { background: var(--warning); animation: agentPulse 1.5s infinite; }
.connection-dot.disconnected,
.connection-dot.error { background: var(--error); }

@media (prefers-reduced-motion: reduce) {
  .connection-dot.connecting { animation: none; }
}
</style>
