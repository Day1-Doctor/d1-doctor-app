import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'

// ── Mock all Tauri APIs (hoisted so factories can reference them) ──────────

const { mockInvoke, mockMainWindow, mockNinjaWindow } = vi.hoisted(() => {
  const mockMainWindow = {
    hide: vi.fn().mockResolvedValue(undefined),
    show: vi.fn().mockResolvedValue(undefined),
    setFocus: vi.fn().mockResolvedValue(undefined),
  }
  const mockNinjaWindow = {
    hide: vi.fn().mockResolvedValue(undefined),
    show: vi.fn().mockResolvedValue(undefined),
  }
  const mockInvoke = vi.fn().mockResolvedValue(undefined)
  return { mockInvoke, mockMainWindow, mockNinjaWindow }
})

vi.mock('@tauri-apps/api/core', () => ({ invoke: mockInvoke }))
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => mockMainWindow),
}))
vi.mock('@tauri-apps/api/webviewWindow', () => ({
  WebviewWindow: {
    getByLabel: vi.fn().mockResolvedValue(mockNinjaWindow),
  },
}))

// ── Tests ──────────────────────────────────────────────────────────────────

describe('useAppStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    // Clear call counts without resetting implementations
    mockInvoke.mockClear()
    mockMainWindow.hide.mockClear()
    mockMainWindow.show.mockClear()
    mockMainWindow.setFocus.mockClear()
    mockNinjaWindow.hide.mockClear()
    mockNinjaWindow.show.mockClear()
    // Default screen dimensions for jsdom
    Object.defineProperty(window.screen, 'availWidth', { value: 1920, configurable: true })
    Object.defineProperty(window.screen, 'availHeight', { value: 1080, configurable: true })
  })

  // Test 1: switchMode('full') calls resize_window with 1200×760
  it('switchMode("full") calls invoke("resize_window", { width: 1200, height: 760 })', async () => {
    const { useAppStore } = await import('../app')
    const store = useAppStore()
    await store.switchMode('full')
    expect(mockInvoke).toHaveBeenCalledWith('resize_window', { width: 1200, height: 760 })
  })

  // Test 2: switchMode('copilot') calls resize_window AND position_window
  it('switchMode("copilot") calls invoke("resize_window", { width: 420, height: 720 }) and invoke("position_window", ...)', async () => {
    const { useAppStore } = await import('../app')
    const store = useAppStore()
    await store.switchMode('copilot')
    expect(mockInvoke).toHaveBeenCalledWith('resize_window', { width: 420, height: 720 })
    expect(mockInvoke).toHaveBeenCalledWith('position_window', expect.objectContaining({
      x: expect.any(Number),
      y: 12,
    }))
  })

  // Test 3: switchMode('ninja') saves previousMode, hides main, shows ninja
  it('switchMode("ninja") saves previousMode, calls mainWindow.hide() and ninjaWindow.show()', async () => {
    const { useAppStore } = await import('../app')
    const store = useAppStore()
    // Start in full mode
    expect(store.uiMode).toBe('full')
    await store.switchMode('ninja')
    expect(store.previousMode).toBe('full')
    expect(store.uiMode).toBe('ninja')
    expect(mockMainWindow.hide).toHaveBeenCalled()
    expect(mockNinjaWindow.show).toHaveBeenCalled()
  })

  // Test 4: switchMode('full') after switchMode('ninja') hides ninja, shows main, resizes
  it('switchMode("full") after ninja: calls ninjaWindow.hide(), mainWindow.show(), and resize', async () => {
    const { useAppStore } = await import('../app')
    const store = useAppStore()
    await store.switchMode('ninja')
    // Clear counts to isolate next switch
    mockMainWindow.hide.mockClear()
    mockMainWindow.show.mockClear()
    mockMainWindow.setFocus.mockClear()
    mockNinjaWindow.hide.mockClear()
    mockNinjaWindow.show.mockClear()
    mockInvoke.mockClear()

    await store.switchMode('full')
    expect(mockNinjaWindow.hide).toHaveBeenCalled()
    expect(mockMainWindow.show).toHaveBeenCalled()
    expect(mockInvoke).toHaveBeenCalledWith('resize_window', { width: 1200, height: 760 })
  })

  // Test 5: init() with saved 'copilot' config sets uiMode to 'copilot'
  it('init() with saved "copilot" config sets uiMode to "copilot"', async () => {
    mockInvoke.mockResolvedValueOnce('copilot')
    const { useAppStore } = await import('../app')
    const store = useAppStore()
    await store.init()
    expect(store.uiMode).toBe('copilot')
  })

  // Test 6: init() with saved 'ninja' config defaults to 'full'
  it('init() with saved "ninja" config defaults uiMode to "full"', async () => {
    mockInvoke.mockResolvedValueOnce('ninja')
    const { useAppStore } = await import('../app')
    const store = useAppStore()
    await store.init()
    expect(store.uiMode).toBe('full')
  })

  // Test 7: init() with missing config (invoke throws) stays 'full'
  it('init() with missing config (invoke throws) stays "full"', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('config not found'))
    const { useAppStore } = await import('../app')
    const store = useAppStore()
    await store.init()
    expect(store.uiMode).toBe('full')
  })

  // Test 8: switchMode() calls set_config for persistence
  it('switchMode() calls invoke("set_config", { key: "ui_mode", value: mode }) for persistence', async () => {
    const { useAppStore } = await import('../app')
    const store = useAppStore()
    await store.switchMode('copilot')
    expect(mockInvoke).toHaveBeenCalledWith('set_config', { key: 'ui_mode', value: 'copilot' })
  })

  // Additional: verify conversation store is NOT cleared during mode switch
  it('switchMode() does not clear conversationStore (conversation state preserved)', async () => {
    const { useConversationStore } = await import('../conversation')
    const { useAppStore } = await import('../app')
    const convStore = useConversationStore()
    const appStore = useAppStore()
    // Add a message
    convStore.appendMessage({ id: '1', role: 'user', content: 'Hello', timestamp: 1000 })
    expect(convStore.messages).toHaveLength(1)
    // Switch modes
    await appStore.switchMode('copilot')
    await appStore.switchMode('full')
    // Message should still be there
    expect(convStore.messages).toHaveLength(1)
    expect(convStore.messages[0].content).toBe('Hello')
  })
})
