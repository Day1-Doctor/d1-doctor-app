<template>
  <div class="app-window">
    <TitleBar />
    <!-- Mode switcher — always visible top-right -->
    <div class="full-mode-bar">
      <ModeBar />
    </div>
    <div class="main-content">
      <Sidebar />
      <ChatWorkspace />
      <UtilityPanel />
    </div>
  </div>
</template>

<script setup lang="ts">
import TitleBar from './TitleBar.vue'
import Sidebar from './Sidebar.vue'
import ChatWorkspace from './ChatWorkspace.vue'
import UtilityPanel from './UtilityPanel.vue'
import ModeBar from '@/shared/components/ModeBar.vue'
</script>

<style scoped>
.app-window {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--background);
  color: var(--text-primary);
  font-family: var(--font-mono);
  overflow: hidden;
  position: relative;
}

.full-mode-bar {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 200;
  -webkit-app-region: no-drag;  /* Tauri: allow clicks in draggable title bar area */
}

.main-content {
  display: flex;
  flex-direction: row;
  flex: 1;
  overflow: hidden;
  min-height: 0;
}
</style>
