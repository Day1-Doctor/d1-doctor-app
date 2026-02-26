import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import CreditBar from '../CreditBar.vue'

describe('CreditBar', () => {
  it('full variant shows label', () => {
    const w = mount(CreditBar, { props: { credits: 50, max: 100, variant: 'full' } })
    expect(w.find('.credit-label').exists()).toBe(true)
    expect(w.find('.credit-label').text()).toBe('Credits')
  })
  it('mini variant hides label', () => {
    const w = mount(CreditBar, { props: { credits: 50, max: 100, variant: 'mini' } })
    expect(w.find('.credit-label').exists()).toBe(false)
  })
  it('progress bar width reflects credits/max ratio', () => {
    const w = mount(CreditBar, { props: { credits: 25, max: 100, variant: 'full' } })
    const fill = w.find('.bar-fill')
    expect(fill.attributes('style')).toContain('25%')
  })
  it('caps bar at 100%', () => {
    const w = mount(CreditBar, { props: { credits: 150, max: 100, variant: 'full' } })
    const fill = w.find('.bar-fill')
    expect(fill.attributes('style')).toContain('100%')
  })
  it('full variant shows buy link', () => {
    const w = mount(CreditBar, { props: { credits: 50, max: 100, variant: 'full' } })
    expect(w.find('.buy-link').exists()).toBe(true)
  })
  it('buy link click emits buy event', async () => {
    const w = mount(CreditBar, { props: { credits: 50, max: 100, variant: 'full' } })
    await w.find('.buy-link').trigger('click')
    expect(w.emitted('buy')).toBeTruthy()
  })
  it('mini variant hides buy link', () => {
    const w = mount(CreditBar, { props: { credits: 50, max: 100, variant: 'mini' } })
    expect(w.find('.buy-link').exists()).toBe(false)
  })
  it('dropdown variant renders with dropdown class', () => {
    const w = mount(CreditBar, { props: { credits: 30, max: 100, variant: 'dropdown' } })
    expect(w.classes()).toContain('dropdown')
    expect(w.find('.bar-track').exists()).toBe(true)
    expect(w.find('.buy-link').exists()).toBe(false)
  })
})
