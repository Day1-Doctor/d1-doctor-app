import { mount } from '@vue/test-utils'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockResolvedValue('full') }))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn().mockResolvedValue(() => {}), emit: vi.fn() }))

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
    expect(w.text()).toContain('Recent Tasks')
  })

  it('renders three stub task items', () => {
    const w = mount(Sidebar)
    const taskItems = w.findAll('.task-item')
    expect(taskItems).toHaveLength(3)
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
})
