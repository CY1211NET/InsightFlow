<template>
  <div class="dashboard">
    <!-- Header -->
    <header class="header">
      <h1 class="title">{{ t('dashboard.title') }}</h1>
      <div class="header-right">
        <span class="date">{{ todayStr }}</span>
        <button class="lang-btn" @click="toggleLocale" :title="localeLabel">
          {{ locale === 'zh-CN' ? 'EN' : '中' }}
        </button>
      </div>
    </header>

    <!-- Summary cards -->
    <div class="summary">
      <div class="card">
        <span class="card-label">{{ t('dashboard.total') }}</span>
        <span class="card-value">{{ fmtDur(data.totalSecs) }}</span>
      </div>
      <div class="card">
        <span class="card-label">{{ t('dashboard.sessions') }}</span>
        <span class="card-value">{{ data.activities.length }}</span>
      </div>
      <div class="card">
        <span class="card-label">{{ t('dashboard.focus') }}</span>
        <span class="card-value">{{ fmtDur(focusSecs) }}</span>
      </div>
    </div>

    <!-- Date range tabs -->
    <div class="date-tabs">
      <button
        v-for="tab in [
          { key: 'today',     label: t('dashboard.today') },
          { key: 'yesterday', label: t('dashboard.yesterday') },
          { key: 'week',      label: t('dashboard.thisWeek') },
          { key: 'month',     label: t('dashboard.thisMonth') },
        ]"
        :key="tab.key"
        class="date-tab"
        :class="{ active: dateRange === tab.key }"
        @click="setDateRange(tab.key as DateRange)"
      >{{ tab.label }}</button>
    </div>


    <div class="sections-layout">
      <div
        v-for="(secName, idx) in layoutOrder"
        :key="secName"
        class="draggable-section"
        draggable="true"
        @dragstart="onSecDragStart($event, idx)"
        @dragover.prevent
        @dragenter="onSecDragEnter($event, idx)"
        @dragend="onSecDragEnd"
      >
        <!-- App usage by category (history) -->
        <section class="section" v-if="secName === 'appUsage'">
          <h2 class="section-title drag-handle">{{ t('dashboard.appUsage') }}</h2>
          <div class="bars">
            <div v-if="categoryApps.length === 0" class="empty-hint">
              {{ t('dashboard.noData') }}
            </div>
            <div
              v-for="cat in categoryApps"
              :key="cat.category"
              class="bar-group"
            >
              <div class="bar-row" @click="toggleCategory(cat.category)">
                <span class="bar-label">{{ catLabel(cat.category) }}</span>
                <div class="bar-track">
                  <div
                    class="bar-fill"
                    :style="{
                      width: barWidth(cat.totalSecs),
                      background: catColor(cat.category),
                    }"
                  />
                </div>
                <span class="bar-value">{{ fmtDur(cat.totalSecs) }}</span>
                <svg class="chevron" :class="{ open: expandedCategory === cat.category }" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
              </div>
              <div v-if="expandedCategory === cat.category" class="cat-apps">
                <div
                  v-for="app in (categoryAppsByCat[cat.category]?.apps ?? [])"
                  :key="app.appName"
                  class="cat-app-row"
                >
                  <span class="cat-app-name">{{ app.appName }}</span>
                  <div class="cat-app-track">
                    <div
                      class="cat-app-fill"
                      :style="{ width: appBarWidth(app.durationSecs, categoryAppsByCat[cat.category]?.totalSecs ?? 0), background: catColor(cat.category) }"
                    />
                  </div>
                  <span class="cat-app-dur">{{ fmtDur(app.durationSecs) }}</span>
                </div>
                <div v-if="(categoryAppsByCat[cat.category]?.apps?.length ?? 0) === 0" class="cat-app-empty">
                  {{ t('dashboard.noData') }}
                </div>
              </div>
            </div>
          </div>
        </section>

        <!-- Module Goals & Progress -->
        <section class="section" v-else-if="secName === 'moduleGoals'">
          <h2 class="section-title drag-handle">{{ t('dashboard.moduleGoals') }}</h2>
          <div class="module-goals-list">
            <div
              v-for="mod in modules"
              :key="mod.category"
              class="module-goal-row"
            >
              <div class="module-goal-left">
                <span class="module-goal-dot" :style="{ background: modCatColor(mod.category) }" />
                <span class="module-goal-name">{{ catLabel(mod.category) }}</span>
                <div class="module-goal-input-wrap">
                  <input
                    type="number"
                    class="module-goal-input"
                    :value="moduleGoalHours(mod.category) || ''"
                    min="0" max="24" step="0.5"
                    :placeholder="t('dashboard.notSet')"
                    @change="saveModuleGoal(mod.category, Number(($event.target as HTMLInputElement).value))"
                  />
                  <span class="module-goal-unit">h</span>
                </div>
              </div>
              <div class="module-goal-right">
                <div class="module-prog-track">
                  <div
                    class="module-prog-fill"
                    :style="{
                      width: mod.goalSecs > 0
                        ? Math.min(mod.actualSecs / mod.goalSecs, 1) * 100 + '%'
                        : Math.min(mod.actualSecs / 14400, 1) * 100 + '%',
                      background: modCatColor(mod.category)
                    }"
                  />
                  <div
                    v-if="mod.goalSecs > 0"
                    class="module-goal-marker"
                    style="left: 100%"
                  />
                </div>
                <span class="module-actual-time">{{ fmtDur(mod.actualSecs) }}</span>
                <span class="module-goal-pct" v-if="mod.goalSecs > 0">
                  {{ mod.goalPct }}%
                </span>
                <span class="module-goal-pct dim" v-else>--</span>
              </div>
            </div>
          </div>
        </section>

        <!-- Module Manager -->
        <section class="section" v-else-if="secName === 'moduleManager'">
          <div class="section-header" @click="toggleSection('moduleManager')">
            <h2 class="section-title drag-handle">{{ t('dashboard.moduleManager') }}</h2>
            <svg class="section-toggle" :class="{ open: !collapsed.moduleManager }" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
          </div>
          <div v-if="!collapsed.moduleManager" class="section-body">
          <div class="module-manager">
            <div v-for="(mod, idx) in moduleConfigs" :key="mod.id + '-' + idx" class="module-config-row">
              <div class="module-config-main">
                <input
                  class="module-config-name"
                  :value="mod.name"
                  :placeholder="t('dashboard.moduleName')"
                  @input="mod.name = ($event.target as HTMLInputElement).value"
                />
                <input
                  class="module-config-color"
                  type="color"
                  :value="mod.color || '#8a8278'"
                  @input="mod.color = ($event.target as HTMLInputElement).value"
                />
              </div>
              <div class="module-config-fields">
                <input
                  class="module-config-input"
                  :value="mod.appKeywords.join(', ')"
                  :placeholder="t('dashboard.appKeywords')"
                  @change="updateKeywords(idx, ($event.target as HTMLInputElement).value)"
                />
                <input
                  class="module-config-input"
                  :value="mod.siteDomains.join(', ')"
                  :placeholder="t('dashboard.siteDomains')"
                  @change="updateDomains(idx, ($event.target as HTMLInputElement).value)"
                />
              </div>
              <div class="module-config-actions">
                <button class="mini-btn" @click="moveModule(idx, -1)">▲</button>
                <button class="mini-btn" @click="moveModule(idx, 1)">▼</button>
                <button class="mini-btn danger" @click="removeModule(idx)">✕</button>
              </div>
            </div>
            <div class="module-config-footer">
              <button class="mini-btn" @click="addModule">{{ t('dashboard.addModule') }}</button>
              <button class="mini-btn primary" @click="saveModuleConfigs">{{ t('dashboard.saveModules') }}</button>
            </div>
          </div>
          </div>
        </section>

        <!-- Weekly trend -->
        <section class="section" v-else-if="secName === 'weeklyTrend'">
          <h2 class="section-title drag-handle">{{ t('dashboard.weeklyTrend') }}</h2>
          <div class="weekly-chart">
            <div v-for="day in weeklySeries" :key="day.date" class="weekly-col">
              <span class="weekly-val" v-if="day.focusSecs > 0">{{ fmtDurShort(day.focusSecs) }}</span>
              <span class="weekly-val" v-else style="opacity:0">-</span>
              <div class="weekly-bar-wrap">
                <div
                  class="weekly-bar-fill"
                  :style="{ height: weeklyBarH(day.focusSecs) + 'px' }"
                />
              </div>
              <span class="weekly-label">{{ dayLabel(day.date) }}</span>
            </div>
          </div>
        </section>

        <!-- Activity list -->
        <section class="section" v-else-if="secName === 'activities'">
          <div class="section-header" @click="toggleSection('activities')">
            <h2 class="section-title drag-handle">{{ t('dashboard.activities') }}</h2>
            <svg class="section-toggle" :class="{ open: !collapsed.activities }" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
          </div>
          <div v-if="!collapsed.activities" class="section-body">
            <div class="activity-list">
              <div v-if="data.activities.length === 0" class="empty-hint">
                {{ t('dashboard.noData') }}
              </div>
              <div
                v-for="act in data.activities.slice().reverse()"
                :key="act.id"
                class="activity-item"
              >
                <span class="cat-dot" :style="{ background: catColor(act.category) }" />
                <div class="act-info">
                  <span class="act-name">{{ act.appName }}</span>
                  <span class="act-title">{{ act.windowTitle }}</span>
                </div>
                <div class="act-meta">
                  <span class="act-duration">{{ fmtDur(act.endTime - act.startTime) }}</span>
                  <span class="act-time">{{ fmtTime(act.startTime) }} - {{ fmtTime(act.endTime) }}</span>
                  <select 
                    class="act-correct-select" 
                    :value="act.category" 
                    @change="correctCategory(act, ($event.target as HTMLSelectElement).value)"
                  >
                    <option v-for="mod in moduleConfigs" :key="mod.id" :value="mod.id">{{ mod.name }}</option>
                    <option value="uncategorized" v-if="!moduleConfigs.find(m => m.id === 'uncategorized')">Uncategorized</option>
                  </select>
                </div>
              </div>
            </div>
          </div>
        </section>

        <!-- Web history (today only) -->
        <section class="section" v-else-if="secName === 'webHistory'">
          <template v-if="webHistory.length > 0 || dateRange === 'today'">
            <div class="section-header" @click="toggleSection('webHistory')">
              <h2 class="section-title drag-handle">{{ t('dashboard.webHistory') }}</h2>
              <svg class="section-toggle" :class="{ open: !collapsed.webHistory }" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
            </div>
            <div v-if="!collapsed.webHistory" class="section-body">
            <div class="activity-list">
              <div
                v-for="visit in webHistory"
                :key="String(visit.id)"
                class="activity-item"
              >
                <span class="cat-dot" style="background:#7a8a9e" />
                <div class="act-info">
                  <span class="act-name">{{ visit.domain }}</span>
                  <span class="act-title">{{ visit.pageTitle }}</span>
                </div>
                <div class="act-meta">
                  <span class="act-duration">{{ visit.visitCount }}×</span>
                  <span class="act-time">{{ fmtTime(visit.lastVisit) }}</span>
                </div>
              </div>
              <div v-if="webHistory.length === 0" class="empty-hint">
                {{ t('dashboard.noWebHistory') }}
              </div>
            </div>
            </div>
          </template>
        </section>

      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { t, loadLocale, getLocale, setLocale, getAvailableLocales } from '../shared/i18n'

