import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { UIMode } from '@/shared/types'

export const useAppStore = defineStore('app', () => {
  const uiMode = ref<UIMode>('full')
  async function switchMode(_mode: UIMode): Promise<void> {}
  async function init(): Promise<void> {}
  return { uiMode, switchMode, init }
})
