<template>
  <Transition name="expand">
    <div v-if="show" class="row-settings">
      <div class="setting-item">
        <span class="setting-label">{{ t('overlay.settings') }}</span>
      </div>
      <div class="setting-item">
        <span class="setting-label">{{ t('overlay.opacity') }}</span>
        <input
          type="range"
          class="slider"
          min="10" max="100" step="5"
          :value="opacityPct"
          @input="$emit('opacityChange', $event)"
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
            @change="$emit('moduleGoalChange', mod.id, $event)"
            @click.stop
          />
        </div>
      </div>

      <div class="setting-item">
        <span class="setting-label">{{ t('overlay.autostart') }}</span>
        <button
          class="toggle-btn"
          :class="{ 'toggle-on': autostartEnabled }"
          @click.stop="$emit('toggleAutostart')"
          :title="autostartEnabled ? t('overlay.autostartOn') : t('overlay.autostartOff')"
          :aria-label="t('overlay.autostart')"
          :aria-pressed="autostartEnabled"
        >
          <span class="toggle-knob" />
        </button>
        <span class="setting-value" style="font-size:9px">{{ autostartEnabled ? 'ON' : 'OFF' }}</span>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { t } from '../shared/i18n'
import type { ModuleConfig } from './types'

defineProps<{
  show: boolean
  moduleConfigs: ModuleConfig[]
  moduleGoals: Record<string, number>
  autostartEnabled: boolean
  opacityPct: number
  catColorOf: (id: string) => string
  moduleGoalHours: (id: string) => string
}>()

defineEmits<{
  toggleAutostart: []
  opacityChange: [e: Event]
  moduleGoalChange: [category: string, e: Event]
}>()
</script>

<style scoped>
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
  background: var(--text-primary);
}

.toggle-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 10px;
  height: 10px;
  background: var(--text-muted);
  border-radius: 50%;
  transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), background 0.2s ease;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.15);
  display: block;
}

.toggle-btn.toggle-on .toggle-knob {
  transform: translateX(14px);
  background: var(--bg-solid);
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
</style>