// ── Interfaces ────────────────────────────────────────────────────────────────

interface Activity {
  id: number | null
  appName: string
  windowTitle: string
  category: string
  startTime: number
  endTime: number
}

interface CategoryStat {
  category: string
  totalSecs: number
}

interface AppUsage {
  appName: string
  durationSecs: number
}

interface ModuleProgress {
  category: string
  actualSecs: number
  goalSecs: number
  goalPct: number
  topApps: AppUsage[]
}

interface ModuleGoals {
  [key: string]: number
}

interface ModuleConfig {
  id: string
  name: string
  color: string
  appKeywords: string[]
  siteDomains: string[]
}

interface CategoryAppBreakdown {
  category: string
  totalSecs: number
  apps: AppUsage[]
}

interface DashboardData {
  activities: Activity[]
  categoryStats: CategoryStat[]
  totalSecs: number
}

interface WebVisit {
  id: number | null
  domain: string
  url: string
  pageTitle: string
  visitCount: number
  lastVisit: number
}

interface DailyFocus {
  date: string       // "YYYY-MM-DD"
  focusSecs: number
}

// ── Existing reactive state ───────────────────────────────────────────────────

const data = ref<DashboardData>({
  activities: [],
  categoryStats: [],
  totalSecs: 0,
})

const modules = ref<ModuleProgress[]>([])
const moduleGoals = ref<ModuleGoals>({})
const moduleConfigs = ref<ModuleConfig[]>([])
const savingGoal = ref<string | null>(null)
const categoryApps = ref<CategoryAppBreakdown[]>([])
const expandedCategory = ref<string | null>(null)
const expandedActCategory = ref<string | null>(null)

