<template>
  <Transition name="update-slide">
    <div
      v-if="visible"
      class="update-banner"
      role="status"
      aria-live="polite"
      data-testid="update-banner"
    >
      <span class="update-banner__message">
        {{ $t('update.ready', { version }) }}
      </span>
      <div class="update-banner__actions">
        <button
          class="update-banner__btn update-banner__btn--primary"
          data-testid="restart-btn"
          @click="$emit('restart')"
        >
          {{ $t('update.restartNow') }}
        </button>
        <button
          class="update-banner__btn update-banner__btn--secondary"
          data-testid="later-btn"
          @click="$emit('dismiss')"
        >
          {{ $t('update.later') }}
        </button>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
defineProps<{
  visible: boolean
  version: string
}>()

defineEmits<{
  restart: []
  dismiss: []
}>()
</script>

<style scoped>
.update-banner {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 9998;
  background: linear-gradient(135deg, var(--success) 0%, var(--success-dark) 100%);
  color: var(--text-contrast);
  padding: var(--space-sm) var(--space-lg);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-md);
  font: var(--font-size-base)/1.4 var(--font-mono, 'SF Mono', monospace);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

.update-banner__message {
  flex: 1;
  font-weight: 500;
}

.update-banner__actions {
  display: flex;
  gap: var(--space-sm);
  flex-shrink: 0;
}

.update-banner__btn {
  border: none;
  border-radius: var(--radius-sm, 6px);
  padding: 4px 12px;
  font: inherit;
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-semibold);
  cursor: pointer;
  transition: opacity var(--duration-fast) var(--easing-default), transform var(--duration-instant) var(--easing-default);
}

.update-banner__btn:hover {
  opacity: 0.9;
}

.update-banner__btn:active {
  transform: scale(0.97);
}

.update-banner__btn--primary {
  background: #fff;
  color: var(--success-dark);
}

.update-banner__btn--secondary {
  background: rgba(255, 255, 255, 0.2);
  color: var(--text-contrast);
}

/* Slide-in from top animation */
.update-slide-enter-active {
  transition: transform 0.35s var(--easing-out), opacity 0.35s ease;
}

.update-slide-leave-active {
  transition: transform 0.25s var(--easing-in), opacity 0.25s ease;
}

.update-slide-enter-from {
  transform: translateY(-100%);
  opacity: 0;
}

.update-slide-leave-to {
  transform: translateY(-100%);
  opacity: 0;
}
</style>
