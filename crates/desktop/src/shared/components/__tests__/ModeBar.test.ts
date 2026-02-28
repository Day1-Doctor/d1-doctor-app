import { mount } from '@vue/test-utils'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockResolvedValue(undefined) }))
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    hide: vi.fn().mockResolvedValue(undefined),
    show: vi.fn().mockResolvedValue(undefined),
    setFocus: vi.fn().mockResolvedValue(undefined),
  })),
}))
vi.mock('@tauri-apps/api/webviewWindow', () => ({
  WebviewWindow: { getByLabel: vi.fn().mockResolvedValue(null) },
}))

import ModeBar from '../ModeBar.vue'
import { useAppStore } from '@/shared/stores/app'

describe('ModeBar', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('renders three mode buttons', () => {
    const w = mount(ModeBar)
    expect(w.findAll('.mode-bar-btn')).toHaveLength(3)
  })

  it('active button matches current uiMode', () => {
    const store = useAppStore()
    store.uiMode = 'copilot'
    const w = mount(ModeBar)
    const activeBtn = w.find('.mode-bar-btn.active')
    expect(activeBtn.attributes('title')).toBe('Copilot mode')
  })

  it('clicking a button calls switchMode', async () => {
    const store = useAppStore()
    const spy = vi.spyOn(store, 'switchMode').mockResolvedValue()
    const w = mount(ModeBar)
    await w.findAll('.mode-bar-btn')[0].trigger('click')
    expect(spy).toHaveBeenCalledWith('full')
  })
})