type CollapsibleSection = 'moduleManager' | 'activities' | 'webHistory'
const collapsed = ref<Record<CollapsibleSection, boolean>>({
  moduleManager: false,
  activities: false,
  webHistory: false,
})

// ── New reactive state ────────────────────────────────────────────────────────

type DateRange = 'today' | 'yesterday' | 'week' | 'month'
const dateRange = ref<DateRange>('today')
const weeklySeries = ref<DailyFocus[]>([])
const webHistory = ref<WebVisit[]>([])

const layoutOrder = ref([
  'appUsage',
  'moduleGoals',
  'moduleManager',
  'weeklyTrend',
  'activities',
  'webHistory'
])

let draggedSecIdx: number | null = null

function onSecDragStart(e: DragEvent, idx: number) {
  draggedSecIdx = idx
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move'
  }
}

function onSecDragEnter(e: DragEvent, idx: number) {
  if (draggedSecIdx !== null && draggedSecIdx !== idx) {
    const list = [...layoutOrder.value]
    const item = list.splice(draggedSecIdx, 1)[0]
    list.splice(idx, 0, item)
    layoutOrder.value = list
    draggedSecIdx = idx
    localStorage.setItem('dashboard-layout-order', JSON.stringify(list))
  }
}

function onSecDragEnd() {
  draggedSecIdx = null
}

