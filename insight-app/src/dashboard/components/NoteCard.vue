<template>
  <div
    class="note-card"
    :class="{ pinned: note.pinned, editing: isEditing }"
    :style="cardStyle"
    draggable="true"
    @dragstart="$emit('dragstart', $event)"
    @dragover.prevent
    @dragenter="$emit('dragenter', $event)"
    @dragend="$emit('dragend')"
    @click="startEdit"
  >
    <!-- Title -->
    <div class="nc-title" v-if="!isEditing">{{ note.title || t('dashboard.noTitle') }}</div>
    <input
      v-else
      class="nc-title-input"
      ref="titleInputRef"
      :value="note.title"
      @input="onTitleInput"
      @blur="finishEdit"
      @keydown.enter="finishEdit"
      @click.stop
      :placeholder="t('dashboard.noTitle')"
    />

    <!-- Content preview -->
    <div class="nc-body" v-if="!isEditing">
      <!-- Checklist preview -->
      <template v-if="note.noteType === 'checklist'">
        <div class="nc-checklist-preview">
          <div
            v-for="item in previewChecklist"
            :key="item.id"
            class="nc-check-item"
          >
            <span class="nc-checkbox" :class="{ done: item.done }">
              <svg v-if="item.done" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
            </span>
            <span class="nc-check-text" :class="{ done: item.done }">{{ item.text }}</span>
          </div>
          <div v-if="checklistItems.length > 3" class="nc-more">+{{ checklistItems.length - 3 }}</div>
          <div class="nc-check-progress" v-if="checklistItems.length > 0">
            {{ checklistDoneCount }}/{{ checklistItems.length }}
          </div>
        </div>
      </template>
      <!-- Markdown preview -->
      <template v-else>
        <div class="nc-md-preview" v-if="note.content" v-html="renderMarkdown(note.content)"></div>
        <div class="nc-empty" v-else>{{ t('dashboard.quickNote') }}</div>
      </template>
    </div>
    <textarea
      v-else
      class="nc-content-input"
      :value="note.content"
      @input="onContentInput"
      @blur="finishEdit"
      @click.stop
      :placeholder="t('dashboard.noteContent')"
    />

    <!-- Actions bar (visible on hover) -->
    <div class="nc-actions" @click.stop>
      <button class="nc-action" :title="t('dashboard.pin')" @click="$emit('pin', !note.pinned)">
        <svg width="12" height="12" viewBox="0 0 24 24" :fill="note.pinned ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
      </button>
      <button class="nc-action" :title="note.noteType === 'checklist' ? t('dashboard.markdown') : t('dashboard.checklist')" @click="$emit('toggleType')">
        <svg v-if="note.noteType === 'checklist'" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
        <svg v-else width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 11 12 14 22 4"/><path d="M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11"/></svg>
      </button>
      <button class="nc-action" :title="t('dashboard.popOut')" @click="$emit('popOut')">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
      </button>
      <div class="nc-color-picker" @click.stop>
        <span
          v-for="c in colors"
          :key="c"
          class="nc-color-dot"
          :style="{ background: c }"
          :class="{ selected: note.color === c }"
          @click="$emit('color', c)"
        />
      </div>
      <button class="nc-action danger" :title="t('dashboard.delete')" @click="$emit('delete')">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/></svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { marked } from 'marked'
import { t } from '../../shared/i18n'
import type { NoteItem, ChecklistItem } from '../../overlay/types'
import { parseChecklistItems } from '../../shared/composables/useNoteCore'

const props = defineProps<{
  note: NoteItem
  colors: readonly string[]
  isNight?: boolean
}>()

const emit = defineEmits<{
  pin: [pinned: boolean]
  delete: []
  color: [color: string]
  toggleType: []
  popOut: []
  updateTitle: [title: string]
  updateContent: [content: string]
  dragstart: [e: DragEvent]
  dragenter: [e: DragEvent]
  dragend: []
}>()

const isEditing = ref(false)
const titleInputRef = ref<HTMLInputElement | null>(null)

const checklistItems = computed<ChecklistItem[]>(() => {
  return props.note._parsedChecklist || parseChecklistItems(props.note.checklistItems || '[]')
})

const previewChecklist = computed(() => checklistItems.value.slice(0, 3))
const checklistDoneCount = computed(() => checklistItems.value.filter(i => i.done).length)

const cardStyle = computed(() => {
  const hex = props.note.color || '#fef3c7'
  if (props.isNight) {
    const rgb = hexToRgb(hex)
    const bg = rgb
      ? `rgba(${Math.round(rgb.r * 0.15 + 20)}, ${Math.round(rgb.g * 0.15 + 20)}, ${Math.round(rgb.b * 0.15 + 20)}, 0.6)`
      : 'rgba(40,40,40,0.6)'
    return { background: bg }
  }
  return { background: hex + '90' }
})

function hexToRgb(hex: string) {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex)
  return result ? {
    r: parseInt(result[1], 16),
    g: parseInt(result[2], 16),
    b: parseInt(result[3], 16)
  } : null
}

function renderMarkdown(content: string): string {
  try {
    const lines = content.split('\n').slice(0, 6).join('\n')
    return marked.parse(lines, { breaks: true }) as string
  } catch {
    return content
  }
}

function startEdit() {
  isEditing.value = true
  nextTick(() => titleInputRef.value?.focus())
}

