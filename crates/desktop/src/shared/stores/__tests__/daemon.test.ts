import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'

describe('useDaemonStore', () => {
  beforeEach(() => { setActivePinia(createPinia()) })

  it('initial state: status="disconnected", activeTasks=0', async () => {
    const { useDaemonStore } = await import('../daemon')
    const store = useDaemonStore()
    expect(store.status).toBe('disconnected')
    expect(store.activeTasks).toBe(0)
    expect(store.daemonVersion).toBeNull()
  })

  it('setStatus() updates status', async () => {
    const { useDaemonStore } = await import('../daemon')
    const store = useDaemonStore()
    store.setStatus('connected')
    expect(store.status).toBe('connected')
  })

  it('setDaemonInfo() updates version and orchestratorConnected', async () => {
    const { useDaemonStore } = await import('../daemon')
    const store = useDaemonStore()
    store.setDaemonInfo({ daemonVersion: '0.4.1', orchestratorConnected: true, activeTasks: 2 })
    expect(store.daemonVersion).toBe('0.4.1')
    expect(store.orchestratorConnected).toBe(true)
    expect(store.activeTasks).toBe(2)
  })

  it('initial currentBobPhrase is null', async () => {
    const { useDaemonStore } = await import('../daemon')
    const store = useDaemonStore()
    expect(store.currentBobPhrase).toBeNull()
  })

  it('setBobPhrase() updates currentBobPhrase', async () => {
    const { useDaemonStore } = await import('../daemon')
    const store = useDaemonStore()
    store.setBobPhrase('Bob is a surgeon, he is operating …')
    expect(store.currentBobPhrase).toBe('Bob is a surgeon, he is operating …')
  })

  it('setBobPhrase(null) clears currentBobPhrase', async () => {
    const { useDaemonStore } = await import('../daemon')
    const store = useDaemonStore()
    store.setBobPhrase('Bob is a detective, he is investigating …')
    store.setBobPhrase(null)
    expect(store.currentBobPhrase).toBeNull()
  })

  it('setError() sets status to error and records message', async () => {
    const { useDaemonStore } = await import('../daemon')
    const store = useDaemonStore()
    store.setError('Daemon failed to start')
    expect(store.status).toBe('error')
    expect(store.errorMessage).toBe('Daemon failed to start')
  })

  it('setCurrentPlanId() stores the plan id', async () => {
    const { useDaemonStore } = await import('../daemon')
    const store = useDaemonStore()
    store.setCurrentPlanId('pln_abc123')
    expect(store.currentPlanId).toBe('pln_abc123')
  })

  it('decrementActiveTasks() reduces activeTasks by 1, not below 0', async () => {
    const { useDaemonStore } = await import('../daemon')
    const store = useDaemonStore()
    store.setDaemonInfo({ daemonVersion: '0.4.1', orchestratorConnected: false, activeTasks: 2 })
    store.decrementActiveTasks()
    expect(store.activeTasks).toBe(1)
    store.decrementActiveTasks()
    expect(store.activeTasks).toBe(0)
    store.decrementActiveTasks()
    expect(store.activeTasks).toBe(0)
  })
})
