<template>
  <div
    class="widget"
    :class="[theme, { hovered: isHovered, fading: isFading }]"
    @mouseenter="onEnter"
    @mouseleave="onLeave"
    @mousedown="onDragStart"
  >
    <!-- 顶栏：整行作为拖拽区域 -->
    <div class="row-main" data-tauri-drag-region>
      <span class="dot" :style="{ background: catColor }" />
      <span class="app-name">{{ display.currentApp }}</span>
      <span class="session">{{ fmtDur(display.sessionSecs) }}</span>
    </div>

    <!-- 今日专注 — 当前模块进度，可展开 -->
    <div class="row-focus" @click.stop="toggleModules">
      <span class="cat-badge" :style="{ color: catColor }">{{ catLabel(display.category) }}</span>
      <span class="focus-time">{{ fmtDur(display.categorySecs) }}</span>
      <div class="track">
        <div class="fill" :style="{ width: categoryBarWidth, background: catColor }" />
      </div>
      <span class="pct" v-if="categoryPctLabel !== null">{{ categoryPctLabel }}%</span>
      <span class="pct" v-else>--</span>
      <svg class="chevron" :class="{ open: showModules }" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
    </div>

    <!-- 操作栏 -->
    <div class="row-actions">
      <button class="icon-btn" :title="t('overlay.historyBtn')" :aria-label="t('overlay.historyBtn')" @click="openDashboard">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18"/><path d="M9 3v18"/></svg>
      </button>
      <button class="icon-btn" :class="{ active: showPomodoro }" :title="t('overlay.pomodoroFocus')" :aria-label="t('overlay.pomodoroFocus')" @click="togglePomodoro">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="13" r="8"/><path d="M12 9v4l2 2"/><path d="M5 3l2 2"/><path d="M19 3l-2 2"/><path d="M12 3v2"/></svg>
      </button>
      <button class="icon-btn" :class="{ active: showTodo }" :title="t('dashboard.todos')" :aria-label="t('dashboard.todos')" @click="toggleTodoList">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11"/></svg>
      </button>
      <button class="icon-btn" :class="{ active: pinned }" :title="pinned ? t('overlay.unpin') : t('overlay.pin')" :aria-label="t('overlay.pin')" :aria-pressed="pinned" @click.stop="togglePinned">
        <svg width="14" height="14" viewBox="0 0 24 24" :fill="pinned ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 17v5"/><path d="M9 10.76a2 2 0 01-1.11 1.79l-1.78.9A2 2 0 005 15.24V17h14v-1.76a2 2 0 00-1.11-1.79l-1.78-.9A2 2 0 0115 10.76V7a1 1 0 011-1 2 2 0 000-4H8a2 2 0 000 4 1 1 0 011 1z"/></svg>
      </button>
      <button class="icon-btn" :class="{ active: clickThrough }" :title="t('overlay.clickThrough') + ' (Ctrl+Shift+I)'" :aria-label="t('overlay.clickThrough')" :aria-pressed="clickThrough" @click="toggleClickThrough">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
      </button>
      <button class="icon-btn" :title="t('overlay.theme')" :aria-label="t('overlay.theme')" @click="toggleTheme">
        <svg v-if="theme === 'day'" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/></svg>
        <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z"/></svg>
      </button>
      <button class="icon-btn" :title="t('overlay.settings')" :aria-label="t('overlay.settings')" @click="toggleSettings">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/></svg>
      </button>
    </div>

    <!-- 设置面板 -->
    <SettingsPanel
      :show="showSettings"
      :moduleConfigs="moduleConfigs"
      :moduleGoals="moduleGoals"
      :autostartEnabled="autostartEnabled"
      :opacityPct="opacityPct"
      :catColorOf="catColorOf"
      :moduleGoalHours="moduleGoalHours"
      @toggleAutostart="toggleAutostart"
      @opacityChange="onOpacityChange"
      @moduleGoalChange="onModuleGoalChange"
    />

    <!-- 模块进度面板 -->
    <ModuleProgressPanel
      :show="showModules"
      :modules="modules"
      :expandedModule="expandedModule"
      :catColorOf="catColorOf"
      :catLabel="catLabel"
      :modBarWidth="modBarWidth"
      :fmtDur="fmtDur"
      @toggleDetail="toggleModuleDetail"
    />

    <!-- 番茄钟面板 -->
    <PomodoroTimer
      :show="showPomodoro"
      :pomoPhase="pomoPhase"
      :pomoDoneCount="pomoDoneCount"
      :pomoRunning="pomoRunning"
      :pomoJustDone="pomoJustDone"
      :pomoDisplay="pomoDisplay"
      :pomodoroToggle="pomodoroToggle"
      :pomodoroReset="pomodoroReset"
      :pomodoroSkip="pomodoroSkip"
    />

    <!-- 待做清单面板 -->
    <TodoList
      :show="showTodo"
      :todos="activeTodos"
      :newTodoText="newTodoText"
      @update:newTodoText="newTodoText = $event"
      @add="addTodo"
      @toggle="toggleTodo"
      @remove="removeTodo"
    />

    <!-- 分心提醒 Toast -->
    <DistractToast
      :show="showDistractAlert"
      :streakMins="distractStreakMins"
      @dismiss="dismissAlert"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { t, loadLocale, watchLocale } from '../shared/i18n'
