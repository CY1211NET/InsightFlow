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
      <button class="icon-btn" :title="t('overlay.historyBtn')" @click="openDashboard">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><rect x="14" y="14" width="7" height="7" rx="1.5"/></svg>
      </button>
      <!-- 番茄钟按钮 -->
      <button class="icon-btn" :class="{ active: showPomodoro }" title="番茄钟" @click="togglePomodoro">
        <span style="font-size:13px">🍅</span>
      </button>
      <button class="icon-btn" :class="{ active: clickThrough }" :title="t('overlay.clickThrough') + ' (Ctrl+Shift+I)'" @click="toggleClickThrough">
        <svg v-if="!clickThrough" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
        <svg v-else width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/><line x1="1" y1="1" x2="23" y2="23"/></svg>
      </button>
      <button class="icon-btn" :title="t('overlay.theme')" @click="toggleTheme">
        <svg v-if="theme === 'day'" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>
        <svg v-else width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/></svg>
      </button>
      <button class="icon-btn" :title="t('overlay.settings')" @click="toggleSettings">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><line x1="4" y1="6" x2="20" y2="6"/><line x1="4" y1="12" x2="20" y2="12"/><line x1="4" y1="18" x2="20" y2="18"/><circle cx="9" cy="6" r="1.5" fill="currentColor"/><circle cx="15" cy="12" r="1.5" fill="currentColor"/><circle cx="9" cy="18" r="1.5" fill="currentColor"/></svg>
      </button>
      <button class="icon-btn" @click="toggleLocale">
        <span class="lang-text">{{ locale === 'zh-CN' ? 'EN' : 'CN' }}</span>
      </button>
    </div>

    <!-- 设置面板 -->
    <Transition name="expand">
      <div v-if="showSettings" class="row-settings">
        <div class="setting-item">
          <span class="setting-label">{{ t('overlay.settings') }}</span>
        </div>
        <div class="setting-item">
          <span class="setting-label">Opacity</span>
          <input
            type="range"
            class="slider"
            min="10" max="100" step="5"
            :value="opacityPct"
            @input="onOpacityChange"
            @click.stop
          />
          <span class="setting-value">{{ opacityPct }}%</span>
        </div>
        <div class="setting-item">
          <span class="setting-label" style="width:100%;text-transform:uppercase;letter-spacing:.06em">{{ t('overlay.moduleGoals') }}</span>
        </div>
        <div class="module-goals-grid">
          <div v-for="mod in moduleConfigs" :key="mod.id" class="module-goal-row">
            <span class="mod-goal-label" :style="{ color: catColorOf(mod.id) }">{{ mod.name }}</span>
            <input
              type="number"
              class="goal-input"
              :value="moduleGoalHours(mod.id)"
              min="0" max="24" step="0.5"
              :placeholder="'0'"
              @change="onModuleGoalChange(mod.id, $event)"
              @click.stop
            />
            <span class="setting-value">h</span>
          </div>
        </div>
        <div class="setting-item">
          <span class="setting-label">{{ t('overlay.autostart') }}</span>
          <button
            class="toggle-btn"
            :class="{ 'toggle-on': autostartEnabled }"
            @click.stop="toggleAutostart"
            :title="autostartEnabled ? t('overlay.autostartOn') : t('overlay.autostartOff')"
          >
            <span class="toggle-knob" />
          </button>
          <span class="setting-value" style="font-size:9px">{{ autostartEnabled ? 'ON' : 'OFF' }}</span>
        </div>
      </div>
    </Transition>

    <!-- 模块进度面板 -->
    <Transition name="expand">
      <div v-if="showModules" class="row-modules">
        <div
          v-for="mod in modules"
          :key="mod.category"
          class="mod-entry"
        >
          <div class="mod-header" @click="toggleModuleDetail(mod.category)">
            <span class="mod-dot" :style="{ background: catColorOf(mod.category) }" />
            <span class="mod-name">{{ catLabel(mod.category) }}</span>
            <div class="mod-track">
              <div class="mod-fill" :style="{ width: modBarWidth(mod), background: catColorOf(mod.category) }" />
            </div>
            <span class="mod-time">{{ fmtDur(mod.actualSecs) }}</span>
            <span class="mod-pct">{{ mod.goalSecs > 0 ? mod.goalPct + '%' : '--' }}</span>
            <svg class="chevron mini" :class="{ open: expandedModule === mod.category }" width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
          </div>
          <div v-if="expandedModule === mod.category" class="mod-apps">
            <div v-for="app in mod.topApps" :key="app.appName" class="mod-app-row">
              <span class="mod-app-name">{{ app.appName }}</span>
              <span class="mod-app-dur">{{ fmtDur(app.durationSecs) }}</span>
            </div>
            <div v-if="mod.topApps.length === 0" class="mod-app-empty">暂无记录</div>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 番茄钟面板 -->
    <Transition name="expand">
      <div v-if="showPomodoro" class="row-pomodoro">
        <div class="pomo-header">
          <span class="pomo-phase" :class="pomoPhase">{{ pomoPhase === 'focus' ? '🍅 专注' : '☕ 休息' }}</span>
          <span class="pomo-count">×{{ pomoDoneCount }}</span>
        </div>
        <div class="pomo-timer" :class="{ ticking: pomoRunning, done: pomoJustDone }">
          {{ pomoDisplay }}
        </div>
        <div class="pomo-actions">
          <button class="pomo-btn" @click.stop="pomodoroToggle">{{ pomoRunning ? '⏸' : '▶' }}</button>
          <button class="pomo-btn" @click.stop="pomodoroReset">↺</button>
          <button class="pomo-btn" @click.stop="pomodoroSkip">⏭</button>
        </div>
      </div>
    </Transition>

    <!-- 分心提醒 Toast -->
    <Transition name="toast">
      <div v-if="showDistractAlert" class="distract-toast" @click.stop="dismissAlert">
        <span class="toast-icon">⚠️</span>
        <span class="toast-text">已{{ distractStreakMins }}分钟未专注，休息一下？</span>
        <button class="toast-close">×</button>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { t, loadLocale, getLocale, setLocale } from '../shared/i18n'

