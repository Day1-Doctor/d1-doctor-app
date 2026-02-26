<template><div class="title-bar"><div class="traffic-lights"><div class="traffic-dot close"/><div class="traffic-dot minimize"/><div class="traffic-dot maximize"/></div><span class="window-title">Day 1 Doctor</span></div></template>
<script setup lang="ts"></script>
<style scoped>
.title-bar { height: 38px; background: var(--surface-title-bar); -webkit-app-region: drag; display: flex; align-items: center; padding: 0 16px; gap: 12px; flex-shrink: 0; }
.traffic-lights { -webkit-app-region: no-drag; display: flex; gap: 8px; }
.traffic-dot { width: 12px; height: 12px; border-radius: 50%; cursor: pointer; }
.close { background: var(--traffic-close); } .minimize { background: var(--traffic-minimize); } .maximize { background: var(--traffic-maximize); }
.window-title { flex: 1; text-align: center; font: 12px var(--font-mono); color: var(--text-muted); }
</style>
