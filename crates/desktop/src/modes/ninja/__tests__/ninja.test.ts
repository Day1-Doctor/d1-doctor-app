import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick } from 'vue'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue('ninja')
}))
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn()
}))

import NinjaBar from '../NinjaBar.vue'
import NinjaDropdown from '../NinjaDropdown.vue'
import StepTimeline from '../StepTimeline.vue'
import NinjaApp from '@/NinjaApp.vue'
import type { Step } from '@/shared/types'

// ---------------------------------------------------------------------------
// NinjaBar
// ---------------------------------------------------------------------------

describe('NinjaBar', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('renders input with placeholder "Ask anything..."', () => {
    const wrapper = mount(NinjaBar)
    const input = wrapper.find('input')
    expect(input.exists()).toBe(true)
    expect(input.attributes('placeholder')).toBe('Ask anything...')
  })

  it('renders send button', () => {
    const wrapper = mount(NinjaBar)
    const btn = wrapper.find('.ninja-send')
    expect(btn.exists()).toBe(true)
  })

  it('emits "submit" with query text on Enter key', async () => {
    const wrapper = mount(NinjaBar)
    const input = wrapper.find('input')

    await input.setValue('how do I fix a build error?')
    await input.trigger('keydown', { key: 'Enter' })

    expect(wrapper.emitted('submit')).toBeTruthy()
    expect(wrapper.emitted('submit')![0]).toEqual(['how do I fix a build error?'])
  })

  it('emits "dismiss" on Esc key', async () => {
    const wrapper = mount(NinjaBar)
    const input = wrapper.find('input')

    await input.trigger('keydown', { key: 'Escape' })

    expect(wrapper.emitted('dismiss')).toBeTruthy()
  })

  it('clears input after submit', async () => {
    const wrapper = mount(NinjaBar)
    const input = wrapper.find('input')

    await input.setValue('test query')
    await input.trigger('keydown', { key: 'Enter' })

    expect((input.element as HTMLInputElement).value).toBe('')
  })

  it('does not emit "submit" when input is empty or whitespace', async () => {
    const wrapper = mount(NinjaBar)
    const input = wrapper.find('input')

    await input.setValue('   ')
    await input.trigger('keydown', { key: 'Enter' })

    expect(wrapper.emitted('submit')).toBeFalsy()
  })

  it('renders the ninja logo element', () => {
    const wrapper = mount(NinjaBar)
    const logo = wrapper.find('.ninja-logo')
    expect(logo.exists()).toBe(true)
    expect(logo.text()).toContain('D1')
  })

  it('renders shortcut hints', () => {
    const wrapper = mount(NinjaBar)
    const hints = wrapper.find('.ninja-shortcuts')
    expect(hints.exists()).toBe(true)
  })

  describe('NinjaBar hints timer', () => {
    beforeEach(() => { vi.useFakeTimers() })
    afterEach(() => { vi.useRealTimers() })

    it('hides shortcut hints after 3s of typing', async () => {
      const wrapper = mount(NinjaBar)
      const input = wrapper.find('input, .ninja-input')
      await input.setValue('test query')
      await input.trigger('input')
      // Before 3s — hints should be visible (not hidden)
      expect(wrapper.find('.ninja-shortcuts').classes()).not.toContain('hints-hidden')
      // After 3s — hints should be hidden
      await vi.advanceTimersByTimeAsync(3000)
      await nextTick()
      expect(wrapper.find('.ninja-shortcuts').classes()).toContain('hints-hidden')
    })
  })
})

// ---------------------------------------------------------------------------
// NinjaApp
// ---------------------------------------------------------------------------