// ──────────────────────────────────────────────
// Types
// ──────────────────────────────────────────────
interface AppUsage {
  appName: string
  durationSecs: number
}

interface ModuleConfig {
  id: string
  name: string
  color: string
  appKeywords: string[]
  siteDomains: string[]
}

interface ModuleProgress {
  category: string
  actualSecs: number
  goalSecs: number
  goalPct: number
  topApps: AppUsage[]
}

interface OverlayData {
  currentApp: string
  category: string
  sessionSecs: number
  focusSecs: number
  goalPct: number
  categorySecs: number   // today's time in current category
  aiHint: string
}

// ──────────────────────────────────────────────
// State
// ──────────────────────────────────────────────
const display = ref<OverlayData>({
  currentApp: t('overlay.initializing'),
  category: 'other',
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
const dailyGoalHours = ref(4)
let fadeTimer: ReturnType<typeof setTimeout> | null = null
const unlisteners: (() => void)[] = []

const showModules = ref(false)
const expandedModule = ref<string | null>(null)
const modules = ref<ModuleProgress[]>([])
const moduleGoals = ref<Record<string, number>>({})
const moduleConfigs = ref<ModuleConfig[]>([])

// ── Pomodoro state ──
const showPomodoro = ref(false)
const POMO_FOCUS_MINS = 25
const POMO_BREAK_MINS = 5
const pomoPhase = ref<'focus' | 'break'>('focus')
const pomoSecsLeft = ref(POMO_FOCUS_MINS * 60)
const pomoRunning = ref(false)
const pomoDoneCount = ref(0)
const pomoJustDone = ref(false)
let pomoTick: ReturnType<typeof setInterval> | null = null

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
        // Phase complete
        pomoRunning.value = false
        clearInterval(pomoTick!)
        pomoTick = null
        pomoJustDone.value = true
        setTimeout(() => { pomoJustDone.value = false }, 2000)
        if (pomoPhase.value === 'focus') {
          pomoDoneCount.value++
          pomoPhase.value = 'break'
          pomoSecsLeft.value = POMO_BREAK_MINS * 60
        } else {
          pomoPhase.value = 'focus'
          pomoSecsLeft.value = POMO_FOCUS_MINS * 60
        }
        // 自动开始下一个阶段
        pomodoroToggle()
      }
    }, 1000)
  }
}

