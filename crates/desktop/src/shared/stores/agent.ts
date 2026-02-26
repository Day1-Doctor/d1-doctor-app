import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { CreditInfo } from '@/shared/types'

export const useAgentStore = defineStore('agent', () => {
  const credits = ref<CreditInfo>({ current: 0, max: 100 })
  const activeAgents = ref<string[]>([])
  const connectionStatus = ref<'connected' | 'disconnected' | 'connecting'>('disconnected')

  function updateCredits(info: CreditInfo): void {
    credits.value = { ...info }
  }

  function setConnectionStatus(status: 'connected' | 'disconnected' | 'connecting'): void {
    connectionStatus.value = status
  }

  function setActiveAgents(agents: string[]): void {
    activeAgents.value = agents
  }

  return { credits, activeAgents, connectionStatus, updateCredits, setConnectionStatus, setActiveAgents }
})
