<template>
  <div>
    <div
      v-if="daemonStore.status === 'error'"
      class="daemon-error-banner"
      role="alert"
    >
      <p>Day 1 Doctor daemon couldn't start.</p>
      <code>Run: d1 start</code>
    </div>
    <Transition name="mode-switch" mode="out-in">
      <FullMode v-if="appStore.uiMode === 'full'" key="full" />
      <CopilotMode v-else-if="appStore.uiMode === 'copilot'" key="copilot" />
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { useAppStore } from '@/shared/stores/app'
import { useAgentEvents } from '@/shared/composables/useAgentEvents'
import { useDaemonConnection } from '@/shared/composables/useDaemonConnection'
import { useDaemonStore } from '@/shared/stores/daemon'
import FullMode from '@/modes/full/FullMode.vue'
import CopilotMode from '@/modes/copilot/CopilotMode.vue'

const appStore = useAppStore()
useAgentEvents() // auto-registers Tauri event listener on mount
// Establishes WebSocket connection; return value not used at this level.
useDaemonConnection()
const daemonStore = useDaemonStore()

let unlistenNinja: UnlistenFn | null = null

onMounted(async () => {
  await appStore.init()
  // Always hide ninja-bar on startup — fixes the "always-showing" bug.
  const ninjaWindow = await WebviewWindow.getByLabel('ninja-bar')
  if (ninjaWindow) await ninjaWindow.hide()

  unlistenNinja = await listen('ninja_dismissed', () => {
    void appStore.switchMode(appStore.previousMode)
  })
})

onUnmounted(() => {
  unlistenNinja?.()
})
</script>

<style scoped>
.daemon-error-banner {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 9999;
  background: var(--error, #ef4444);
  color: #fff;
  padding: 8px 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  font: 12px var(--font-mono);
}

.daemon-error-banner p {
  margin: 0;
}

.daemon-error-banner code {
  background: rgba(0, 0, 0, 0.2);
  padding: 2px 6px;
  border-radius: 4px;
}

.mode-switch-enter-active,
.mode-switch-leave-active {
  transition: opacity 0.3s ease;
}
.mode-switch-enter-from,
.mode-switch-leave-to {
  opacity: 0;
}
</style>