function pomodoroReset() {
  pomoRunning.value = false
  if (pomoTick) { clearInterval(pomoTick); pomoTick = null }
  pomoPhase.value = 'focus'
  pomoSecsLeft.value = POMO_FOCUS_MINS * 60
  pomoDoneCount.value = 0
}

function pomodoroSkip() {
  pomoRunning.value = false
  if (pomoTick) { clearInterval(pomoTick); pomoTick = null }
  if (pomoPhase.value === 'focus') {
    pomoDoneCount.value++
    pomoPhase.value = 'break'
    pomoSecsLeft.value = POMO_BREAK_MINS * 60
  } else {
    pomoPhase.value = 'focus'
    pomoSecsLeft.value = POMO_FOCUS_MINS * 60
  }
}

async function togglePomodoro() {
  showPomodoro.value = !showPomodoro.value
  if (showSettings.value) showSettings.value = false
  if (showModules.value) { showModules.value = false; expandedModule.value = null }
  const h = showPomodoro.value ? COLLAPSED_H + 90 : COLLAPSED_H
  try { await invoke('resize_overlay', { height: h }) } catch {}
}

// ── Distraction alert state ──
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
      // Auto dismiss after 8 seconds
      setTimeout(() => { showDistractAlert.value = false }, 8000)
    } else if (!state.isDistracted) {
      showDistractAlert.value = false
    }
  } catch {
    // not available yet
  }
}

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
  if (cat === 'uncategorized') return t('category.other')
  return moduleById.value[cat]?.name ?? cat
}

const catColor = computed(() => catColorOf(display.value.category))

const barWidth = computed(() => Math.min(display.value.goalPct, 100) + '%')

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

// Short display label for each category (handled by module config)

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
// Locale toggle
// ──────────────────────────────────────────────
const locale = getLocale()

