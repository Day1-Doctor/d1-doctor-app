import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'

const mockSubmitTask = vi.fn().mockReturnValue('tsk_test')
vi.mock('@/shared/composables/useDaemonConnection', () => ({
  useDaemonConnection: vi.fn(() => ({ submitTask: mockSubmitTask })),
}))

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockResolvedValue(undefined) }))
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn()
}))

const mockClose = vi.fn().mockResolvedValue(undefined)
const mockMinimize = vi.fn().mockResolvedValue(undefined)
const mockToggleMaximize = vi.fn().mockResolvedValue(undefined)

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    close: mockClose,
    minimize: mockMinimize,
    toggleMaximize: mockToggleMaximize,
  }))
}))

import CopilotMode from '../CopilotMode.vue'
import CopilotHeader from '../CopilotHeader.vue'
import SessionBar from '../SessionBar.vue'
import CopilotInput from '../CopilotInput.vue'

describe('CopilotMode', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  const copilotModeStubs = {
    CopilotHeader: true,
    SessionBar: true,
    CopilotInput: true,
    CreditBar: true,
    MessageBubble: true,
    PlanCard: true,
  }

  it('renders without errors', () => {
    const w = mount(CopilotMode, {
      global: { stubs: copilotModeStubs }
    })
    expect(w.exists()).toBe(true)
  })

  it('has .copilot-panel root element', () => {
    const w = mount(CopilotMode, {
      global: { stubs: copilotModeStubs }
    })
    expect(w.find('.copilot-panel').exists()).toBe(true)
  })

  it('has .copilot class on the root for deep style overrides', () => {
    const w = mount(CopilotMode, {
      global: { stubs: copilotModeStubs }
    })
    expect(w.find('.copilot').exists()).toBe(true)
  })

  it('renders message list area', () => {
    const w = mount(CopilotMode, {
      global: { stubs: copilotModeStubs }
    })
    expect(w.find('.copilot-messages').exists()).toBe(true)
  })

  it('renders credit bar footer', () => {
    const w = mount(CopilotMode, {
      global: { stubs: copilotModeStubs }
    })
    expect(w.find('.credit-footer').exists()).toBe(true)
  })

  it('renders CreditBar with variant="mini" wired to agentStore credits', async () => {
    const { useAgentStore } = await import('@/shared/stores/agent')
    const agentStore = useAgentStore()
    agentStore.updateCredits({ current: 55, max: 200 })
    const wrapper = mount(CopilotMode, {
      global: {
        stubs: {
          CopilotHeader: true,
          CopilotInput: true,
          MessageBubble: true,
          PlanCard: true,
        }
      }
    })
    const creditBar = wrapper.findComponent({ name: 'CreditBar' })
    expect(creditBar.exists()).toBe(true)
    expect(creditBar.props('variant')).toBe('mini')
    expect(creditBar.props('credits')).toBe(55)
    expect(creditBar.props('max')).toBe(200)
  })

  it('renders PlanCard when currentPlan is set', async () => {
    const { useConversationStore } = await import('@/shared/stores/conversation')
    const conversationStore = useConversationStore()
    conversationStore.appendMessage({ id: 'seed', role: 'user', content: 'go', timestamp: Date.now() })
    conversationStore.setPlan({
      steps: [
        { id: '1', label: 'Step one', state: 'pending', index: 0 },
        { id: '2', label: 'Step two', state: 'pending', index: 1 },
      ],
      approved: null,
    })
    const wrapper = mount(CopilotMode, {
      global: {
        stubs: {
          CopilotHeader: true,
          SessionBar: true,
          CopilotInput: true,
          CreditBar: true,
          MessageBubble: true,
        }
      }
    })
    const planCard = wrapper.findComponent({ name: 'PlanCard' })
    expect(planCard.exists()).toBe(true)
  })

  it('passes agentStore connectionStatus as statusDot to SessionBar', async () => {
    const { useAgentStore } = await import('@/shared/stores/agent')
    const agentStore = useAgentStore()
    agentStore.setConnectionStatus('connected')
    const wrapper = mount(CopilotMode, {
      global: {
        stubs: {
          CopilotHeader: true,
          CopilotInput: true,
          CreditBar: true,
          MessageBubble: true,
          PlanCard: true,
        }
      }
    })
    const sessionBar = wrapper.findComponent({ name: 'SessionBar' })
    expect(sessionBar.exists()).toBe(true)
    expect(sessionBar.props('statusDot')).toBe('connected')
  })
})

