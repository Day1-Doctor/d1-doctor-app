<template>
  <div class="chat-workspace">
    <!-- Message list -->
    <div class="message-list" ref="listEl" @scroll="onScroll">
      <div v-if="conversationStore.messages.length === 0 && !conversationStore.currentPlan" class="empty-state">
        <div class="empty-icon">💬</div>
        <div class="empty-title">Start a conversation</div>
        <div class="empty-subtitle">Ask Day1 Doctor anything to get started.</div>
      </div>

      <template v-else>
        <MessageBubble
          v-for="msg in conversationStore.messages"
          :key="msg.id"
          :role="msg.role"
          :content="msg.content"
          :timestamp="msg.timestamp"
        />
        <PlanCard
          v-if="conversationStore.currentPlan"
          :steps="conversationStore.currentPlan.steps"
          @approve="onApprove"
          @reject="onReject"
        />
      </template>
    </div>

    <!-- New messages badge -->
    <div v-if="!conversationStore.scrollPinned" class="new-messages-badge" @click="scrollToBottom">
      ↓ New messages
    </div>

    <!-- Disconnection banner -->
    <div
      v-if="daemonStore.status !== 'connected'"
      class="disconnection-banner"
    >
      ● Daemon offline — trying to reconnect…
    </div>

    <!-- Input bar -->
    <div class="input-bar">
      <div class="input-pill">
        <textarea
          class="chat-textarea"
          ref="textareaEl"
          v-model="inputText"
          placeholder="Ask anything..."
          rows="1"
          @keydown="onKeydown"
          @input="autoResize"
        />
        <button
          class="send-btn"
          :disabled="!inputText.trim() || daemonStore.status !== 'connected'"
          :title="daemonStore.status !== 'connected' ? 'Daemon offline' : 'Send'"
          @click="submitMessage"
        >
          ↩
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { useConversationStore } from '@/shared/stores/conversation'
import { useDaemonConnection } from '@/shared/composables/useDaemonConnection'
import { useDaemonStore } from '@/shared/stores/daemon'
import MessageBubble from '@/shared/components/MessageBubble.vue'
import PlanCard from '@/shared/components/PlanCard.vue'

const SCROLL_UNPIN_THRESHOLD_PX = 100
const TEXTAREA_MAX_HEIGHT_PX = 160

const conversationStore = useConversationStore()
const daemonStore = useDaemonStore()
const { submitTask, approvePlan } = useDaemonConnection()

const listEl = ref<HTMLDivElement | null>(null)
const textareaEl = ref<HTMLTextAreaElement | null>(null)
const inputText = ref('')
const lastMessage = ref('')

// Auto-scroll: watch messages and scroll to bottom if pinned
watch(
  () => conversationStore.messages.length,
  () => {
    if (conversationStore.scrollPinned) {
      nextTick(scrollToBottom)
    }
  }
)

function scrollToBottom(): void {
  if (listEl.value) {
    listEl.value.scrollTop = listEl.value.scrollHeight
    conversationStore.setScrollPinned(true)
  }
}

function onScroll(): void {
  if (!listEl.value) return
  const el = listEl.value
  const distFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight
  conversationStore.setScrollPinned(distFromBottom <= SCROLL_UNPIN_THRESHOLD_PX)
}

function submitMessage(): void {
  const text = inputText.value.trim()
  if (!text) return
  if (daemonStore.status !== 'connected') return  // guard: don't submit when offline
  lastMessage.value = text
  conversationStore.appendMessage({
    id: Date.now().toString(),
    role: 'user',
    content: text,
    timestamp: Date.now(),
  })
  submitTask(text)
  inputText.value = ''
  nextTick(() => {
    if (textareaEl.value) textareaEl.value.style.height = 'auto'
  })
}

function onKeydown(e: KeyboardEvent): void {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    submitMessage()
  } else if (e.key === 'ArrowUp' && !inputText.value) {
    // Recall last sent message
    inputText.value = lastMessage.value
    nextTick(() => {
      if (textareaEl.value) {
        textareaEl.value.selectionStart = textareaEl.value.value.length
        textareaEl.value.selectionEnd = textareaEl.value.value.length
      }
    })
  }
  // Shift+Enter: default browser behavior inserts newline — no interception needed
}

