import { setActivePinia, createPinia } from 'pinia'
import { describe, it, expect, beforeEach, vi } from 'vitest'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue('full'),
}))

describe('ConversationStore — setScrollPinned / approvePlan', () => {
  beforeEach(() => { setActivePinia(createPinia()) })

  it('setScrollPinned(true) sets scrollPinned to true', async () => {
    const { useConversationStore } = await import('../conversation')
    const store = useConversationStore()
    store.setScrollPinned(false)
    store.setScrollPinned(true)
    expect(store.scrollPinned).toBe(true)
  })

  it('setScrollPinned(false) sets scrollPinned to false', async () => {
    const { useConversationStore } = await import('../conversation')
    const store = useConversationStore()
    store.setScrollPinned(false)
    expect(store.scrollPinned).toBe(false)
  })

  it('approvePlan(true) sets currentPlan.approved to true when plan exists', async () => {
    const { useConversationStore } = await import('../conversation')
    const store = useConversationStore()
    store.setPlan({ steps: [{ id: 's1', label: 'Step 1', state: 'pending', index: 0 }], approved: null })
    store.approvePlan(true)
    expect(store.currentPlan!.approved).toBe(true)
  })

  it('approvePlan(false) sets currentPlan.approved to false when plan exists', async () => {
    const { useConversationStore } = await import('../conversation')
    const store = useConversationStore()
    store.setPlan({ steps: [{ id: 's1', label: 'Step 1', state: 'pending', index: 0 }], approved: null })
    store.approvePlan(false)
    expect(store.currentPlan!.approved).toBe(false)
  })

  it('approvePlan(true) does nothing when currentPlan is null', async () => {
    const { useConversationStore } = await import('../conversation')
    const store = useConversationStore()
    expect(store.currentPlan).toBeNull()
    store.approvePlan(true)
    expect(store.currentPlan).toBeNull()
  })
})