describe('CopilotHeader', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('renders traffic lights (3 dots)', () => {
    const w = mount(CopilotHeader)
    const dots = w.findAll('.traffic-dot')
    expect(dots).toHaveLength(3)
  })

  it('has close, minimize, maximize traffic dot classes', () => {
    const w = mount(CopilotHeader)
    expect(w.find('.traffic-dot.close').exists()).toBe(true)
    expect(w.find('.traffic-dot.minimize').exists()).toBe(true)
    expect(w.find('.traffic-dot.maximize').exists()).toBe(true)
  })

  it('renders title "Day1 Doctor"', () => {
    const w = mount(CopilotHeader)
    expect(w.text()).toContain('Day1 Doctor')
  })

  it('has .copilot-header root element', () => {
    const w = mount(CopilotHeader)
    expect(w.find('.copilot-header').exists()).toBe(true)
  })

  it('renders settings icon button', () => {
    const w = mount(CopilotHeader)
    const btns = w.findAll('.icon-btn')
    expect(btns.length).toBeGreaterThanOrEqual(1)
  })

  it('renders at least two icon buttons (settings and mode-switch)', () => {
    const w = mount(CopilotHeader)
    const btns = w.findAll('.icon-btn')
    expect(btns.length).toBeGreaterThanOrEqual(2)
  })

  it('has D1 logo element', () => {
    const w = mount(CopilotHeader)
    expect(w.find('.app-logo').exists()).toBe(true)
  })

  it('calls appWindow.close() when close dot is clicked', async () => {
    mockClose.mockClear()
    const w = mount(CopilotHeader)
    await w.find('.traffic-dot.close').trigger('click')
    expect(mockClose).toHaveBeenCalled()
  })

  it('calls appWindow.minimize() when minimize dot is clicked', async () => {
    mockMinimize.mockClear()
    const w = mount(CopilotHeader)
    await w.find('.traffic-dot.minimize').trigger('click')
    expect(mockMinimize).toHaveBeenCalled()
  })

  it('calls appWindow.toggleMaximize() when maximize dot is clicked', async () => {
    mockToggleMaximize.mockClear()
    const w = mount(CopilotHeader)
    await w.find('.traffic-dot.maximize').trigger('click')
    expect(mockToggleMaximize).toHaveBeenCalled()
  })
})

describe('SessionBar', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('renders session name from prop', () => {
    const w = mount(SessionBar, {
      props: {
        sessionName: 'My Test Session',
        statusDot: 'connected' as const,
        creditEstimate: '~2.4 credits',
      }
    })
    expect(w.text()).toContain('My Test Session')
  })

  it('renders the sessionName prop passed to SessionBar', () => {
    const w = mount(SessionBar, {
      props: {
        sessionName: 'Active Session',
        statusDot: 'connected' as const,
        creditEstimate: '~0 credits',
      }
    })
    expect(w.text()).toContain('Active Session')
  })

  it('renders credit estimate', () => {
    const w = mount(SessionBar, {
      props: {
        sessionName: 'Session',
        statusDot: 'connected' as const,
        creditEstimate: '~3.7 credits',
      }
    })
    expect(w.text()).toContain('~3.7 credits')
  })

  it('renders green status dot when prop = connected', () => {
    const w = mount(SessionBar, {
      props: {
        sessionName: 'Active Session',
        statusDot: 'connected' as const,
        creditEstimate: '~0 credits',
      }
    })
    const dot = w.find('.status-dot')
    expect(dot.exists()).toBe(true)
    // connected → data-status="connected" attribute for style binding verification
    expect(dot.attributes('data-status')).toBe('connected')
  })

  it('renders grey status dot when prop = disconnected', () => {
    const w = mount(SessionBar, {
      props: {
        sessionName: 'Active Session',
        statusDot: 'disconnected' as const,
        creditEstimate: '~0 credits',
      }
    })
    const dot = w.find('.status-dot')
    expect(dot.exists()).toBe(true)
    expect(dot.attributes('data-status')).toBe('disconnected')
  })

  it('renders connecting status dot when prop = connecting', () => {
    const w = mount(SessionBar, {
      props: {
        sessionName: 'Active Session',
        statusDot: 'connecting' as const,
        creditEstimate: '~0 credits',
      }
    })
    const dot = w.find('.status-dot')
    expect(dot.exists()).toBe(true)
    expect(dot.attributes('data-status')).toBe('connecting')
  })

  it('has .session-bar root class', () => {
    const w = mount(SessionBar, {
      props: {
        sessionName: 'Session',
        statusDot: 'connected' as const,
        creditEstimate: '~0 credits',
      }
    })
    expect(w.find('.session-bar').exists()).toBe(true)
  })
})