async function toggleLocale() {
  const next = locale.value === 'zh-CN' ? 'en' : 'zh-CN'
  await setLocale(next)
  await loadLocale()
  await refresh()
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
    // default to day
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

async function onGoalChange(e: Event) {
  const hours = Number((e.target as HTMLInputElement).value)
  dailyGoalHours.value = hours
  try {
    await invoke('set_daily_goal', { goalSecs: Math.round(hours * 3600) })
    await refresh()
  } catch (err) {
    console.warn('set_daily_goal failed:', err)
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
    const [opacity, goalSecs] = await Promise.all([
      invoke<number>('get_opacity'),
      invoke<number>('get_daily_goal'),
    ])
    opacityPct.value = Math.round(opacity * 100)
    dailyGoalHours.value = +(goalSecs / 3600).toFixed(1)
  } catch {
    // defaults
  }
}

async function loadAutostart() {
  try {
    autostartEnabled.value = await invoke<boolean>('get_autostart')
  } catch {
    // default false
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
const EXPANDED_H = 320
const EXPANDED_MODS_H = 360   // modules panel
const EXPANDED_SETS_H = 340   // settings panel (enlarged for per-category goals)

async function toggleSettings() {
  if (!showSettings.value) {
    if (showModules.value) { showModules.value = false; expandedModule.value = null }
    await loadModuleConfigs()
    await loadModuleGoals()
    try { await invoke('resize_overlay', { height: EXPANDED_SETS_H }) } catch (e) { console.warn('resize_overlay failed:', e) }
    showSettings.value = true
  } else {
    showSettings.value = false
    setTimeout(async () => {
      if (!showModules.value)
        try { await invoke('resize_overlay', { height: COLLAPSED_H }) } catch (e) { console.warn('resize_overlay failed:', e) }
    }, 200)
  }
}

async function toggleModules() {
  if (!showModules.value) {
    if (showSettings.value) { showSettings.value = false }
    await loadModuleConfigs()
    await loadModuleProgress()
    try { await invoke('resize_overlay', { height: EXPANDED_MODS_H }) } catch {}
    showModules.value = true
  } else {
    showModules.value = false
    expandedModule.value = null
    setTimeout(async () => {
      if (!showSettings.value)
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
    display.value.focusSecs += 1
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
  await loadTheme()
  await loadSettings()
  await loadAutostart()
  await loadModuleConfigs()
  await loadModuleGoals()
  await loadModuleProgress()
  await refresh()
  startSessionTick()

  unlisteners.push(await listen<OverlayData>('activity-changed', (event) => {
    display.value = event.payload
    stopSessionTick()
    startSessionTick()
  }))

  unlisteners.push(await listen('toggle-click-through', () => {
    toggleClickThrough()
  }))

  setInterval(async () => {
    await refresh()
    await loadModuleConfigs()
    await loadModuleGoals()
    await loadModuleProgress()
  }, 30_000)

  // 每 60 秒检查一次分心状态
  alertCheckTimer = setInterval(() => {
    checkDistractionAlert()
  }, 60_000)
})

onUnmounted(() => {
  unlisteners.forEach(fn => fn())
  stopSessionTick()
  if (fadeTimer) clearTimeout(fadeTimer)
  if (pomoTick) clearInterval(pomoTick)
  if (alertCheckTimer) clearInterval(alertCheckTimer)
})
</script>

<style scoped>
/* ─── Theme variables ─── */
.widget {
  --bg: rgba(245, 240, 235, 0.95);
  --bg-solid: #F5F0EB;
  --border: rgba(0, 0, 0, 0.06);
  --text-primary: #2C2420;
  --text-secondary: #6B5E54;
  --text-muted: #A89A8E;
  --surface: rgba(0, 0, 0, 0.04);
  --surface-hover: rgba(0, 0, 0, 0.07);
  --track: rgba(0, 0, 0, 0.06);
  --shadow: 0 4px 24px rgba(0, 0, 0, 0.06), 0 1px 3px rgba(0, 0, 0, 0.04);
  --shadow-hover: 0 8px 40px rgba(0, 0, 0, 0.1), 0 2px 6px rgba(0, 0, 0, 0.06);

  width: calc(100vw - 20px);
  margin: 10px;
  box-sizing: border-box;
  background: var(--bg);
  backdrop-filter: blur(24px) saturate(130%);
  -webkit-backdrop-filter: blur(24px) saturate(130%);
  border: 1px solid var(--border);
  border-radius: 25px;
  padding: 18px 20px 14px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  opacity: 1;
  transition: opacity 1.2s ease, box-shadow 0.3s ease, background 0.4s ease;
  box-shadow: var(--shadow);
  cursor: default;
  overflow: hidden;
  font-family: 'Outfit', sans-serif;
  position: relative;
}

/* subtle paper texture */
.widget::before {
  content: '';
  position: absolute;
  inset: 0;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='0.02'/%3E%3C/svg%3E");
  background-size: 128px 128px;
  pointer-events: none;
  border-radius: inherit;
  overflow: hidden;
}

/* ─── Night theme ─── */
.widget.night {
  --bg: rgba(28, 25, 23, 0.92);
  --bg-solid: #1C1917;
  --border: rgba(255, 255, 255, 0.06);
  --text-primary: #E8E0D8;
  --text-secondary: #9E958C;
  --text-muted: #5A544E;
  --surface: rgba(255, 255, 255, 0.05);
  --surface-hover: rgba(255, 255, 255, 0.08);
  --track: rgba(255, 255, 255, 0.06);
  --shadow: 0 4px 24px rgba(0, 0, 0, 0.3), 0 1px 3px rgba(0, 0, 0, 0.2);
  --shadow-hover: 0 8px 40px rgba(0, 0, 0, 0.5), 0 2px 6px rgba(0, 0, 0, 0.3);
}

.widget.hovered {
  box-shadow: var(--shadow-hover);
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
  background: rgba(196, 122, 90, 0.15);
  color: #c47a5a;
  border-color: rgba(196, 122, 90, 0.2);
}

.night .icon-btn.active {
  background: rgba(196, 122, 90, 0.2);
  border-color: rgba(196, 122, 90, 0.3);
}

.lang-text {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.04em;
}

/* ─── Row: settings ─── */
.row-settings {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding-top: 4px;
  border-top: 1px solid var(--track);
}

.setting-item {
  display: flex;
  align-items: center;
  gap: 10px;
}

.setting-label {
  font-size: 10px;
  font-weight: 500;
  color: var(--text-muted);
  width: 50px;
  flex-shrink: 0;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.setting-value {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: var(--text-secondary);
  width: 34px;
  text-align: right;
  flex-shrink: 0;
}

.slider {
  flex: 1;
  height: 3px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--track);
  border-radius: 2px;
  outline: none;
  cursor: pointer;
}

.slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--text-secondary);
  border: 2px solid var(--bg-solid);
  cursor: pointer;
  transition: background 0.15s ease, transform 0.15s ease;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}

.slider::-webkit-slider-thumb:hover {
  background: var(--text-primary);
  transform: scale(1.15);
}

.goal-input {
  flex: 1;
  height: 26px;
  background: var(--surface);
  border: 1px solid var(--track);
  border-radius: 8px;
  color: var(--text-primary);
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  text-align: center;
  outline: none;
  padding: 0 6px;
  -moz-appearance: textfield;
  transition: border-color 0.15s ease;
}

.goal-input:focus {
  border-color: var(--text-muted);
}

.goal-input::-webkit-inner-spin-button,
.goal-input::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
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

/* ─── Toggle button (autostart) ─── */
.toggle-btn {
  position: relative;
  width: 32px;
  height: 16px;
  background: var(--track);
  border: none;
  border-radius: 8px;
  cursor: pointer;
  padding: 0;
  transition: background 0.2s ease;
  flex-shrink: 0;
  margin-right: auto;
}

.toggle-btn.toggle-on {
  background: #c47a5a;
}

.toggle-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 12px;
  height: 12px;
  background: #fff;
  border-radius: 50%;
  transition: transform 0.2s ease;
  box-shadow: 0 1px 3px rgba(0,0,0,0.2);
  display: block;
}

.toggle-btn.toggle-on .toggle-knob {
  transform: translateX(16px);
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

/* ─── Modules panel ─── */
.row-modules {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding-top: 4px;
  border-top: 1px solid var(--track);
  max-height: 220px;
  overflow-y: auto;
}
.row-modules::-webkit-scrollbar { width: 3px; }
.row-modules::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.08); border-radius: 2px; }

.mod-entry {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.mod-header {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 5px 4px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.12s;
}
.mod-header:hover { background: var(--surface); }

.mod-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  flex-shrink: 0;
}

.mod-name {
  font-size: 10px;
  font-weight: 500;
  color: var(--text-secondary);
  width: 30px;
  flex-shrink: 0;
}

.mod-track {
  flex: 1;
  height: 3px;
  background: var(--track);
  border-radius: 2px;
  overflow: hidden;
}

.mod-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.5s cubic-bezier(0.22,1,0.36,1);
}

