import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock the Tauri updater plugin
const mockCheck = vi.fn()
const mockRelaunch = vi.fn()

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: (...args: unknown[]) => mockCheck(...args),
}))

vi.mock('@tauri-apps/plugin-process', () => ({
  relaunch: (...args: unknown[]) => mockRelaunch(...args),
}))

// Must import after mocks are set up
import { useAutoUpdater } from '../useAutoUpdater'

describe('useAutoUpdater', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.useFakeTimers()
  })

  it('exports expected API shape', () => {
    const api = useAutoUpdater()
    expect(api).toHaveProperty('updateReady')
    expect(api).toHaveProperty('updateVersion')
    expect(api).toHaveProperty('bannerDismissed')
    expect(api).toHaveProperty('restartNow')
    expect(api).toHaveProperty('dismissBanner')
  })

  it('initial state: no update ready, banner not dismissed', () => {
    const api = useAutoUpdater()
    expect(api.updateReady.value).toBe(false)
    expect(api.updateVersion.value).toBe('')
    expect(api.bannerDismissed.value).toBe(false)
  })

  it('dismissBanner sets bannerDismissed to true', () => {
    const api = useAutoUpdater()
    api.dismissBanner()
    expect(api.bannerDismissed.value).toBe(true)
  })

  it('restartNow calls relaunch', async () => {
    mockRelaunch.mockResolvedValue(undefined)
    const api = useAutoUpdater()
    await api.restartNow()
    expect(mockRelaunch).toHaveBeenCalledOnce()
  })

  it('restartNow handles relaunch failure gracefully', async () => {
    mockRelaunch.mockRejectedValue(new Error('relaunch failed'))
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    const api = useAutoUpdater()
    await api.restartNow()
    expect(consoleSpy).toHaveBeenCalledWith(
      '[useAutoUpdater] Relaunch failed:',
      expect.any(Error),
    )
    consoleSpy.mockRestore()
  })
})
