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
        :disabled="!inputText.trim()"
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

const conversationStore = useConversationStore()

const textareaEl = ref<HTMLTextAreaElement | null>(null)
const inputText = ref('')
const isFocused = ref(false)

const MAX_TEXTAREA_HEIGHT_PX = 50 // must match CSS max-height on .copilot-textarea

function submitMessage(): void {
  const text = inputText.value.trim()
  if (!text) return
  conversationStore.appendMessage({
    id: crypto.randomUUID(),
    role: 'user',
    content: text,
    timestamp: Date.now(),
  })
  inputText.value = ''
  nextTick(() => {
    if (textareaEl.value) {
      textareaEl.value.style.height = 'auto'
    }
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
  padding: 8px 12px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

.input-pill {
  background: var(--muted);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 8px 12px;
  display: flex;
  align-items: flex-end;
  gap: 8px;
  transition: border-color 0.15s, box-shadow 0.15s;
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
  color: var(--text-disabled);
}

.send-btn {
  background: var(--accent);
  border: none;
  border-radius: var(--radius-sm);
  width: 24px;
  height: 24px;
  color: #000;
  cursor: pointer;
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: background 0.15s, opacity 0.15s;
  padding: 0;
  line-height: 1;
}

.send-btn:hover:not(:disabled) {
  background: var(--accent-hover);
}

.send-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}
</style>
