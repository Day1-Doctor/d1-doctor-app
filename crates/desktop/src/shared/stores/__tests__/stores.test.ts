import { setActivePinia, createPinia } from 'pinia'
import { describe, it, expect, beforeEach, vi } from 'vitest'

// Mock Tauri invoke — stores use it but it won't be available in jsdom
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue('full'),
}))

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    hide: vi.fn().mockResolvedValue(undefined),
    show: vi.fn().mockResolvedValue(undefined),
    setFocus: vi.fn().mockResolvedValue(undefined),
  })),
}))

vi.mock('@tauri-apps/api/webviewWindow', () => ({
  WebviewWindow: {
    getByLabel: vi.fn().mockResolvedValue({
      hide: vi.fn().mockResolvedValue(undefined),
      show: vi.fn().mockResolvedValue(undefined),
    }),
  },
}))

describe('ConversationStore', () => {
  beforeEach(() => { setActivePinia(createPinia()) })

  it('starts with empty messages', async () => {
    const { useConversationStore } = await import('../conversation')
    const store = useConversationStore()
    expect(store.messages).toHaveLength(0)
  })

  it('appendMessage adds to messages array', async () => {
    const { useConversationStore } = await import('../conversation')
    const store = useConversationStore()
    store.appendMessage({ id: '1', role: 'user', content: 'Hello', timestamp: 1000 })
    expect(store.messages).toHaveLength(1)
    expect(store.messages[0].content).toBe('Hello')
  })

  it('setPlan sets currentPlan', async () => {
    const { useConversationStore } = await import('../conversation')
    const store = useConversationStore()
    store.setPlan({ steps: [{ id: 's1', label: 'Step 1', state: 'pending', index: 0 }], approved: null })
    expect(store.currentPlan).not.toBeNull()
    expect(store.currentPlan!.steps).toHaveLength(1)
  })

  it('updateStep changes step state', async () => {
    const { useConversationStore } = await import('../conversation')
    const store = useConversationStore()
    store.setPlan({ steps: [{ id: 's1', label: 'Step 1', state: 'pending', index: 0 }], approved: null })
    store.updateStep('s1', 'active')
    expect(store.currentPlan!.steps[0].state).toBe('active')
  })

  it('updateStep does nothing for unknown stepId', async () => {
    const { useConversationStore } = await import('../conversation')
    const store = useConversationStore()
    store.setPlan({ steps: [{ id: 's1', label: 'Step 1', state: 'pending', index: 0 }], approved: null })
    store.updateStep('nonexistent', 'done')
    expect(store.currentPlan!.steps[0].state).toBe('pending')
  })

  it('scrollPinned starts as true', async () => {
    const { useConversationStore } = await import('../conversation')
    const store = useConversationStore()
    expect(store.scrollPinned).toBe(true)
  })
})

describe('AgentStore', () => {
  beforeEach(() => { setActivePinia(createPinia()) })

  it('starts with 0 credits', async () => {
    const { useAgentStore } = await import('../agent')
    const store = useAgentStore()
    expect(store.credits.current).toBe(0)
  })

  it('updateCredits updates current and max', async () => {
    const { useAgentStore } = await import('../agent')
    const store = useAgentStore()
    store.updateCredits({ current: 75, max: 200 })
    expect(store.credits.current).toBe(75)
    expect(store.credits.max).toBe(200)
  })

  it('starts disconnected', async () => {
    const { useAgentStore } = await import('../agent')
    const store = useAgentStore()
    expect(store.connectionStatus).toBe('disconnected')
  })
})

describe('AppStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('starts with full mode', async () => {
    const { useAppStore } = await import('../app')
    const store = useAppStore()
    expect(store.uiMode).toBe('full')
  })

  it('switchMode updates uiMode and calls invoke', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue(undefined)
    const { useAppStore } = await import('../app')
    const store = useAppStore()
    await store.switchMode('copilot')
    expect(store.uiMode).toBe('copilot')
    expect(mockInvoke).toHaveBeenCalledWith('set_config', { key: 'ui_mode', value: 'copilot' })
  })
})
