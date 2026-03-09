import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick } from 'vue'

// ── Hoisted mocks ─────────────────────────────────────────────────────────

const { mockInvoke } = vi.hoisted(() => {
  const mockInvoke = vi.fn().mockResolvedValue(undefined)
  return { mockInvoke }
})

vi.mock('@tauri-apps/api/core', () => ({ invoke: mockInvoke }))

import LoginScreen from '../LoginScreen.vue'
import { useAuthStore } from '@/shared/stores/auth'

// ── Tests ─────────────────────────────────────────────────────────────────

describe('LoginScreen.vue', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    mockInvoke.mockClear()
  })

  it('renders email and password fields', () => {
    const wrapper = mount(LoginScreen)
    expect(wrapper.find('#login-email').exists()).toBe(true)
    expect(wrapper.find('#login-password').exists()).toBe(true)
  })

  it('renders sign in button and Google SSO button', () => {
    const wrapper = mount(LoginScreen)
    const buttons = wrapper.findAll('button')
    const signInBtn = buttons.find(b => b.text().includes('Sign In'))
    const googleBtn = buttons.find(b => b.text().includes('Continue with Google'))
    expect(signInBtn).toBeDefined()
    expect(googleBtn).toBeDefined()
  })

  it('renders the Day1 Doctor branding', () => {
    const wrapper = mount(LoginScreen)
    expect(wrapper.find('.logo-mark').text()).toBe('D1')
    expect(wrapper.find('.app-title').text()).toBe('Day1 Doctor')
    expect(wrapper.find('.app-subtitle').text()).toBe('Sign in to continue')
  })

  it('shows email validation error on empty email submit', async () => {
    const wrapper = mount(LoginScreen)
    await wrapper.find('form').trigger('submit')
    await nextTick()

    expect(wrapper.find('.field--error').exists()).toBe(true)
    expect(wrapper.text()).toContain('Email is required.')
  })

  it('shows email format validation error', async () => {
    const wrapper = mount(LoginScreen)
    const emailInput = wrapper.find('#login-email')
    await emailInput.setValue('notanemail')
    await wrapper.find('form').trigger('submit')
    await nextTick()

    expect(wrapper.text()).toContain('Please enter a valid email address.')
  })

  it('shows password required error', async () => {
    const wrapper = mount(LoginScreen)
    const emailInput = wrapper.find('#login-email')
    await emailInput.setValue('test@example.com')
    await wrapper.find('form').trigger('submit')
    await nextTick()

    expect(wrapper.text()).toContain('Password is required.')
  })

  it('calls authStore.login on valid form submission', async () => {
    const wrapper = mount(LoginScreen)
    const authStore = useAuthStore()
    const loginSpy = vi.spyOn(authStore, 'login').mockResolvedValue(true)

    await wrapper.find('#login-email').setValue('test@example.com')
    await wrapper.find('#login-password').setValue('password123')
    await wrapper.find('form').trigger('submit')
    await flushPromises()

    expect(loginSpy).toHaveBeenCalledWith('test@example.com', 'password123')
  })

  it('calls authStore.loginWithGoogle on Google button click', async () => {
    const wrapper = mount(LoginScreen)
    const authStore = useAuthStore()
    const googleSpy = vi.spyOn(authStore, 'loginWithGoogle').mockResolvedValue(true)

    const googleBtn = wrapper.findAll('button').find(b => b.text().includes('Continue with Google'))
    await googleBtn!.trigger('click')
    await flushPromises()

    expect(googleSpy).toHaveBeenCalled()
  })

  it('shows loading state when auth is in progress', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const authStore = useAuthStore()
    authStore.loading = true

    const wrapper = mount(LoginScreen, { global: { plugins: [pinia] } })
    expect(wrapper.text()).toContain('Signing in...')
    expect(wrapper.find('.spinner').exists()).toBe(true)
  })

  it('shows error banner when authStore has an error', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const authStore = useAuthStore()
    authStore.error = { type: 'invalid_credentials', message: 'Invalid email or password.' }

    const wrapper = mount(LoginScreen, { global: { plugins: [pinia] } })
    expect(wrapper.find('.error-banner').exists()).toBe(true)
    expect(wrapper.text()).toContain('Invalid email or password.')
  })

  it('clears field error on input', async () => {
    const wrapper = mount(LoginScreen)

    // Trigger validation error
    await wrapper.find('form').trigger('submit')
    await nextTick()
    expect(wrapper.text()).toContain('Email is required.')

    // Type in the email field — error should clear
    await wrapper.find('#login-email').setValue('a')
    await wrapper.find('#login-email').trigger('input')
    await nextTick()
    expect(wrapper.text()).not.toContain('Email is required.')
  })

  it('disables inputs during loading', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const authStore = useAuthStore()
    authStore.loading = true

    const wrapper = mount(LoginScreen, { global: { plugins: [pinia] } })
    const emailInput = wrapper.find('#login-email').element as HTMLInputElement
    const passwordInput = wrapper.find('#login-password').element as HTMLInputElement
    expect(emailInput.disabled).toBe(true)
    expect(passwordInput.disabled).toBe(true)
  })

  it('dismisses error banner when dismiss button is clicked', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const authStore = useAuthStore()
    authStore.error = { type: 'network_error', message: 'Network error.' }

    const wrapper = mount(LoginScreen, { global: { plugins: [pinia] } })
    expect(wrapper.find('.error-banner').exists()).toBe(true)

    await wrapper.find('.error-dismiss').trigger('click')
    await nextTick()
    expect(wrapper.find('.error-banner').exists()).toBe(false)
  })
})
