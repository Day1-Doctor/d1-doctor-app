// Day1 Doctor — Auto-updater composable
// Checks for updates on app launch, downloads in the background,
// and exposes reactive state for the UpdateBanner component.

import { ref, readonly, onMounted } from 'vue'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'

/** How often to re-check after initial check (ms). */
const RECHECK_INTERVAL_MS = 4 * 60 * 60 * 1000 // 4 hours

export function useAutoUpdater() {
  const updateReady = ref(false)
  const updateVersion = ref('')
  const bannerDismissed = ref(false)

  let pendingUpdate: Update | null = null
  let recheckTimer: ReturnType<typeof setInterval> | null = null

  async function checkForUpdate() {
    try {
      const update = await check()
      if (update) {
        pendingUpdate = update
        updateVersion.value = update.version

        // Download in the background (non-blocking)
        await update.downloadAndInstall()

        // Download + install completed — ready to relaunch
        updateReady.value = true
        bannerDismissed.value = false
      }
    } catch (err) {
      // Update check/download failures are non-critical — log and move on
      console.warn('[useAutoUpdater] Update check failed:', err)
    }
  }

  async function restartNow() {
    try {
      await relaunch()
    } catch (err) {
      console.error('[useAutoUpdater] Relaunch failed:', err)
    }
  }

  function dismissBanner() {
    bannerDismissed.value = true
  }

  onMounted(() => {
    // Initial check shortly after launch (2s delay to avoid blocking startup)
    setTimeout(() => {
      void checkForUpdate()
    }, 2000)

    // Periodic re-check for long-running sessions
    recheckTimer = setInterval(() => {
      if (!updateReady.value) {
        void checkForUpdate()
      }
    }, RECHECK_INTERVAL_MS)
  })

  return {
    /** True when an update has been downloaded and is ready to install. */
    updateReady: readonly(updateReady),
    /** The version string of the pending update. */
    updateVersion: readonly(updateVersion),
    /** Whether the user dismissed the banner for this session. */
    bannerDismissed: readonly(bannerDismissed),
    /** Restart the app to apply the update. */
    restartNow,
    /** Dismiss the update banner for this session. */
    dismissBanner,
  }
}
