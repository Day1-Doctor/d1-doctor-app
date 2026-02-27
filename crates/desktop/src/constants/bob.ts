// Day 1 Doctor — Dr. Bob loading state phrases
// Format: "Bob is [a/an] {noun}, {pronoun} is {verb+ing} …"
// Selected randomly per step_started event. Static array — no backend involvement.
// Spec: progress/v2.4.1.md §Dr. Bob Character Loading States

export const BOB_LOADING_PHRASES: ReadonlyArray<string> = [
  'Bob is a handyman, he is troubleshooting …',
  'Bob is a surgeon, he is operating …',
  'Bob is a detective, he is investigating …',
  'Bob is a chef, he is cooking …',
  'Bob is a mountaineer, he is climbing …',
  'Bob is a cartographer, he is mapping …',
  'Bob is a diver, he is diving …',
  'Bob is an archaeologist, he is excavating …',
  'Bob is a lighthouse keeper, he is scanning …',
  'Bob is a jazz musician, he is improvising …',
  'Bob is a beekeeper, he is managing …',
  'Bob is a telescope operator, he is focusing …',
  'Bob is a submarine captain, he is navigating …',
  'Bob is a geologist, he is analyzing …',
  'Bob is a falconer, he is scouting …',
  'Bob is an air traffic controller, he is coordinating …',
  'Bob is a chess grandmaster, he is calculating …',
  'Bob is a locksmith, he is unlocking …',
  'Bob is a botanist, he is cultivating …',
  'Bob is a watchmaker, he is assembling …',
  'Bob is a navigator, he is charting …',
  'Bob is an astronomer, he is observing …',
] as const

/**
 * Returns a randomly selected Bob loading phrase.
 * Called on each `step_started` event — a new phrase per step, not per session.
 */
export function getRandomBobPhrase(): string {
  return BOB_LOADING_PHRASES[Math.floor(Math.random() * BOB_LOADING_PHRASES.length)]
}
