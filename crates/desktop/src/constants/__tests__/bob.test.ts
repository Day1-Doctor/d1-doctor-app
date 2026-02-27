import { describe, it, expect } from 'vitest'
import { BOB_LOADING_PHRASES, getRandomBobPhrase } from '../bob'

describe('bob loading phrases', () => {
  it('BOB_LOADING_PHRASES has at least 20 entries', () => {
    expect(BOB_LOADING_PHRASES.length).toBeGreaterThanOrEqual(20)
  })

  it('each phrase matches format "Bob is [a/an] {noun}, {pronoun} is {verb+ing} …"', () => {
    for (const phrase of BOB_LOADING_PHRASES) {
      expect(phrase).toMatch(/^Bob is (a|an) .+, (he|she|they) is .+ing …$/)
    }
  })

  it('getRandomBobPhrase() returns a phrase from the list', () => {
    const phrase = getRandomBobPhrase()
    expect(BOB_LOADING_PHRASES).toContain(phrase)
  })

  it('getRandomBobPhrase() called 100 times returns different phrases', () => {
    const results = new Set(Array.from({ length: 100 }, () => getRandomBobPhrase()))
    expect(results.size).toBeGreaterThan(1)
  })
})
