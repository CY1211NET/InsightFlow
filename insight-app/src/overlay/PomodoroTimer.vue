<template>
  <Transition name="expand">
    <div v-if="show" class="row-pomodoro">
      <div class="pomo-header">
        <span class="pomo-phase" :class="pomoPhase">{{ pomoPhase === 'focus' ? t('overlay.pomodoroFocus') : t('overlay.pomodoroBreak') }}</span>
        <span class="pomo-count">×{{ pomoDoneCount }}</span>
      </div>
      <div class="pomo-timer" :class="{ ticking: pomoRunning, done: pomoJustDone }">
        {{ pomoDisplay }}
      </div>
      <div class="pomo-actions">
        <button class="pomo-btn" :aria-label="pomoRunning ? t('overlay.pause') : t('overlay.start')" @click.stop="pomodoroToggle">{{ pomoRunning ? '⏸' : '▶' }}</button>
        <button class="pomo-btn" :aria-label="t('overlay.reset')" @click.stop="pomodoroReset">↺</button>
        <button class="pomo-btn" :aria-label="t('overlay.skip')" @click.stop="pomodoroSkip">⏭</button>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { t } from '../shared/i18n'

defineProps<{
  show: boolean
  pomoPhase: 'focus' | 'break'
  pomoDoneCount: number
  pomoRunning: boolean
  pomoJustDone: boolean
  pomoDisplay: string
  pomodoroToggle: () => void
  pomodoroReset: () => void
  pomodoroSkip: () => void
}>()
</script>

<style scoped>
/* ─── Pomodoro Timer ─── */
.row-pomodoro {
  padding: 8px 10px 10px;
  border-top: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.pomo-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.pomo-phase {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.04em;
  color: var(--text-primary);
}

.pomo-phase.focus { color: #c47a5a; }
.pomo-phase.break { color: #7a9e7e; }

.pomo-count {
  font-size: 10px;
  color: var(--text-muted);
  font-family: 'JetBrains Mono', monospace;
}

.pomo-timer {
  font-family: 'JetBrains Mono', monospace;
  font-size: 26px;
  font-weight: 700;
  color: var(--text-primary);
  text-align: center;
  letter-spacing: 0.06em;
  transition: color 0.3s ease;
}

.pomo-timer.ticking {
  color: #c47a5a;
}

.pomo-timer.done {
  color: #7a9e7e;
  animation: pomo-done-flash 0.4s ease 3;
}

@keyframes pomo-done-flash {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.pomo-actions {
  display: flex;
  justify-content: center;
  gap: 10px;
}

.pomo-btn {
  width: 30px;
  height: 24px;
  background: var(--btn-bg, var(--surface));
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s ease;
}

.pomo-btn:hover {
  background: var(--btn-hover, var(--surface-hover));
}
</style>
