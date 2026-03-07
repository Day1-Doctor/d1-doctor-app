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

  // --- D1D-75: credit remaining text and shared queue ---

  it('full variant shows "XX/50 credits remaining" text', () => {
    const w = mount(CreditBar, { props: { credits: 30, max: 50, variant: 'full' } })
    const remaining = w.find('[data-testid="credit-remaining"]')
    expect(remaining.exists()).toBe(true)
    expect(remaining.text()).toBe('30/50 credits remaining')
  })

  it('shows depleted class when credits reach 0', () => {
    const w = mount(CreditBar, { props: { credits: 0, max: 50, variant: 'full' } })
    expect(w.classes()).toContain('depleted')
  })

  it('does not show depleted class when credits > 0', () => {
    const w = mount(CreditBar, { props: { credits: 10, max: 50, variant: 'full' } })
    expect(w.classes()).not.toContain('depleted')
  })

  it('shows shared queue notice when depleted and isQueued', () => {
    const w = mount(CreditBar, { props: { credits: 0, max: 50, variant: 'full', isQueued: true } })
    const notice = w.find('[data-testid="shared-queue-notice"]')
    expect(notice.exists()).toBe(true)
    expect(notice.text()).toBe('Shared queue - responses may be slower')
  })

  it('hides shared queue notice when credits > 0 even if isQueued', () => {
    const w = mount(CreditBar, { props: { credits: 10, max: 50, variant: 'full', isQueued: true } })
    expect(w.find('[data-testid="shared-queue-notice"]').exists()).toBe(false)
  })

  it('hides shared queue notice when depleted but isQueued is false', () => {
    const w = mount(CreditBar, { props: { credits: 0, max: 50, variant: 'full', isQueued: false } })
    expect(w.find('[data-testid="shared-queue-notice"]').exists()).toBe(false)
  })

  it('bar fill has depleted modifier when credits = 0', () => {
    const w = mount(CreditBar, { props: { credits: 0, max: 50, variant: 'full' } })
    expect(w.find('.bar-fill--depleted').exists()).toBe(true)
  })

  it('mini variant does not show credit-remaining text', () => {
    const w = mount(CreditBar, { props: { credits: 30, max: 50, variant: 'mini' } })
    expect(w.find('[data-testid="credit-remaining"]').exists()).toBe(false)
  })
})
