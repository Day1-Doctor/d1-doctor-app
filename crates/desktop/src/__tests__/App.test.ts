import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick } from 'vue'

// ── Hoisted mocks — vi.mock factories are hoisted to top of file ───────────

const { mockInvoke, mockListen, mockNinjaListeners, mockNinjaWindow } = vi.hoisted(() => {
  const mockNinjaListeners: Array<() => void> = []
  const mockInvoke = vi.fn().mockResolvedValue(undefined)
  const mockListen = vi.fn(async (event: string, handler: () => void) => {
    if (event === 'ninja_dismissed') {
      mockNinjaListeners.push(handler)
    }
    return () => {}
  })
  const mockNinjaWindow = { hide: vi.fn().mockResolvedValue(undefined) }
  return { mockInvoke, mockListen, mockNinjaListeners, mockNinjaWindow }
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
    getByLabel: vi.fn().mockResolvedValue(mockNinjaWindow),
  },
}))

// Stub child mode components to avoid their internal dependencies
vi.mock('@/modes/full/FullMode.vue', () => ({
  default: { name: 'FullMode', template: '<div class="full-mode-stub" />' },
}))
vi.mock('@/modes/copilot/CopilotMode.vue', () => ({
  default: { name: 'CopilotMode', template: '<div class="copilot-mode-stub" />' },
}))
vi.mock('@/shared/components/LoginScreen.vue', () => ({
  default: { name: 'LoginScreen', template: '<div class="login-screen-stub" />' },
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
    mockNinjaListeners.length = 0
    mockInvoke.mockClear()
    mockListen.mockClear()
    mockNinjaWindow.hide.mockClear()
    Object.defineProperty(window.screen, 'availWidth', { value: 1920, configurable: true })
  })

  it('renders FullMode when uiMode is "full" and authenticated', async () => {
    const { useAppStore } = await import('@/shared/stores/app')
    const { useAuthStore } = await import('@/shared/stores/auth')
    const pinia = createPinia()
    setActivePinia(pinia)
    const appStore = useAppStore()
    const authStore = useAuthStore()
    appStore.uiMode = 'full'
    authStore.status = 'authenticated'

    const wrapper = mount(App, { global: { plugins: [pinia] } })
    await nextTick()

    expect(wrapper.find('.full-mode-stub').exists()).toBe(true)
    expect(wrapper.find('.copilot-mode-stub').exists()).toBe(false)
    expect(wrapper.find('.login-screen-stub').exists()).toBe(false)
  })

  it('renders CopilotMode when uiMode is "copilot" and authenticated', async () => {
    const { useAppStore } = await import('@/shared/stores/app')
    const { useAuthStore } = await import('@/shared/stores/auth')
    const pinia = createPinia()
    setActivePinia(pinia)
    const appStore = useAppStore()
    const authStore = useAuthStore()
    appStore.uiMode = 'copilot'
    authStore.status = 'authenticated'

    const wrapper = mount(App, { global: { plugins: [pinia] } })
    await nextTick()

    expect(wrapper.find('.copilot-mode-stub').exists()).toBe(true)
    expect(wrapper.find('.full-mode-stub').exists()).toBe(false)
    expect(wrapper.find('.login-screen-stub').exists()).toBe(false)
  })

  it('renders LoginScreen when unauthenticated', async () => {
    const { useAuthStore } = await import('@/shared/stores/auth')
    const pinia = createPinia()
    setActivePinia(pinia)
    const authStore = useAuthStore()
    authStore.status = 'unauthenticated'

    const wrapper = mount(App, { global: { plugins: [pinia] } })
    await nextTick()

    expect(wrapper.find('.login-screen-stub').exists()).toBe(true)
    expect(wrapper.find('.full-mode-stub').exists()).toBe(false)
    expect(wrapper.find('.copilot-mode-stub').exists()).toBe(false)
  })

  it('listens for "ninja_dismissed" and calls switchMode with previousMode', async () => {
    const { useAppStore } = await import('@/shared/stores/app')
    const pinia = createPinia()
    setActivePinia(pinia)
    const appStore = useAppStore()
    appStore.uiMode = 'ninja'
    appStore.previousMode = 'full'

    mount(App, { global: { plugins: [pinia] } })
    await flushPromises()

    expect(mockListen).toHaveBeenCalledWith('ninja_dismissed', expect.any(Function))
    expect(mockNinjaListeners).toHaveLength(1)

    const switchModeSpy = vi.spyOn(appStore, 'switchMode').mockResolvedValue(undefined)
    mockNinjaListeners[0]()
    await nextTick()

    expect(switchModeSpy).toHaveBeenCalledWith('full')
  })

  it('hides ninja-bar window on startup regardless of saved mode', async () => {
    mockInvoke.mockResolvedValueOnce('ninja')
    mount(App, { global: { stubs: { FullMode: true, CopilotMode: true, LoginScreen: true } } })
    await flushPromises()
    expect(mockNinjaWindow.hide).toHaveBeenCalled()
  })
})
