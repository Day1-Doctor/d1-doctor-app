import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import PlanCard from '../PlanCard.vue'
import type { Step } from '@/shared/types'

const makeSteps = (): Step[] => [
  { id: 's1', label: 'Step 1', state: 'done', index: 0 },
  { id: 's2', label: 'Step 2', state: 'active', index: 1 },
  { id: 's3', label: 'Step 3', state: 'pending', index: 2 },
]

describe('PlanCard', () => {
  it('renders all steps', () => {
    const w = mount(PlanCard, { props: { steps: makeSteps() } })
    const labels = w.findAll('.step-label')
    expect(labels).toHaveLength(3)
  })
  it('emits approve on button click', async () => {
    const w = mount(PlanCard, { props: { steps: makeSteps() } })
    await w.find('.btn-approve').trigger('click')
    expect(w.emitted('approve')).toBeTruthy()
  })
  it('emits reject on button click', async () => {
    const w = mount(PlanCard, { props: { steps: makeSteps() } })
    await w.find('.btn-reject').trigger('click')
    expect(w.emitted('reject')).toBeTruthy()
  })
  it('progress bar reflects done/total ratio', () => {
    const w = mount(PlanCard, { props: { steps: makeSteps() } })
    const fill = w.find('.progress-fill')
    // 1 of 3 done = 33.33%
    expect(fill.attributes('style')).toContain('33')
  })
})