import { CATEGORY, isFocus } from '../shared/constants'
import type { ModuleConfig, ModuleProgress, OverlayData } from './types'
import { usePomodoro } from './composables/usePomodoro'
import { useDistraction } from './composables/useDistraction'
import { useTodo } from './composables/useTodo'
import SettingsPanel from './SettingsPanel.vue'
import ModuleProgressPanel from './ModuleProgress.vue'
import PomodoroTimer from './PomodoroTimer.vue'
import DistractToast from './DistractToast.vue'
import TodoList from './TodoList.vue'

// ──────────────────────────────────────────────
// State
// ──────────────────────────────────────────────
const display = ref<OverlayData>({
  currentApp: t('overlay.initializing'),
  category: CATEGORY.OTHER,
  sessionSecs: 0,
  focusSecs: 0,
  goalPct: 0,
  categorySecs: 0,
  aiHint: t('overlay.aiHint'),
})

const isHovered = ref(false)
const isFading = ref(false)
const clickThrough = ref(false)
const showSettings = ref(false)
const theme = ref<'day' | 'night'>('day')
const autostartEnabled = ref(false)
const opacityPct = ref(100)
const pinned = ref(false)
let fadeTimer: ReturnType<typeof setTimeout> | null = null
const unlisteners: (() => void)[] = []

const showModules = ref(false)
const expandedModule = ref<string | null>(null)
const modules = ref<ModuleProgress[]>([])
const moduleGoals = ref<Record<string, number>>({})
const moduleConfigs = ref<ModuleConfig[]>([])

// ── Composables ──
const {
  showPomodoro, pomoPhase, pomoRunning, pomoDoneCount,
  pomoJustDone, pomoDisplay,
  pomodoroToggle, pomodoroReset, pomodoroSkip,
  cleanup: cleanupPomodoro,
  loadPomodoroSettings,
} = usePomodoro()

const {
  showDistractAlert, distractStreakMins,
  dismissAlert, startChecking: startDistractionCheck,
  cleanup: cleanupDistraction,
} = useDistraction()

const {
  showTodo, activeTodos, newTodoText,
  loadTodos, addTodo, toggleTodo, removeTodo,
} = useTodo()

function closeOtherPanels(except: 'settings' | 'modules' | 'pomodoro' | 'todo') {
  if (except !== 'settings' && showSettings.value) showSettings.value = false
  if (except !== 'modules' && showModules.value) { showModules.value = false; expandedModule.value = null }
  if (except !== 'pomodoro' && showPomodoro.value) showPomodoro.value = false
  if (except !== 'todo' && showTodo.value) showTodo.value = false
}

