import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import AgentAvatar from '../AgentAvatar.vue'

describe('AgentAvatar', () => {
  it('renders first letter uppercase', () => {
    const w = mount(AgentAvatar, { props: { agent: 'planner', active: false } })
    expect(w.text()).toBe('P')
  })
  it('applies active class when active=true', () => {
    const w = mount(AgentAvatar, { props: { agent: 'X', active: true } })
    expect(w.classes()).toContain('active')
  })
  it('does not apply active class when false', () => {
    const w = mount(AgentAvatar, { props: { agent: 'X', active: false } })
    expect(w.classes()).not.toContain('active')
  })
})
