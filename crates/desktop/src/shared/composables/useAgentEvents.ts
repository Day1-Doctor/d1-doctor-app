export function useAgentEvents() {
  async function startListening(): Promise<void> {}
  function stopListening(): void {}
  return { startListening, stopListening }
}
