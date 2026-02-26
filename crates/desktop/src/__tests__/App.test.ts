import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick } from 'vue'

// ── Hoisted mocks — vi.mock factories are hoisted to top of file ───────────

const { mockInvoke, mockListen, mockNinjaListeners } = vi.hoisted(() => {
  const mockNinjaListeners: Array<() => void> = []
  const mockInvoke = vi.fn().mockResolvedValue(undefined)
  const mockListen = vi.fn(async (event: string, handler: () => void) => {
    if (event === 'ninja_dismissed') {
      mockNinjaListeners.push(handler)
    }
    return () => {}
  })
  return { mockInvoke, mockListen, mockNinjaListeners }
})

vi.mock('@tauri-apps/api/core', () => ({ invoke: mockInvoke }))
vi.mock('@tauri-apps/api/event', () => ({
  listen: mockListen,
  emit: vi.fn(),
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

// Stub child mode components to avoid their internal dependencies
vi.mock('@/modes/full/FullMode.vue', () => ({
  default: { name: 'FullMode', template: '<div class="full-mode-stub" />' },
}))
vi.mock('@/modes/copilot/CopilotMode.vue', () => ({
  default: { name: 'CopilotMode', template: '<div class="copilot-mode-stub" />' },
}))

// Stub useAgentEvents to avoid real Tauri event registration
vi.mock('@/shared/composables/useAgentEvents', () => ({
  useAgentEvents: vi.fn(() => ({
    startListening: vi.fn(),
    stopListening: vi.fn(),
    onEvent: vi.fn(),
  })),
}))

import App from '../App.vue'

// ── Tests ──────────────────────────────────────────────────────────────────

describe('App.vue', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    // Clear call counts without resetting implementations
    mockNinjaListeners.length = 0
    mockInvoke.mockClear()
    mockListen.mockClear()
    // Set screen dimensions
    Object.defineProperty(window.screen, 'availWidth', { value: 1920, configurable: true })
  })

  it('renders FullMode when uiMode is "full"', async () => {
    const { useAppStore } = await import('@/shared/stores/app')
    const pinia = createPinia()
    setActivePinia(pinia)
    const appStore = useAppStore()
    appStore.uiMode = 'full'

    const wrapper = mount(App, {
      global: { plugins: [pinia] },
    })
    await nextTick()

    expect(wrapper.find('.full-mode-stub').exists()).toBe(true)
    expect(wrapper.find('.copilot-mode-stub').exists()).toBe(false)
  })

  it('renders CopilotMode when uiMode is "copilot"', async () => {
    const { useAppStore } = await import('@/shared/stores/app')
    const pinia = createPinia()
    setActivePinia(pinia)
    const appStore = useAppStore()
    appStore.uiMode = 'copilot'

    const wrapper = mount(App, {
      global: { plugins: [pinia] },
    })
    await nextTick()

    expect(wrapper.find('.copilot-mode-stub').exists()).toBe(true)
    expect(wrapper.find('.full-mode-stub').exists()).toBe(false)
  })

  it('listens for "ninja_dismissed" and calls switchMode with previousMode', async () => {
    const { useAppStore } = await import('@/shared/stores/app')
    const pinia = createPinia()
    setActivePinia(pinia)
    const appStore = useAppStore()
    // Set up pre-condition: simulate being in ninja mode with 'full' as previous
    appStore.uiMode = 'ninja'
    appStore.previousMode = 'full'

    mount(App, {
      global: { plugins: [pinia] },
    })

    // Flush all pending promises (onMounted is async)
    await flushPromises()

    // Verify that listen was called for ninja_dismissed
    expect(mockListen).toHaveBeenCalledWith('ninja_dismissed', expect.any(Function))

    // Simulate the ninja_dismissed event firing
    expect(mockNinjaListeners).toHaveLength(1)

    // Capture what switchMode does by checking if it's called
    const switchModeSpy = vi.spyOn(appStore, 'switchMode').mockResolvedValue(undefined)

    mockNinjaListeners[0]()
    await nextTick()

    // switchMode should have been called with the previousMode value ('full')
    expect(switchModeSpy).toHaveBeenCalledWith('full')
  })
})
