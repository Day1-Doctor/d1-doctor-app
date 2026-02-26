<template>
  <FullMode v-if="appStore.uiMode === 'full'" />
  <CopilotMode v-else-if="appStore.uiMode === 'copilot'" />
  <!-- Ninja mode runs in its own window (NinjaApp.vue). Main window hides when ninja is active. -->
  <!-- If this renders, uiMode is 'ninja' but ninja window didn't show — shouldn't happen in production. -->
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useAppStore } from '@/shared/stores/app'
import FullMode from '@/modes/full/FullMode.vue'
import CopilotMode from '@/modes/copilot/CopilotMode.vue'

const appStore = useAppStore()
onMounted(() => appStore.init())
</script>