async function togglePomodoro() {
  if (!showPomodoro.value) {
    closeOtherPanels('pomodoro')
    try { await invoke('resize_overlay', { height: COLLAPSED_H + 130 }) } catch {}
    showPomodoro.value = true
  } else {
    showPomodoro.value = false
    setTimeout(async () => {
      if (!showSettings.value && !showModules.value && !showTodo.value)
        try { await invoke('resize_overlay', { height: COLLAPSED_H }) } catch {}
    }, 200)
  }
}

async function toggleTodoList() {
  if (!showTodo.value) {
    closeOtherPanels('todo')
    await loadTodos()
    try { await invoke('resize_overlay', { height: COLLAPSED_H + 160 }) } catch {}
    showTodo.value = true
  } else {
    showTodo.value = false
    setTimeout(async () => {
      if (!showSettings.value && !showModules.value && !showPomodoro.value)
        try { await invoke('resize_overlay', { height: COLLAPSED_H }) } catch {}
    }, 200)
  }
}

let refreshTimer: ReturnType<typeof setInterval> | null = null

// ──────────────────────────────────────────────
// Computed
// ──────────────────────────────────────────────
const moduleById = computed(() => {
  const map: Record<string, ModuleConfig> = {}
  moduleConfigs.value.forEach(m => { map[m.id] = m })
  return map
})

function catColorOf(cat: string): string {
  return moduleById.value[cat]?.color ?? '#8a8278'
}

function catLabel(cat: string) {
  if (cat === CATEGORY.UNCATEGORIZED) return t('category.other')
  return moduleById.value[cat]?.name ?? cat
}

const catColor = computed(() => catColorOf(display.value.category))

// Goal for current active category (seconds)
const currentModuleGoalSecs = computed(() =>
  moduleGoals.value[display.value.category] ?? 0
)

// Width of the category progress bar
const categoryBarWidth = computed(() => {
  const secs = display.value.categorySecs
  const goal = currentModuleGoalSecs.value
  if (goal > 0) return Math.min(secs / goal, 1) * 100 + '%'
  return Math.min(secs / 14400, 1) * 100 + '%'  // 4h reference when no goal
})

// Percentage label for current category
const categoryPctLabel = computed(() => {
  if (currentModuleGoalSecs.value === 0) return null
  return Math.min(Math.round((display.value.categorySecs / currentModuleGoalSecs.value) * 100), 100)
})

// ──────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────
function fmtDur(secs: number): string {
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  const s = secs % 60
  if (h > 0) return `${h}h ${m.toString().padStart(2,'0')}m`
  if (m > 0) return `${m}m ${s.toString().padStart(2,'0')}s`
  return `${s}s`
}

async function refresh() {
  try {
    const data = await invoke<OverlayData>('get_overlay_data')
    display.value = data
  } catch (e) {
    console.warn('get_overlay_data failed:', e)
  }
}

// ──────────────────────────────────────────────
// Mouse interaction
// ──────────────────────────────────────────────
function onEnter() {
  if (fadeTimer) { clearTimeout(fadeTimer); fadeTimer = null }
  isFading.value = false
  isHovered.value = true
}

function onLeave() {
  isHovered.value = false
  if (pinned.value) return
  fadeTimer = setTimeout(() => { isFading.value = true }, 3000)
}

async function onDragStart(e: MouseEvent) {
  if (e.button !== 0) return
  const target = e.target as HTMLElement | null
  if (!target) return
  if (target.closest('.row-focus, .row-actions, .row-settings, .row-modules, button, input, textarea, .toggle-btn')) {
    return
  }
  try {
    await getCurrentWindow().startDragging()
  } catch (err) {
    console.warn('startDragging failed:', err)
  }
}

// ──────────────────────────────────────────────
// Click-through toggle
// ──────────────────────────────────────────────
async function toggleClickThrough() {
  clickThrough.value = !clickThrough.value
  try {
    const win = getCurrentWindow()
    await win.setIgnoreCursorEvents(clickThrough.value)
  } catch (e) {
    console.warn('setIgnoreCursorEvents failed:', e)
  }
}

