import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import StepIndicator from '../StepIndicator.vue'

describe('StepIndicator', () => {
  it('shows check for done state', () => {
    const w = mount(StepIndicator, { props: { state: 'done', label: 'Test', index: 0 } })
    expect(w.find('.step-dot').text()).toContain('✓')
    expect(w.classes()).toContain('done')
  })
  it('shows X for error state', () => {
    const w = mount(StepIndicator, { props: { state: 'error', label: 'Test', index: 0 } })
    expect(w.find('.step-dot').text()).toContain('✕')
    expect(w.classes()).toContain('error')
  })
  it('shows index+1 for pending state', () => {
    const w = mount(StepIndicator, { props: { state: 'pending', label: 'Step', index: 2 } })
    expect(w.find('.step-dot').text()).toBe('3')
    expect(w.classes()).toContain('pending')
  })
  it('shows label text', () => {
    const w = mount(StepIndicator, { props: { state: 'active', label: 'Installing...', index: 0 } })
    expect(w.find('.step-label').text()).toBe('Installing...')
  })
})