describe('CopilotInput', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('renders textarea', () => {
    const w = mount(CopilotInput)
    expect(w.find('textarea').exists()).toBe(true)
  })

  it('renders send button', () => {
    const w = mount(CopilotInput)
    expect(w.find('.send-btn').exists()).toBe(true)
  })

  it('has .input-pill container', () => {
    const w = mount(CopilotInput)
    expect(w.find('.input-pill').exists()).toBe(true)
  })

  it('submits on Enter key and appends message to store', async () => {
    const { useDaemonStore } = await import('@/shared/stores/daemon')
    useDaemonStore().setStatus('connected')
    const { useConversationStore } = await import('@/shared/stores/conversation')
    const store = useConversationStore()
    const initialCount = store.messages.length
    const w = mount(CopilotInput)
    const textarea = w.find('textarea')

    await textarea.setValue('Hello from Enter key test')
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })

    expect(store.messages.length).toBeGreaterThan(initialCount)
    // The textarea should be cleared after submit
    expect((textarea.element as HTMLTextAreaElement).value).toBe('')
  })

  it('does NOT submit on Shift+Enter', async () => {
    const w = mount(CopilotInput)
    const textarea = w.find('textarea')

    await textarea.setValue('Hello shift enter')
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: true })

    // Text should NOT be cleared — Shift+Enter inserts newline, doesn't submit
    expect((textarea.element as HTMLTextAreaElement).value).toBe('Hello shift enter')
  })

  it('clears textarea after submit via Enter', async () => {
    const { useDaemonStore } = await import('@/shared/stores/daemon')
    useDaemonStore().setStatus('connected')
    const w = mount(CopilotInput)
    const textarea = w.find('textarea')

    await textarea.setValue('Clear me after submit')
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })

    expect((textarea.element as HTMLTextAreaElement).value).toBe('')
  })

  it('clears textarea after clicking send button', async () => {
    const { useDaemonStore } = await import('@/shared/stores/daemon')
    useDaemonStore().setStatus('connected')
    const w = mount(CopilotInput)
    const textarea = w.find('textarea')
    const sendBtn = w.find('.send-btn')

    await textarea.setValue('Click send test')
    await sendBtn.trigger('click')

    expect((textarea.element as HTMLTextAreaElement).value).toBe('')
  })

  it('does not submit empty messages', async () => {
    const w = mount(CopilotInput)
    const { useConversationStore } = await import('@/shared/stores/conversation')
    const store = useConversationStore()

    const initialCount = store.messages.length
    const textarea = w.find('textarea')
    await textarea.setValue('')
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })

    expect(store.messages.length).toBe(initialCount)
  })

  it('appends message to conversation store on Enter submit', async () => {
    const { useDaemonStore } = await import('@/shared/stores/daemon')
    useDaemonStore().setStatus('connected')
    const w = mount(CopilotInput)
    const { useConversationStore } = await import('@/shared/stores/conversation')
    const store = useConversationStore()

    const initialCount = store.messages.length
    const textarea = w.find('textarea')
    await textarea.setValue('Test store message')
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })

    expect(store.messages.length).toBe(initialCount + 1)
    expect(store.messages[store.messages.length - 1].content).toBe('Test store message')
    expect(store.messages[store.messages.length - 1].role).toBe('user')
  })

  it('generates message IDs via crypto.randomUUID()', async () => {
    const { useDaemonStore } = await import('@/shared/stores/daemon')
    useDaemonStore().setStatus('connected')
    const uuidSpy = vi.spyOn(crypto, 'randomUUID')
    const w = mount(CopilotInput)
    const textarea = w.find('textarea')

    await textarea.setValue('UUID test message')
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })

    expect(uuidSpy).toHaveBeenCalled()
    uuidSpy.mockRestore()
  })

  it('calls submitTask when Enter is pressed with content and daemon is connected', async () => {
    const { useDaemonStore } = await import('@/shared/stores/daemon')
    const daemonStore = useDaemonStore()
    daemonStore.setStatus('connected')
    mockSubmitTask.mockClear()
    const w = mount(CopilotInput)
    const textarea = w.find('textarea')
    await textarea.setValue('Install Docker')
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })
    expect(mockSubmitTask).toHaveBeenCalledWith('Install Docker')
  })

  it('send button is disabled when daemon is disconnected', async () => {
    const { useDaemonStore } = await import('@/shared/stores/daemon')
    const daemonStore = useDaemonStore()
    daemonStore.setStatus('disconnected')
    const w = mount(CopilotInput)
    await w.find('textarea').setValue('Hello')
    expect((w.find('.send-btn').element as HTMLButtonElement).disabled).toBe(true)
  })
})
