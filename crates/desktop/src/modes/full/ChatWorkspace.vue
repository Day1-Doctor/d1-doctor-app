<template><div class="chat-workspace"><div class="message-list"/><div class="input-bar"><input placeholder="Ask anything..." /></div></div></template>
<script setup lang="ts"></script>
<style scoped>.chat-workspace { flex: 1; background: var(--surface-chat); display: flex; flex-direction: column; overflow: hidden; } .message-list { flex: 1; overflow-y: auto; padding: 24px; } .input-bar { padding: 16px; border-top: 1px solid var(--border); }</style>
