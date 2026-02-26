import { describe, it, expect, vi, beforeEach } from 'vitest'
import { flushPromises } from '@vue/test-utils'
import { mount } from '@vue/test-utils'
import { defineComponent } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import { useAgentEvents } from '../useAgentEvents'
import { useConversationStore } from '@/shared/stores/conversation'
import { useAgentStore } from '@/shared/stores/agent'

// Capture the listener callback so we can fire mock events
let capturedListener: ((event: { payload: any }) => void) | null = null
let capturedUnlisten: ReturnType<typeof vi.fn>

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async (_eventName: string, handler: (event: { payload: any }) => void) => {
    capturedListener = handler
    capturedUnlisten = vi.fn()
    return capturedUnlisten
  }),
}))

// Mock @tauri-apps/api/core so store imports don't break
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue('full'),
}))

const TestComponent = defineComponent({
  setup() {
    useAgentEvents()
  },
  template: '<div />',
})

function fireEvent(type: string, payload: Record<string, unknown>) {
  capturedListener?.({ payload: { type, payload } })
}

describe('useAgentEvents', () => {
  let pinia: ReturnType<typeof createPinia>

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    capturedListener = null
    capturedUnlisten = vi.fn() // reset here
  })

  it('plan_proposed → conversationStore.setPlan() called', async () => {
    const wrapper = mount(TestComponent, { global: { plugins: [pinia] } })
    await flushPromises()

    const store = useConversationStore()
    const plan = {
      steps: [{ id: 's1', label: 'Step 1', state: 'pending' as const, index: 0 }],
      approved: null,
    }
    fireEvent('plan_proposed', plan)
    expect(store.currentPlan).toEqual(plan)

    wrapper.unmount()
  })

  it('step_started → step state becomes active', async () => {
    const wrapper = mount(TestComponent, { global: { plugins: [pinia] } })
    await flushPromises()

    const store = useConversationStore()
    store.setPlan({
      steps: [{ id: 's1', label: 'Step 1', state: 'pending', index: 0 }],
      approved: null,
    })

    fireEvent('step_started', { stepId: 's1' })
    expect(store.currentPlan!.steps[0].state).toBe('active')

    wrapper.unmount()
  })

  it('step_completed → step state becomes done', async () => {
    const wrapper = mount(TestComponent, { global: { plugins: [pinia] } })
    await flushPromises()

    const store = useConversationStore()
    store.setPlan({
      steps: [{ id: 's1', label: 'Step 1', state: 'active', index: 0 }],
      approved: null,
    })

    fireEvent('step_completed', { stepId: 's1' })
    expect(store.currentPlan!.steps[0].state).toBe('done')

    wrapper.unmount()
  })

  it('step_failed → step state becomes error', async () => {
    const wrapper = mount(TestComponent, { global: { plugins: [pinia] } })
    await flushPromises()

    const store = useConversationStore()
    store.setPlan({
      steps: [{ id: 's1', label: 'Step 1', state: 'active', index: 0 }],
      approved: null,
    })

    fireEvent('step_failed', { stepId: 's1' })
    expect(store.currentPlan!.steps[0].state).toBe('error')

    wrapper.unmount()
  })

  it('agent_message → message appended to store.messages', async () => {
    const wrapper = mount(TestComponent, { global: { plugins: [pinia] } })
    await flushPromises()

    const store = useConversationStore()
    const msg = {
      id: 'msg-1',
      role: 'agent' as const,
      content: 'Hello from agent',
      timestamp: 1000,
    }

    fireEvent('agent_message', msg)
    expect(store.messages).toHaveLength(1)
    expect(store.messages[0]).toEqual(msg)

    wrapper.unmount()
  })

  it('result_ready → message appended with title and detail combined', async () => {
    const wrapper = mount(TestComponent, { global: { plugins: [pinia] } })
    await flushPromises()

    const store = useConversationStore()

    fireEvent('result_ready', { title: 'Diagnosis Complete', detail: 'Patient is healthy' })
    expect(store.messages).toHaveLength(1)
    expect(store.messages[0].content).toBe('Diagnosis Complete\n\nPatient is healthy')
    expect(store.messages[0].role).toBe('agent')
    expect(typeof store.messages[0].id).toBe('string')
    expect(typeof store.messages[0].timestamp).toBe('number')

    wrapper.unmount()
  })

  it('result_ready without detail → message appended with title only', async () => {
    const wrapper = mount(TestComponent, { global: { plugins: [pinia] } })
    await flushPromises()

    const store = useConversationStore()

    fireEvent('result_ready', { title: 'Diagnosis Complete', detail: '' })
    expect(store.messages).toHaveLength(1)
    expect(store.messages[0].content).toBe('Diagnosis Complete')
    expect(store.messages[0].role).toBe('agent')

    wrapper.unmount()
  })

  it('credits_updated → agentStore.credits updated', async () => {
    const wrapper = mount(TestComponent, { global: { plugins: [pinia] } })
    await flushPromises()

    const store = useAgentStore()

    fireEvent('credits_updated', { current: 42, max: 200 })
    expect(store.credits).toEqual({ current: 42, max: 200 })

    wrapper.unmount()
  })

  it('permission_requested → message appended with "Permission requested:" prefix', async () => {
    const wrapper = mount(TestComponent, { global: { plugins: [pinia] } })
    await flushPromises()

    const store = useConversationStore()

    fireEvent('permission_requested', { permission: 'camera' })
    expect(store.messages).toHaveLength(1)
    expect(store.messages[0].content).toBe('Permission requested: camera')
    expect(store.messages[0].role).toBe('agent')

    wrapper.unmount()
  })

  it('onUnmounted → unlisten() is called', async () => {
    const wrapper = mount(TestComponent, { global: { plugins: [pinia] } })
    await flushPromises()

    const unlistenFn = capturedUnlisten
    expect(unlistenFn).toBeDefined()

    wrapper.unmount()
    expect(unlistenFn).toHaveBeenCalledTimes(1)
  })

  it('unknown event type is silently ignored without corrupting store', async () => {
    const wrapper = mount(TestComponent, { global: { plugins: [pinia] } })
    await flushPromises()

    const conversationStore = useConversationStore()
    const agentStore = useAgentStore()

    // Verify initial state
    const messagesBefore = conversationStore.messages.length
    const creditsBefore = agentStore.credits

    // Fire an unknown event type
    expect(() => {
      fireEvent('totally_unknown_event_type', { someField: 'someValue' })
    }).not.toThrow()

    await flushPromises()

    // Stores should be unchanged
    expect(conversationStore.messages).toHaveLength(messagesBefore)
    expect(agentStore.credits).toEqual(creditsBefore)

    wrapper.unmount()
  })

  it('multiple events in sequence → all dispatched correctly', async () => {
    const wrapper = mount(TestComponent, { global: { plugins: [pinia] } })
    await flushPromises()

    const conversationStore = useConversationStore()
    const agentStore = useAgentStore()

    // Set up a plan first
    const plan = {
      steps: [
        { id: 's1', label: 'Step 1', state: 'pending' as const, index: 0 },
        { id: 's2', label: 'Step 2', state: 'pending' as const, index: 1 },
      ],
      approved: null,
    }
    fireEvent('plan_proposed', plan)
    expect(conversationStore.currentPlan).toEqual(plan)

    // Start step 1
    fireEvent('step_started', { stepId: 's1' })
    expect(conversationStore.currentPlan!.steps[0].state).toBe('active')

    // Complete step 1
    fireEvent('step_completed', { stepId: 's1' })
    expect(conversationStore.currentPlan!.steps[0].state).toBe('done')

    // Start step 2
    fireEvent('step_started', { stepId: 's2' })
    expect(conversationStore.currentPlan!.steps[1].state).toBe('active')

    // Fail step 2
    fireEvent('step_failed', { stepId: 's2' })
    expect(conversationStore.currentPlan!.steps[1].state).toBe('error')

    // Agent message
    const msg = { id: 'msg-1', role: 'agent' as const, content: 'Done', timestamp: 2000 }
    fireEvent('agent_message', msg)
    expect(conversationStore.messages).toHaveLength(1)

    // Credits updated
    fireEvent('credits_updated', { current: 10, max: 100 })
    expect(agentStore.credits).toEqual({ current: 10, max: 100 })

    // Permission requested
    fireEvent('permission_requested', { permission: 'microphone' })
    expect(conversationStore.messages).toHaveLength(2)
    expect(conversationStore.messages[1].content).toBe('Permission requested: microphone')

    // Result ready — content should combine title and detail
    fireEvent('result_ready', { title: 'Report Ready', detail: 'See below' })
    expect(conversationStore.messages).toHaveLength(3)
    expect(conversationStore.messages[2].content).toBe('Report Ready\n\nSee below')

    wrapper.unmount()
  })
})
