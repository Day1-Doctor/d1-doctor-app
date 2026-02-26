<template>
  <Transition name="mode-switch" mode="out-in">
    <FullMode v-if="appStore.uiMode === 'full'" key="full" />
    <CopilotMode v-else-if="appStore.uiMode === 'copilot'" key="copilot" />
  </Transition>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useAppStore } from '@/shared/stores/app'
import { useAgentEvents } from '@/shared/composables/useAgentEvents'
import FullMode from '@/modes/full/FullMode.vue'
import CopilotMode from '@/modes/copilot/CopilotMode.vue'

const appStore = useAppStore()
useAgentEvents() // auto-registers Tauri event listener on mount

let unlistenNinja: UnlistenFn | null = null

onMounted(async () => {
  await appStore.init()
  unlistenNinja = await listen('ninja_dismissed', () => {
    void appStore.switchMode(appStore.previousMode)
  })
})

onUnmounted(() => {
  unlistenNinja?.()
})
</script>

<style scoped>
.mode-switch-enter-active,
.mode-switch-leave-active {
  transition: opacity 0.3s ease;
}
.mode-switch-enter-from,
.mode-switch-leave-to {
  opacity: 0;
}
</style>
