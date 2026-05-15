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
        <span class="card-value">{{ fmtDur(activeTotalSecs) }}</span>
        <span v-if="compareData" class="card-diff" :class="diffPct(activeTotalSecs, compareData.totalSecs - (compareData.categoryStats.find(s => s.category === CATEGORY.AFK)?.totalSecs ?? 0)) >= 0 ? 'up' : 'down'">
          {{ diffPct(activeTotalSecs, compareData.totalSecs - (compareData.categoryStats.find(s => s.category === CATEGORY.AFK)?.totalSecs ?? 0)) >= 0 ? '+' : '' }}{{ diffPct(activeTotalSecs, compareData.totalSecs - (compareData.categoryStats.find(s => s.category === CATEGORY.AFK)?.totalSecs ?? 0)) }}%
        </span>
      </div>
      <div class="card">
        <span class="card-label">{{ t('dashboard.sessions') }}</span>
        <span class="card-value">{{ activeSessionCount }}</span>
        <span v-if="compareData" class="card-diff" :class="diffPct(activeSessionCount, compareData.activities.filter(a => a.category !== CATEGORY.AFK).length) >= 0 ? 'up' : 'down'">
          {{ diffPct(activeSessionCount, compareData.activities.filter(a => a.category !== CATEGORY.AFK).length) >= 0 ? '+' : '' }}{{ diffPct(activeSessionCount, compareData.activities.filter(a => a.category !== CATEGORY.AFK).length) }}%
        </span>
      </div>
      <div class="card">
        <span class="card-label">{{ t('dashboard.focus') }}</span>
        <span class="card-value">{{ fmtDur(focusSecs) }}</span>
        <span v-if="compareData" class="card-diff" :class="diffPct(focusSecs, compareData.categoryStats.filter(s => isFocus(s.category)).reduce((sum, s) => sum + s.totalSecs, 0)) >= 0 ? 'up' : 'down'">
          {{ diffPct(focusSecs, compareData.categoryStats.filter(s => isFocus(s.category)).reduce((sum, s) => sum + s.totalSecs, 0)) >= 0 ? '+' : '' }}{{ diffPct(focusSecs, compareData.categoryStats.filter(s => isFocus(s.category)).reduce((sum, s) => sum + s.totalSecs, 0)) }}%
        </span>
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
      <div class="custom-range" :class="{ active: dateRange === 'custom' }">
        <input type="date" class="date-input" v-model="customStart" @change="dateRange = 'custom'; setDateRange('custom')" />
        <span class="range-sep">~</span>
        <input type="date" class="date-input" v-model="customEnd" @change="dateRange = 'custom'; setDateRange('custom')" />
      </div>
      <button class="date-tab compare-btn" :class="{ active: compareMode }" @click="toggleCompare()">
        {{ t('dashboard.compare') }}
      </button>
    </div>

    <!-- Compare date picker -->
    <div v-if="compareMode" class="compare-bar">
      <span class="compare-label">{{ t('dashboard.vs') }}:</span>
      <input type="date" class="date-input" v-model="compareStart" @change="loadCompareData()" />
      <span class="range-sep">~</span>
      <input type="date" class="date-input" v-model="compareEnd" @change="loadCompareData()" />
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
              v-for="cat in categoryApps.filter(c => c.category !== CATEGORY.AFK)"
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

        <!-- Hourly Distribution -->
        <section class="section" v-else-if="secName === 'hourlyDist'">
          <div class="section-header" @click="toggleSection('hourlyDist')">
            <h2 class="section-title drag-handle">{{ t('dashboard.hourlyDistribution') }}</h2>
            <svg class="section-toggle" :class="{ open: !collapsed.hourlyDist }" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
          </div>
          <div v-if="!collapsed.hourlyDist" class="section-body">
            <div v-if="hourlyData.length === 0" class="empty-hint">
              {{ t('dashboard.noData') }}
            </div>
            <div v-else class="hourly-chart">
              <div v-for="h in 24" :key="h - 1" class="hourly-col">
                <span class="hourly-val">{{ hourlyDataMap[h - 1] ? fmtDurShort(hourlyDataMap[h - 1]) : '' }}</span>
                <div class="hourly-bar-wrap">
                  <div
                    class="hourly-bar-fill"
                    :style="{ height: hourlyBarH(hourlyDataMap[h - 1] ?? 0) + 'px' }"
                  />
                </div>
                <span class="hourly-label">{{ (h - 1) % 2 === 0 ? (h - 1) : '' }}</span>
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
            <div class="act-filter-bar">
              <input
                class="act-search"
                v-model="activitySearch"
                :placeholder="t('dashboard.searchActivities')"
              />
              <div class="act-cat-tags">
                <button
                  class="cat-tag"
                  :class="{ active: !activityCategoryFilter }"
                  @click="activityCategoryFilter = null"
                >All</button>
                <button
                  v-for="stat in data.categoryStats.filter(s => s.category !== CATEGORY.AFK)"
                  :key="stat.category"
                  class="cat-tag"
                  :class="{ active: activityCategoryFilter === stat.category }"
                  :style="activityCategoryFilter === stat.category ? { background: catColor(stat.category), borderColor: catColor(stat.category), color: 'var(--bg)' } : {}"
                  @click="activityCategoryFilter = activityCategoryFilter === stat.category ? null : stat.category"
                >{{ catLabel(stat.category) }}</button>
              </div>
            </div>
            <div class="activity-list">
              <div v-if="filteredActivities.length === 0" class="empty-hint">
                {{ t('dashboard.noData') }}
              </div>
              <div
                v-for="(act, idx) in filteredActivities"
                :key="act.id ?? idx"
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
                    <option value="uncategorized" v-if="!moduleConfigs.find(m => m.id === CATEGORY.UNCATEGORIZED)">Uncategorized</option>
                  </select>
                </div>
              </div>
            </div>
          </div>
        </section>

        <!-- Web history (today only) -->
        <section class="section" v-else-if="secName === 'webHistory'">
          <template v-if="dateRange === 'today'">
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
                  <span class="act-duration">{{ fmtDur(visit.totalDuration) }}</span>
                  <span class="act-count">{{ visit.visitCount }}×</span>
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

        <!-- Pomodoro Settings -->
        <section class="section" v-else-if="secName === 'pomodoroSettings'">
          <div class="section-header" @click="toggleSection('pomodoroSettings')">
            <h2 class="section-title drag-handle">{{ t('dashboard.pomodoroSettings') }}</h2>
            <svg class="section-toggle" :class="{ open: !collapsed.pomodoroSettings }" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
          </div>
          <div v-if="!collapsed.pomodoroSettings" class="section-body">
            <div class="module-goals-list">
              <div class="module-goal-row">
                <div class="module-goal-left">
                  <span class="module-goal-name">{{ t('dashboard.focusDuration') }}</span>
                  <div class="module-goal-input-wrap">
                    <input
                      type="number"
                      class="module-goal-input"
                      v-model.number="pomodoroFocusMins"
                      min="1" max="120" step="1"
                      @change="savePomodoroSettings"
                    />
                    <span class="module-goal-unit">min</span>
                  </div>
                </div>
              </div>
              <div class="module-goal-row">
                <div class="module-goal-left">
                  <span class="module-goal-name">{{ t('dashboard.breakDuration') }}</span>
                  <div class="module-goal-input-wrap">
                    <input
                      type="number"
                      class="module-goal-input"
                      v-model.number="pomodoroBreakMins"
                      min="1" max="30" step="1"
                      @change="savePomodoroSettings"
                    />
                    <span class="module-goal-unit">min</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        <!-- Data Management -->
        <section class="section" v-else-if="secName === 'dataManagement'">
          <div class="section-header" @click="toggleSection('dataManagement')">
            <h2 class="section-title drag-handle">{{ t('dashboard.dataManagement') }}</h2>
            <svg class="section-toggle" :class="{ open: !collapsed.dataManagement }" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
          </div>
          <div v-if="!collapsed.dataManagement" class="section-body">
            <div class="clear-options">
              <label class="clear-option">
                <input type="checkbox" v-model="clearOpts.activities" />
                <span>{{ t('dashboard.clearActivities') }}</span>
              </label>
              <label class="clear-option">
                <input type="checkbox" v-model="clearOpts.webHistory" />
                <span>{{ t('dashboard.clearWebHistory') }}</span>
              </label>
              <label class="clear-option">
                <input type="checkbox" v-model="clearOpts.moduleConfig" />
                <span>{{ t('dashboard.clearModuleConfig') }}</span>
              </label>
              <label class="clear-option">
                <input type="checkbox" v-model="clearOpts.windowSettings" />
                <span>{{ t('dashboard.clearWindowSettings') }}</span>
              </label>
            </div>
            <div class="module-config-footer">
              <button
                class="mini-btn danger"
                :class="{ confirming: clearConfirming }"
                :disabled="!anyClearSelected"
                @click="handleClearData"
              >{{ clearConfirming ? t('dashboard.clearDataConfirm') : t('dashboard.clearData') }}</button>
            </div>
            <div class="export-row">
              <button class="mini-btn" @click="exportJSON">{{ t('dashboard.exportJSON') }}</button>
              <button class="mini-btn" @click="exportCSV">{{ t('dashboard.exportCSV') }}</button>
            </div>
          </div>
        </section>

      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { t, loadLocale, getLocale, setLocale } from '../shared/i18n'