// ── Locale helpers ────────────────────────────────────────────────────────────

const locale = getLocale()
const localeLabel = computed(() => {
  return locale.value === 'zh-CN' ? 'Switch to English' : '切换到中文'
})
const todayStr = computed(() => {
  const loc = locale.value === 'en' ? 'en-US' : 'zh-CN'
  return new Date().toLocaleDateString(loc, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    weekday: 'long',
  })
})

async function toggleLocale() {
  const next = locale.value === 'zh-CN' ? 'en' : 'zh-CN'
  await setLocale(next)
  await load()
}

// ── Computed: focus seconds ───────────────────────────────────────────────────

const focusSecs = computed(() => {
  return data.value.categoryStats
    .filter(s => s.category === 'dev' || s.category === 'productivity')
    .reduce((sum, s) => sum + s.totalSecs, 0)
})



// ── Computed: date-range timestamps ──────────────────────────────────────────

const rangeTimestamps = computed(() => {
  const now = new Date()
  const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const todayStartTs = Math.floor(todayStart.getTime() / 1000)

  switch (dateRange.value) {
    case 'today':
      return { start: todayStartTs, end: todayStartTs + 86400 }
    case 'yesterday':
      return { start: todayStartTs - 86400, end: todayStartTs }
    case 'week':
      return { start: todayStartTs - 6 * 86400, end: todayStartTs + 86400 }
    case 'month': {
      const monthStart = new Date(now.getFullYear(), now.getMonth(), 1)
      return { start: Math.floor(monthStart.getTime() / 1000), end: todayStartTs + 86400 }
    }
  }
})

const categoryAppsByCat = computed(() => {
  const map: Record<string, CategoryAppBreakdown> = {}
  categoryApps.value.forEach(item => { map[item.category] = item })
  return map
})

// ── Category helpers ──────────────────────────────────────────────────────────

const maxSecs = computed(() => {
  return Math.max(...data.value.categoryStats.map(s => s.totalSecs), 1)
})

const moduleById = computed(() => {
  const map: Record<string, ModuleConfig> = {}
  moduleConfigs.value.forEach(m => { map[m.id] = m })
  return map
})

function catColor(cat: string): string {
  return moduleById.value[cat]?.color ?? '#8a8278'
}

function modCatColor(cat: string): string {
  return catColor(cat)
}

function catLabel(cat: string): string {
  if (cat === 'uncategorized') return t('category.other')
  const name = moduleById.value[cat]?.name
  if (name) return name
  const fallback = t(`category.${cat}`)
  return fallback === `category.${cat}` ? cat : fallback
}

function barWidth(secs: number): string {
  return ((secs / maxSecs.value) * 100).toFixed(1) + '%'
}

// ── Duration / time formatters ────────────────────────────────────────────────

function fmtDur(secs: number): string {
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  if (h > 0) return `${h}h ${m.toString().padStart(2, '0')}m`
  if (m > 0) return `${m}m`
  return `${secs}s`
}

function fmtTime(ts: number): string {
  const d = new Date(ts * 1000)
  const loc = locale.value === 'en' ? 'en-US' : 'zh-CN'
  return d.toLocaleTimeString(loc, { hour: '2-digit', minute: '2-digit' })
}

// ── Weekly chart helpers ──────────────────────────────────────────────────────

const maxWeeklyFocus = computed(() =>
  Math.max(...weeklySeries.value.map(d => d.focusSecs), 1)
)

