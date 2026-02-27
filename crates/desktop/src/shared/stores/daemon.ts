import { defineStore } from 'pinia'
import { ref } from 'vue'

export type DaemonStatus = 'connecting' | 'connected' | 'disconnected' | 'error'

export const useDaemonStore = defineStore('daemon', () => {
  const status = ref<DaemonStatus>('disconnected')
  const daemonVersion = ref<string | null>(null)
  const orchestratorConnected = ref(false)
  const activeTasks = ref(0)
  const errorMessage = ref<string | null>(null)
  const currentBobPhrase = ref<string | null>(null)
  const currentPlanId = ref<string | null>(null)

  function setStatus(s: DaemonStatus) { status.value = s }

  function setDaemonInfo(info: {
    daemonVersion: string
    orchestratorConnected: boolean
    activeTasks: number
  }) {
    daemonVersion.value = info.daemonVersion
    orchestratorConnected.value = info.orchestratorConnected
    activeTasks.value = info.activeTasks
  }

  function setError(msg: string) {
    status.value = 'error'
    errorMessage.value = msg
  }

  function setBobPhrase(phrase: string | null) {
    currentBobPhrase.value = phrase
  }

  function setCurrentPlanId(id: string | null) { currentPlanId.value = id }

  function decrementActiveTasks() {
    activeTasks.value = Math.max(0, activeTasks.value - 1)
  }

  return {
    status,
    daemonVersion,
    orchestratorConnected,
    activeTasks,
    errorMessage,
    currentBobPhrase,
    currentPlanId,
    setStatus,
    setDaemonInfo,
    setError,
    setBobPhrase,
    setCurrentPlanId,
    decrementActiveTasks,
  }
})
