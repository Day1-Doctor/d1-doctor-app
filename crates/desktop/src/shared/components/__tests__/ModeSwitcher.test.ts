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

import ModeSwitcher from '../ModeSwitcher.vue'
import { useAppStore } from '@/shared/stores/app'

describe('ModeSwitcher', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('renders three mode buttons', () => {
    const w = mount(ModeSwitcher)
    expect(w.findAll('.mode-btn')).toHaveLength(3)
  })

  it('labels are Full, Copilot, Ninja', () => {
    const w = mount(ModeSwitcher)
    const text = w.text()
    expect(text).toContain('Full')
    expect(text).toContain('Copilot')
    expect(text).toContain('Ninja')
  })

  it('Full button is active when uiMode is full', () => {
    const store = useAppStore()
    store.uiMode = 'full'
    const w = mount(ModeSwitcher)
    const fullBtn = w.findAll('.mode-btn').find(b => b.text().includes('Full'))
    expect(fullBtn!.classes()).toContain('active')
  })

  it('Copilot button is active when uiMode is copilot', () => {
    const store = useAppStore()
    store.uiMode = 'copilot'
    const w = mount(ModeSwitcher)
    const copilotBtn = w.findAll('.mode-btn').find(b => b.text().includes('Copilot'))
    expect(copilotBtn!.classes()).toContain('active')
  })

  it('clicking Copilot button calls switchMode("copilot")', async () => {
    const store = useAppStore()
    const spy = vi.spyOn(store, 'switchMode').mockResolvedValue()
    const w = mount(ModeSwitcher)
    const copilotBtn = w.findAll('.mode-btn').find(b => b.text().includes('Copilot'))
    await copilotBtn!.trigger('click')
    expect(spy).toHaveBeenCalledWith('copilot')
  })

  it('clicking Ninja button calls switchMode("ninja")', async () => {
    const store = useAppStore()
    const spy = vi.spyOn(store, 'switchMode').mockResolvedValue()
    const w = mount(ModeSwitcher)
    const ninjaBtn = w.findAll('.mode-btn').find(b => b.text().includes('Ninja'))
    await ninjaBtn!.trigger('click')
    expect(spy).toHaveBeenCalledWith('ninja')
  })

  it('clicking Full button calls switchMode("full")', async () => {
    const store = useAppStore()
    store.uiMode = 'copilot'
    const spy = vi.spyOn(store, 'switchMode').mockResolvedValue()
    const w = mount(ModeSwitcher)
    const fullBtn = w.findAll('.mode-btn').find(b => b.text().includes('Full'))
    await fullBtn!.trigger('click')
    expect(spy).toHaveBeenCalledWith('full')
  })

  it('has .mode-switcher root element', () => {
    const w = mount(ModeSwitcher)
    expect(w.find('.mode-switcher').exists()).toBe(true)
  })
})
