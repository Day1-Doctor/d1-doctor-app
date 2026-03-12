<template>
  <div class="copilot-input">
    <div class="input-pill" :class="{ focused: isFocused }">
      <textarea
        ref="textareaEl"
        v-model="inputText"
        placeholder="Ask anything..."
        rows="1"
        @keydown="onKeydown"
        @input="autoResize"
        @focus="isFocused = true"
        @blur="isFocused = false"
      />
      <button
        class="send-btn"
        :disabled="!inputText.trim() || daemonStore.status !== 'connected'"
        @click="submitMessage"
        title="Send"
        aria-label="Send message"
      >
        ↩
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'
import { useConversationStore } from '@/shared/stores/conversation'
import { useDaemonConnection } from '@/shared/composables/useDaemonConnection'
import { useDaemonStore } from '@/shared/stores/daemon'

const conversationStore = useConversationStore()
const daemonStore = useDaemonStore()
const { submitTask } = useDaemonConnection()

const textareaEl = ref<HTMLTextAreaElement | null>(null)
const inputText = ref('')
const isFocused = ref(false)

const MAX_TEXTAREA_HEIGHT_PX = 50 // must match CSS max-height on .copilot-textarea

function submitMessage(): void {
  const text = inputText.value.trim()
  if (!text) return
  if (daemonStore.status !== 'connected') return
  conversationStore.appendMessage({
    id: crypto.randomUUID(),
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
  }
  // Shift+Enter: default browser behavior inserts newline — no interception needed
}

function autoResize(): void {
  if (!textareaEl.value) return
  textareaEl.value.style.height = 'auto'
  textareaEl.value.style.height = Math.min(textareaEl.value.scrollHeight, MAX_TEXTAREA_HEIGHT_PX) + 'px'
}
</script>

<style scoped>
.copilot-input {
  padding: var(--space-sm) var(--space-md);
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

.input-pill {
  background: var(--muted);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: var(--space-sm) var(--space-md);
  display: flex;
  align-items: flex-end;
  gap: var(--space-sm);
  transition: border-color var(--duration-fast), box-shadow var(--duration-fast);
}

.input-pill.focused {
  border-color: var(--accent-border);
  box-shadow: 0 0 0 2px var(--accent-soft);
}

textarea {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: var(--text-primary);
  font: 12px/1.5 var(--font-mono);
  resize: none;
  max-height: 50px;
  min-height: 18px;
  overflow-y: auto;
}

textarea::placeholder {
  color: var(--text-placeholder);
}

.send-btn {
  background: var(--accent);
  border: none;
  border-radius: var(--radius-sm);
  width: var(--space-xl);
  height: var(--space-xl);
  color: var(--text-on-accent);
  cursor: pointer;
  font-size: var(--font-size-base);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: background var(--duration-fast), opacity var(--duration-fast);
  padding: 0;
  line-height: 1;
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