// ──────────────────────────────────────────────
// Open dashboard
// ──────────────────────────────────────────────
async function openDashboard() {
  try {
    await invoke('show_dashboard')
  } catch (e) {
    console.warn('show_dashboard failed:', e)
  }
}

// ──────────────────────────────────────────────
// Theme toggle
// ──────────────────────────────────────────────
async function toggleTheme() {
  theme.value = theme.value === 'day' ? 'night' : 'day'
  try {
    await invoke('set_theme', { theme: theme.value })
  } catch (e) {
    console.warn('set_theme failed:', e)
  }
}

async function loadTheme() {
  try {
    const saved = await invoke<string>('get_theme')
    if (saved === 'day' || saved === 'night') {
      theme.value = saved
    }
  } catch {
    console.warn('get_theme failed, using default')
  }
}

// ──────────────────────────────────────────────
// Opacity & daily goal
// ──────────────────────────────────────────────
async function onOpacityChange(e: Event) {
  const val = Number((e.target as HTMLInputElement).value)
  opacityPct.value = val
  try {
    await invoke('set_opacity', { opacity: val / 100 })
  } catch (err) {
    console.warn('set_opacity failed:', err)
  }
}

async function onModuleGoalChange(category: string, e: Event) {
  const hours = Number((e.target as HTMLInputElement).value)
  const goalSecs = Math.round(hours * 3600)
  moduleGoals.value[category] = goalSecs
  try {
    await invoke('set_module_goal', { category, goalSecs })
    await loadModuleProgress()
  } catch (err) {
    console.warn('set_module_goal failed:', err)
  }
}

function moduleGoalHours(cat: string): string {
  const secs = moduleGoals.value[cat] ?? 0
  return secs > 0 ? (secs / 3600).toFixed(1) : ''
}

async function loadSettings() {
  try {
    const opacity = await invoke<number>('get_opacity')
    opacityPct.value = Math.round(opacity * 100)
  } catch {
    console.warn('get_opacity failed, using default')
  }
}

async function loadAutostart() {
  try {
    autostartEnabled.value = await invoke<boolean>('get_autostart')
  } catch {
    // default false
  }
}

async function loadPinned() {
  try {
    pinned.value = await invoke<boolean>('get_pinned')
    if (pinned.value) {
      if (fadeTimer) { clearTimeout(fadeTimer); fadeTimer = null }
      isFading.value = false
    }
  } catch (e) {
    console.warn('get_pinned failed:', e)
  }
}

async function togglePinned() {
  const next = !pinned.value
  pinned.value = next

  if (next) {
    if (fadeTimer) { clearTimeout(fadeTimer); fadeTimer = null }
    isFading.value = false
  } else {
    // 恢复自动淡出逻辑：若当前不在悬停，重新启动计时
    if (!isHovered.value) {
      if (fadeTimer) { clearTimeout(fadeTimer); fadeTimer = null }
      fadeTimer = setTimeout(() => { isFading.value = true }, 3000)
    }
  }

  try {
    await invoke('set_pinned', { pinned: next })
  } catch (e) {
    console.warn('set_pinned failed:', e)
  }
}

async function loadModuleConfigs() {
  try {
    moduleConfigs.value = await invoke<ModuleConfig[]>('get_modules')
  } catch (e) {
    console.warn('get_modules failed:', e)
  }
}

async function loadModuleGoals() {
  try {
    moduleGoals.value = await invoke<Record<string, number>>('get_module_goals')
  } catch (e) {
    console.warn('get_module_goals failed:', e)
  }
}

async function loadModuleProgress() {
  try {
    const mods = await invoke<ModuleProgress[]>('get_module_progress')
    modules.value = mods
  } catch (e) {
    console.warn('get_module_progress failed:', e)
  }
}

