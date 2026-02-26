<template>
  <div class="app-window">
    <TitleBar />
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
}

.main-content {
  display: flex;
  flex-direction: row;
  flex: 1;
  overflow: hidden;
  min-height: 0;
}
</style>
