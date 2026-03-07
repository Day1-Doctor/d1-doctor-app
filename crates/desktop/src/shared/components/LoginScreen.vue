<template>
  <div class="login-screen">
    <div class="login-card">
      <div class="login-header">
        <div class="logo-mark">D1</div>
        <h1 class="app-title">{{ $t('login.title') }}</h1>
        <p class="app-subtitle">{{ $t('login.subtitle') }}</p>
      </div>
      <Transition name="error-fade">
        <div v-if="authStore.error" class="error-banner" role="alert">
          <span class="error-icon">!</span>
          <span class="error-message">{{ authStore.error.message }}</span>
          <button class="error-dismiss" @click="authStore.clearError()" :aria-label="$t('login.dismissError')">&#x2715;</button>
        </div>
      </Transition>
      <form class="login-form" @submit.prevent="handleSubmit" novalidate>
        <div class="field" :class="{ 'field--error': emailError }">
          <label for="login-email" class="field-label">{{ $t('login.emailLabel') }}</label>
          <input id="login-email" v-model.trim="email" type="email" class="field-input" :placeholder="$t('login.emailPlaceholder')" autocomplete="email" :disabled="authStore.loading" @blur="validateEmail" @input="clearFieldError('email')" />
          <p v-if="emailError" class="field-error">{{ emailError }}</p>
        </div>
        <div class="field" :class="{ 'field--error': passwordError }">
          <label for="login-password" class="field-label">{{ $t('login.passwordLabel') }}</label>
          <input id="login-password" v-model="password" type="password" class="field-input" :placeholder="$t('login.passwordPlaceholder')" autocomplete="current-password" :disabled="authStore.loading" @blur="validatePassword" @input="clearFieldError('password')" />
          <p v-if="passwordError" class="field-error">{{ passwordError }}</p>
        </div>
        <button type="submit" class="btn btn-primary" :disabled="authStore.loading">
          <span v-if="authStore.loading" class="spinner" aria-hidden="true"></span>
          <span v-if="authStore.loading">{{ $t('login.signingIn') }}</span>
          <span v-else>{{ $t('login.signIn') }}</span>
        </button>
      </form>
      <div class="divider">
        <span class="divider-line"></span>
        <span class="divider-text">{{ $t('login.or') }}</span>
        <span class="divider-line"></span>
      </div>
      <button class="btn btn-google" :disabled="authStore.loading" @click="handleGoogleSSO">
        <svg class="google-icon" viewBox="0 0 24 24" width="18" height="18" aria-hidden="true">
          <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 01-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z" fill="#4285F4"/>
          <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/>
          <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/>
          <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/>
        </svg>
        <span>{{ $t('login.continueWithGoogle') }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/shared/stores/auth'

const { t } = useI18n()
const authStore = useAuthStore()

const email = ref('')
const password = ref('')
const emailError = ref('')
const passwordError = ref('')

const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/

function validateEmail(): boolean {
  if (!email.value) {
    emailError.value = t('login.emailRequired')
    return false
  }
  if (!EMAIL_REGEX.test(email.value)) {
    emailError.value = t('login.emailInvalid')
    return false
  }
  emailError.value = ''
  return true
}

function validatePassword(): boolean {
  if (!password.value) {
    passwordError.value = t('login.passwordRequired')
    return false
  }
  passwordError.value = ''
  return true
}

function clearFieldError(field: 'email' | 'password') {
  if (field === 'email') emailError.value = ''
  if (field === 'password') passwordError.value = ''
}

async function handleSubmit() {
  const emailValid = validateEmail()
  const passwordValid = validatePassword()
  if (!emailValid || !passwordValid) return
  await authStore.login(email.value, password.value)
}

async function handleGoogleSSO() {
  await authStore.loginWithGoogle()
}
</script>

<style scoped>
.login-screen {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  background: var(--background);
  color: var(--text-primary);
  font-family: var(--font-mono);
  padding: 24px;
}
.login-card {
  width: 100%;
  max-width: 380px;
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 40px 32px;
}
.login-header { text-align: center; margin-bottom: 32px; }
.logo-mark {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  border-radius: var(--radius-md);
  background: var(--accent);
  color: #fff;
  font-size: 20px;
  font-weight: 700;
  letter-spacing: -0.5px;
  margin-bottom: 16px;
}
.app-title {
  margin: 0 0 4px;
  font-size: 22px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.3px;
}
.app-subtitle { margin: 0; font-size: 13px; color: var(--text-secondary); }
.error-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  margin-bottom: 20px;
  background: var(--error-soft);
  border: 1px solid var(--error-border);
  border-radius: var(--radius-sm);
  font-size: 12px;
  color: var(--error);
}
.error-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--error);
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  flex-shrink: 0;
}
.error-message { flex: 1; }
.error-dismiss {
  background: none;
  border: none;
  color: var(--error);
  cursor: pointer;
  padding: 0 2px;
  font-size: 12px;
  opacity: 0.7;
  transition: opacity 0.15s;
}
.error-dismiss:hover { opacity: 1; }
.error-fade-enter-active,
.error-fade-leave-active { transition: opacity 0.2s ease, transform 0.2s ease; }
.error-fade-enter-from,
.error-fade-leave-to { opacity: 0; transform: translateY(-4px); }
.login-form { display: flex; flex-direction: column; gap: 16px; }
.field { display: flex; flex-direction: column; gap: 6px; }
.field-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  letter-spacing: 0.3px;
}
.field-input {
  width: 100%;
  padding: 10px 12px;
  background: var(--muted);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 13px;
  outline: none;
  transition: border-color 0.15s, box-shadow 0.15s;
  box-sizing: border-box;
}
.field-input::placeholder { color: var(--text-disabled); }
.field-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-glow);
}
.field-input:disabled { opacity: 0.5; cursor: not-allowed; }
.field--error .field-input { border-color: var(--error); }
.field--error .field-input:focus { box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.2); }
.field-error { margin: 0; font-size: 11px; color: var(--error); }
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 11px 16px;
  border: none;
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s, opacity 0.15s, box-shadow 0.15s;
}
.btn:disabled { opacity: 0.55; cursor: not-allowed; }
.btn-primary { background: var(--accent); color: #fff; }
.btn-primary:hover:not(:disabled) {
  background: var(--accent-hover);
  box-shadow: 0 0 12px var(--accent-glow);
}
.btn-google {
  background: var(--muted);
  color: var(--text-primary);
  border: 1px solid var(--border);
}
.btn-google:hover:not(:disabled) { background: var(--border); }
.google-icon { flex-shrink: 0; }
.divider { display: flex; align-items: center; gap: 12px; margin: 20px 0; }
.divider-line { flex: 1; height: 1px; background: var(--border); }
.divider-text {
  font-size: 11px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.spinner {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
