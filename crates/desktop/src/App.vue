<template>
  <div>
    <Transition name="mode-switch" mode="out-in">
      <LoginScreen v-if="authStore.isUnauthenticated" key="login" />
      <div v-else key="app">
        <!-- Update banner: shown when a new version has been downloaded and is ready -->
        <UpdateBanner
          :visible="showUpdateBanner"
          :version="updateVersion"
          @restart="restartNow"
          @dismiss="dismissUpdate"
        />

        <!--
          Error banner: shown when ensure_daemon_running failed (errorMessage is set)
          AND we are not yet connected. Dismissed manually or auto-hides on connect.
        -->
        <div
          v-if="showErrorBanner"
          class="daemon-error-banner"
          role="alert"
          aria-live="assertive"
          aria-atomic="true"
        >
          <p>{{ bannerMessage }}</p>
          <code v-if="showStartCmd">d1 start</code>
          <button class="banner-dismiss" @click="dismissBanner" :aria-label="$t('banner.dismiss')">&#x2715;</button>
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
import { useI18n } from 'vue-i18n'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { useAppStore } from '@/shared/stores/app'
import { useAuthStore } from '@/shared/stores/auth'
import { useAgentEvents } from '@/shared/composables/useAgentEvents'
import { useDaemonConnection } from '@/shared/composables/useDaemonConnection'
import { useDaemonStore } from '@/shared/stores/daemon'
import FullMode from '@/modes/full/FullMode.vue'
import CopilotMode from '@/modes/copilot/CopilotMode.vue'
import UpdateBanner from '@/shared/components/UpdateBanner.vue'
import LoginScreen from '@/shared/components/LoginScreen.vue'

const { t } = useI18n()
const appStore = useAppStore()
const authStore = useAuthStore()
useAgentEvents() // auto-registers Tauri event listener on mount
// Establishes WebSocket connection; return value not used at this level.
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
    || t('banner.daemonError')
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
  background: var(--error);
  color: var(--text-contrast);
  padding: var(--space-sm) var(--space-lg);
  display: flex;
  align-items: center;
  gap: var(--space-md);
  font: var(--font-size-base) var(--font-mono);
}
.daemon-error-banner p { margin: 0; }
.daemon-error-banner code {
  background: var(--color-disabled-bg);
  padding: var(--space-2xs) var(--space-xs);
  border-radius: var(--space-xs);
}
.banner-dismiss {
  margin-left: auto;
  background: transparent;
  border: none;
  color: var(--text-placeholder);
  cursor: pointer;
  font-size: var(--font-size-lg);
  line-height: 1;
  padding: 0 var(--space-xs);
  transition: color var(--duration-instant);
}
.banner-dismiss:hover { color: var(--text-contrast); }
.mode-switch-enter-active,
.mode-switch-leave-active { transition: opacity var(--duration-fast) var(--easing-default); }
.mode-switch-enter-from,
.mode-switch-leave-to { opacity: 0; }
</style>