function weeklyBarH(secs: number): number {
  if (secs === 0) return 2
  return Math.max((secs / maxWeeklyFocus.value) * 60, 4)
}

function fmtDurShort(secs: number): string {
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  if (h > 0) return `${h}h`
  if (m > 0) return `${m}m`
  return `<1m`
}

function dayLabel(dateStr: string): string {
  const parts = dateStr.split('-')
  return `${parseInt(parts[1])}/${parseInt(parts[2])}`
}

// ── Data loaders ──────────────────────────────────────────────────────────────

async function load() {
  await loadLocale()
  try {
    const { start, end } = rangeTimestamps.value
    data.value = await invoke<DashboardData>('get_dashboard_data_range', {
      startTs: start,
      endTs: end,
    })
  } catch (e) {
    console.warn('get_dashboard_data_range failed, falling back:', e)
    try {
      data.value = await invoke<DashboardData>('get_dashboard_data')
    } catch {}
  }
}

async function loadWeeklySeries() {
  try {
    weeklySeries.value = await invoke<DailyFocus[]>('get_weekly_focus_series')
  } catch (e) {
    console.warn('get_weekly_focus_series failed:', e)
  }
}

async function loadWebHistory() {
  try {
    webHistory.value = await invoke<WebVisit[]>('get_web_history')
  } catch (e) {
    console.warn('get_web_history failed:', e)
  }
}

async function loadModules() {
  try {
    modules.value = await invoke<ModuleProgress[]>('get_module_progress')
  } catch (e) {
    console.warn('get_module_progress failed:', e)
  }
}

async function correctCategory(act: { appName: string, category: string }, newCategory: string) {
  if (act.category === newCategory) return
  
  try {
    await invoke('correct_activity_category', {
      appName: act.appName,
      newCategory
    })
    
    // Optimistic UI update
    act.category = newCategory
    
    // Reload full data in background to refresh charts
    setTimeout(() => {
      load()
      loadModules()
    }, 500)
  } catch (e) {
    console.error('Failed to correct category:', e)
  }
}

async function loadModuleGoals() {
  try {
    moduleGoals.value = await invoke<ModuleGoals>('get_module_goals')
  } catch (e) {
    console.warn('get_module_goals failed:', e)
  }
}

async function loadModuleConfigs() {
  try {
    moduleConfigs.value = await invoke<ModuleConfig[]>('get_modules')
  } catch (e) {
    console.warn('get_modules failed:', e)
  }
}

async function saveModuleConfigs() {
  try {
    await invoke('save_modules', { modules: moduleConfigs.value })
    await Promise.all([loadModuleConfigs(), loadModules()])
  } catch (e) {
    console.warn('save_modules failed:', e)
  }
}

function addModule() {
  moduleConfigs.value.push({
    id: '',
    name: '',
    color: '#8a8278',
    appKeywords: [],
    siteDomains: [],
  })
}

function removeModule(index: number) {
  moduleConfigs.value.splice(index, 1)
}

function moveModule(index: number, delta: number) {
  const next = index + delta
  if (next < 0 || next >= moduleConfigs.value.length) return
  const temp = moduleConfigs.value[index]
  moduleConfigs.value[index] = moduleConfigs.value[next]
  moduleConfigs.value[next] = temp
}

function updateKeywords(index: number, value: string) {
  moduleConfigs.value[index].appKeywords = value.split(',').map(s => s.trim()).filter(Boolean)
}

function updateDomains(index: number, value: string) {
  moduleConfigs.value[index].siteDomains = value.split(',').map(s => s.trim()).filter(Boolean)
}

async function loadCategoryApps() {
  try {
    const { start, end } = rangeTimestamps.value
    categoryApps.value = await invoke<CategoryAppBreakdown[]>('get_category_app_breakdown', {
      startTs: start,
      endTs: end,
    })
  } catch (e) {
    console.warn('get_category_app_breakdown failed:', e)
  }
}

async function saveModuleGoal(category: string, hours: number) {
  const goalSecs = Math.round(hours * 3600)
  ;(moduleGoals.value as Record<string, number>)[category] = goalSecs
  savingGoal.value = category
  try {
    await invoke('set_module_goal', { category, goalSecs })
    await loadModules()
  } catch (e) {
    console.warn('set_module_goal failed:', e)
  } finally {
    setTimeout(() => { if (savingGoal.value === category) savingGoal.value = null }, 600)
  }
}

function moduleGoalHours(cat: string): number {
  return ((moduleGoals.value as Record<string, number>)[cat] ?? 0) / 3600
}

