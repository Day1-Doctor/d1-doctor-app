<template>
  <FullMode v-if="appStore.uiMode === 'full'" />
  <CopilotMode v-else-if="appStore.uiMode === 'copilot'" />
</template>
<script setup lang="ts">
import { onMounted } from 'vue'
import { useAppStore } from '@/shared/stores/app'
import FullMode from '@/modes/full/FullMode.vue'
import CopilotMode from '@/modes/copilot/CopilotMode.vue'
const appStore = useAppStore()
onMounted(() => appStore.init())
</script>
