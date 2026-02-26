import { mount } from '@vue/test-utils'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockResolvedValue('full') }))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn().mockResolvedValue(() => {}), emit: vi.fn() }))

import UtilityPanel from '../UtilityPanel.vue'
import { useAgentStore } from '@/shared/stores/agent'

describe('UtilityPanel', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('renders the utility-panel element', () => {
    const w = mount(UtilityPanel)
    expect(w.find('.utility-panel').exists()).toBe(true)
  })

  it('renders 5 panel sections', () => {
    const w = mount(UtilityPanel)
    expect(w.findAll('.panel-section')).toHaveLength(5)
  })

  it('renders Task Info section header', () => {
    const w = mount(UtilityPanel)
    expect(w.text()).toContain('Task Info')
  })

  it('renders Agents section header', () => {
    const w = mount(UtilityPanel)
    expect(w.text()).toContain('Agents')
  })

  it('renders Permissions section header', () => {
    const w = mount(UtilityPanel)
    expect(w.text()).toContain('Permissions')
  })

  it('renders System Health section header', () => {
    const w = mount(UtilityPanel)
    expect(w.text()).toContain('System Health')
  })

  it('renders Connection section header', () => {
    const w = mount(UtilityPanel)
    expect(w.text()).toContain('Connection')
  })

  it('all sections are expanded by default', () => {
    const w = mount(UtilityPanel)
    const bodies = w.findAll('.section-body')
    expect(bodies).toHaveLength(5)
  })

  it('clicking a section header collapses that section', async () => {
    const w = mount(UtilityPanel)
    const headers = w.findAll('.section-header')
    // Collapse first section (Task Info)
    await headers[0].trigger('click')
    // Now only 4 section-bodies should be visible
    const bodies = w.findAll('.section-body')
    expect(bodies).toHaveLength(4)
  })

  it('clicking a collapsed section header re-expands it', async () => {
    const w = mount(UtilityPanel)
    const headers = w.findAll('.section-header')
    // Collapse
    await headers[0].trigger('click')
    expect(w.findAll('.section-body')).toHaveLength(4)
    // Re-expand
    await headers[0].trigger('click')
    expect(w.findAll('.section-body')).toHaveLength(5)
  })

  it('shows OS: macOS in System Health', () => {
    const w = mount(UtilityPanel)
    expect(w.text()).toContain('macOS')
  })

  it('shows Daemon status in System Health', () => {
    const w = mount(UtilityPanel)
    expect(w.text()).toContain('Daemon')
    expect(w.text()).toContain('Running')
  })

  it('shows backend and gateway in Connection section', () => {
    const w = mount(UtilityPanel)
    expect(w.text()).toContain('Backend')
    expect(w.text()).toContain('Gateway')
  })

  it('shows no active agents message when activeAgents is empty', () => {
    const w = mount(UtilityPanel)
    expect(w.text()).toContain('No active agents')
  })

  it('renders AgentAvatar for each active agent', () => {
    const store = useAgentStore()
    store.setActiveAgents(['planner', 'executor'])
    const w = mount(UtilityPanel)
    const avatars = w.findAll('.agent-avatar')
    expect(avatars).toHaveLength(2)
  })

  it('shows connection status from agent store', () => {
    const store = useAgentStore()
    store.setConnectionStatus('connected')
    const w = mount(UtilityPanel)
    expect(w.text()).toContain('Connected')
  })

  it('chevron rotates when section is collapsed', async () => {
    const w = mount(UtilityPanel)
    const headers = w.findAll('.section-header')
    // Initially not collapsed
    const chevron = headers[0].find('.chevron')
    expect(chevron.classes()).not.toContain('collapsed')
    // Collapse
    await headers[0].trigger('click')
    expect(chevron.classes()).toContain('collapsed')
  })

  it('Permissions section shows empty state by default', () => {
    const w = mount(UtilityPanel)
    expect(w.text()).toContain('No permissions requested')
  })

  it('registers setInterval for health refresh on mount', () => {
    vi.useFakeTimers()
    const setIntervalSpy = vi.spyOn(globalThis, 'setInterval')
    mount(UtilityPanel)
    expect(setIntervalSpy).toHaveBeenCalledWith(expect.any(Function), 5000)
    vi.useRealTimers()
    setIntervalSpy.mockRestore()
  })

  it('clears health refresh interval on unmount', () => {
    vi.useFakeTimers()
    const clearIntervalSpy = vi.spyOn(globalThis, 'clearInterval')
    const w = mount(UtilityPanel)
    w.unmount()
    expect(clearIntervalSpy).toHaveBeenCalled()
    vi.useRealTimers()
    clearIntervalSpy.mockRestore()
  })
})
