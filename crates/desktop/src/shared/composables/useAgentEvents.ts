import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { AgentEvent } from '@/shared/types'

type AgentEventHandler = (event: AgentEvent) => void

export function useAgentEvents() {
  let unlisten: UnlistenFn | null = null
  const handlers: AgentEventHandler[] = []

  async function startListening(): Promise<void> {
    if (unlisten) return // already listening
    unlisten = await listen<AgentEvent>('agent_event', ({ payload }) => {
      handlers.forEach(h => h(payload))
    })
  }

  function stopListening(): void {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
    handlers.length = 0
  }

  function onEvent(handler: AgentEventHandler): void {
    handlers.push(handler)
  }

  return { startListening, stopListening, onEvent }
}
