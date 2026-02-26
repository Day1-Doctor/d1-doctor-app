import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import type { UIMode } from '@/shared/types'

const FULL_WINDOW_WIDTH = 1200
const FULL_WINDOW_HEIGHT = 760
const COPILOT_WINDOW_WIDTH = 420
const COPILOT_WINDOW_HEIGHT = 720
const COPILOT_WINDOW_MARGIN = 12

function isUIMode(value: string): value is UIMode {
  return value === 'full' || value === 'copilot' || value === 'ninja'
}

export const useAppStore = defineStore('app', () => {
  const uiMode = ref<UIMode>('full')
  const previousMode = ref<UIMode>('full')

  async function switchMode(mode: UIMode): Promise<void> {
    // Persist to config
    await invoke('set_config', { key: 'ui_mode', value: mode })

    const mainWindow = getCurrentWindow()
    const ninjaWindow = await WebviewWindow.getByLabel('ninja-bar')

    if (mode === 'ninja') {
      // Save previous mode before entering ninja
      previousMode.value = uiMode.value
      uiMode.value = mode
      await mainWindow.hide()
      if (ninjaWindow) await ninjaWindow.show()
    } else {
      // Hide ninja bar if visible
      if (ninjaWindow) await ninjaWindow.hide()
      uiMode.value = mode
      await mainWindow.show()
      await mainWindow.setFocus()

      if (mode === 'full') {
        await invoke('resize_window', { width: FULL_WINDOW_WIDTH, height: FULL_WINDOW_HEIGHT })
      } else if (mode === 'copilot') {
        await invoke('resize_window', { width: COPILOT_WINDOW_WIDTH, height: COPILOT_WINDOW_HEIGHT })
        const x = Math.max(0, window.screen.availWidth - COPILOT_WINDOW_WIDTH - COPILOT_WINDOW_MARGIN)
        const y = COPILOT_WINDOW_MARGIN
        await invoke('position_window', { x, y })
      }
    }
  }

  async function init(): Promise<void> {
    try {
      const savedMode = await invoke<string>('get_config', { key: 'ui_mode' })
      if (savedMode && isUIMode(savedMode)) {
        // On startup in ninja mode, restore to previous non-ninja mode
        uiMode.value = savedMode === 'ninja' ? 'full' : savedMode
      }
    } catch (err) {
      console.warn('[AppStore] Failed to load ui_mode from config, defaulting to full:', err)
    }
  }

  return { uiMode, previousMode, switchMode, init }
})
