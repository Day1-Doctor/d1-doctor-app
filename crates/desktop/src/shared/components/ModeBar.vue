<template>
  <div class="mode-bar">
    <button
      v-for="m in modes"
      :key="m.id"
      class="mode-bar-btn"
      :class="{ active: appStore.uiMode === m.id }"
      :title="m.label"
      @click="appStore.switchMode(m.id)"
    >
      {{ m.icon }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { useAppStore } from '@/shared/stores/app'

const appStore = useAppStore()

const modes = [
  { id: 'full' as const,    icon: '⊞', label: 'Full mode'    },
  { id: 'copilot' as const, icon: '◫', label: 'Copilot mode' },
  { id: 'ninja' as const,   icon: '◈', label: 'Ninja mode'   },
]
</script>

<style scoped>
.mode-bar {
  display: flex;
  gap: 2px;
  background: var(--surface-2, rgba(0,0,0,0.3));
  border-radius: var(--radius-sm, 4px);
  padding: 2px;
}

.mode-bar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  background: transparent;
  border: none;
  border-radius: 3px;
  color: var(--text-disabled);
  cursor: pointer;
  font-size: 12px;
  transition: background 0.1s, color 0.1s;
}

.mode-bar-btn:hover:not(.active) {
  background: var(--muted);
  color: var(--text-primary);
}

.mode-bar-btn.active {
  background: var(--accent-soft, rgba(99,102,241,0.15));
  color: var(--accent, #6366f1);
}
</style>
