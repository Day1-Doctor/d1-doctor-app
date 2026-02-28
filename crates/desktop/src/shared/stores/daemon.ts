import { defineStore } from 'pinia'
import { ref } from 'vue'

export type DaemonStatus = 'connecting' | 'connected' | 'disconnected'

export const useDaemonStore = defineStore('daemon', () => {
  const status = ref<DaemonStatus>('disconnected')
  const daemonVersion = ref<string | null>(null)
  const orchestratorConnected = ref(false)
  const activeTasks = ref(0)
  const errorMessage = ref<string | null>(null)
  const currentBobPhrase = ref<string | null>(null)
  const currentPlanId = ref<string | null>(null)
  const currentTaskId = ref<string | null>(null)

  function setStatus(s: DaemonStatus) {
    status.value = s
    // Only clear the error message when fully connected — not when merely
    // transitioning to 'connecting', so the hint stays visible during
    // reconnect attempts after an ensure_daemon_running failure.
    if (s === 'connected') errorMessage.value = null
  }

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
    // Store the error message for the UI to display as a hint/banner.
    // Do NOT force status to 'error' here — the WebSocket connection attempt
    // that follows ensure_daemon_running will set the real status ('connecting',
    // then 'connected' or 'disconnected'). Keeping status separate from the
    // error message avoids blocking the connect() call chain.
    errorMessage.value = msg
  }

  function setBobPhrase(phrase: string | null) {
    currentBobPhrase.value = phrase
  }

  function setCurrentPlanId(id: string | null) { currentPlanId.value = id }

  function setCurrentTaskId(id: string | null) { currentTaskId.value = id }

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
    currentTaskId,
    setStatus,
    setDaemonInfo,
    setError,
    setBobPhrase,
    setCurrentPlanId,
    setCurrentTaskId,
    decrementActiveTasks,
  }
})