// ──────────────────────────────────────────────
// Overlay resize heights
// ──────────────────────────────────────────────
const COLLAPSED_H = 180

const EXPANDED_MODS_H = 370   // modules panel

async function toggleSettings() {
  if (!showSettings.value) {
    closeOtherPanels('settings')
    await loadModuleConfigs()
    await loadModuleGoals()
    const expandedSetsHeight = 260 + Math.ceil(moduleConfigs.value.length / 2) * 40;
    try { await invoke('resize_overlay', { height: expandedSetsHeight }) } catch (e) { console.warn('resize_overlay failed:', e) }
    showSettings.value = true
  } else {
    showSettings.value = false
    setTimeout(async () => {
      if (!showModules.value && !showPomodoro.value && !showTodo.value)
        try { await invoke('resize_overlay', { height: COLLAPSED_H }) } catch (e) { console.warn('resize_overlay failed:', e) }
    }, 200)
  }
}

async function toggleModules() {
  if (!showModules.value) {
    closeOtherPanels('modules')
    await loadModuleConfigs()
    await loadModuleProgress()
    try { await invoke('resize_overlay', { height: EXPANDED_MODS_H }) } catch {}
    showModules.value = true
  } else {
    showModules.value = false
    expandedModule.value = null
    setTimeout(async () => {
      if (!showSettings.value && !showPomodoro.value && !showTodo.value)
        try { await invoke('resize_overlay', { height: COLLAPSED_H }) } catch {}
    }, 200)
  }
}

function toggleModuleDetail(cat: string) {
  expandedModule.value = expandedModule.value === cat ? null : cat
}

function modBarWidth(mod: ModuleProgress): string {
  if (mod.goalSecs > 0) return Math.min(mod.actualSecs / mod.goalSecs, 1) * 100 + '%'
  return Math.min(mod.actualSecs / 14400, 1) * 100 + '%'
}

async function toggleAutostart() {
  try {
    const next = !autostartEnabled.value
    await invoke('set_autostart', { enabled: next })
    autostartEnabled.value = next
  } catch (e) {
    console.warn('set_autostart failed:', e)
  }
}

// ──────────────────────────────────────────────
// Session timer
// ──────────────────────────────────────────────
let sessionTick: ReturnType<typeof setInterval> | null = null

function startSessionTick() {
  stopSessionTick()
  sessionTick = setInterval(() => {
    display.value.sessionSecs += 1
    if (isFocus(display.value.category)) {
      display.value.focusSecs += 1
    }
    display.value.categorySecs += 1
  }, 1000)
}

function stopSessionTick() {
  if (sessionTick) { clearInterval(sessionTick); sessionTick = null }
}

// ──────────────────────────────────────────────
// Lifecycle
// ──────────────────────────────────────────────
onMounted(async () => {
  await loadLocale()
  watchLocale()
  await loadTheme()
  await loadSettings()
  await loadAutostart()
  await loadPinned()
  await loadModuleConfigs()
  await loadModuleGoals()
  await loadModuleProgress()
  await refresh()
  startSessionTick()
  startDistractionCheck()

  unlisteners.push(await listen<OverlayData>('activity-changed', (event) => {
    display.value = event.payload
    stopSessionTick()
    startSessionTick()
  }))

  unlisteners.push(await listen('toggle-click-through', () => {
    toggleClickThrough()
  }))

  refreshTimer = setInterval(async () => {
    await refresh()
    await loadModuleConfigs()
    await loadModuleGoals()
    await loadModuleProgress()
    await loadPomodoroSettings()
  }, 30_000)
})

onUnmounted(() => {
  unlisteners.forEach(fn => fn())
  stopSessionTick()
  if (fadeTimer) clearTimeout(fadeTimer)
  if (refreshTimer) clearInterval(refreshTimer)
  cleanupPomodoro()
  cleanupDistraction()
})
</script>

