import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import UpdateBanner from '../UpdateBanner.vue'

describe('UpdateBanner', () => {
  it('renders nothing when visible is false', () => {
    const w = mount(UpdateBanner, {
      props: { visible: false, version: '2.5.0' },
    })
    expect(w.find('[data-testid="update-banner"]').exists()).toBe(false)
  })

  it('renders the banner when visible is true', () => {
    const w = mount(UpdateBanner, {
      props: { visible: true, version: '2.5.0' },
    })
    expect(w.find('[data-testid="update-banner"]').exists()).toBe(true)
  })

  it('displays the version string in the message', () => {
    const w = mount(UpdateBanner, {
      props: { visible: true, version: '3.0.1' },
    })
    expect(w.text()).toContain('v3.0.1')
    expect(w.text()).toContain('Restart to update')
  })

  it('emits "restart" when Restart Now is clicked', async () => {
    const w = mount(UpdateBanner, {
      props: { visible: true, version: '2.5.0' },
    })
    await w.find('[data-testid="restart-btn"]').trigger('click')
    expect(w.emitted('restart')).toHaveLength(1)
  })

  it('emits "dismiss" when Later is clicked', async () => {
    const w = mount(UpdateBanner, {
      props: { visible: true, version: '2.5.0' },
    })
    await w.find('[data-testid="later-btn"]').trigger('click')
    expect(w.emitted('dismiss')).toHaveLength(1)
  })

  it('has role="status" for accessibility', () => {
    const w = mount(UpdateBanner, {
      props: { visible: true, version: '2.5.0' },
    })
    expect(w.find('[data-testid="update-banner"]').attributes('role')).toBe('status')
  })

  it('shows Restart Now and Later buttons', () => {
    const w = mount(UpdateBanner, {
      props: { visible: true, version: '2.5.0' },
    })
    expect(w.find('[data-testid="restart-btn"]').text()).toBe('Restart Now')
    expect(w.find('[data-testid="later-btn"]').text()).toBe('Later')
  })
})
