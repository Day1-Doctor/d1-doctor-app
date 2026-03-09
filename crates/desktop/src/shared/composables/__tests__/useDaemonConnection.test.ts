import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { mount, flushPromises } from '@vue/test-utils'
import { defineComponent } from 'vue'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}))

vi.mock('@/shared/stores/auth', () => ({
  useAuthStore: vi.fn(() => ({
    status: 'unknown',
    loading: false,
    error: null,
    userEmail: null,
    isAuthenticated: false,
    isUnauthenticated: false,
    checkAuth: vi.fn(),
    login: vi.fn(),
    loginWithGoogle: vi.fn(),
    logout: vi.fn(),
    clearError: vi.fn(),
    setUnauthenticated: vi.fn(),
  })),
}))

class MockWebSocket {
  static OPEN = 1
  static CLOSING = 2
  static CLOSED = 3
  readyState = MockWebSocket.OPEN
  url: string
  onopen: ((ev: Event) => void) | null = null
  onmessage: ((ev: MessageEvent) => void) | null = null
  onclose: ((ev: CloseEvent) => void) | null = null
  onerror: ((ev: Event) => void) | null = null
  send = vi.fn()
  close = vi.fn()

  constructor(url: string) {
    this.url = url
    MockWebSocket.instances.push(this)
    Promise.resolve().then(() => this.onopen?.(new Event('open')))
  }

  simulateMessage(data: unknown) {
    this.onmessage?.(new MessageEvent('message', { data: JSON.stringify(data) }))
  }

  simulateClose() {
    this.readyState = MockWebSocket.CLOSED
    this.onclose?.(new CloseEvent('close'))
  }

  static instances: MockWebSocket[] = []
  static reset() { MockWebSocket.instances = [] }
}

vi.stubGlobal('WebSocket', MockWebSocket)

async function mountComposable() {
  const { useDaemonConnection } = await import('../useDaemonConnection')
  const { useDaemonStore } = await import('../../stores/daemon')
  const { useConversationStore } = await import('../../stores/conversation')

  let exposed: ReturnType<typeof useDaemonConnection>

  const TestComponent = defineComponent({
    setup() {
      exposed = useDaemonConnection()
      return exposed
    },
    template: '<div />',
  })

  const pinia = createPinia()
  setActivePinia(pinia)

  const wrapper = mount(TestComponent, { global: { plugins: [pinia] } })
  await flushPromises()

  return {
    wrapper,
    exposed: exposed!,
    daemonStore: useDaemonStore(),
    conversationStore: useConversationStore(),
  }
}

describe('useDaemonConnection', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    MockWebSocket.reset()
    vi.clearAllMocks()
  })

  afterEach(() => { vi.clearAllMocks() })

  it('on mount: calls invoke("ensure_daemon_running") then opens WebSocket', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    await mountComposable()
    expect(invoke).toHaveBeenCalledWith('ensure_daemon_running')
    expect(MockWebSocket.instances).toHaveLength(1)
    expect(MockWebSocket.instances[0].url).toBe('ws://localhost:9876/ws')
  })

  it('on daemon.status message: sets daemonStore.status to connected', async () => {
    const { daemonStore } = await mountComposable()
    const ws = MockWebSocket.instances[0]
    ws.simulateMessage({
      v: 1, id: 'msg-1', ts: Date.now(), type: 'daemon.status',
      payload: { daemon_version: '0.4.1', protocol_version: 1, orchestrator_connected: true, orchestrator_url: 'wss://api.day1doctor.com', active_tasks: 0, device_id: 'dev-abc' },
    })
    await flushPromises()
    expect(daemonStore.status).toBe('connected')
    expect(daemonStore.daemonVersion).toBe('0.4.1')
  })

  it('on step.started message: calls updateStep and sets bobPhrase', async () => {
    const { daemonStore, conversationStore } = await mountComposable()
    const ws = MockWebSocket.instances[0]
    conversationStore.setPlan({ steps: [{ id: 'step-1', label: 'Install deps', state: 'pending', index: 0 }], approved: null })
    ws.simulateMessage({
      v: 1, id: 'msg-2', ts: Date.now(), type: 'step.started',
      payload: { task_id: 'tsk-abc', step_id: 'step-1', order: 0, description: 'Install deps', agent: 'executor', started_at: Date.now() },
    })
    await flushPromises()
    expect(conversationStore.currentPlan?.steps.find(s => s.id === 'step-1')?.state).toBe('active')
    expect(daemonStore.currentBobPhrase).not.toBeNull()
  })

  it('on ws.close event: daemonStore.status becomes disconnected', async () => {
    const { daemonStore } = await mountComposable()
    const ws = MockWebSocket.instances[0]
    ws.simulateMessage({
      v: 1, id: 'msg-1', ts: Date.now(), type: 'daemon.status',
      payload: { daemon_version: '0.4.1', protocol_version: 1, orchestrator_connected: true, orchestrator_url: '', active_tasks: 0, device_id: 'x' },
    })
    await flushPromises()
    expect(daemonStore.status).toBe('connected')
    ws.simulateClose()
    await flushPromises()
    expect(daemonStore.status).toBe('disconnected')
  })

  it('submitTask() sends ws message with type task.submit', async () => {
    const { exposed } = await mountComposable()
    const ws = MockWebSocket.instances[0]
    exposed.submitTask('install openclaw')
    expect(ws.send).toHaveBeenCalledTimes(1)
    const sentData = JSON.parse(ws.send.mock.calls[0][0])
    expect(sentData.type).toBe('task.submit')
    expect(sentData.payload.input).toBe('install openclaw')
  })

  it('plan.proposed message stores plan_id in daemon store', async () => {
    const { daemonStore, conversationStore } = await mountComposable()
    const ws = MockWebSocket.instances[0]
    ws.simulateMessage({
      v: 1, id: 'msg-plan', ts: Date.now(), type: 'plan.proposed',
      payload: { task_id: 'tsk-abc', plan_id: 'pln_test123', requires_approval: true, steps: [{ step_id: 'stp-1', description: 'Install dependencies', order: 0 }, { step_id: 'stp-2', description: 'Run tests', order: 1 }] },
    })
    await flushPromises()
    expect(daemonStore.currentPlanId).toBe('pln_test123')
    expect(conversationStore.currentPlan?.steps).toHaveLength(2)
  })

  it('task.completed message decrements activeTasks', async () => {
    const { daemonStore } = await mountComposable()
    const ws = MockWebSocket.instances[0]
    daemonStore.setDaemonInfo({ daemonVersion: '0.4.1', orchestratorConnected: true, activeTasks: 2 })
    ws.simulateMessage({
      v: 1, id: 'msg-done', ts: Date.now(), type: 'task.completed',
      payload: { task_id: 'tsk-abc', summary: 'All done!', steps_completed: 3, steps_total: 3, artifacts: [], completed_at: Date.now() },
    })
    await flushPromises()
    expect(daemonStore.activeTasks).toBe(1)
  })
})
