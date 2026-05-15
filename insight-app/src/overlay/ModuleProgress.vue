<template>
  <Transition name="expand">
    <div v-if="show" class="row-modules">
      <div
        v-for="mod in modules"
        :key="mod.category"
        class="mod-entry"
      >
        <div class="mod-header" @click="$emit('toggleDetail', mod.category)">
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
          <div v-if="mod.topApps.length === 0" class="mod-app-empty">{{ t('overlay.noRecord') }}</div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { t } from '../shared/i18n'
import type { ModuleProgress } from './types'

defineProps<{
  show: boolean
  modules: ModuleProgress[]
  expandedModule: string | null
  catColorOf: (cat: string) => string
  catLabel: (cat: string) => string
  modBarWidth: (mod: ModuleProgress) => string
  fmtDur: (secs: number) => string
}>()

defineEmits<{
  toggleDetail: [category: string]
}>()
</script>

<style scoped>
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
</style>
