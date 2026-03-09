import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock the Tauri os plugin before importing the module under test
vi.mock('@tauri-apps/plugin-os', () => ({
  locale: vi.fn(),
}))

import { normalizeLocale, detectLocale } from '../i18n'

describe('normalizeLocale', () => {
  it('returns "en" for en-US', () => {
    expect(normalizeLocale('en-US')).toBe('en')
  })

  it('returns "en" for en-GB', () => {
    expect(normalizeLocale('en-GB')).toBe('en')
  })

  it('returns "en" for plain "en"', () => {
    expect(normalizeLocale('en')).toBe('en')
  })

  it('returns "zh-CN" for zh-CN', () => {
    expect(normalizeLocale('zh-CN')).toBe('zh-CN')
  })

  it('returns "zh-CN" for zh_CN (underscore)', () => {
    expect(normalizeLocale('zh_CN')).toBe('zh-CN')
  })

  it('returns "zh-CN" for zh-Hans', () => {
    expect(normalizeLocale('zh-Hans')).toBe('zh-CN')
  })

  it('returns "zh-CN" for zh-Hans-CN', () => {
    expect(normalizeLocale('zh-Hans-CN')).toBe('zh-CN')
  })

  it('returns "zh-CN" for bare "zh"', () => {
    expect(normalizeLocale('zh')).toBe('zh-CN')
  })

  it('falls back to "en" for unrecognized locale (fr-FR)', () => {
    expect(normalizeLocale('fr-FR')).toBe('en')
  })

  it('falls back to "en" for unrecognized locale (ja-JP)', () => {
    expect(normalizeLocale('ja-JP')).toBe('en')
  })

  it('is case-insensitive', () => {
    expect(normalizeLocale('EN-US')).toBe('en')
    expect(normalizeLocale('ZH-CN')).toBe('zh-CN')
    expect(normalizeLocale('Zh-Hans')).toBe('zh-CN')
  })
})

describe('detectLocale', () => {
  beforeEach(() => {
    vi.resetModules()
  })

  it('returns "en" when Tauri plugin throws and navigator.language is en-US', async () => {
    const { locale: mockLocale } = await import('@tauri-apps/plugin-os')
    ;(mockLocale as ReturnType<typeof vi.fn>).mockRejectedValue(new Error('not available'))

    Object.defineProperty(navigator, 'language', {
      value: 'en-US',
      configurable: true,
    })

    const result = await detectLocale()
    expect(result).toBe('en')
  })

  it('returns "zh-CN" when Tauri plugin returns zh-CN', async () => {
    const { locale: mockLocale } = await import('@tauri-apps/plugin-os')
    ;(mockLocale as ReturnType<typeof vi.fn>).mockResolvedValue('zh-CN')

    const result = await detectLocale()
    expect(result).toBe('zh-CN')
  })

  it('returns "zh-CN" when Tauri plugin returns zh_Hans_CN', async () => {
    const { locale: mockLocale } = await import('@tauri-apps/plugin-os')
    ;(mockLocale as ReturnType<typeof vi.fn>).mockResolvedValue('zh_Hans_CN')

    const result = await detectLocale()
    expect(result).toBe('zh-CN')
  })

  it('falls back to "en" for unsupported Tauri locale', async () => {
    const { locale: mockLocale } = await import('@tauri-apps/plugin-os')
    ;(mockLocale as ReturnType<typeof vi.fn>).mockResolvedValue('de-DE')

    const result = await detectLocale()
    expect(result).toBe('en')
  })

  it('falls back to "en" when Tauri returns null and navigator is unavailable', async () => {
    const { locale: mockLocale } = await import('@tauri-apps/plugin-os')
    ;(mockLocale as ReturnType<typeof vi.fn>).mockResolvedValue(null)

    Object.defineProperty(navigator, 'language', {
      value: '',
      configurable: true,
    })

    const result = await detectLocale()
    expect(result).toBe('en')
  })
})
