<template>
  <div>
    <Transition name="mode-switch" mode="out-in">
      <LoginScreen v-if="authStore.isUnauthenticated" key="login" />
      <div v-else key="app">
        <div
          v-if="showErrorBanner"
          class="daemon-error-banner"
          role="alert"
        >
          <p>{{ bannerMessage }}</p>
          <code v-if="showStartCmd">d1 start</code>
          <button class="banner-dismiss" @click="dismissBanner" aria-label="Dismiss">&#x2715;</button>
        </div>
        <Transition name="mode-switch" mode="out-in">
          <FullMode v-if="appStore.uiMode === 'full'" key="full" />
          <CopilotMode v-else-if="appStore.uiMode === 'copilot'" key="copilot" />
        </Transition>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { useAppStore } from '@/shared/stores/app'
import { useAuthStore } from '@/shared/stores/auth'
import { useAgentEvents } from '@/shared/composables/useAgentEvents'
import { useDaemonConnection } from '@/shared/composables/useDaemonConnection'
import { useDaemonStore } from '@/shared/stores/daemon'
import FullMode from '@/modes/full/FullMode.vue'
import CopilotMode from '@/modes/copilot/CopilotMode.vue'
import LoginScreen from '@/shared/components/LoginScreen.vue'

const appStore = useAppStore()
const authStore = useAuthStore()
useAgentEvents()
useDaemonConnection()
const daemonStore = useDaemonStore()

const bannerDismissed = ref(false)

const showErrorBanner = computed(() => {
  if (bannerDismissed.value) return false
  if (!daemonStore.errorMessage) return false
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

watch(() => daemonStore.errorMessage, (newMsg) => {
  if (newMsg) bannerDismissed.value = false
})

let unlistenNinja: UnlistenFn | null = null

onMounted(async () => {
  await authStore.checkAuth()
  await appStore.init()
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
.daemon-error-banner p { margin: 0; }
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
.banner-dismiss:hover { color: #fff; }
.mode-switch-enter-active,
.mode-switch-leave-active { transition: opacity 0.3s ease; }
.mode-switch-enter-from,
.mode-switch-leave-to { opacity: 0; }
</style>