<style scoped>
/* ─── Theme variables ─── */
.widget {
  --bg: rgba(247, 247, 245, 0.95);
  --bg-solid: #F7F7F5;
  --border: rgba(0, 0, 0, 0.08);
  --text-primary: #1A1A1A;
  --text-secondary: #5A5A5A;
  --text-muted: #9A9A9A;
  --surface: rgba(0, 0, 0, 0.04);
  --surface-hover: rgba(0, 0, 0, 0.07);
  --track: rgba(0, 0, 0, 0.08);

  width: calc(100vw - 20px);
  margin: 10px;
  box-sizing: border-box;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 25px;
  padding: 18px 20px 24px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  opacity: 1;
  transition: opacity 1.2s ease, background 0.4s ease;
  cursor: default;
  overflow: hidden;
  font-family: 'Outfit', sans-serif;
  position: relative;
}

/* ─── Night theme ─── */
.widget.night {
  --bg: rgba(26, 26, 26, 0.92);
  --bg-solid: #1A1A1A;
  --border: rgba(255, 255, 255, 0.08);
  --text-primary: #E0E0E0;
  --text-secondary: #A0A0A0;
  --text-muted: #606060;
  --surface: rgba(255, 255, 255, 0.06);
  --surface-hover: rgba(255, 255, 255, 0.10);
  --track: rgba(255, 255, 255, 0.08);
}

.widget.fading {
  opacity: 0.15;
}

/* ─── Row: main ─── */
.row-main {
  display: flex;
  align-items: center;
  gap: 10px;
}

.dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
  box-shadow: 0 0 6px currentColor;
}

.app-name {
  flex: 1;
  font-size: 13.5px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  letter-spacing: -0.01em;
}

.session {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  color: var(--text-muted);
  letter-spacing: 0.03em;
}

/* ─── 禁止非顶栏行触发拖拽（Tauri 透明窗口关键修复）─── */
.row-focus,
.row-actions,
.row-settings,
.row-modules {
  -webkit-app-region: no-drag;
  pointer-events: auto;
}

/* ─── Row: focus ─── */
.row-focus {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
}

.focus-time {
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.track {
  flex: 1;
  height: 3px;
  background: var(--track);
  border-radius: 2px;
  overflow: hidden;
}

.fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.6s cubic-bezier(0.22, 1, 0.36, 1);
}

.pct {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 500;
  color: var(--text-muted);
  font-variant-numeric: tabular-nums;
  width: 32px;
  text-align: right;
  letter-spacing: 0.02em;
}

/* ─── Row: actions ─── */
.row-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.icon-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--surface);
  border: 1px solid transparent;
  border-radius: 8px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s ease;
  padding: 0;
}

.icon-btn:hover {
  background: var(--surface-hover);
  color: var(--text-primary);
}

.icon-btn.active {
  background: rgba(196, 122, 90, 0.12);
  color: #c47a5a;
  border-color: rgba(196, 122, 90, 0.2);
}

.night .icon-btn.active {
  background: rgba(196, 122, 90, 0.2);
  color: #d4906e;
  border-color: rgba(196, 122, 90, 0.3);
}

.lang-text {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.04em;
}

/* ─── Transition: expand ─── */
.expand-enter-active {
  transition: opacity 0.25s cubic-bezier(0.22, 1, 0.36, 1),
              transform 0.25s cubic-bezier(0.22, 1, 0.36, 1);
}
.expand-leave-active {
  transition: opacity 0.15s ease,
              transform 0.15s ease;
}
.expand-enter-from, .expand-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
.expand-enter-to, .expand-leave-from {
  opacity: 1;
  transform: translateY(0);
}

/* ─── row-focus: category-aware ─── */
.cat-badge {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  white-space: nowrap;
  flex-shrink: 0;
  min-width: 30px;
}

.chevron {
  flex-shrink: 0;
  color: var(--text-muted);
  transition: transform 0.2s ease;
  margin-left: 2px;
}
.chevron.open { transform: rotate(180deg); }
.chevron.mini { opacity: 0.6; }
</style>
