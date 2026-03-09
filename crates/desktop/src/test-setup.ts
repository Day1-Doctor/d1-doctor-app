/**
 * Vitest setup file — registers global plugins so that component tests
 * work without manually providing vue-i18n in every mount() call.
 */
import { config } from '@vue/test-utils'
import { createI18n } from 'vue-i18n'
import en from '@/locales/en.json'
import zhCN from '@/locales/zh-CN.json'

const i18n = createI18n({
  legacy: false,
  locale: 'en',
  fallbackLocale: 'en',
  messages: { en, 'zh-CN': zhCN },
})

config.global.plugins.push(i18n)
