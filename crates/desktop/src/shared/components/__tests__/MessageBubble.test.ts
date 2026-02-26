import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import MessageBubble from '../MessageBubble.vue'

describe('MessageBubble', () => {
  it('renders content', () => {
    const w = mount(MessageBubble, { props: { role: 'agent', content: 'Hello!', timestamp: 0 } })
    expect(w.text()).toContain('Hello!')
  })
  it('agent role has agent class', () => {
    const w = mount(MessageBubble, { props: { role: 'agent', content: '', timestamp: 0 } })
    expect(w.classes()).toContain('agent')
  })
  it('user role has user class', () => {
    const w = mount(MessageBubble, { props: { role: 'user', content: '', timestamp: 0 } })
    expect(w.classes()).toContain('user')
  })
  it('shows timestamp', () => {
    const w = mount(MessageBubble, { props: { role: 'agent', content: 'Hi', timestamp: 1706745600000 } })
    expect(w.find('.timestamp').exists()).toBe(true)
  })
  it('shows "Day 1 Doctor" label for agent role', () => {
    const w = mount(MessageBubble, { props: { role: 'agent', content: '', timestamp: 0 } })
    expect(w.find('.role-label').exists()).toBe(true)
    expect(w.find('.role-label').text()).toBe('Day 1 Doctor')
  })
  it('does not show role-label for user role', () => {
    const w = mount(MessageBubble, { props: { role: 'user', content: '', timestamp: 0 } })
    expect(w.find('.role-label').exists()).toBe(false)
  })
})
