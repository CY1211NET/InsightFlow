import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const DEFAULT_FOCUS_MINS = 25
const DEFAULT_BREAK_MINS = 5

export function usePomodoro(opts?: { focusMins?: number; breakMins?: number }) {
  const focusMins = ref(opts?.focusMins ?? DEFAULT_FOCUS_MINS)
  const breakMins = ref(opts?.breakMins ?? DEFAULT_BREAK_MINS)

  const showPomodoro = ref(false)
  const pomoPhase = ref<'focus' | 'break'>('focus')
  const pomoSecsLeft = ref(focusMins.value * 60)
  const pomoRunning = ref(false)
  const pomoDoneCount = ref(0)
  const pomoJustDone = ref(false)
  let pomoTick: ReturnType<typeof setInterval> | null = null

  async function loadPomodoroSettings() {
    try {
      const [fMins, bMins] = await invoke<[number, number]>('get_pomodoro_settings')
      if (fMins && bMins) {
        focusMins.value = fMins
        breakMins.value = bMins
        if (!pomoRunning.value) {
          if (pomoPhase.value === 'focus') {
            pomoSecsLeft.value = fMins * 60
          } else {
            pomoSecsLeft.value = bMins * 60
          }
        }
      }
    } catch {}
  }

  // Initial load
  loadPomodoroSettings()

  const pomoDisplay = computed(() => {
    const m = Math.floor(pomoSecsLeft.value / 60)
    const s = pomoSecsLeft.value % 60
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  })

  function pomodoroToggle() {
    if (pomoRunning.value) {
      pomoRunning.value = false
      if (pomoTick) { clearInterval(pomoTick); pomoTick = null }
    } else {
      pomoRunning.value = true
      pomoTick = setInterval(() => {
        if (pomoSecsLeft.value > 0) {
          pomoSecsLeft.value--
        } else {
          pomoRunning.value = false
          clearInterval(pomoTick!)
          pomoTick = null
          pomoJustDone.value = true
          setTimeout(() => { pomoJustDone.value = false }, 2000)
          if (pomoPhase.value === 'focus') {
            pomoDoneCount.value++
            pomoPhase.value = 'break'
            pomoSecsLeft.value = breakMins.value * 60
          } else {
            pomoPhase.value = 'focus'
            pomoSecsLeft.value = focusMins.value * 60
          }
          pomodoroToggle()
        }
      }, 1000)
    }
  }

  function pomodoroReset() {
    pomoRunning.value = false
    if (pomoTick) { clearInterval(pomoTick); pomoTick = null }
    pomoPhase.value = 'focus'
    pomoSecsLeft.value = focusMins.value * 60
    pomoDoneCount.value = 0
  }

  function pomodoroSkip() {
    pomoRunning.value = false
    if (pomoTick) { clearInterval(pomoTick); pomoTick = null }
    if (pomoPhase.value === 'focus') {
      pomoDoneCount.value++
      pomoPhase.value = 'break'
      pomoSecsLeft.value = breakMins.value * 60
    } else {
      pomoPhase.value = 'focus'
      pomoSecsLeft.value = focusMins.value * 60
    }
  }

  function setFocusMins(mins: number) {
    focusMins.value = Math.max(1, Math.min(120, mins))
    if (!pomoRunning.value && pomoPhase.value === 'focus') {
      pomoSecsLeft.value = focusMins.value * 60
    }
  }

  function setBreakMins(mins: number) {
    breakMins.value = Math.max(1, Math.min(30, mins))
    if (!pomoRunning.value && pomoPhase.value === 'break') {
      pomoSecsLeft.value = breakMins.value * 60
    }
  }

  function cleanup() {
    if (pomoTick) { clearInterval(pomoTick); pomoTick = null }
  }

  return {
    showPomodoro,
    pomoPhase,
    pomoSecsLeft,
    pomoRunning,
    pomoDoneCount,
    pomoJustDone,
    pomoDisplay,
    focusMins,
    breakMins,
    pomodoroToggle,
    pomodoroReset,
    pomodoroSkip,
    setFocusMins,
    setBreakMins,
    cleanup,
    loadPomodoroSettings,
  }
}
