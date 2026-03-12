<template>
  <div class="ninja-bar-wrapper">
    <div class="ninja-bar">
      <div class="ninja-logo">dr. D1</div>
      <input
        ref="inputRef"
        v-model="query"
        class="ninja-input"
        placeholder="Ask anything..."
        autofocus
        aria-describedby="ninja-hints"
        @keydown.enter.prevent="submit"
        @keydown.esc.prevent="emit('dismiss')"
        @input="onInput"
      />
      <button class="ninja-send" @click="submit" aria-label="Send">→</button>
    </div>

    <div id="ninja-hints" :class="{ 'hints-hidden': !showHints }" class="ninja-shortcuts">
      <span>⌘⇧D Toggle · Esc Dismiss · ⏎ Send</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const emit = defineEmits<{ submit: [query: string]; dismiss: [] }>()

const inputRef = ref<HTMLInputElement | null>(null)
const query = ref('')
const showHints = ref(true)
let hintsTimer: ReturnType<typeof setTimeout> | null = null

function submit(): void {
  const trimmed = query.value.trim()
  if (!trimmed) return
  emit('submit', trimmed)
  query.value = ''
  // Restore hints when input is cleared
  showHints.value = true
  if (hintsTimer !== null) {
    clearTimeout(hintsTimer)
    hintsTimer = null
  }
}

function onInput(): void {
  if (!query.value) {
    // Input cleared — show hints again
    showHints.value = true
    if (hintsTimer !== null) {
      clearTimeout(hintsTimer)
      hintsTimer = null
    }
    return
  }
  // First keystroke (or after clear): schedule hints fade-out after 3s
  if (showHints.value && hintsTimer === null) {
    hintsTimer = setTimeout(() => {
      showHints.value = false
      hintsTimer = null
    }, 3000)
  }
}

onMounted(() => {
  inputRef.value?.focus()
})

onUnmounted(() => {
  if (hintsTimer !== null) {
    clearTimeout(hintsTimer)
    hintsTimer = null
  }
})
</script>

<style scoped>
.ninja-bar-wrapper {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.ninja-bar {
  width: 680px;
  height: 64px;
  background: var(--surface-ninja-bar);
  backdrop-filter: var(--backdrop-xl);
  -webkit-backdrop-filter: var(--backdrop-xl);
  border-radius: 20px;
  border: 1px solid var(--border-translucent);
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.7), 0 0 0 1px rgba(249, 115, 22, 0.1);
  display: flex;
  align-items: center;
  padding: 0 var(--space-xl);
  gap: var(--space-lg);
}

.ninja-logo {
  width: 40px;
  height: 40px;
  border-radius: var(--space-md);
  background: var(--gradient-accent);
  box-shadow: 0 0 20px rgba(249, 115, 22, 0.1);
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-contrast);
  font-size: var(--font-size-2xl);
  font-weight: var(--font-weight-bold);
  font-family: var(--font-mono);
}

.ninja-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: var(--text-primary);
  font: var(--font-size-xl)/1 var(--font-mono);
  caret-color: var(--accent);
}

.ninja-input::placeholder {
  color: var(--text-placeholder);
}

.ninja-send {
  background: var(--accent);
  border: none;
  border-radius: var(--space-sm);
  width: var(--space-2xl);
  height: var(--space-2xl);
  color: var(--text-on-accent);
  font-size: var(--font-size-xl);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: background var(--duration-fast);
}

.ninja-send:hover {
  background: var(--accent-hover);
  box-shadow: 0 0 16px var(--accent-glow);
}

.ninja-shortcuts {
  position: absolute;
  top: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: var(--space-xl);
  opacity: 0.6;
  font: var(--font-size-sm) var(--font-mono);
  color: var(--text-muted);
  pointer-events: none;
  white-space: nowrap;
  transition: opacity var(--duration-slower) var(--easing-default);
}

.ninja-shortcuts.hints-hidden {
  opacity: 0;
  pointer-events: none;
}

/* Reappear on hover */
.ninja-bar-wrapper:hover .ninja-shortcuts.hints-hidden {
  opacity: 0.6;
}
</style>
