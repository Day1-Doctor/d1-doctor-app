<template>
  <div class="ninja-dropdown">
    <div class="query-echo">> {{ query }}</div>
    <StepTimeline :steps="steps" />
    <div class="dropdown-footer">
      <div class="progress-track"><div class="progress-fill" :style="{ width: '30%' }"/></div>
      <button @click="$emit('dismiss')" class="btn-dismiss">✕</button>
    </div>
  </div>
</template>
<script setup lang="ts">
import type { Step } from '@/shared/types'
import StepTimeline from './StepTimeline.vue'
defineProps<{ query: string; steps: Step[] }>()
defineEmits<{ dismiss: [] }>()
</script>
<style scoped>
.ninja-dropdown { width: 680px; background: var(--surface-ninja-dropdown); backdrop-filter: blur(50px) saturate(160%); -webkit-backdrop-filter: blur(50px) saturate(160%); border-radius: 16px; border: 1px solid rgba(255,255,255,0.06); box-shadow: 0 24px 64px rgba(0,0,0,0.8); overflow: hidden; animation: slideDown 0.25s cubic-bezier(0.16, 1, 0.3, 1) forwards; margin-top: 4px; }
.query-echo { padding: 14px 20px 8px; font: 13px var(--font-mono); color: var(--text-muted); }
.dropdown-footer { display: flex; align-items: center; gap: 12px; padding: 12px 20px; border-top: 1px solid var(--border); }
.progress-track { flex: 1; height: 3px; background: var(--border); border-radius: 2px; overflow: hidden; }
.progress-fill { height: 100%; background: var(--accent); border-radius: 2px; }
.btn-dismiss { background: none; border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 5px 12px; color: var(--text-secondary); font: 12px var(--font-mono); cursor: pointer; }
</style>