function autoResize(): void {
  if (!textareaEl.value) return
  textareaEl.value.style.height = 'auto'
  textareaEl.value.style.height = Math.min(textareaEl.value.scrollHeight, TEXTAREA_MAX_HEIGHT_PX) + 'px'
}

function onApprove(): void {
  conversationStore.approvePlan(true)
  const taskId = daemonStore.currentTaskId
  const planId = daemonStore.currentPlanId
  if (taskId && planId) {
    approvePlan(taskId, planId, 'APPROVE')
  }
}

function onReject(): void {
  conversationStore.approvePlan(false)
  const taskId = daemonStore.currentTaskId
  const planId = daemonStore.currentPlanId
  if (taskId && planId) {
    approvePlan(taskId, planId, 'REJECT')
  }
}
</script>

<style scoped>
.chat-workspace {
  flex: 1;
  background: var(--surface-chat);
  backdrop-filter: var(--backdrop-sm);
  -webkit-backdrop-filter: var(--backdrop-sm);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
}

.message-list {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-xl);
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.message-list::-webkit-scrollbar {
  width: 4px;
}

.message-list::-webkit-scrollbar-track {
  background: transparent;
}

.message-list::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 2px;
}

/* Empty state */
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-sm);
  padding: var(--space-3xl) var(--space-xl);
  color: var(--text-disabled);
}

.empty-icon {
  font-size: 32px;
  margin-bottom: var(--space-sm);
}

.empty-title {
  font: var(--font-weight-semibold) var(--font-size-lg) var(--font-mono);
  color: var(--text-muted);
}

.empty-subtitle {
  font: var(--font-size-base) var(--font-mono);
  color: var(--text-disabled);
  text-align: center;
}

/* New messages badge */
.new-messages-badge {
  position: absolute;
  bottom: 88px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--accent);
  color: var(--text-on-accent);
  font: var(--font-weight-bold) var(--font-size-sm) var(--font-mono);
  padding: var(--space-xs) var(--space-lg);
  border-radius: 999px;
  cursor: pointer;
  user-select: none;
  z-index: 10;
  box-shadow: 0 2px 12px var(--accent-glow);
  /* fadeIn keyframe defined globally in src/shared/styles/animations.css */
  animation: fadeIn var(--duration-fast) var(--easing-default);
  transition: opacity 0.15s;
}

.new-messages-badge:hover {
  opacity: 0.85;
}

.disconnection-banner {
  padding: var(--space-xs) var(--space-lg);
  background: var(--error-soft);
  border-top: 1px solid var(--error-border);
  color: var(--error);
  font: var(--font-size-sm) var(--font-mono, monospace);
  flex-shrink: 0;
  text-align: center;
}

/* Input bar */
.input-bar {
  padding: var(--space-lg) var(--space-xl);
  background: rgba(13, 13, 13, 0.6);
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

.input-pill {
  background: var(--muted);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: var(--space-md) var(--space-lg);
  display: flex;
  align-items: flex-end;
  gap: var(--space-md);
  transition: border-color var(--duration-fast), box-shadow var(--duration-fast);
}

.input-pill:focus-within {
  border-color: var(--accent-border);
  box-shadow: 0 0 0 3px var(--accent-soft);
}

.chat-textarea {
  flex: 1;
  background: transparent;
  border: none;
  color: var(--text-primary);
  font: 13px/1.6 var(--font-mono);
  resize: none;
  outline: none;
  max-height: 160px;
  min-height: 20px;
  overflow-y: auto;
}

.chat-textarea::placeholder {
  color: var(--text-placeholder);
}

.send-btn {
  background: var(--accent);
  border: none;
  color: var(--text-on-accent);
  font-size: var(--font-size-lg);
  width: 28px;
  height: 28px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: background var(--duration-fast), opacity var(--duration-fast);
  padding: 0;
}

.send-btn:hover:not(:disabled) {
  background: var(--accent-hover);
  box-shadow: 0 0 16px var(--accent-glow);
}

.send-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}
</style>
