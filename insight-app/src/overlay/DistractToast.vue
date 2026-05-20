<template>
  <Transition name="toast">
    <div v-if="show" class="distract-toast" role="alert" @click.stop="$emit('dismiss')">
      <span class="toast-icon" aria-hidden="true">⚠️</span>
      <span class="toast-text">{{ t('overlay.distractMsg').replace('{mins}', streakMins.toString()) }}</span>
      <button class="toast-close" :aria-label="t('overlay.closeAlert')" @click="$emit('dismiss')">×</button>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { t } from '../shared/i18n'

defineProps<{
  show: boolean
  streakMins: number
}>()

defineEmits<{
  dismiss: []
}>()
</script>

<style scoped>
/* ─── Distraction Alert Toast ─── */
.distract-toast {
  position: absolute;
  bottom: 6px;
  left: 8px;
  right: 8px;
  background: rgba(196, 122, 90, 0.12);
  border: 1px solid rgba(196, 122, 90, 0.35);
  border-radius: 8px;
  padding: 6px 10px;
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  z-index: 99;
}

.toast-icon {
  font-size: 12px;
  flex-shrink: 0;
}

.toast-text {
  font-size: 10px;
  color: var(--accent);
  flex: 1;
  font-weight: 500;
}

.toast-close {
  font-size: 11px;
  color: var(--text-muted);
  background: none;
  border: none;
  cursor: pointer;
  padding: 0 2px;
  flex-shrink: 0;
}

/* Toast transitions */
.toast-enter-active, .toast-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}
.toast-enter-from, .toast-leave-to {
  opacity: 0;
  transform: translateY(8px);
}
</style>