function toggleCategory(cat: string) {
  expandedCategory.value = expandedCategory.value === cat ? null : cat
}

function toggleActCategory(cat: string) {
  expandedActCategory.value = expandedActCategory.value === cat ? null : cat
}


function appBarWidth(appSecs: number, totalSecs: number): string {
  if (totalSecs <= 0) return '0%'
  return Math.max((appSecs / totalSecs) * 100, 3) + '%'
}

async function setDateRange(range: DateRange) {
  dateRange.value = range
  await Promise.all([load(), loadCategoryApps()])
}

function toggleSection(key: CollapsibleSection) {
  collapsed.value[key] = !collapsed.value[key]
}

// ── Lifecycle ─────────────────────────────────────────────────────────────────

onMounted(async () => {
  const savedOrder = localStorage.getItem('dashboard-layout-order')
  if (savedOrder) {
    try { layoutOrder.value = JSON.parse(savedOrder) } catch (e) {}
  }

  await Promise.all([
    load(),
    loadWeeklySeries(),
    loadWebHistory(),
    loadModules(),
    loadModuleGoals(),
    loadModuleConfigs(),
    loadCategoryApps(),
  ])
})
</script>

<style scoped>
.dashboard {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  padding: 24px;
  background: #1c1917;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.dashboard::-webkit-scrollbar {
  width: 4px;
}
.dashboard::-webkit-scrollbar-track {
  background: transparent;
}
.dashboard::-webkit-scrollbar-thumb {
  background: rgba(255,255,255,0.08);
  border-radius: 2px;
}

/* Header */
.header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
}

.title {
  font-size: 18px;
  font-weight: 600;
  color: #e8e0d8;
  letter-spacing: -0.02em;
}

.date {
  font-size: 12px;
  color: #6e6760;
  letter-spacing: 0.02em;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.lang-btn {
  background: rgba(255,255,255,0.05);
  border: 1px solid rgba(255,255,255,0.08);
  border-radius: 6px;
  padding: 2px 8px;
  font-size: 11px;
  font-weight: 500;
  color: #9e958c;
  cursor: pointer;
  transition: all 0.15s ease;
  font-family: 'Outfit', sans-serif;
}

.lang-btn:hover {
  background: rgba(255,255,255,0.1);
  color: #e8e0d8;
  border-color: rgba(255,255,255,0.12);
}

/* Summary */
.summary {
  display: flex;
  gap: 12px;
}

.card {
  flex: 1;
  background: rgba(255,255,255,0.03);
  border: 1px solid rgba(255,255,255,0.05);
  border-radius: 12px;
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.card-label {
  font-size: 10.5px;
  font-weight: 400;
  color: #6e6760;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.card-value {
  font-family: 'JetBrains Mono', monospace;
  font-size: 20px;
  font-weight: 600;
  color: #e8e0d8;
  letter-spacing: -0.02em;
}

/* Date tabs */
.date-tabs {
  display: flex;
  gap: 6px;
}

.date-tab {
  flex: 1;
  height: 28px;
  background: rgba(255,255,255,0.03);
  border: 1px solid rgba(255,255,255,0.05);
  border-radius: 8px;
  font-size: 11px;
  font-weight: 500;
  color: #6e6760;
  cursor: pointer;
  transition: all 0.15s ease;
  font-family: 'Outfit', sans-serif;
}

.date-tab:hover {
  background: rgba(255,255,255,0.06);
  color: #9e958c;
}

.date-tab.active {
  background: rgba(196,122,90,0.15);
  border-color: rgba(196,122,90,0.25);
  color: #c47a5a;
}

/* Section */
.section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-title {
  font-size: 11px;
  font-weight: 500;
  color: #6e6760;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  cursor: pointer;
}

.section-toggle {
  flex-shrink: 0;
  color: #5a544e;
  transition: transform 0.2s ease;
}

.section-toggle.open {
  transform: rotate(180deg);
}

.section-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* Bars */
.bars {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.bar-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.bar-row {
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  padding: 2px 0;
  border-radius: 6px;
  transition: background 0.15s ease;
}

.bar-row:hover {
  background: rgba(255,255,255,0.03);
}

.bar-row.plain {
  cursor: default;
}

.bar-row.plain:hover {
  background: transparent;
}

.chevron {
  flex-shrink: 0;
  color: #5a544e;
  transition: transform 0.2s ease;
  margin-left: 4px;
}

.chevron.open {
  transform: rotate(180deg);
}

.bar-label {
  font-size: 12px;
  color: #9e958c;
  width: 90px;
  flex-shrink: 0;
}

.bar-track {
  flex: 1;
  height: 6px;
  background: rgba(255,255,255,0.04);
  border-radius: 3px;
  overflow: hidden;
}

.bar-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.6s cubic-bezier(0.22, 1, 0.36, 1);
  min-width: 2px;
}

.bar-value {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: #6e6760;
  width: 56px;
  text-align: right;
  flex-shrink: 0;
}

.cat-apps {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-left: 102px;
}

.cat-app-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.cat-app-name {
  font-size: 11px;
  color: #6e6760;
  width: 120px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex-shrink: 0;
}

.cat-app-track {
  flex: 1;
  height: 4px;
  background: rgba(255,255,255,0.03);
  border-radius: 2px;
  overflow: hidden;
}

.cat-app-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.4s cubic-bezier(0.22, 1, 0.36, 1);
  min-width: 2px;
}

.cat-app-dur {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: #5a544e;
  width: 56px;
  text-align: right;
  flex-shrink: 0;
}

.cat-app-empty {
  font-size: 11px;
  color: #5a544e;
  font-style: italic;
}

/* Weekly chart */
.weekly-chart {
  display: flex;
  align-items: flex-end;
  gap: 6px;
  padding-top: 4px;
}

.weekly-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.weekly-val {
  font-size: 8px;
  color: #6e6760;
  font-family: 'JetBrains Mono', monospace;
  white-space: nowrap;
  height: 12px;
  line-height: 12px;
}

.weekly-bar-wrap {
  width: 100%;
  height: 60px;
  display: flex;
  align-items: flex-end;
  background: rgba(255,255,255,0.03);
  border-radius: 3px;
  overflow: hidden;
}

.weekly-bar-fill {
  width: 100%;
  background: #c47a5a;
  border-radius: 3px 3px 0 0;
  transition: height 0.6s cubic-bezier(0.22, 1, 0.36, 1);
  min-height: 2px;
}

.weekly-label {
  font-size: 9px;
  color: #5a544e;
  font-family: 'JetBrains Mono', monospace;
}

/* Activity list */
.activity-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.activity-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 8px;
  transition: background 0.15s;
}

