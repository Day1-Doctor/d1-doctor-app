import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { CreditInfo } from '@/shared/types'

export const useAgentStore = defineStore('agent', () => {
  const credits = ref<CreditInfo>({ current: 0, max: 100 })
  const activeAgents = ref<string[]>([])
  const connectionStatus = ref<'connected' | 'disconnected' | 'connecting'>('disconnected')
  function updateCredits(_info: CreditInfo): void {}
  return { credits, activeAgents, connectionStatus, updateCredits }
})
