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
      :creditEstimate="creditEstimate"
      @approve="onApprove"
      @dismiss="onDismiss"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { emit } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import NinjaBar from '@/modes/ninja/NinjaBar.vue'
import NinjaDropdown from '@/modes/ninja/NinjaDropdown.vue'
import { useConversationStore } from '@/shared/stores/conversation'
import type { Step } from '@/shared/types'

const conversationStore = useConversationStore()

const appEl = ref<HTMLDivElement | null>(null)
const showDropdown = ref(false)
const query = ref('')
const creditEstimate = '~0.5 credits'

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

function onApprove(): void {
  if (conversationStore.currentPlan) {
    conversationStore.setPlan({
      ...conversationStore.currentPlan,
      approved: true,
    })
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

// Expose for testing only — allows tests to assert internal query state
defineExpose({ query, dismissNinjaWindow })
</script>

<style scoped>
.ninja-app {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-top: 0;
  outline: none;
}
</style>
