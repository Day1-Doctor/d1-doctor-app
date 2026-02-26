import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import KbdShortcut from '../KbdShortcut.vue'

describe('KbdShortcut', () => {
  it('renders one kbd element per key', () => {
    const w = mount(KbdShortcut, { props: { keys: ['⌘', '⇧', 'D'] } })
    const keys = w.findAll('.kbd-key')
    expect(keys).toHaveLength(3)
  })
  it('renders key text correctly', () => {
    const w = mount(KbdShortcut, { props: { keys: ['Esc'] } })
    expect(w.find('.kbd-key').text()).toBe('Esc')
  })
  it('renders empty when no keys', () => {
    const w = mount(KbdShortcut, { props: { keys: [] } })
    expect(w.findAll('.kbd-key')).toHaveLength(0)
  })
})
