<template>
  <div class="credit-bar" :class="[variant, { depleted: isDepleted }]">
    <span v-if="variant === 'full'" class="credit-label">{{ $t('credits.label') }}</span>
    <div class="bar-track">
      <div class="bar-fill" :class="{ 'bar-fill--depleted': isDepleted }" :style="{ width: pct + '%' }" />
    </div>
    <span class="credit-value">{{ credits }}<template v-if="variant === 'full'">/{{ max }}</template></span>
    <span v-if="variant === 'full'" class="credit-remaining" data-testid="credit-remaining">
      {{ $t('credits.remaining', { current: credits, max: max }) }}
    </span>
    <button v-if="variant === 'full'" class="buy-link" @click="emit('buy')">{{ $t('credits.buy') }}</button>
    <!-- Shared queue notice when credits are depleted -->
    <div
      v-if="isDepleted && isQueued"
      class="shared-queue-notice"
      data-testid="shared-queue-notice"
    >
      {{ $t('credits.sharedQueue') }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
const props = withDefaults(
  defineProps<{
    credits: number
    max: number
    variant: 'full' | 'mini' | 'dropdown'
    isQueued?: boolean
  }>(),
  { isQueued: false },
)
const emit = defineEmits<{ buy: [] }>()
const pct = computed(() => Math.min(100, props.max > 0 ? (props.credits / props.max) * 100 : 0))
const isDepleted = computed(() => props.credits <= 0)
</script>

<style scoped>
.credit-bar {
  display: flex; align-items: center; gap: var(--space-sm); flex-wrap: wrap;
  font: var(--font-size-sm) var(--font-mono); color: var(--text-muted);
}
.credit-label { white-space: nowrap; color: var(--text-disabled); }
.bar-track {
  flex: 1; height: 3px; background: var(--border);
  border-radius: 2px; overflow: hidden; min-width: 40px;
}
.bar-fill { height: 100%; background: var(--accent); border-radius: 2px; transition: width var(--duration-slow); }
.bar-fill--depleted { background: var(--error); }
.credit-value { white-space: nowrap; color: var(--text-secondary); }
.credit-remaining {
  white-space: nowrap;
  color: var(--text-muted);
  font-size: var(--font-size-xs);
}
.depleted .credit-remaining { color: var(--error); }
.buy-link { background: none; border: none; padding: 0; cursor: pointer; color: var(--accent); text-decoration: none; font: var(--font-size-xs) var(--font-mono); }
.buy-link:hover { text-decoration: underline; }
.shared-queue-notice {
  width: 100%;
  font: var(--font-size-xs) var(--font-mono);
  color: var(--warning);
  padding: 4px 0 0;
}
.mini .bar-track { min-width: 30px; }
/* dropdown variant: compact horizontal layout for popover/menu contexts */
.dropdown { gap: var(--space-xs); padding: 4px 0; }
.dropdown .bar-track { min-width: 60px; height: 2px; }
.dropdown .credit-value { font-size: var(--font-size-xs); color: var(--text-muted); }
</style>
