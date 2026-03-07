import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export type AuthStatus = 'unknown' | 'authenticated' | 'unauthenticated'

export interface AuthError {
  type: 'invalid_credentials' | 'network_error' | 'unknown_error'
  message: string
}

export const useAuthStore = defineStore('auth', () => {
  const status = ref<AuthStatus>('unknown')
  const loading = ref(false)
  const error = ref<AuthError | null>(null)
  const userEmail = ref<string | null>(null)

  const isAuthenticated = computed(() => status.value === 'authenticated')
  const isUnauthenticated = computed(() => status.value === 'unauthenticated')

  async function checkAuth(): Promise<void> {
    try {
      const token = await invoke<string | null>('get_auth_token')
      if (token) {
        status.value = 'authenticated'
        try {
          const payload = JSON.parse(atob(token.split('.')[1]))
          userEmail.value = payload.email ?? null
        } catch {
          // Token may not be a standard JWT
        }
      } else {
        status.value = 'unauthenticated'
      }
    } catch {
      status.value = 'unauthenticated'
    }
  }

  async function login(email: string, password: string): Promise<boolean> {
    loading.value = true
    error.value = null

    try {
      const token = await invoke<string>('auth_login', { email, password })
      await invoke('store_auth_token', { token })
      userEmail.value = email
      status.value = 'authenticated'
      loading.value = false
      return true
    } catch (err) {
      const message = String(err)
      if (message.includes('invalid') || message.includes('credentials') || message.includes('401')) {
        error.value = { type: 'invalid_credentials', message: 'Invalid email or password.' }
      } else if (message.includes('network') || message.includes('connect') || message.includes('timeout')) {
        error.value = { type: 'network_error', message: 'Network error. Please check your connection and try again.' }
      } else {
        error.value = { type: 'unknown_error', message: 'An unexpected error occurred. Please try again.' }
      }
      loading.value = false
      return false
    }
  }

  async function loginWithGoogle(): Promise<boolean> {
    loading.value = true
    error.value = null

    try {
      const token = await invoke<string>('auth_google_sso')
      await invoke('store_auth_token', { token })
      status.value = 'authenticated'
      try {
        const payload = JSON.parse(atob(token.split('.')[1]))
        userEmail.value = payload.email ?? null
      } catch {
        // Not a standard JWT
      }
      loading.value = false
      return true
    } catch (err) {
      const message = String(err)
      if (message.includes('network') || message.includes('connect') || message.includes('timeout')) {
        error.value = { type: 'network_error', message: 'Network error. Please check your connection and try again.' }
      } else {
        error.value = { type: 'unknown_error', message: 'Google sign-in failed. Please try again.' }
      }
      loading.value = false
      return false
    }
  }

  async function logout(): Promise<void> {
    try {
      await invoke('clear_auth_token')
    } catch {
      // Best effort
    }
    status.value = 'unauthenticated'
    userEmail.value = null
    error.value = null
  }

  function clearError(): void {
    error.value = null
  }

  function setUnauthenticated(): void {
    status.value = 'unauthenticated'
  }

  return {
    status,
    loading,
    error,
    userEmail,
    isAuthenticated,
    isUnauthenticated,
    checkAuth,
    login,
    loginWithGoogle,
    logout,
    clearError,
    setUnauthenticated,
  }
})
