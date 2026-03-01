<template>
  <div>
    <!--
      Error banner: shown when ensure_daemon_running failed (errorMessage is set)
      AND we are not yet connected. Dismissed manually or auto-hides on connect.
    -->
    <div
      v-if="showErrorBanner"
      class="daemon-error-banner"
      role="alert"
    >
      <p>{{ bannerMessage }}</p>
      <code v-if="showStartCmd">d1 start</code>
      <button class="banner-dismiss" @click="dismissBanner" aria-label="Dismiss">✕</button>
    </div>
    <Transition name="mode-switch" mode="out-in">
      <FullMode v-if="appStore.uiMode === 'full'" key="full" />
      <CopilotMode v-else-if="appStore.uiMode === 'copilot'" key="copilot" />
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
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

/** User can dismiss the banner manually; it also hides on successful connect. */
const bannerDismissed = ref(false)

const showErrorBanner = computed(() => {
  if (bannerDismissed.value) return false
  if (!daemonStore.errorMessage) return false
  // Hide once connected — error was transient (dev-mode warning)
  if (daemonStore.status === 'connected') return false
  return true
})

const bannerMessage = computed(() => {
  const msg = daemonStore.errorMessage ?? ''
  return msg.replace(/\.\s*Start it with:.*$/i, '.').trim()
    || 'Day1 Doctor daemon couldn\'t start.'
})

const showStartCmd = computed(() => {
  return !!daemonStore.errorMessage?.toLowerCase().includes('start')
})

function dismissBanner() {
  bannerDismissed.value = true
}

// Re-show the banner if a new error arrives (e.g. after a reconnect failure)
watch(() => daemonStore.errorMessage, (newMsg) => {
  if (newMsg) bannerDismissed.value = false
})

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

.banner-dismiss {
  margin-left: auto;
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.8);
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
  padding: 0 4px;
  transition: color 0.1s;
}

.banner-dismiss:hover {
  color: #fff;
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
