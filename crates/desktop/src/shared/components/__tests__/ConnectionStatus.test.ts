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

  // --- D1D-75: overall connection status indicator (3 states) ---

  it('renders conn-summary element', () => {
    const w = mount(ConnectionStatus)
    expect(w.find('[data-testid="conn-summary"]').exists()).toBe(true)
  })

  it('shows "Connected" (green) when daemon + platform are both connected', async () => {
    const { useDaemonStore } = await import('@/shared/stores/daemon')
    const store = useDaemonStore()
    store.setStatus('connected')
    store.setDaemonInfo({ daemonVersion: '1.0', orchestratorConnected: true, activeTasks: 0 })
    const w = mount(ConnectionStatus)
    const summary = w.find('[data-testid="conn-summary"]')
    expect(summary.find('.status-dot').attributes('data-status')).toBe('connected')
    expect(summary.find('.conn-summary-text').text()).toBe('Connected')
    expect(summary.find('.conn-summary-text').classes()).toContain('connected')
  })

  it('shows "Local only" (yellow) when daemon connected but platform offline', async () => {
    const { useDaemonStore } = await import('@/shared/stores/daemon')
    const store = useDaemonStore()
    store.setStatus('connected')
    store.setDaemonInfo({ daemonVersion: '1.0', orchestratorConnected: false, activeTasks: 0 })
    const w = mount(ConnectionStatus)
    const summary = w.find('[data-testid="conn-summary"]')
    expect(summary.find('.status-dot').attributes('data-status')).toBe('local-only')
    expect(summary.find('.conn-summary-text').text()).toBe('Local only')
    expect(summary.find('.conn-summary-text').classes()).toContain('local-only')
  })

  it('shows "Offline" (red) when daemon is disconnected', async () => {
    const { useDaemonStore } = await import('@/shared/stores/daemon')
    const store = useDaemonStore()
    store.setStatus('disconnected')
    const w = mount(ConnectionStatus)
    const summary = w.find('[data-testid="conn-summary"]')
    expect(summary.find('.status-dot').attributes('data-status')).toBe('offline')
    expect(summary.find('.conn-summary-text').text()).toBe('Offline')
    expect(summary.find('.conn-summary-text').classes()).toContain('offline')
  })
})
