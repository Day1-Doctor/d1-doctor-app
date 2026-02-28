import { mount } from '@vue/test-utils'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockResolvedValue(undefined) }))

import ConnectionStatus from '../ConnectionStatus.vue'

describe('ConnectionStatus', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('renders .connection-status root element', () => {
    const w = mount(ConnectionStatus)
    expect(w.find('.connection-status').exists()).toBe(true)
  })

  it('shows two status rows: Daemon and Platform', () => {
    const w = mount(ConnectionStatus)
    const text = w.text()
    expect(text).toContain('Daemon')
    expect(text).toContain('Platform')
  })

  it('daemon dot has data-status="connected" when daemon store is connected', async () => {
    const { useDaemonStore } = await import('@/shared/stores/daemon')
    const store = useDaemonStore()
    store.setStatus('connected')
    const w = mount(ConnectionStatus)
    const daemonRow = w.find('[data-row="daemon"]')
    expect(daemonRow.find('.status-dot').attributes('data-status')).toBe('connected')
  })

  it('daemon dot has data-status="disconnected" when daemon store is disconnected', async () => {
    const { useDaemonStore } = await import('@/shared/stores/daemon')
    const store = useDaemonStore()
    store.setStatus('disconnected')
    const w = mount(ConnectionStatus)
    expect(w.find('[data-row="daemon"] .status-dot').attributes('data-status')).toBe('disconnected')
  })

  it('platform dot has data-status="connected" when orchestratorConnected is true', async () => {
    const { useDaemonStore } = await import('@/shared/stores/daemon')
    const store = useDaemonStore()
    store.setStatus('connected')
    store.setDaemonInfo({ daemonVersion: '1.0', orchestratorConnected: true, activeTasks: 0 })
    const w = mount(ConnectionStatus)
    expect(w.find('[data-row="platform"] .status-dot').attributes('data-status')).toBe('connected')
  })

  it('platform dot is disconnected when daemon is disconnected (cascades)', async () => {
    const { useDaemonStore } = await import('@/shared/stores/daemon')
    const store = useDaemonStore()
    store.setStatus('disconnected')
    const w = mount(ConnectionStatus)
    expect(w.find('[data-row="platform"] .status-dot').attributes('data-status')).toBe('disconnected')
  })

  it('renders a Reconnect button', () => {
    const w = mount(ConnectionStatus)
    expect(w.find('.reconnect-btn').exists()).toBe(true)
  })
})
