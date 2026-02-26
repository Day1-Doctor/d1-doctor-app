import type { Config } from '@/shared/types'
export function useConfig() {
  async function getConfig<K extends keyof Config>(_key: K): Promise<Config[K] | null> { return null }
  async function setConfig<K extends keyof Config>(_key: K, _value: Config[K]): Promise<void> {}
  return { getConfig, setConfig }
}
