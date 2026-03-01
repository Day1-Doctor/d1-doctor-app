import { mount } from '@vue/test-utils'
import { describe, it, expect, vi } from 'vitest'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockResolvedValue('full') }))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn().mockResolvedValue(() => {}), emit: vi.fn() }))

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

import TitleBar from '../TitleBar.vue'

describe('TitleBar', () => {
  it('renders three traffic light dots', () => {
    const w = mount(TitleBar)
    const dots = w.findAll('.traffic-dot')
    expect(dots).toHaveLength(3)
  })

  it('traffic dot classes include close, minimize, maximize', () => {
    const w = mount(TitleBar)
    expect(w.find('.traffic-dot.close').exists()).toBe(true)
    expect(w.find('.traffic-dot.minimize').exists()).toBe(true)
    expect(w.find('.traffic-dot.maximize').exists()).toBe(true)
  })

  it('renders the window title text', () => {
    const w = mount(TitleBar)
    expect(w.text()).toContain('Day1 Doctor')
    expect(w.text()).toContain('Workspace')
  })

  it('has a .title-bar root element', () => {
    const w = mount(TitleBar)
    expect(w.find('.title-bar').exists()).toBe(true)
  })

  it('renders right-side icon buttons', () => {
    const w = mount(TitleBar)
    expect(w.find('.title-actions').exists()).toBe(true)
    const btns = w.findAll('.icon-btn')
    expect(btns.length).toBeGreaterThanOrEqual(2)
  })

  it('traffic lights container has .traffic-lights class', () => {
    const w = mount(TitleBar)
    expect(w.find('.traffic-lights').exists()).toBe(true)
  })

  it('clicking the close dot calls appWindow.close()', async () => {
    mockClose.mockClear()
    const w = mount(TitleBar)
    await w.find('.traffic-dot.close').trigger('click')
    expect(mockClose).toHaveBeenCalled()
  })

  it('clicking the minimize dot calls appWindow.minimize()', async () => {
    mockMinimize.mockClear()
    const w = mount(TitleBar)
    await w.find('.traffic-dot.minimize').trigger('click')
    expect(mockMinimize).toHaveBeenCalled()
  })

  it('clicking the maximize dot calls appWindow.toggleMaximize()', async () => {
    mockToggleMaximize.mockClear()
    const w = mount(TitleBar)
    await w.find('.traffic-dot.maximize').trigger('click')
    expect(mockToggleMaximize).toHaveBeenCalled()
  })
})
