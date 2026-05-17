<template>
  <div class="date-picker-wrap" ref="wrapRef">
    <!-- Trigger input look-alike -->
    <div class="date-input-trigger" :class="{ active: isOpen }" @click="toggle" :title="title">
      <svg class="calendar-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
        <line x1="16" y1="2" x2="16" y2="6"></line>
        <line x1="8" y1="2" x2="8" y2="6"></line>
        <line x1="3" y1="10" x2="21" y2="10"></line>
      </svg>
      <span class="date-text" :class="{ placeholder: !modelValue }">{{ displayValue }}</span>
      <button v-if="modelValue && clearable" class="clear-btn" @click.stop="clearDate">✕</button>
    </div>

    <!-- Calendar Popup -->
    <Transition name="fade-down">
      <div v-if="isOpen" class="calendar-popup" ref="popupRef" :class="{ 'align-right': alignRight, 'align-top': alignTop }" @click.stop>
        <div class="calendar-header">
          <button class="cal-nav-btn" @click="prevMonth">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"></polyline></svg>
          </button>
          <span class="cal-title">{{ year }}年 {{ month + 1 }}月</span>
          <button class="cal-nav-btn" @click="nextMonth">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"></polyline></svg>
          </button>
        </div>
        
        <div class="calendar-weekdays">
          <span v-for="wd in ['日', '一', '二', '三', '四', '五', '六']" :key="wd">{{ wd }}</span>
        </div>
        
        <div class="calendar-days">
          <span v-for="b in firstDayOffset" :key="'blank-'+b" class="cal-day blank"></span>
          <button 
            v-for="d in daysInMonth" 
            :key="d" 
            class="cal-day"
            :class="{ 
              selected: isSelected(d), 
              today: isToday(d),
              disabled: isDisabled(d)
            }"
            :disabled="isDisabled(d)"
            @click="selectDay(d)"
          >
            {{ d }}
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { t } from '../../shared/i18n'

const props = defineProps<{
  modelValue: string // YYYY-MM-DD
  min?: string       // YYYY-MM-DD
  max?: string       // YYYY-MM-DD
  title?: string
  clearable?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'change': [value: string]
}>()

const wrapRef = ref<HTMLElement | null>(null)
const popupRef = ref<HTMLElement | null>(null)
const isOpen = ref(false)
const alignRight = ref(false)
const alignTop = ref(false)

const viewDate = ref(new Date())
const year = computed(() => viewDate.value.getFullYear())
const month = computed(() => viewDate.value.getMonth())

watch(() => props.modelValue, (val) => {
  if (val) {
    const d = new Date(val + 'T00:00:00')
    if (!isNaN(d.getTime())) {
      viewDate.value = d
    }
  }
}, { immediate: true })

const displayValue = computed(() => {
  if (!props.modelValue) return t('dashboard.addDate') || 'YYYY / MM / DD'
  const [y, m, d] = props.modelValue.split('-')
  return `${y}/${m}/${d}`
})

const firstDayOffset = computed(() => {
  const firstDay = new Date(year.value, month.value, 1)
  return firstDay.getDay()
})

const daysInMonth = computed(() => {
  return new Date(year.value, month.value + 1, 0).getDate()
})

function formatDateStr(y: number, m: number, d: number) {
  return `${y}-${String(m + 1).padStart(2, '0')}-${String(d).padStart(2, '0')}`
}

function isSelected(day: number) {
  return props.modelValue === formatDateStr(year.value, month.value, day)
}

function isToday(day: number) {
  const dStr = formatDateStr(year.value, month.value, day)
  const t = new Date()
  const todayFormat = `${t.getFullYear()}-${String(t.getMonth()+1).padStart(2, '0')}-${String(t.getDate()).padStart(2, '0')}`
  return dStr === todayFormat
}

function isDisabled(day: number) {
  const dStr = formatDateStr(year.value, month.value, day)
  if (props.min && dStr < props.min) return true
  if (props.max && dStr > props.max) return true
  return false
}

function prevMonth() {
  viewDate.value = new Date(year.value, month.value - 1, 1)
}

function nextMonth() {
  viewDate.value = new Date(year.value, month.value + 1, 1)
}

function selectDay(day: number) {
  const dStr = formatDateStr(year.value, month.value, day)
  emit('update:modelValue', dStr)
  emit('change', dStr)
  isOpen.value = false
}

