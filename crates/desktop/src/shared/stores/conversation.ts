import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Message, Plan, Step } from '@/shared/types'

export const useConversationStore = defineStore('conversation', () => {
  const messages = ref<Message[]>([])
  const currentPlan = ref<Plan | null>(null)
  const scrollPinned = ref(true)

  function appendMessage(msg: Message): void {
    messages.value.push(msg)
  }

  function setPlan(plan: Plan): void {
    currentPlan.value = plan
  }

  function updateStep(stepId: string, state: Step['state']): void {
    if (!currentPlan.value) return
    const step = currentPlan.value.steps.find(s => s.id === stepId)
    if (step) step.state = state
  }

  function clearMessages(): void {
    messages.value = []
    currentPlan.value = null
    scrollPinned.value = true
  }

  function setScrollPinned(pinned: boolean): void {
    scrollPinned.value = pinned
  }

  function approvePlan(approved: boolean): void {
    if (!currentPlan.value) return
    currentPlan.value = { ...currentPlan.value, approved }
  }

  function clearPlan(): void {
    currentPlan.value = null
  }

  return { messages, currentPlan, scrollPinned, appendMessage, setPlan, updateStep, clearMessages, setScrollPinned, approvePlan, clearPlan }
})
