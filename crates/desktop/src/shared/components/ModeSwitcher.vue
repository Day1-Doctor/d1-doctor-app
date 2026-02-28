<template>
  <div class="mode-switcher">
    <div class="mode-switcher-label">MODE</div>
    <div class="mode-btns">
      <button
        v-for="m in modes"
        :key="m.id"
        class="mode-btn"
        :class="{ active: appStore.uiMode === m.id }"
        :title="m.label"
        @click="appStore.switchMode(m.id)"
      >
        <span class="mode-icon">{{ m.icon }}</span>
        <span class="mode-label">{{ m.label }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useAppStore } from '@/shared/stores/app'

const appStore = useAppStore()

// Use the actual UIMode type from the store
const modes = [
  { id: 'full' as const,    icon: '⊞',  label: 'Full'    },
  { id: 'copilot' as const, icon: '◫',  label: 'Copilot' },
  { id: 'ninja' as const,   icon: '◈',  label: 'Ninja'   },
]
</script>

<style scoped>
.mode-switcher {
  padding: 8px 16px;
}

.mode-switcher-label {
  font: 700 10px var(--font-mono, monospace);
  color: var(--text-disabled);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  margin-bottom: 6px;
}

.mode-btns {
  display: flex;
  gap: 4px;
}

.mode-btn {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  padding: 6px 4px;
  background: transparent;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm, 4px);
  color: var(--text-secondary);
  cursor: pointer;
  transition: background 0.12s, color 0.12s, border-color 0.12s;
  font-family: var(--font-mono, monospace);
}

.mode-btn:hover:not(.active) {
  background: var(--muted);
  color: var(--text-primary);
}

.mode-btn.active {
  background: var(--accent-soft);
  border-color: var(--accent-border);
  color: var(--accent);
}

.mode-icon {
  font-size: 14px;
  line-height: 1;
}

.mode-label {
  font-size: 9px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}
</style>