function finishEdit() {
  isEditing.value = false
}

function onTitleInput(e: Event) {
  emit('updateTitle', (e.target as HTMLInputElement).value)
}

function onContentInput(e: Event) {
  emit('updateContent', (e.target as HTMLTextAreaElement).value)
}
</script>

<style scoped>
.note-card {
  position: relative;
  border-radius: 12px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  cursor: pointer;
  transition: transform 0.15s, box-shadow 0.15s;
  border: 1px solid rgba(0,0,0,0.06);
  min-height: 80px;
  word-break: break-word;
}
.note-card:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.08);
}
.note-card.pinned {
  border-top: 2px solid var(--accent, #c47a5a);
}
.note-card.editing {
  cursor: default;
  box-shadow: 0 4px 16px rgba(0,0,0,0.12);
}

/* Title */
.nc-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary, #1a1a1a);
  line-height: 1.3;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.nc-title-input {
  width: 100%;
  border: none;
  background: transparent;
  font-size: 13px;
  font-weight: 600;
  font-family: 'Outfit', sans-serif;
  color: var(--text-primary, #1a1a1a);
  outline: none;
  padding: 0;
  line-height: 1.3;
}

/* Body */
.nc-body {
  flex: 1;
  overflow: hidden;
  max-height: 120px;
}

/* Markdown preview */
.nc-md-preview {
  font-size: 11px;
  line-height: 1.5;
  color: var(--text-secondary, #5a5a5a);
  overflow: hidden;
}
.nc-md-preview :deep(p) { margin: 0 0 3px; }
.nc-md-preview :deep(h1),
.nc-md-preview :deep(h2),
.nc-md-preview :deep(h3) { margin: 4px 0 2px; font-size: 12px; font-weight: 600; }
.nc-md-preview :deep(ul),
.nc-md-preview :deep(ol) { padding-left: 14px; margin: 2px 0; }
.nc-md-preview :deep(code) {
  background: rgba(0,0,0,0.06);
  border-radius: 3px;
  padding: 0 2px;
  font-size: 10px;
}
.nc-md-preview :deep(a) { color: var(--accent, #c47a5a); }
.nc-md-preview :deep(blockquote) {
  border-left: 2px solid var(--accent, #c47a5a);
  padding-left: 6px;
  margin: 3px 0;
  opacity: 0.8;
}

.nc-empty {
  font-size: 11px;
  color: var(--text-muted, #9a9a9a);
  font-style: italic;
}

/* Checklist preview */
.nc-checklist-preview {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.nc-check-item {
  display: flex;
  align-items: center;
  gap: 6px;
}
.nc-checkbox {
  width: 14px;
  height: 14px;
  border-radius: 3px;
  border: 1.5px solid rgba(0,0,0,0.2);
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: transparent;
}
.nc-checkbox.done {
  background: var(--accent, #c47a5a);
  border-color: var(--accent, #c47a5a);
  color: white;
}
.nc-check-text {
  font-size: 11px;
  color: var(--text-secondary, #5a5a5a);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.nc-check-text.done {
  text-decoration: line-through;
  opacity: 0.5;
}
.nc-more {
  font-size: 10px;
  color: var(--text-muted, #9a9a9a);
  padding-left: 20px;
}
.nc-check-progress {
  font-size: 10px;
  color: var(--text-muted, #9a9a9a);
  font-family: 'JetBrains Mono', monospace;
  margin-top: 2px;
}

/* Content input (edit mode) */
.nc-content-input {
  width: 100%;
  min-height: 60px;
  border: none;
  background: transparent;
  font-size: 11px;
  font-family: 'Outfit', sans-serif;
  line-height: 1.5;
  color: var(--text-secondary, #5a5a5a);
  outline: none;
  resize: none;
  padding: 0;
}

/* Actions bar */
.nc-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.15s;
  flex-wrap: wrap;
}
.note-card:hover .nc-actions {
  opacity: 1;
}
.nc-action {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-muted, #9a9a9a);
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
}
.nc-action:hover {
  background: rgba(0,0,0,0.06);
  color: var(--text-secondary, #5a5a5a);
}
.nc-action.danger:hover {
  background: rgba(220,50,50,0.1);
  color: #dc3232;
}

/* Color picker */
.nc-color-picker {
  display: flex;
  gap: 3px;
  align-items: center;
  margin-left: auto;
}
.nc-color-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  cursor: pointer;
  border: 1.5px solid transparent;
  transition: all 0.15s;
}
.nc-color-dot:hover {
  transform: scale(1.2);
}
.nc-color-dot.selected {
  border-color: rgba(0,0,0,0.3);
  transform: scale(1.15);
}

/* Night mode */
.night .nc-title,
.night .nc-title-input { color: var(--text-primary, #e0e0e0); }
.night .nc-check-text { color: var(--text-secondary, #a0a0a0); }
.night .nc-md-preview { color: var(--text-secondary, #a0a0a0); }
.night .nc-md-preview :deep(code) { background: rgba(255,255,255,0.08); }
.night .nc-checkbox { border-color: rgba(255,255,255,0.2); }
.night .nc-color-dot.selected { border-color: rgba(255,255,255,0.5); }
.night .note-card { border-color: rgba(255,255,255,0.06); }
.night .nc-action:hover { background: rgba(255,255,255,0.08); }
</style>
