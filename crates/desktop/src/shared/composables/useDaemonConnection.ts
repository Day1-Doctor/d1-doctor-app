// Day 1 Doctor — Daemon WebSocket connection composable
// Connects to ws://localhost:9876/ws, handles all daemon message types.

import { onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { createMessage, type DaemonMessage } from '@/types/daemon'
import { getRandomBobPhrase } from '@/constants/bob'
import { useDaemonStore } from '@/shared/stores/daemon'
import { useConversationStore } from '@/shared/stores/conversation'
import { useAgentStore } from '@/shared/stores/agent'

const DAEMON_WS_URL = 'ws://localhost:9876/ws'
const HEARTBEAT_INTERVAL_MS = 30_000
const RECONNECT_DELAYS_MS = [1000, 2000, 4000, 8000, 8000]

export function useDaemonConnection() {
  const daemonStore = useDaemonStore()
  const conversationStore = useConversationStore()
  const agentStore = useAgentStore()

  let ws: WebSocket | null = null
  let heartbeatTimer: ReturnType<typeof setInterval> | null = null
  let reconnectAttempt = 0
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null

  function handleMessage(event: MessageEvent) {
    let msg: DaemonMessage
    try {
      msg = JSON.parse(event.data) as DaemonMessage
    } catch {
      console.error('[useDaemonConnection] Failed to parse message:', event.data)
      return
    }

    switch (msg.type) {
      case 'daemon.status': {
        const p = msg.payload
        daemonStore.setStatus('connected')
        daemonStore.setDaemonInfo({
          daemonVersion: p.daemon_version,
          orchestratorConnected: p.orchestrator_connected,
          activeTasks: p.active_tasks,
        })
        break
      }
      case 'plan.proposed': {
        const p = msg.payload
        daemonStore.setCurrentPlanId(p.plan_id)
        conversationStore.setPlan({
          steps: p.steps.map((s: any, i: number) => ({
            id: s.step_id,
            label: s.description,
            state: 'pending' as const,
            index: i,
          })),
          approved: null,
        })
        break
      }
      case 'step.started': {
        const p = msg.payload
        conversationStore.updateStep(p.step_id, 'active')
        daemonStore.setBobPhrase(getRandomBobPhrase())
        break
      }
      case 'step.completed': {
        const p = msg.payload
        conversationStore.updateStep(p.step_id, 'done')
        daemonStore.setBobPhrase(null)
        break
      }
      case 'step.failed': {
        const p = msg.payload
        conversationStore.updateStep(p.step_id, 'error')
        daemonStore.setBobPhrase(null)
        break
      }
      case 'agent.message': {
        const p = msg.payload
        conversationStore.appendMessage({
          id: crypto.randomUUID(),
          role: 'agent',
          content: p.message,
          timestamp: Date.now(),
        })
        break
      }
      case 'task.completed': {
        const p = msg.payload
        conversationStore.appendMessage({
          id: crypto.randomUUID(),
          role: 'agent',
          content: p.summary,
          timestamp: Date.now(),
        })
        daemonStore.decrementActiveTasks()
        break
      }
      case 'task.failed': {
        const p = msg.payload
        conversationStore.appendMessage({
          id: crypto.randomUUID(),
          role: 'agent',
          content: `Task failed: ${p.error.message}`,
          timestamp: Date.now(),
        })
        break
      }
      case 'credits.updated': {
        agentStore.updateCredits({
          current: msg.payload.daily_balance + msg.payload.bonus_balance,
          max: 100,
        })
        break
      }
      case 'permission.requested': {
        const p = msg.payload
        conversationStore.appendMessage({
          id: crypto.randomUUID(),
          role: 'agent',
          content: `Permission requested: ${p.description}`,
          timestamp: Date.now(),
        })
        break
      }
      case 'heartbeat': break
      case 'error': {
        console.error('[useDaemonConnection] Protocol error:', msg.payload)
        if (msg.payload.code === 'PROTOCOL_VERSION_MISMATCH') {
          daemonStore.setError('Day 1 Doctor app is out of date. Please update.')
        }
        break
      }
    }
  }

  function startHeartbeat() {
    heartbeatTimer = setInterval(() => {
      if (ws?.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(createMessage('heartbeat', { ping: true })))
      }
    }, HEARTBEAT_INTERVAL_MS)
  }

  function stopHeartbeat() {
    if (heartbeatTimer) { clearInterval(heartbeatTimer); heartbeatTimer = null }
  }

  function connect() {
    daemonStore.setStatus('connecting')
    ws = new WebSocket(DAEMON_WS_URL)
    ws.onopen = () => { reconnectAttempt = 0; startHeartbeat() }
    ws.onmessage = handleMessage
    ws.onclose = () => { stopHeartbeat(); daemonStore.setStatus('disconnected'); scheduleReconnect() }
    ws.onerror = (err) => { console.error('[useDaemonConnection] WS error:', err) }
  }

  function scheduleReconnect() {
    const delayMs = RECONNECT_DELAYS_MS[Math.min(reconnectAttempt, RECONNECT_DELAYS_MS.length - 1)]
    reconnectAttempt++
    reconnectTimer = setTimeout(connect, delayMs)
  }

  function disconnect() {
    stopHeartbeat()
    if (reconnectTimer) { clearTimeout(reconnectTimer); reconnectTimer = null }
    ws?.close()
    ws = null
  }

  function submitTask(input: string): string {
    const taskId = `tsk_${crypto.randomUUID().slice(0, 8)}`
    const msg = createMessage('task.submit', {
      task_id: taskId,
      input,
      context: { cwd: undefined, env: {} },
    })
    ws?.send(JSON.stringify(msg))
    return taskId
  }

  function approvePlan(taskId: string, planId: string, action: 'APPROVE' | 'REJECT') {
    const msg = createMessage('plan.approve', {
      task_id: taskId,
      plan_id: planId,
      action,
      modifications: null,
    })
    ws?.send(JSON.stringify(msg))
  }

  onMounted(async () => {
    try {
      await invoke('ensure_daemon_running')
    } catch (err) {
      console.error('[useDaemonConnection] Failed to start daemon:', err)
      daemonStore.setError('Daemon failed to start. Run: d1 start')
      return
    }
    connect()
  })

  onUnmounted(() => { disconnect() })

  return { submitTask, approvePlan }
}
