import { mount } from '@vue/test-utils'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick } from 'vue'
import { flushPromises } from '@vue/test-utils'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === 'list_recent_tasks') return Promise.resolve([
      { id: 'a', title: 'Fix login bug', status: 'completed', created_at: 1700000000 },
      { id: 'b', title: 'Setup Docker', status: 'failed', created_at: 1700000100 },
    ])
    return Promise.resolve('full')  // default for get_config etc.
  })
}))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn().mockResolvedValue(() => {}), emit: vi.fn() }))
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

import Sidebar from '../Sidebar.vue'
import { useAgentStore } from '@/shared/stores/agent'

describe('Sidebar', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('renders the sidebar element', () => {
    const w = mount(Sidebar)
    expect(w.find('.sidebar').exists()).toBe(true)
  })

  it('renders logo circle with D1 text', () => {
    const w = mount(Sidebar)
    expect(w.find('.logo-circle').exists()).toBe(true)
    expect(w.find('.logo-circle').text()).toBe('D1')
  })

  it('renders app name and version badge', () => {
    const w = mount(Sidebar)
    expect(w.text()).toContain('Day 1 Doctor')
    expect(w.text()).toContain('v2.4.0')
  })

  it('renders all four nav items', () => {
    const w = mount(Sidebar)
    const navItems = w.findAll('.nav-item')
    expect(navItems).toHaveLength(4)
  })

  it('Chat nav item is active by default', () => {
    const w = mount(Sidebar)
    const chatItem = w.findAll('.nav-item').find(item => item.text().includes('Chat'))
    expect(chatItem).toBeDefined()
    expect(chatItem!.classes()).toContain('active')
  })

  it('contains Tasks, Knowledge, Settings nav items', () => {
    const w = mount(Sidebar)
    const text = w.text()
    expect(text).toContain('Tasks')
    expect(text).toContain('Knowledge')
    expect(text).toContain('Settings')
  })

  it('renders Recent Tasks section', () => {
    const w = mount(Sidebar)
    expect(w.text()).toContain('RECENT')
  })

  it('renders CreditBar component', () => {
    const w = mount(Sidebar)
    expect(w.find('.credit-bar').exists()).toBe(true)
  })

  it('renders user section with email', () => {
    const w = mount(Sidebar)
    expect(w.find('.sidebar-user').exists()).toBe(true)
    expect(w.text()).toContain('user@example.com')
  })

  it('clicking a non-active nav item sets it as active', async () => {
    const w = mount(Sidebar)
    const navItems = w.findAll('.nav-item')
    const tasksItem = navItems.find(item => item.text().includes('Tasks'))
    expect(tasksItem).toBeDefined()
    await tasksItem!.trigger('click')
    expect(tasksItem!.classes()).toContain('active')
    // Chat should no longer be active
    const chatItem = navItems.find(item => item.text().includes('Chat'))
    expect(chatItem!.classes()).not.toContain('active')
  })

  it('passes variant="full" to CreditBar', () => {
    const w = mount(Sidebar)
    const creditBar = w.findComponent({ name: 'CreditBar' })
    expect(creditBar.exists()).toBe(true)
    expect(creditBar.props('variant')).toBe('full')
  })

  it('passes agentStore credits to CreditBar', () => {
    const agentStore = useAgentStore()
    agentStore.updateCredits({ current: 42, max: 100 })
    const w = mount(Sidebar)
    const creditBar = w.findComponent({ name: 'CreditBar' })
    expect(creditBar.props('credits')).toBe(42)
    expect(creditBar.props('max')).toBe(100)
  })

  it('renders ModeSwitcher component', () => {
    const w = mount(Sidebar)
    expect(w.findComponent({ name: 'ModeSwitcher' }).exists()).toBe(true)
  })

  it('renders ConnectionStatus component', () => {
    const w = mount(Sidebar)
    expect(w.findComponent({ name: 'ConnectionStatus' }).exists()).toBe(true)
  })

  it('loads tasks from list_recent_tasks invoke on mount', async () => {
    const w = mount(Sidebar)
    await flushPromises()
    const taskItems = w.findAll('.task-item')
    expect(taskItems.length).toBeGreaterThanOrEqual(2)
  })

  it('renders task titles from invoked task list', async () => {
    const w = mount(Sidebar)
    await flushPromises()
    expect(w.text()).toContain('Fix login bug')
    expect(w.text()).toContain('Setup Docker')
  })

  it('shows "No recent tasks" when task list is empty', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    vi.mocked(invoke).mockResolvedValueOnce([])
    const w = mount(Sidebar)
    await flushPromises()
    expect(w.text()).toContain('No recent tasks')
  })
})
