import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export function useDistraction() {
  const showDistractAlert = ref(false)
  const distractStreakMins = ref(0)
  let alertCheckTimer: ReturnType<typeof setInterval> | null = null

  function dismissAlert() {
    showDistractAlert.value = false
  }

  async function checkDistractionAlert() {
    try {
      const state = await invoke<{ streakSecs: number; isDistracted: boolean }>('get_distraction_state')
      if (state.isDistracted && !showDistractAlert.value) {
        distractStreakMins.value = Math.floor(state.streakSecs / 60)
        showDistractAlert.value = true
        setTimeout(() => { showDistractAlert.value = false }, 8000)
      } else if (!state.isDistracted) {
        showDistractAlert.value = false
      }
    } catch {
      // not available yet
    }
  }

  function startChecking() {
    alertCheckTimer = setInterval(() => {
      checkDistractionAlert()
    }, 60_000)
  }

  function cleanup() {
    if (alertCheckTimer) { clearInterval(alertCheckTimer); alertCheckTimer = null }
  }

  return {
    showDistractAlert,
    distractStreakMins,
    dismissAlert,
    checkDistractionAlert,
    startChecking,
    cleanup,
  }
}
