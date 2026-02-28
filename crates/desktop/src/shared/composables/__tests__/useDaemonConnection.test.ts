import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { mount, flushPromises } from '@vue/test-utils'
import { defineComponent } from 'vue'

// ── Mock Tauri invoke ────────────────────────────────────────────────────────
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}))

// ── WebSocket mock ────────────────────────────────────────────────────────────
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
    // Call onopen asynchronously to simulate real WS
    Promise.resolve().then(() => this.onopen?.(new Event('open')))
  }

  /** Simulate receiving a message from the server */
  simulateMessage(data: unknown) {
    this.onmessage?.(new MessageEvent('message', { data: JSON.stringify(data) }))
  }

  /** Simulate the connection closing */
  simulateClose() {
    this.readyState = MockWebSocket.CLOSED
    this.onclose?.(new CloseEvent('close'))
  }

  static instances: MockWebSocket[] = []
  static reset() { MockWebSocket.instances = [] }
}

// Attach MockWebSocket to global so the composable picks it up
vi.stubGlobal('WebSocket', MockWebSocket)

// ── Helper: mount composable in a minimal component ──────────────────────────
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

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('on mount: calls invoke("ensure_daemon_running") then opens WebSocket to ws://localhost:9876/ws', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    await mountComposable()

    expect(invoke).toHaveBeenCalledWith('ensure_daemon_running')
    expect(MockWebSocket.instances).toHaveLength(1)
    expect(MockWebSocket.instances[0].url).toBe('ws://localhost:9876/ws')
  })

  it('on daemon.status message: sets daemonStore.status to connected and updates daemonVersion', async () => {
    const { daemonStore } = await mountComposable()
    const ws = MockWebSocket.instances[0]

    ws.simulateMessage({
      v: 1,
      id: 'msg-1',
      ts: Date.now(),
      type: 'daemon.status',
      payload: {
        daemon_version: '0.4.1',
        protocol_version: 1,
        orchestrator_connected: true,
        orchestrator_url: 'wss://api.day1doctor.com',
        active_tasks: 0,
        device_id: 'dev-abc',
      },
    })

    await flushPromises()
    expect(daemonStore.status).toBe('connected')
    expect(daemonStore.daemonVersion).toBe('0.4.1')
    expect(daemonStore.orchestratorConnected).toBe(true)
  })

  it('on step.started message: calls updateStep(stepId, "active") and sets a non-null currentBobPhrase in store', async () => {
    const { daemonStore, conversationStore } = await mountComposable()
    const ws = MockWebSocket.instances[0]

    // Set up a plan so updateStep has something to work with
    conversationStore.setPlan({
      steps: [
        { id: 'step-1', label: 'Install deps', state: 'pending', index: 0 },
      ],
      approved: null,
    })

    ws.simulateMessage({
      v: 1,
      id: 'msg-2',
      ts: Date.now(),
      type: 'step.started',
      payload: {
        task_id: 'tsk-abc',
        step_id: 'step-1',
        order: 0,
        description: 'Install deps',
        agent: 'executor',
        started_at: Date.now(),
      },
    })

    await flushPromises()

    const step = conversationStore.currentPlan?.steps.find(s => s.id === 'step-1')
    expect(step?.state).toBe('active')
    expect(daemonStore.currentBobPhrase).not.toBeNull()
    expect(typeof daemonStore.currentBobPhrase).toBe('string')
  })

  it('on ws.close event: daemonStore.status becomes disconnected', async () => {
    const { daemonStore } = await mountComposable()
    const ws = MockWebSocket.instances[0]

    // First get connected
    ws.simulateMessage({
      v: 1, id: 'msg-1', ts: Date.now(), type: 'daemon.status',
      payload: { daemon_version: '0.4.1', protocol_version: 1, orchestrator_connected: true, orchestrator_url: '', active_tasks: 0, device_id: 'x' },
    })
    await flushPromises()
    expect(daemonStore.status).toBe('connected')

    // Simulate close
    ws.simulateClose()
    await flushPromises()
    expect(daemonStore.status).toBe('disconnected')
  })

  it('submitTask() sends ws message with type task.submit and the given input', async () => {
    const { exposed } = await mountComposable()
    const ws = MockWebSocket.instances[0]

    exposed.submitTask('install openclaw')

    expect(ws.send).toHaveBeenCalledTimes(1)
    const sentData = JSON.parse(ws.send.mock.calls[0][0])
    expect(sentData.type).toBe('task.submit')
    expect(sentData.payload.input).toBe('install openclaw')
    expect(sentData.payload.task_id).toMatch(/^tsk_/)
  })

  it('plan.proposed message stores plan_id in daemon store', async () => {
    const { daemonStore, conversationStore } = await mountComposable()
    const ws = MockWebSocket.instances[0]

    ws.simulateMessage({
      v: 1,
      id: 'msg-plan',
      ts: Date.now(),
      type: 'plan.proposed',
      payload: {
        task_id: 'tsk-abc',
        plan_id: 'pln_test123',
        steps: [
          { step_id: 'stp-1', description: 'Install dependencies', order: 0 },
          { step_id: 'stp-2', description: 'Run tests', order: 1 },
        ],
      },
    })

    await flushPromises()

    // plan_id must be stored in the daemon store
    expect(daemonStore.currentPlanId).toBe('pln_test123')
    // steps must be mapped into the conversation store
    expect(conversationStore.currentPlan).not.toBeNull()
    expect(conversationStore.currentPlan?.steps).toHaveLength(2)
    expect(conversationStore.currentPlan?.steps[0].id).toBe('stp-1')
    expect(conversationStore.currentPlan?.steps[1].label).toBe('Run tests')
  })

  it('task.completed message decrements activeTasks via decrementActiveTasks()', async () => {
    const { daemonStore } = await mountComposable()
    const ws = MockWebSocket.instances[0]

    // Set active tasks to 2
    daemonStore.setDaemonInfo({ daemonVersion: '0.4.1', orchestratorConnected: true, activeTasks: 2 })

    ws.simulateMessage({
      v: 1,
      id: 'msg-done',
      ts: Date.now(),
      type: 'task.completed',
      payload: {
        task_id: 'tsk-abc',
        summary: 'All done!',
        steps_completed: 3,
        steps_total: 3,
        artifacts: [],
        completed_at: Date.now(),
      },
    })

    await flushPromises()
    expect(daemonStore.activeTasks).toBe(1)
  })
})