import { CATEGORY, isFocus } from '../shared/constants'

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
  totalDuration: number
}

interface DailyFocus {
  date: string       // "YYYY-MM-DD"
  focusSecs: number
}

interface HourlyStat {
  hour: number
  totalSecs: number
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


type CollapsibleSection = 'moduleManager' | 'activities' | 'webHistory' | 'dataManagement' | 'hourlyDist' | 'pomodoroSettings'
const collapsed = ref<Record<CollapsibleSection, boolean>>({
  moduleManager: true,
  activities: true,
  webHistory: true,
  dataManagement: true,
  hourlyDist: true,
  pomodoroSettings: true,
})

// ── New reactive state ────────────────────────────────────────────────────────

type DateRange = 'today' | 'yesterday' | 'week' | 'month' | 'custom'
const dateRange = ref<DateRange>('today')
const customStart = ref('')
const customEnd = ref('')
const weeklySeries = ref<DailyFocus[]>([])
const webHistory = ref<WebVisit[]>([])
const hourlyData = ref<HourlyStat[]>([])

// Search & filter
const activitySearch = ref('')
const activityCategoryFilter = ref<string | null>(null)

// Compare mode
const compareMode = ref(false)
const compareStart = ref('')
const compareEnd = ref('')
const compareData = ref<DashboardData | null>(null)
const compareCategoryApps = ref<CategoryAppBreakdown[]>([])

const clearOpts = ref({ activities: false, webHistory: false, moduleConfig: false, windowSettings: false })
const clearConfirming = ref(false)
let clearTimer: ReturnType<typeof setTimeout> | null = null

const layoutOrder = ref([
  'appUsage',
  'moduleGoals',
  'hourlyDist',
  'pomodoroSettings',
  'moduleManager',
  'weeklyTrend',
  'activities',
  'webHistory',
  'dataManagement',
])

let draggedSecIdx: number | null = null

function onSecDragStart(e: DragEvent, idx: number) {
  draggedSecIdx = idx
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move'
  }
}

