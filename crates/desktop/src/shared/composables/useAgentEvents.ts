import { onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import type { AgentEvent, Message, Plan, CreditInfo } from '@/shared/types'
import { useConversationStore } from '@/shared/stores/conversation'
import { useAgentStore } from '@/shared/stores/agent'

export function useAgentEvents(): void {
  const conversationStore = useConversationStore()
  const agentStore = useAgentStore()
  let unlisten: (() => void) | null = null

  onMounted(async () => {
    try {
      unlisten = await listen<AgentEvent>('agent_event', ({ payload }) => {
        const { type, payload: data } = payload

        // Payload arrives as Record<string, unknown> from Tauri IPC.
        // Each case narrows the payload to the expected shape for the store action.
        // The backend guarantees the payload structure matches these types.
        switch (type) {
          case 'plan_proposed': {
            // Backend ensures payload matches Plan shape
            conversationStore.setPlan(data as unknown as Plan)
            break
          }
          case 'step_started': {
            const { stepId } = data as { stepId: string }
            conversationStore.updateStep(stepId, 'active')
            break
          }
          case 'step_completed': {
            const { stepId } = data as { stepId: string }
            conversationStore.updateStep(stepId, 'done')
            break
          }
          case 'step_failed': {
            const { stepId } = data as { stepId: string }
            conversationStore.updateStep(stepId, 'error')
            break
          }
          case 'agent_message': {
            // Backend ensures payload matches Message shape
            conversationStore.appendMessage(data as unknown as Message)
            break
          }
          case 'result_ready': {
            const { title, detail } = data as { title: string; detail: string }
            const content = detail ? `${title}\n\n${detail}` : title
            conversationStore.appendMessage({
              id: crypto.randomUUID(),
              role: 'agent',
              content,
              timestamp: Date.now(),
            })
            break
          }
          case 'credits_updated': {
            // Backend ensures payload matches CreditInfo shape
            agentStore.updateCredits(data as unknown as CreditInfo)
            break
          }
          case 'permission_requested': {
            const { permission } = data as { permission: string }
            conversationStore.appendMessage({
              id: crypto.randomUUID(),
              role: 'agent',
              content: `Permission requested: ${permission}`,
              timestamp: Date.now(),
            })
            break
          }
        }
      })
    } catch (err) {
      console.error('[useAgentEvents] Failed to register Tauri event listener:', err)
    }
  })

  onUnmounted(() => {
    unlisten?.()
    unlisten = null
  })
}
