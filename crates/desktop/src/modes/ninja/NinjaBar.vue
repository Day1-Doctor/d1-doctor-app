<template>
  <div class="ninja-bar">
    <div class="ninja-logo">D1</div>
    <input ref="inputRef" v-model="query" class="ninja-input" placeholder="Ask Day 1 Doctor..." @keydown.enter="submit" @keydown.esc="$emit('esc')" autofocus />
    <button class="ninja-send" @click="submit">→</button>
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue'
const emit = defineEmits<{ submit: [query: string]; esc: [] }>()
const query = ref('')
function submit() { if (query.value.trim()) { emit('submit', query.value.trim()); query.value = '' } }
</script>
<style scoped>
.ninja-bar { width: 680px; height: 64px; background: var(--surface-ninja-bar); backdrop-filter: blur(50px) saturate(180%); -webkit-backdrop-filter: blur(50px) saturate(180%); border-radius: 20px; border: 1px solid rgba(255,255,255,0.08); box-shadow: 0 24px 64px rgba(0,0,0,0.7), 0 0 0 1px rgba(249,115,22,0.1); display: flex; align-items: center; padding: 0 20px; gap: 14px; }
.ninja-logo { width: 40px; height: 40px; border-radius: 12px; background: linear-gradient(135deg, var(--accent), var(--accent-hover)); box-shadow: 0 0 20px var(--accent-glow); flex-shrink: 0; display: flex; align-items: center; justify-content: center; color: white; font: 700 18px var(--font-mono); }
.ninja-input { flex: 1; background: transparent; border: none; outline: none; color: var(--text-primary); font: 16px var(--font-mono); caret-color: var(--accent); }
.ninja-input::placeholder { color: var(--text-disabled); }
.ninja-send { background: var(--accent); border: none; border-radius: 8px; width: 32px; height: 32px; color: white; font-size: 16px; cursor: pointer; display: flex; align-items: center; justify-content: center; }
</style>
