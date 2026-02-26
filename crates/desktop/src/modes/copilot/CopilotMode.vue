<template>
  <div class="copilot copilot-panel">
    <CopilotHeader />
    <SessionBar />
    <div class="copilot-messages"/>
    <CopilotInput />
  </div>
</template>
<script setup lang="ts">
import CopilotHeader from './CopilotHeader.vue'
import SessionBar from './SessionBar.vue'
import CopilotInput from './CopilotInput.vue'
</script>
<style scoped>
.copilot-panel { width: 420px; height: 100vh; background: var(--surface-copilot); backdrop-filter: blur(40px) saturate(160%); -webkit-backdrop-filter: blur(40px) saturate(160%); border: 1px solid var(--border); border-radius: 12px; overflow: hidden; display: flex; flex-direction: column; box-shadow: 0 24px 64px rgba(0,0,0,0.6); }
.copilot-messages { flex: 1; overflow-y: auto; padding: 12px; }
</style>
