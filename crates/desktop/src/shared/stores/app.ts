import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { UIMode } from '@/shared/types'

export const useAppStore = defineStore('app', () => {
  const uiMode = ref<UIMode>('full')

  async function switchMode(mode: UIMode): Promise<void> {
    await invoke('set_config', { key: 'ui_mode', value: mode })
    uiMode.value = mode

    // Resize/reposition window for the target mode
    if (mode === 'full') {
      await invoke('resize_window', { width: 1200, height: 760 }).catch(() => {})
    } else if (mode === 'copilot') {
      await invoke('resize_window', { width: 420, height: 720 }).catch(() => {})
      // Position 12px from right edge — best effort, screen dimensions come from CSS/JS
    }
    // Ninja mode: handled by NinjaApp.vue window visibility
  }

  async function init(): Promise<void> {
    try {
      const savedMode = await invoke<string>('get_config', { key: 'ui_mode' })
      if (savedMode && (['full', 'copilot', 'ninja'] as string[]).includes(savedMode)) {
        uiMode.value = savedMode as UIMode
      }
    } catch {
      // Default to full mode on error
    }
  }

  return { uiMode, switchMode, init }
})
