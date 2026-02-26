import { invoke } from '@tauri-apps/api/core'
import type { Config } from '@/shared/types'

export function useConfig() {
  async function getConfig<K extends keyof Config>(key: K): Promise<Config[K] | null> {
    try {
      const raw = await invoke<string>('get_config', { key: String(key) })
      if (!raw) return null
      // Parse numeric values
      const numericKeys: Array<keyof Config> = ['last_copilot_x', 'last_copilot_y']
      if (numericKeys.includes(key)) {
        return Number(raw) as unknown as Config[K]
      }
      return raw as unknown as Config[K]
    } catch {
      return null
    }
  }

  async function setConfig<K extends keyof Config>(key: K, value: Config[K]): Promise<void> {
    await invoke('set_config', { key: String(key), value: String(value) })
  }

  return { getConfig, setConfig }
}
