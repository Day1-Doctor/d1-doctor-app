import { createI18n } from 'vue-i18n'
import en from '@/locales/en.json'
import zhCN from '@/locales/zh-CN.json'

export type MessageSchema = typeof en

/**
 * Detect the user's system locale via the Tauri os plugin.
 * Falls back to navigator.language, then 'en'.
 */
export async function detectLocale(): Promise<string> {
  try {
    const { locale } = await import('@tauri-apps/plugin-os')
    const systemLocale = await locale()
    if (systemLocale) return normalizeLocale(systemLocale)
  } catch {
    // Tauri plugin not available (e.g. in tests or browser dev mode)
  }

  // Fallback: browser API
  if (typeof navigator !== 'undefined' && navigator.language) {
    return normalizeLocale(navigator.language)
  }

  return 'en'
}

/**
 * Map a BCP-47 locale tag to one of our supported locales.
 * Supports: 'en', 'zh-CN'.  Anything else falls back to 'en'.
 */
export function normalizeLocale(tag: string): string {
  const lower = tag.toLowerCase().replace(/_/g, '-')

  // Chinese simplified variants
  if (lower.startsWith('zh-cn') || lower.startsWith('zh-hans') || lower === 'zh') {
    return 'zh-CN'
  }

  // English
  if (lower.startsWith('en')) {
    return 'en'
  }

  // Unrecognized — fallback to English
  return 'en'
}

const i18n = createI18n<[MessageSchema], 'en' | 'zh-CN'>({
  legacy: false,
  locale: 'en', // default; updated asynchronously after mount
  fallbackLocale: 'en',
  messages: {
    en,
    'zh-CN': zhCN,
  },
})

/**
 * Initialize locale detection asynchronously.
 * Call this after the app is mounted.
 */
export async function initLocale(): Promise<void> {
  const detected = await detectLocale()
  i18n.global.locale.value = detected as 'en' | 'zh-CN'
}

export default i18n