.activity-item:hover {
  background: rgba(255,255,255,0.03);
}

.cat-dot {
  width: 6px;
  height: 6px;
  border-radius: 2px;
  flex-shrink: 0;
}

.act-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.act-name {
  font-size: 12.5px;
  font-weight: 500;
  color: #d4cdc5;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.act-title {
  font-size: 10.5px;
  color: #5a544e;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.act-meta {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
  flex-shrink: 0;
}

.act-duration {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  font-weight: 500;
  color: #9e958c;
}

.act-time {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: #5a544e;
  letter-spacing: 0.02em;
}

.act-correct-select {
  font-size: 10px;
  padding: 2px 4px;
  border-radius: 4px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: transparent;
  color: #9e958c;
  outline: none;
  cursor: pointer;
  transition: all 0.2s ease;
}

.act-correct-select:hover {
  border-color: #c47a5a;
  color: #d4cdc5;
}
.act-correct-select option {
  background: #1e1e1e;
  color: #d4cdc5;
}

/* Gantt Chart Styles */
.gantt-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-left: 102px;
}

.gantt-ruler {
  position: relative;
  height: 16px;
  border-bottom: 1px solid rgba(255,255,255,0.06);
  margin-bottom: 4px;
}

.gantt-tick {
  position: absolute;
  top: 0;
  transform: translateX(-50%);
  font-family: 'JetBrains Mono', monospace;
  font-size: 9px;
  color: #5a544e;
}

.gantt-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.gantt-app-name {
  font-size: 11px;
  color: #9e958c;
  width: 90px;
  flex-shrink: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  text-align: right;
}

.gantt-track-wrap {
  flex: 1;
  position: relative;
  height: 20px;
  background: rgba(255,255,255,0.02);
  border-radius: 4px;
}

.gantt-segment {
  position: absolute;
  top: 4px;
  height: 12px;
  border-radius: 3px;
  min-width: 2px;
  cursor: pointer;
  transition: opacity 0.2s;
}

.gantt-segment:hover {
  opacity: 0.8;
}

/* Tooltip for Gantt segments */
.gantt-segment .tooltip {
  visibility: hidden;
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%) translateY(-4px);
  background: #2C2420;
  color: #E8E0D8;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 10px;
  font-family: 'JetBrains Mono', monospace;
  white-space: nowrap;
  z-index: 10;
  opacity: 0;
  transition: opacity 0.2s;
  pointer-events: none;
  box-shadow: 0 2px 8px rgba(0,0,0,0.2);
}

