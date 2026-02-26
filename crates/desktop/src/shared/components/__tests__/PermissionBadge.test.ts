import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import PermissionBadge from '../PermissionBadge.vue'

describe('PermissionBadge', () => {
  it('renders label', () => {
    const w = mount(PermissionBadge, { props: { state: 'allow', label: 'Read Files' } })
    expect(w.text()).toBe('Read Files')
  })
  it('applies allow class', () => {
    const w = mount(PermissionBadge, { props: { state: 'allow', label: 'x' } })
    expect(w.classes()).toContain('allow')
  })
  it('applies ask class', () => {
    const w = mount(PermissionBadge, { props: { state: 'ask', label: 'x' } })
    expect(w.classes()).toContain('ask')
  })
  it('applies deny class', () => {
    const w = mount(PermissionBadge, { props: { state: 'deny', label: 'x' } })
    expect(w.classes()).toContain('deny')
  })
})