describe('NinjaApp', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('two-stage Esc: first hides dropdown, second clears query', async () => {
    const wrapper = mount(NinjaApp)
    // Trigger submit via real DOM interaction
    const input = wrapper.find('.ninja-input')
    await input.setValue('test query')
    await input.trigger('keydown', { key: 'Enter' })
    await nextTick()
    // Dropdown should be visible
    expect(wrapper.findComponent({ name: 'NinjaDropdown' }).exists()).toBe(true)
    // First Esc — hides dropdown
    await wrapper.trigger('keydown', { key: 'Escape' })
    await nextTick()
    expect(wrapper.findComponent({ name: 'NinjaDropdown' }).exists()).toBe(false)
    // Second Esc — bar state cleared (dropdown still gone, dismissNinjaWindow not called)
    await wrapper.trigger('keydown', { key: 'Escape' })
    await nextTick()
    expect(wrapper.findComponent({ name: 'NinjaDropdown' }).exists()).toBe(false)
    // Query was cleared (not dismissed): emit('ninja_dismissed') should NOT have been called yet
    const { emit: tauriEmit } = await import('@tauri-apps/api/event')
    expect(vi.mocked(tauriEmit)).not.toHaveBeenCalledWith('ninja_dismissed')
  })

  it('passes conversationStore plan steps to NinjaDropdown', async () => {
    const { useConversationStore } = await import('@/shared/stores/conversation')
    const conversationStore = useConversationStore()
    conversationStore.setPlan({
      steps: [
        { id: '1', label: 'Step A', state: 'pending', index: 0 },
        { id: '2', label: 'Step B', state: 'done', index: 1 },
      ],
      approved: null,
    })
    const wrapper = mount(NinjaApp)
    // Trigger submit via real DOM interaction
    const input = wrapper.find('.ninja-input')
    await input.setValue('show dropdown')
    await input.trigger('keydown', { key: 'Enter' })
    await nextTick()
    const ninjaDropdown = wrapper.findComponent({ name: 'NinjaDropdown' })
    expect(ninjaDropdown.exists()).toBe(true)
    expect(ninjaDropdown.props('steps')).toHaveLength(2)
    expect(ninjaDropdown.props('steps')[0].label).toBe('Step A')
  })
})

// ---------------------------------------------------------------------------
// NinjaDropdown
// ---------------------------------------------------------------------------

describe('NinjaDropdown', () => {
  const baseProps = {
    query: 'diagnose my startup',
    steps: [] as Step[],
    creditEstimate: '~0.5 credits',
  }

  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('renders query echo with ">" prefix', () => {
    const wrapper = mount(NinjaDropdown, { props: baseProps })
    const echo = wrapper.find('.query-echo')
    expect(echo.exists()).toBe(true)
    expect(echo.text()).toContain('>')
    expect(echo.text()).toContain('diagnose my startup')
  })

  it('renders approve button', () => {
    const wrapper = mount(NinjaDropdown, { props: baseProps })
    const btn = wrapper.find('.btn-approve')
    expect(btn.exists()).toBe(true)
  })

  it('renders dismiss button', () => {
    const wrapper = mount(NinjaDropdown, { props: baseProps })
    const btn = wrapper.find('.btn-dismiss')
    expect(btn.exists()).toBe(true)
  })

  it('emits "approve" when approve button is clicked', async () => {
    const wrapper = mount(NinjaDropdown, { props: baseProps })
    await wrapper.find('.btn-approve').trigger('click')
    expect(wrapper.emitted('approve')).toBeTruthy()
  })

  it('emits "dismiss" when dismiss button is clicked', async () => {
    const wrapper = mount(NinjaDropdown, { props: baseProps })
    await wrapper.find('.btn-dismiss').trigger('click')
    expect(wrapper.emitted('dismiss')).toBeTruthy()
  })

  it('renders credit estimate text', () => {
    const wrapper = mount(NinjaDropdown, { props: baseProps })
    expect(wrapper.text()).toContain('~0.5 credits')
  })

  it('does not render ResultCard when resultTitle is absent', () => {
    const wrapper = mount(NinjaDropdown, { props: baseProps })
    expect(wrapper.find('.result-card').exists()).toBe(false)
  })

  it('renders ResultCard when resultTitle prop is provided', () => {
    const wrapper = mount(NinjaDropdown, {
      props: { ...baseProps, resultTitle: 'Analysis complete', resultDetail: 'All good' }
    })
    expect(wrapper.find('.result-card').exists()).toBe(true)
  })

  it('renders agent label in header', () => {
    const wrapper = mount(NinjaDropdown, { props: baseProps })
    const label = wrapper.find('.agent-label')
    expect(label.exists()).toBe(true)
    expect(label.text()).toContain('Day1 Doctor')
  })

  it('renders progress bar in footer', () => {
    const wrapper = mount(NinjaDropdown, { props: baseProps })
    expect(wrapper.find('.progress-track').exists()).toBe(true)
    expect(wrapper.find('.progress-fill').exists()).toBe(true)
  })

  it('shows 0% progress fill when no steps are done', async () => {
    const steps: Step[] = [
      { id: 's1', label: 'Step A', state: 'pending', index: 0 },
      { id: 's2', label: 'Step B', state: 'active', index: 1 },
    ]
    const wrapper = mount(NinjaDropdown, {
      props: { ...baseProps, steps }
    })
    const fill = wrapper.find('.progress-fill')
    expect(fill.attributes('style')).toContain('width: 0%')
  })

  it('shows 50% progress fill when half of steps are done', async () => {
    const steps: Step[] = [
      { id: 's1', label: 'Step A', state: 'done', index: 0 },
      { id: 's2', label: 'Step B', state: 'pending', index: 1 },
    ]
    const wrapper = mount(NinjaDropdown, {
      props: { ...baseProps, steps }
    })
    const fill = wrapper.find('.progress-fill')
    expect(fill.attributes('style')).toContain('width: 50%')
  })
})