.gantt-segment:hover .tooltip {
  visibility: visible;
  opacity: 1;
}

.empty-hint {
  font-size: 12px;
  color: #5a544e;
  text-align: center;
  padding: 32px 0;
  font-style: italic;
}

/* ── Module goals section ── */
.module-goals-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

/* ── Module manager ── */
.module-manager {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.module-config-row {
  display: grid;
  grid-template-columns: 1fr;
  gap: 8px;
  padding: 10px;
  border: 1px solid rgba(255,255,255,0.04);
  border-radius: 10px;
  background: rgba(255,255,255,0.02);
}

.module-config-main {
  display: flex;
  align-items: center;
  gap: 8px;
}

.module-config-name {
  flex: 1;
  height: 26px;
  background: rgba(255,255,255,0.04);
  border: 1px solid rgba(255,255,255,0.06);
  border-radius: 6px;
  color: #d4cdc5;
  font-size: 12px;
  padding: 0 8px;
  outline: none;
}

.module-config-color {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  padding: 0;
}

.module-config-fields {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
}

.module-config-input {
  height: 26px;
  background: rgba(255,255,255,0.04);
  border: 1px solid rgba(255,255,255,0.06);
  border-radius: 6px;
  color: #d4cdc5;
  font-size: 11px;
  padding: 0 8px;
  outline: none;
}

.module-config-actions {
  display: flex;
  gap: 6px;
}

.module-config-footer {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.mini-btn {
  height: 24px;
  padding: 0 8px;
  border-radius: 6px;
  background: rgba(255,255,255,0.04);
  border: 1px solid rgba(255,255,255,0.06);
  color: #9e958c;
  font-size: 11px;
  cursor: pointer;
}

.mini-btn:hover {
  background: rgba(255,255,255,0.08);
  color: #e8e0d8;
}

.mini-btn.primary {
  background: rgba(196,122,90,0.15);
  border-color: rgba(196,122,90,0.25);
  color: #c47a5a;
}

.mini-btn.danger {
  background: rgba(220, 90, 90, 0.15);
  border-color: rgba(220, 90, 90, 0.25);
  color: #d4726a;
}

.module-goal-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.module-goal-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  width: 160px;
}

.module-goal-dot {
  width: 6px;
  height: 6px;
  border-radius: 2px;
  flex-shrink: 0;
}

.module-goal-name {
  font-size: 12px;
  color: #9e958c;
  width: 44px;
  flex-shrink: 0;
}

.module-goal-input-wrap {
  display: flex;
  align-items: center;
  gap: 3px;
  flex: 1;
}

.module-goal-input {
  width: 52px;
  height: 22px;
  background: rgba(255,255,255,0.04);
  border: 1px solid rgba(255,255,255,0.06);
  border-radius: 6px;
  color: #d4cdc5;
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  text-align: center;
  outline: none;
  padding: 0 4px;
  -moz-appearance: textfield;
  transition: border-color 0.15s;
}
.module-goal-input:focus { border-color: rgba(255,255,255,0.15); }
.module-goal-input::-webkit-inner-spin-button,
.module-goal-input::-webkit-outer-spin-button { -webkit-appearance: none; }
.module-goal-input::placeholder { color: #5a544e; }

.module-goal-unit {
  font-size: 10px;
  color: #5a544e;
}

.module-goal-right {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 10px;
}

.module-prog-track {
  flex: 1;
  height: 5px;
  background: rgba(255,255,255,0.04);
  border-radius: 3px;
  overflow: visible;
  position: relative;
}

.module-prog-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.6s cubic-bezier(0.22,1,0.36,1);
  min-width: 2px;
}

.module-goal-marker {
  position: absolute;
  top: -2px;
  width: 2px;
  height: 9px;
  background: rgba(255,255,255,0.25);
  border-radius: 1px;
  transform: translateX(-50%);
}

.module-actual-time {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: #6e6760;
  width: 48px;
  text-align: right;
  flex-shrink: 0;
}

.module-goal-pct {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: #9e958c;
  width: 32px;
  text-align: right;
  flex-shrink: 0;
}
.module-goal-pct.dim { color: #5a544e; }
/* ── Draggable Sections ── */
.sections-layout {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.draggable-section {
  transition: transform 0.2s ease;
}

.draggable-section.dragging {
  opacity: 0.5;
}

.drag-handle {
  cursor: grab;
}
.drag-handle:active {
  cursor: grabbing;
}
</style>
