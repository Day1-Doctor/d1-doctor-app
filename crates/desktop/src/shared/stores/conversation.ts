import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Message, Plan, Step } from '@/shared/types'

export const useConversationStore = defineStore('conversation', () => {
  const messages = ref<Message[]>([])
  const currentPlan = ref<Plan | null>(null)
  const scrollPinned = ref(true)
  function appendMessage(_msg: Message): void {}
  function setPlan(_plan: Plan): void {}
  function updateStep(_stepId: string, _state: Step['state']): void {}
  return { messages, currentPlan, scrollPinned, appendMessage, setPlan, updateStep }
})
