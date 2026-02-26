import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import type { UIMode } from '@/shared/types'

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
        await invoke('resize_window', { width: 1200, height: 760 })
      } else if (mode === 'copilot') {
        await invoke('resize_window', { width: 420, height: 720 })
        const x = Math.max(0, window.screen.availWidth - 420 - 12)
        const y = 12
        await invoke('position_window', { x, y })
      }
    }
  }

  async function init(): Promise<void> {
    try {
      const savedMode = await invoke<string>('get_config', { key: 'ui_mode' })
      if (savedMode && (['full', 'copilot', 'ninja'] as string[]).includes(savedMode)) {
        // On startup in ninja mode, restore to previous non-ninja mode
        uiMode.value = (savedMode === 'ninja' ? 'full' : savedMode) as UIMode
      }
    } catch {
      // Default to full mode on error
    }
  }

  return { uiMode, previousMode, switchMode, init }
})
