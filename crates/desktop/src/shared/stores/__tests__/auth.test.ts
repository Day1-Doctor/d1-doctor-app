import { describe, it, expect, vi, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

const { mockInvoke } = vi.hoisted(() => {
  const mockInvoke = vi.fn().mockResolvedValue(undefined)
  return { mockInvoke }
})

vi.mock('@tauri-apps/api/core', () => ({ invoke: mockInvoke }))

import { useAuthStore } from '../auth'

describe('auth store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    mockInvoke.mockReset()
  })

  describe('checkAuth', () => {
    it('sets authenticated when token exists', async () => {
      const fakePayload = btoa(JSON.stringify({ email: 'test@example.com' }))
      const fakeToken = 'header.' + fakePayload + '.signature'
      mockInvoke.mockResolvedValueOnce(fakeToken)
      const store = useAuthStore()
      await store.checkAuth()
      expect(store.status).toBe('authenticated')
      expect(store.userEmail).toBe('test@example.com')
    })

    it('sets unauthenticated when no token', async () => {
      mockInvoke.mockResolvedValueOnce(null)
      const store = useAuthStore()
      await store.checkAuth()
      expect(store.status).toBe('unauthenticated')
    })

    it('sets unauthenticated on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('keyring unavailable'))
      const store = useAuthStore()
      await store.checkAuth()
      expect(store.status).toBe('unauthenticated')
    })
  })

  describe('login', () => {
    it('stores token and sets authenticated on success', async () => {
      mockInvoke.mockResolvedValueOnce('jwt-token-here').mockResolvedValueOnce(undefined)
      const store = useAuthStore()
      const result = await store.login('test@example.com', 'password')
      expect(result).toBe(true)
      expect(store.status).toBe('authenticated')
      expect(store.userEmail).toBe('test@example.com')
      expect(store.loading).toBe(false)
    })

    it('sets invalid_credentials error on 401-like failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('invalid credentials'))
      const store = useAuthStore()
      const result = await store.login('test@example.com', 'wrong')
      expect(result).toBe(false)
      expect(store.error?.type).toBe('invalid_credentials')
    })

    it('sets network_error on connection failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('network timeout'))
      const store = useAuthStore()
      const result = await store.login('test@example.com', 'pass')
      expect(result).toBe(false)
      expect(store.error?.type).toBe('network_error')
    })

    it('sets unknown_error on unexpected failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('something weird'))
      const store = useAuthStore()
      const result = await store.login('test@example.com', 'pass')
      expect(result).toBe(false)
      expect(store.error?.type).toBe('unknown_error')
    })
  })

  describe('loginWithGoogle', () => {
    it('stores token and sets authenticated on success', async () => {
      const fakePayload = btoa(JSON.stringify({ email: 'google@example.com' }))
      const fakeToken = 'header.' + fakePayload + '.signature'
      mockInvoke.mockResolvedValueOnce(fakeToken).mockResolvedValueOnce(undefined)
      const store = useAuthStore()
      const result = await store.loginWithGoogle()
      expect(result).toBe(true)
      expect(store.status).toBe('authenticated')
      expect(store.userEmail).toBe('google@example.com')
    })

    it('sets error on failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('popup closed'))
      const store = useAuthStore()
      const result = await store.loginWithGoogle()
      expect(result).toBe(false)
      expect(store.error?.type).toBe('unknown_error')
    })
  })

  describe('logout', () => {
    it('clears token and sets unauthenticated', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      const store = useAuthStore()
      store.status = 'authenticated'
      store.userEmail = 'test@example.com'
      await store.logout()
      expect(store.status).toBe('unauthenticated')
      expect(store.userEmail).toBeNull()
    })

    it('still clears state even if clear_auth_token fails', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('keyring error'))
      const store = useAuthStore()
      store.status = 'authenticated'
      await store.logout()
      expect(store.status).toBe('unauthenticated')
    })
  })

  describe('setUnauthenticated', () => {
    it('sets status to unauthenticated', () => {
      const store = useAuthStore()
      store.status = 'authenticated'
      store.setUnauthenticated()
      expect(store.status).toBe('unauthenticated')
    })
  })

  describe('clearError', () => {
    it('clears the error', () => {
      const store = useAuthStore()
      store.error = { type: 'network_error', message: 'oops' }
      store.clearError()
      expect(store.error).toBeNull()
    })
  })

  describe('computed properties', () => {
    it('isAuthenticated is true when authenticated', () => {
      const store = useAuthStore()
      store.status = 'authenticated'
      expect(store.isAuthenticated).toBe(true)
      expect(store.isUnauthenticated).toBe(false)
    })

    it('isUnauthenticated is true when unauthenticated', () => {
      const store = useAuthStore()
      store.status = 'unauthenticated'
      expect(store.isUnauthenticated).toBe(true)
      expect(store.isAuthenticated).toBe(false)
    })
  })
})