.mod-time {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: var(--text-muted);
  width: 38px;
  text-align: right;
  flex-shrink: 0;
}

.mod-pct {
  font-family: 'JetBrains Mono', monospace;
  font-size: 9px;
  color: var(--text-muted);
  width: 26px;
  text-align: right;
  flex-shrink: 0;
}

.mod-apps {
  padding: 2px 4px 6px 16px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.mod-app-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}

.mod-app-name {
  font-size: 10px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.mod-app-dur {
  font-family: 'JetBrains Mono', monospace;
  font-size: 9px;
  color: var(--text-muted);
  flex-shrink: 0;
}

.mod-app-empty {
  font-size: 9px;
  color: var(--text-muted);
  font-style: italic;
}

/* ─── Module goals grid in settings ─── */
.module-goals-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px 6px;
}

.module-goal-row {
  display: flex;
  align-items: center;
  gap: 5px;
}

.mod-goal-label {
  font-size: 9px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  width: 26px;
  flex-shrink: 0;
}

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
  background: var(--btn-bg);
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
  background: var(--btn-hover);
}

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
  backdrop-filter: blur(6px);
  z-index: 99;
}

.toast-icon {
  font-size: 12px;
  flex-shrink: 0;
}

.toast-text {
  font-size: 10px;
  color: #c47a5a;
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

