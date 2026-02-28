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

  it('setError() records errorMessage without changing status', async () => {
    const { useDaemonStore } = await import('../daemon')
    const store = useDaemonStore()
    // setError() stores the message for the UI but does NOT force status to
    // 'error' — the WebSocket connect() call that follows determines real status.
    store.setError('Daemon failed to start')
    expect(store.errorMessage).toBe('Daemon failed to start')
    // Status should remain at its initial value ('disconnected'), unchanged.
    expect(store.status).toBe('disconnected')
  })

  it('setStatus("connected") clears errorMessage', async () => {
    const { useDaemonStore } = await import('../daemon')
    const store = useDaemonStore()
    store.setError('Daemon not running')
    store.setStatus('connected')
    expect(store.errorMessage).toBeNull()
    expect(store.status).toBe('connected')
  })

  it('setStatus("connecting") preserves errorMessage', async () => {
    const { useDaemonStore } = await import('../daemon')
    const store = useDaemonStore()
    store.setError('Daemon not running')
    store.setStatus('connecting')
    // errorMessage stays visible while we are still trying to connect
    expect(store.errorMessage).toBe('Daemon not running')
    expect(store.status).toBe('connecting')
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
