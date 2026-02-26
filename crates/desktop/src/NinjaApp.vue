<template>
  <div class="ninja-app">
    <NinjaBar @submit="onSubmit" @esc="onEsc" />
    <NinjaDropdown v-if="showDropdown" :query="lastQuery" :steps="[]" @dismiss="showDropdown = false" />
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue'
import NinjaBar from '@/modes/ninja/NinjaBar.vue'
import NinjaDropdown from '@/modes/ninja/NinjaDropdown.vue'
const showDropdown = ref(false)
const lastQuery = ref('')
function onSubmit(q: string) { lastQuery.value = q; showDropdown.value = true }
function onEsc() { if (showDropdown.value) { showDropdown.value = false } }
</script>
<style scoped>
.ninja-app { display: flex; flex-direction: column; align-items: center; padding-top: 0; }
</style>
