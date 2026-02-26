import { mount } from '@vue/test-utils'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { nextTick } from 'vue'
import { createPinia, setActivePinia } from 'pinia'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockResolvedValue('full') }))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn().mockResolvedValue(() => {}), emit: vi.fn() }))

import ChatWorkspace from '../ChatWorkspace.vue'
import { useConversationStore } from '@/shared/stores/conversation'

describe('ChatWorkspace', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('renders the chat-workspace element', () => {
    const w = mount(ChatWorkspace)
    expect(w.find('.chat-workspace').exists()).toBe(true)
  })

  it('renders the message-list container', () => {
    const w = mount(ChatWorkspace)
    expect(w.find('.message-list').exists()).toBe(true)
  })

  it('shows empty state when no messages', () => {
    const w = mount(ChatWorkspace)
    expect(w.find('.empty-state').exists()).toBe(true)
    expect(w.text()).toContain('Start a conversation')
  })

  it('renders textarea in input bar', () => {
    const w = mount(ChatWorkspace)
    expect(w.find('textarea').exists()).toBe(true)
  })

  it('renders input pill with send button', () => {
    const w = mount(ChatWorkspace)
    expect(w.find('.input-pill').exists()).toBe(true)
    expect(w.find('.send-btn').exists()).toBe(true)
  })

  it('send button is disabled when textarea is empty', () => {
    const w = mount(ChatWorkspace)
    const btn = w.find('.send-btn')
    expect((btn.element as HTMLButtonElement).disabled).toBe(true)
  })

  it('send button is enabled when textarea has content', async () => {
    const w = mount(ChatWorkspace)
    const textarea = w.find('textarea')
    await textarea.setValue('Hello world')
    const btn = w.find('.send-btn')
    expect((btn.element as HTMLButtonElement).disabled).toBe(false)
  })

  it('Enter key submits message and appends to store', async () => {
    const store = useConversationStore()
    const w = mount(ChatWorkspace)
    const textarea = w.find('textarea')
    await textarea.setValue('Test message')
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })
    expect(store.messages).toHaveLength(1)
    expect(store.messages[0].content).toBe('Test message')
    expect(store.messages[0].role).toBe('user')
  })

  it('Enter key clears the textarea after submit', async () => {
    const w = mount(ChatWorkspace)
    const textarea = w.find('textarea')
    await textarea.setValue('Hello')
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })
    expect((textarea.element as HTMLTextAreaElement).value).toBe('')
  })

  it('Shift+Enter does not submit', async () => {
    const store = useConversationStore()
    const w = mount(ChatWorkspace)
    const textarea = w.find('textarea')
    await textarea.setValue('Draft')
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: true })
    expect(store.messages).toHaveLength(0)
  })

  it('renders MessageBubble for each message in store', async () => {
    const store = useConversationStore()
    store.appendMessage({ id: '1', role: 'user', content: 'Hello', timestamp: Date.now() })
    store.appendMessage({ id: '2', role: 'agent', content: 'Hi there', timestamp: Date.now() })
    const w = mount(ChatWorkspace)
    const bubbles = w.findAll('.message-bubble')
    expect(bubbles).toHaveLength(2)
  })

  it('hides empty state when messages exist', async () => {
    const store = useConversationStore()
    store.appendMessage({ id: '1', role: 'user', content: 'Hi', timestamp: Date.now() })
    const w = mount(ChatWorkspace)
    expect(w.find('.empty-state').exists()).toBe(false)
  })

  it('renders PlanCard when currentPlan is set', () => {
    const store = useConversationStore()
    store.setPlan({ steps: [{ id: 's1', label: 'Step 1', state: 'pending', index: 0 }], approved: null })
    const w = mount(ChatWorkspace)
    expect(w.find('.plan-card').exists()).toBe(true)
  })

  it('new-messages-badge is hidden when scrollPinned is true', () => {
    const store = useConversationStore()
    store.setScrollPinned(true)
    const w = mount(ChatWorkspace)
    expect(w.find('.new-messages-badge').exists()).toBe(false)
  })

  it('new-messages-badge appears when scrollPinned is false', () => {
    const store = useConversationStore()
    store.setScrollPinned(false)
    const w = mount(ChatWorkspace)
    expect(w.find('.new-messages-badge').exists()).toBe(true)
  })

  it('clicking new-messages-badge calls setScrollPinned(true)', async () => {
    const store = useConversationStore()
    store.setScrollPinned(false)
    const spy = vi.spyOn(store, 'setScrollPinned')
    const w = mount(ChatWorkspace)
    await w.find('.new-messages-badge').trigger('click')
    expect(spy).toHaveBeenCalledWith(true)
  })

  it('ArrowUp recalls last sent message', async () => {
    const w = mount(ChatWorkspace)
    const textarea = w.find('textarea')
    // Send a message first
    await textarea.setValue('Previous message')
    await textarea.trigger('keydown', { key: 'Enter', shiftKey: false })
    // Press ArrowUp to recall
    await textarea.trigger('keydown', { key: 'ArrowUp' })
    expect((textarea.element as HTMLTextAreaElement).value).toBe('Previous message')
  })

  it('does not auto-scroll when user has scrolled up (scrollPinned = false)', async () => {
    const store = useConversationStore()
    store.setScrollPinned(false)
    const w = mount(ChatWorkspace)
    // scrollPinned should still be false after mounting
    expect(store.scrollPinned).toBe(false)
    // Adding a message should not change scrollPinned (no real DOM scroll happens in jsdom)
    store.appendMessage({ id: '1', role: 'agent', content: 'test', timestamp: Date.now() })
    await nextTick()
    expect(store.scrollPinned).toBe(false)
  })

  it('onApprove calls approvePlan(true) via store action', async () => {
    const store = useConversationStore()
    store.setPlan({ steps: [{ id: 's1', label: 'Step 1', state: 'pending', index: 0 }], approved: null })
    const spy = vi.spyOn(store, 'approvePlan')
    const w = mount(ChatWorkspace)
    // Click the Approve button inside PlanCard to trigger the Vue emit
    await w.find('.btn-approve').trigger('click')
    expect(spy).toHaveBeenCalledWith(true)
  })

  it('onReject calls approvePlan(false) via store action', async () => {
    const store = useConversationStore()
    store.setPlan({ steps: [{ id: 's1', label: 'Step 1', state: 'pending', index: 0 }], approved: null })
    const spy = vi.spyOn(store, 'approvePlan')
    const w = mount(ChatWorkspace)
    // Click the Reject button inside PlanCard to trigger the Vue emit
    await w.find('.btn-reject').trigger('click')
    expect(spy).toHaveBeenCalledWith(false)
  })
})