function clearDate() {
  emit('update:modelValue', '')
  emit('change', '')
  isOpen.value = false
}

async function toggle() {
  isOpen.value = !isOpen.value
  if (isOpen.value) {
    if (props.modelValue) {
      const d = new Date(props.modelValue + 'T00:00:00')
      if (!isNaN(d.getTime())) {
        viewDate.value = d
      }
    }
    // Calculate positioning
    alignRight.value = false
    alignTop.value = false
    await nextTick()
    if (popupRef.value && wrapRef.value) {
      const rect = popupRef.value.getBoundingClientRect()
      const triggerRect = wrapRef.value.getBoundingClientRect()
      
      // Check horizontal overflow
      if (rect.right > window.innerWidth - 10) {
        alignRight.value = true
      }
      
      // Check vertical overflow
      if (rect.bottom > window.innerHeight - 10 && triggerRect.top > rect.height) {
        alignTop.value = true
      }
    }
  }
}

function onClickOutside(e: MouseEvent) {
  if (isOpen.value && wrapRef.value && !wrapRef.value.contains(e.target as Node)) {
    isOpen.value = false
  }
}

onMounted(() => document.addEventListener('click', onClickOutside))
onUnmounted(() => document.removeEventListener('click', onClickOutside))
</script>

<style scoped>
.date-picker-wrap {
  position: relative;
  display: inline-block;
  font-family: 'Outfit', sans-serif;
}

.date-input-trigger {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--surface-03);
  border: 1px solid var(--surface-05);
  border-radius: 25px;
  color: var(--text-secondary);
  font-size: 11px;
  padding: 4px 10px;
  height: 28px;
  cursor: pointer;
  transition: all 0.15s ease;
  user-select: none;
  min-width: 100px;
}

.date-input-trigger:hover, .date-input-trigger.active {
  border-color: var(--surface-10);
  background: var(--surface-05);
  color: var(--text-primary);
}

.date-input-trigger.active {
  border-color: rgba(196, 122, 90, 0.4);
  box-shadow: 0 0 0 2px rgba(196, 122, 90, 0.1);
  color: #c47a5a;
}

.calendar-icon {
  opacity: 0.8;
  flex-shrink: 0;
}

.date-text {
  flex: 1;
}

.date-text.placeholder {
  color: var(--text-dim);
}

.clear-btn {
  background: none;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  padding: 0 2px;
  font-size: 10px;
  display: flex;
  align-items: center;
  opacity: 0;
  transition: opacity 0.15s, color 0.15s;
}

.date-input-trigger:hover .clear-btn {
  opacity: 1;
}

.clear-btn:hover {
  color: #e74c3c;
}

.calendar-popup {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  width: 220px;
  background: var(--bg-solid);
  border: 1px solid var(--surface-10);
  border-radius: 16px;
  padding: 12px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  z-index: 100;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.calendar-popup.align-right {
  left: auto;
  right: 0;
}

.calendar-popup.align-top {
  top: auto;
  bottom: calc(100% + 6px);
}

.calendar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.cal-nav-btn {
  background: var(--surface-03);
  border: none;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: var(--text-secondary);
  transition: all 0.15s;
}

.cal-nav-btn:hover {
  background: var(--surface-08);
  color: var(--text-primary);
}

.cal-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.calendar-weekdays {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  text-align: center;
  font-size: 10px;
  color: var(--text-muted);
  font-weight: 500;
  margin-bottom: 4px;
}

.calendar-days {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 2px;
}

.cal-day {
  aspect-ratio: 1;
  background: none;
  border: none;
  border-radius: 8px;
  font-size: 11px;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.1s;
}

.cal-day.blank {
  cursor: default;
}

.cal-day:not(.blank):not(.disabled):hover {
  background: var(--surface-08);
  color: var(--text-primary);
}

.cal-day.today {
  color: #c47a5a;
  font-weight: 600;
}

.cal-day.selected {
  background: #c47a5a;
  color: #fff;
  font-weight: 600;
}

.cal-day.disabled {
  opacity: 0.3;
  cursor: not-allowed;
  text-decoration: line-through;
}

/* Transitions */
.fade-down-enter-active,
.fade-down-leave-active {
  transition: opacity 0.2s ease, transform 0.2s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}
.fade-down-enter-from,
.fade-down-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.98);
}
</style>