function onSecDragEnter(_e: DragEvent, idx: number) {
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

const theme = ref<'day' | 'night'>('day')
const locale = getLocale()

async function loadTheme() {
  try {
    const saved = await invoke<string>('get_theme')
    if (saved === 'day' || saved === 'night') {
      theme.value = saved
      if (theme.value === 'night') {
        document.documentElement.classList.add('night')
      } else {
        document.documentElement.classList.remove('night')
      }
    }
  } catch (e) {
    console.warn('loadTheme failed', e)
  }
}

const localeLabel = computed(() => {
  return t('overlay.switchLang')
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
    .filter(s => isFocus(s.category))
    .reduce((sum, s) => sum + s.totalSecs, 0)
})

// 排除 afk 后的总时长和活动数
const activeTotalSecs = computed(() => {
  const afkSecs = data.value.categoryStats
    .filter(s => s.category === CATEGORY.AFK)
    .reduce((sum, s) => sum + s.totalSecs, 0)
  return Math.max(0, data.value.totalSecs - afkSecs)
})
const activeSessionCount = computed(() => {
  return data.value.activities.filter(a => a.category !== CATEGORY.AFK).length
})

// Filtered activities for search/filter
const filteredActivities = computed(() => {
  return data.value.activities
    .filter(a => a.category !== CATEGORY.AFK)
    .filter(a => !activityCategoryFilter.value || a.category === activityCategoryFilter.value)
    .filter(a => {
      if (!activitySearch.value) return true
      const q = activitySearch.value.toLowerCase()
      return a.appName.toLowerCase().includes(q) || a.windowTitle.toLowerCase().includes(q)
    })
    .slice().reverse()
})

// Compare diff helpers
function diffPct(main: number, cmp: number): number {
  if (cmp === 0) return main > 0 ? 100 : 0
  return Math.round(((main - cmp) / cmp) * 100)
}



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
    case 'custom': {
      if (!customStart.value || !customEnd.value) {
        return { start: todayStartTs, end: todayStartTs + 86400 }
      }
      const cs = new Date(customStart.value + 'T00:00:00')
      const ce = new Date(customEnd.value + 'T00:00:00')
      const endTs = Math.floor(ce.getTime() / 1000) + 86400
      return { start: Math.floor(cs.getTime() / 1000), end: endTs }
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
  return Math.max(...data.value.categoryStats.filter(s => s.category !== CATEGORY.AFK).map(s => s.totalSecs), 1)
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
  if (cat === CATEGORY.UNCATEGORIZED) return t('category.other')
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

// ── Hourly chart helpers ─────────────────────────────────────────────────────

const hourlyDataMap = computed(() => {
  const map: Record<number, number> = {}
  hourlyData.value.forEach(h => { map[h.hour] = h.totalSecs })
  return map
})

const maxHourlySecs = computed(() =>
  Math.max(...hourlyData.value.map(h => h.totalSecs), 1)
)

function hourlyBarH(secs: number): number {
  if (secs === 0) return 2
  return Math.max((secs / maxHourlySecs.value) * 80, 4)
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

const pomodoroFocusMins = ref(25)
const pomodoroBreakMins = ref(5)

async function loadPomodoroSettings() {
  try {
    const [f, b] = await invoke<[number, number]>('get_pomodoro_settings')
    pomodoroFocusMins.value = f
    pomodoroBreakMins.value = b
  } catch (e) {
    console.warn('get_pomodoro_settings failed:', e)
  }
}

async function savePomodoroSettings() {
  try {
    await invoke('set_pomodoro_settings', {
      focusMins: pomodoroFocusMins.value,
      breakMins: pomodoroBreakMins.value
    })
  } catch (e) {
    console.warn('set_pomodoro_settings failed:', e)
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

const anyClearSelected = computed(() => {
  const o = clearOpts.value
  return o.activities || o.webHistory || o.moduleConfig || o.windowSettings
})

function handleClearData() {
  if (!clearConfirming.value) {
    clearConfirming.value = true
    if (clearTimer) clearTimeout(clearTimer)
    clearTimer = setTimeout(() => { clearConfirming.value = false }, 3000)
    return
  }
  if (clearTimer) { clearTimeout(clearTimer); clearTimer = null }
  clearConfirming.value = false

  const opts = { ...clearOpts.value }
  invoke('clear_data', { options: opts }).then(() => {
    clearOpts.value = { activities: false, webHistory: false, moduleConfig: false, windowSettings: false }
    load()
    loadModules()
    loadModuleGoals()
    loadModuleConfigs()
  }).catch(e => {
    console.warn('clear_data failed:', e)
  })
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

async function loadHourlyData() {
  try {
    const { start, end } = rangeTimestamps.value
    hourlyData.value = await invoke<HourlyStat[]>('get_hourly_distribution', {
      startTs: start,
      endTs: end,
    })
  } catch (e) {
    console.warn('get_hourly_distribution failed:', e)
  }
}

// ── Compare mode ─────────────────────────────────────────────────────────────

async function toggleCompare() {
  if (compareMode.value) {
    compareMode.value = false
    compareData.value = null
    compareCategoryApps.value = []
    return
  }
  compareMode.value = true
  // Default: same length period immediately before current range
  const { start, end } = rangeTimestamps.value
  const len = end - start
  const cs = new Date((start - len) * 1000)
  const ce = new Date((start - 1) * 1000)
  compareStart.value = cs.toISOString().slice(0, 10)
  compareEnd.value = ce.toISOString().slice(0, 10)
  await loadCompareData()
}

async function loadCompareData() {
  if (!compareStart.value || !compareEnd.value) return
  const cs = new Date(compareStart.value + 'T00:00:00')
  const ce = new Date(compareEnd.value + 'T00:00:00')
  const startTs = Math.floor(cs.getTime() / 1000)
  const endTs = Math.floor(ce.getTime() / 1000) + 86400
  try {
    const [dd, ca] = await Promise.all([
      invoke<DashboardData>('get_dashboard_data_range', { startTs, endTs }),
      invoke<CategoryAppBreakdown[]>('get_category_app_breakdown', { startTs, endTs }),
    ])
    compareData.value = dd
    compareCategoryApps.value = ca
  } catch (e) {
    console.warn('loadCompareData failed:', e)
  }
}

// ── Data export ──────────────────────────────────────────────────────────────

function downloadFile(content: string, filename: string, type: string) {
  const blob = new Blob([content], { type })
  const a = document.createElement('a')
  a.href = URL.createObjectURL(blob)
  a.download = filename
  a.click()
  URL.revokeObjectURL(a.href)
}

function exportJSON() {
  const payload = {
    dateRange: dateRange.value,
    range: rangeTimestamps.value,
    activities: data.value.activities,
    categoryStats: data.value.categoryStats,
    categoryApps: categoryApps.value,
    hourlyDistribution: hourlyData.value,
    webHistory: webHistory.value,
    weeklySeries: weeklySeries.value,
  }
  downloadFile(JSON.stringify(payload, null, 2), 'insightflow-export.json', 'application/json')
}

function exportCSV() {
  const rows = [['app_name', 'window_title', 'category', 'start', 'end', 'duration_secs']]
  data.value.activities.forEach(a => {
    rows.push([
      a.appName,
      a.windowTitle,
      a.category,
      new Date(a.startTime * 1000).toISOString(),
      new Date(a.endTime * 1000).toISOString(),
      String(a.endTime - a.startTime),
    ])
  })
  downloadFile(
    rows.map(r => r.map(c => `"${c.replace(/"/g, '""')}"`).join(',')).join('\n'),
    'insightflow-export.csv',
    'text/csv',
  )
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




function appBarWidth(appSecs: number, totalSecs: number): string {
  if (totalSecs <= 0) return '0%'
  return Math.max((appSecs / totalSecs) * 100, 3) + '%'
}

async function setDateRange(range: DateRange) {
  dateRange.value = range
  await Promise.all([load(), loadCategoryApps(), loadHourlyData(), loadWebHistory()])
}

function toggleSection(key: CollapsibleSection) {
  collapsed.value[key] = !collapsed.value[key]
}

// ── Lifecycle ─────────────────────────────────────────────────────────────────

onMounted(async () => {
  await loadTheme()
  listen('theme-changed', (event: any) => {
    theme.value = event.payload
    if (theme.value === 'night') {
      document.documentElement.classList.add('night')
    } else {
      document.documentElement.classList.remove('night')
    }
  })

  const savedOrder = localStorage.getItem('dashboard-layout-order')
  if (savedOrder) {
    try { 
      const parsed = JSON.parse(savedOrder)
      if (Array.isArray(parsed)) {
        const defaultOrder = [
          'appUsage', 'moduleGoals', 'hourlyDist', 'pomodoroSettings', 
          'moduleManager', 'weeklyTrend', 'activities', 'webHistory', 'dataManagement'
        ]
        for (const item of defaultOrder) {
          if (!parsed.includes(item)) {
            parsed.push(item)
          }
        }
        layoutOrder.value = parsed
      }
    } catch (e) {}
  }

  await Promise.all([
    load(),
    loadWeeklySeries(),
    loadWebHistory(),
    loadModules(),
    loadPomodoroSettings(),
    loadModuleGoals(),
    loadModuleConfigs(),
    loadCategoryApps(),
    loadHourlyData(),
  ])
})
</script>

<style scoped>
.dashboard {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  padding: 24px;
  background: var(--bg);
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
  background: var(--surface-08);
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
  color: var(--text-primary);
  letter-spacing: -0.02em;
}

.date {
  font-size: 12px;
  color: var(--text-muted);
  letter-spacing: 0.02em;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.lang-btn {
  background: var(--surface-05);
  border: 1px solid var(--surface-08);
  border-radius: 6px;
  padding: 2px 8px;
  font-size: 11px;
  font-weight: 500;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s ease;
  font-family: 'Outfit', sans-serif;
}

.lang-btn:hover {
  background: var(--surface-10);
  color: var(--text-primary);
  border-color: var(--surface-12);
}

/* Summary */
.summary {
  display: flex;
  gap: 12px;
}

.card {
  flex: 1;
  background: var(--surface-03);
  border: 1px solid var(--surface-05);
  border-radius: 12px;
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.card-label {
  font-size: 10.5px;
  font-weight: 400;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.card-value {
  font-family: 'JetBrains Mono', monospace;
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
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
  background: var(--surface-03);
  border: 1px solid var(--surface-05);
  border-radius: 8px;
  font-size: 11px;
  font-weight: 500;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s ease;
  font-family: 'Outfit', sans-serif;
}

.date-tab:hover {
  background: var(--surface-06);
  color: var(--text-secondary);
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
  color: var(--text-muted);
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
  color: var(--text-dim);
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
  background: var(--surface-03);
}

.bar-row.plain {
  cursor: default;
}

.bar-row.plain:hover {
  background: transparent;
}

.chevron {
  flex-shrink: 0;
  color: var(--text-dim);
  transition: transform 0.2s ease;
  margin-left: 4px;
}

.chevron.open {
  transform: rotate(180deg);
}

.bar-label {
  font-size: 12px;
  color: var(--text-secondary);
  width: 90px;
  flex-shrink: 0;
}

.bar-track {
  flex: 1;
  height: 6px;
  background: var(--surface-04);
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
  color: var(--text-muted);
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
  color: var(--text-muted);
  width: 120px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex-shrink: 0;
}

.cat-app-track {
  flex: 1;
  height: 4px;
  background: var(--surface-03);
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
  color: var(--text-dim);
  width: 56px;
  text-align: right;
  flex-shrink: 0;
}

.cat-app-empty {
  font-size: 11px;
  color: var(--text-dim);
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
  color: var(--text-muted);
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
  background: var(--surface-03);
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
  color: var(--text-dim);
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
  background: var(--surface-03);
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
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.act-title {
  font-size: 10.5px;
  color: var(--text-dim);
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
  color: var(--text-secondary);
}

.act-count {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: var(--text-muted);
}

.act-time {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: var(--text-dim);
  letter-spacing: 0.02em;
}

.act-correct-select {
  font-size: 10px;
  padding: 2px 4px;
  border-radius: 4px;
  border: 1px solid var(--surface-10);
  background: transparent;
  color: var(--text-secondary);
  outline: none;
  cursor: pointer;
  transition: all 0.2s ease;
}

.act-correct-select:hover {
  border-color: #c47a5a;
  color: var(--text-primary);
}
.act-correct-select option {
  background: #1e1e1e;
  color: var(--text-primary);
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
  border-bottom: 1px solid var(--surface-06);
  margin-bottom: 4px;
}

.gantt-tick {
  position: absolute;
  top: 0;
  transform: translateX(-50%);
  font-family: 'JetBrains Mono', monospace;
  font-size: 9px;
  color: var(--text-dim);
}

.gantt-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.gantt-app-name {
  font-size: 11px;
  color: var(--text-secondary);
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
  background: var(--surface-02);
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
  color: var(--text-dim);
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
  border: 1px solid var(--surface-04);
  border-radius: 10px;
  background: var(--surface-02);
}

.module-config-main {
  display: flex;
  align-items: center;
  gap: 8px;
}

.module-config-name {
  flex: 1;
  height: 26px;
  background: var(--surface-04);
  border: 1px solid var(--surface-06);
  border-radius: 6px;
  color: var(--text-primary);
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
  background: var(--surface-04);
  border: 1px solid var(--surface-06);
  border-radius: 6px;
  color: var(--text-primary);
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
  background: var(--surface-04);
  border: 1px solid var(--surface-06);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
}

.mini-btn:hover {
  background: var(--surface-08);
  color: var(--text-primary);
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

.mini-btn.danger.confirming {
  background: rgba(220, 90, 90, 0.35);
  border-color: rgba(220, 90, 90, 0.5);
  color: #e85555;
  animation: pulse-danger 0.8s ease infinite;
}

@keyframes pulse-danger {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

.mini-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.clear-options {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 10px;
}

.clear-option {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-secondary, var(--text-secondary));
  cursor: pointer;
}

.clear-option input[type="checkbox"] {
  accent-color: #c47a5a;
  width: 14px;
  height: 14px;
  cursor: pointer;
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
  color: var(--text-secondary);
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
  background: var(--surface-04);
  border: 1px solid var(--surface-06);
  border-radius: 6px;
  color: var(--text-primary);
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  text-align: center;
  outline: none;
  padding: 0 4px;
  -moz-appearance: textfield;
  transition: border-color 0.15s;
}
.module-goal-input:focus { border-color: var(--surface-15); }
.module-goal-input::-webkit-inner-spin-button,
.module-goal-input::-webkit-outer-spin-button { -webkit-appearance: none; }
.module-goal-input::placeholder { color: var(--text-dim); }

.module-goal-unit {
  font-size: 10px;
  color: var(--text-dim);
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
  background: var(--surface-04);
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
  background: var(--surface-25);
  border-radius: 1px;
  transform: translateX(-50%);
}

.module-actual-time {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: var(--text-muted);
  width: 48px;
  text-align: right;
  flex-shrink: 0;
}

.module-goal-pct {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: var(--text-secondary);
  width: 32px;
  text-align: right;
  flex-shrink: 0;
}
.module-goal-pct.dim { color: var(--text-dim); }
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

/* ── Custom date range ──────────────────────────────────────────────────────── */

.custom-range {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: 4px;
}

.range-sep {
  color: var(--text-dim);
  font-size: 11px;
}

.date-input {
  background: var(--surface-03);
  border: 1px solid var(--surface-05);
  border-radius: 8px;
  color: var(--text-muted);
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  padding: 4px 6px;
  height: 28px;
  cursor: pointer;
  transition: all 0.15s ease;
}
.date-input:hover {
  border-color: var(--surface-10);
  color: var(--text-secondary);
}
.date-input:focus {
  outline: none;
  border-color: rgba(196, 122, 90, 0.25);
  color: #c47a5a;
}

.custom-range.active .date-input {
  background: rgba(196, 122, 90, 0.15);
  border-color: rgba(196, 122, 90, 0.25);
  color: #c47a5a;
}

/* Color scheme for date input calendar icon */
.date-input::-webkit-calendar-picker-indicator {
  filter: invert(0.5);
  cursor: pointer;
}

/* ── Compare mode ──────────────────────────────────────────────────────────── */

.compare-btn {
  margin-left: auto;
  flex-shrink: 0;
}

.compare-bar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 16px 10px;
}

.compare-label {
  font-size: 11px;
  color: var(--text-dim);
  font-family: 'Outfit', sans-serif;
}

.card-diff {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 500;
  margin-top: 2px;
}
.card-diff.up {
  color: #5a9e6f;
}
.card-diff.down {
  color: #d4726a;
}

/* ── Hourly chart ──────────────────────────────────────────────────────────── */

.hourly-chart {
  display: flex;
  align-items: flex-end;
  gap: 3px;
  padding-top: 4px;
}

.hourly-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
}

.hourly-val {
  font-size: 7px;
  color: var(--text-muted);
  font-family: 'JetBrains Mono', monospace;
  white-space: nowrap;
  height: 10px;
  line-height: 10px;
}

.hourly-bar-wrap {
  width: 100%;
  height: 80px;
  display: flex;
  align-items: flex-end;
  background: var(--surface-03);
  border-radius: 3px;
  overflow: hidden;
}

.hourly-bar-fill {
  width: 100%;
  background: #c47a5a;
  border-radius: 3px 3px 0 0;
  transition: height 0.6s cubic-bezier(0.22, 1, 0.36, 1);
  min-height: 2px;
}

.hourly-label {
  font-size: 8px;
  color: var(--text-dim);
  font-family: 'JetBrains Mono', monospace;
}

/* ── Activity search & filter ──────────────────────────────────────────────── */

.act-filter-bar {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 10px;
}

.act-search {
  width: 100%;
  background: var(--surface-04);
  border: 1px solid var(--surface-05);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 11px;
  font-family: 'Outfit', sans-serif;
  padding: 6px 10px;
  transition: all 0.15s ease;
  box-sizing: border-box;
}
.act-search::placeholder {
  color: var(--text-dim);
}
.act-search:focus {
  outline: none;
  border-color: rgba(196, 122, 90, 0.25);
  background: var(--surface-06);
}

.act-cat-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.cat-tag {
  background: var(--surface-03);
  border: 1px solid var(--surface-05);
  border-radius: 4px;
  color: var(--text-muted);
  font-size: 10px;
  font-family: 'Outfit', sans-serif;
  padding: 2px 8px;
  cursor: pointer;
  transition: all 0.15s ease;
}
.cat-tag:hover {
  background: var(--surface-06);
  color: var(--text-secondary);
}
.cat-tag.active {
  background: var(--surface-08);
  border-color: var(--surface-12);
  color: var(--text-primary);
}

/* ── Export buttons ────────────────────────────────────────────────────────── */

.export-row {
  display: flex;
  gap: 6px;
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--surface-05);
}
</style>
