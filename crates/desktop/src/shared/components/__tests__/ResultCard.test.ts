import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import ResultCard from '../ResultCard.vue'

describe('ResultCard', () => {
  it('renders title and detail', () => {
    const w = mount(ResultCard, { props: { title: 'Done', detail: 'All good' } })
    expect(w.find('.result-title').text()).toContain('Done')
    expect(w.find('.result-detail').text()).toBe('All good')
  })
  it('shows code block when code prop present', () => {
    const w = mount(ResultCard, { props: { title: 'T', detail: 'D', code: 'npm install' } })
    expect(w.find('.result-code').exists()).toBe(true)
    expect(w.find('.result-code').text()).toBe('npm install')
  })
  it('hides code block when no code prop', () => {
    const w = mount(ResultCard, { props: { title: 'T', detail: 'D' } })
    expect(w.find('.result-code').exists()).toBe(false)
  })
})
