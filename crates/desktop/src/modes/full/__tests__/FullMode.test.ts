import { mount } from '@vue/test-utils'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useDaemonStore } from '@/shared/stores/daemon'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockResolvedValue('full') }))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn().mockResolvedValue(() => {}), emit: vi.fn() }))
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    close: vi.fn().mockResolvedValue(undefined),
    minimize: vi.fn().mockResolvedValue(undefined),
    toggleMaximize: vi.fn().mockResolvedValue(undefined),
  }))
}))

import FullMode from '../FullMode.vue'
import TitleBar from '../TitleBar.vue'
import Sidebar from '../Sidebar.vue'
import ChatWorkspace from '../ChatWorkspace.vue'
import UtilityPanel from '../UtilityPanel.vue'

describe('FullMode', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('renders the app-window root element', () => {
    const w = mount(FullMode)
    expect(w.find('.app-window').exists()).toBe(true)
  })

  it('renders the main-content flex container', () => {
    const w = mount(FullMode)
    expect(w.find('.main-content').exists()).toBe(true)
  })

  it('renders TitleBar component', () => {
    const w = mount(FullMode)
    // TitleBar renders .title-bar
    expect(w.find('.title-bar').exists()).toBe(true)
  })

  it('renders Sidebar component', () => {
    const w = mount(FullMode)
    // Sidebar renders .sidebar (aside)
    expect(w.find('.sidebar').exists()).toBe(true)
  })

  it('renders ChatWorkspace component', () => {
    const w = mount(FullMode)
    // ChatWorkspace renders .chat-workspace
    expect(w.find('.chat-workspace').exists()).toBe(true)
  })

  it('renders UtilityPanel component', () => {
    const w = mount(FullMode)
    // UtilityPanel renders .utility-panel (aside)
    expect(w.find('.utility-panel').exists()).toBe(true)
  })

  it('has three panes inside main-content', () => {
    const w = mount(FullMode)
    const mainContent = w.find('.main-content')
    // sidebar + chat-workspace + utility-panel
    expect(mainContent.find('.sidebar').exists()).toBe(true)
    expect(mainContent.find('.chat-workspace').exists()).toBe(true)
    expect(mainContent.find('.utility-panel').exists()).toBe(true)
  })

  it('TitleBar is outside main-content (direct child of app-window)', () => {
    const w = mount(FullMode)
    const appWindow = w.find('.app-window')
    // TitleBar (.title-bar) should be a direct child of app-window, not inside main-content
    const mainContent = w.find('.main-content')
    expect(mainContent.find('.title-bar').exists()).toBe(false)
    expect(appWindow.find('.title-bar').exists()).toBe(true)
  })

  it('imports and uses TitleBar', () => {
    // Component identity check: FullMode should use TitleBar
    // We verify by checking TitleBar renders its characteristic class
    const w = mount(FullMode)
    expect(w.findComponent(TitleBar).exists()).toBe(true)
  })

  it('imports and uses Sidebar', () => {
    const w = mount(FullMode)
    expect(w.findComponent(Sidebar).exists()).toBe(true)
  })

  it('imports and uses ChatWorkspace', () => {
    const w = mount(FullMode)
    expect(w.findComponent(ChatWorkspace).exists()).toBe(true)
  })

  it('imports and uses UtilityPanel', () => {
    const w = mount(FullMode)
    expect(w.findComponent(UtilityPanel).exists()).toBe(true)
  })

  it('shows bob phrase when daemonStore.currentBobPhrase is set', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const daemonStore = useDaemonStore()
    daemonStore.setBobPhrase('Bob is a doctor, he is diagnosing …')

    const wrapper = mount(FullMode, { global: { plugins: [pinia] } })
    expect(wrapper.find('[data-testid="bob-phrase"]').text()).toBe('Bob is a doctor, he is diagnosing …')
  })

  it('hides bob phrase when daemonStore.currentBobPhrase is null', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const daemonStore = useDaemonStore()
    daemonStore.setBobPhrase(null)

    const wrapper = mount(FullMode, { global: { plugins: [pinia] } })
    expect(wrapper.find('[data-testid="bob-phrase"]').exists()).toBe(false)
  })

  it('shows connection dot with correct class for connected status', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const daemonStore = useDaemonStore()
    daemonStore.setStatus('connected')

    const wrapper = mount(FullMode, { global: { plugins: [pinia] } })
    expect(wrapper.find('[data-testid="connection-dot"]').classes()).toContain('connected')
  })

  it('shows connection dot with disconnected class for disconnected status', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const daemonStore = useDaemonStore()
    daemonStore.setStatus('disconnected')

    const wrapper = mount(FullMode, { global: { plugins: [pinia] } })
    expect(wrapper.find('[data-testid="connection-dot"]').classes()).toContain('disconnected')
  })
})
