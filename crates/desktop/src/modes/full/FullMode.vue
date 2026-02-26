<template>
  <div class="full-mode">
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
.full-mode { display: flex; flex-direction: column; height: 100vh; background: var(--background); color: var(--text-primary); font-family: var(--font-mono); }
.main-content { display: flex; flex: 1; overflow: hidden; }
</style>
