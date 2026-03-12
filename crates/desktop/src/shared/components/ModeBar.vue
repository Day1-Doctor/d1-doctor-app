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
  gap: var(--space-2xs);
  background: var(--muted);
  border-radius: var(--radius-sm, 4px);
  padding: var(--space-2xs);
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
  font-size: var(--font-size-base);
  transition: background var(--duration-instant), color var(--duration-instant);
}

.mode-bar-btn:hover:not(.active) {
  background: var(--muted);
  color: var(--text-primary);
}

.mode-bar-btn.active {
  background: var(--accent-soft);
  color: var(--accent);
}
</style>