// ---------------------------------------------------------------------------
// StepTimeline
// ---------------------------------------------------------------------------

describe('StepTimeline', () => {
  const makeStep = (id: string, label: string, state: Step['state'], index: number): Step => ({
    id, label, state, index
  })

  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('renders a step-item for each step', () => {
    const steps: Step[] = [
      makeStep('s1', 'Analyse codebase', 'done', 0),
      makeStep('s2', 'Generate plan', 'active', 1),
      makeStep('s3', 'Apply changes', 'pending', 2),
    ]
    const wrapper = mount(StepTimeline, { props: { steps } })
    expect(wrapper.findAll('.step-item')).toHaveLength(3)
  })

  it('shows ✓ for done steps', () => {
    const steps: Step[] = [makeStep('s1', 'Done step', 'done', 0)]
    const wrapper = mount(StepTimeline, { props: { steps } })
    const dot = wrapper.find('.step-dot')
    expect(dot.text()).toContain('✓')
  })

  it('shows number (index + 1) for pending steps', () => {
    const steps: Step[] = [makeStep('s1', 'Pending step', 'pending', 2)]
    const wrapper = mount(StepTimeline, { props: { steps } })
    const dot = wrapper.find('.step-dot')
    expect(dot.text()).toContain('3')
  })

  it('shows ✕ for error steps', () => {
    const steps: Step[] = [makeStep('s1', 'Error step', 'error', 0)]
    const wrapper = mount(StepTimeline, { props: { steps } })
    const dot = wrapper.find('.step-dot')
    expect(dot.text()).toContain('✕')
  })

  it('shows number for active steps', () => {
    const steps: Step[] = [makeStep('s1', 'Active step', 'active', 1)]
    const wrapper = mount(StepTimeline, { props: { steps } })
    const dot = wrapper.find('.step-dot')
    expect(dot.text()).toContain('2')
  })

  it('applies correct class for each step state', () => {
    const steps: Step[] = [
      makeStep('s1', 'Done', 'done', 0),
      makeStep('s2', 'Active', 'active', 1),
      makeStep('s3', 'Pending', 'pending', 2),
      makeStep('s4', 'Error', 'error', 3),
    ]
    const wrapper = mount(StepTimeline, { props: { steps } })
    const items = wrapper.findAll('.step-item')
    expect(items[0].classes()).toContain('done')
    expect(items[1].classes()).toContain('active')
    expect(items[2].classes()).toContain('pending')
    expect(items[3].classes()).toContain('error')
  })

  it('renders step labels', () => {
    const steps: Step[] = [makeStep('s1', 'Analyse dependencies', 'pending', 0)]
    const wrapper = mount(StepTimeline, { props: { steps } })
    expect(wrapper.find('.step-label').text()).toContain('Analyse dependencies')
  })

  it('renders empty timeline when steps is empty', () => {
    const wrapper = mount(StepTimeline, { props: { steps: [] } })
    expect(wrapper.findAll('.step-item')).toHaveLength(0)
  })
})
