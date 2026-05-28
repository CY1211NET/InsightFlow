<template>
  <Transition name="slide-left">
    <div v-if="show" class="notes-panel" :class="theme" @click.stop @mouseleave="expandedNoteId = null">
      <!-- Panel Header -->
      <div class="panel-header" data-tauri-drag-region>
        <span class="panel-title" data-tauri-drag-region>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" class="title-icon"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
          {{ t('dashboard.notes') }}
        </span>
        <div class="header-actions">
          <button class="action-btn" :title="t('dashboard.add')" @click.stop="$emit('addNote')">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          </button>
          <button class="action-btn" :class="{ active: pinned }" :title="pinned ? t('overlay.unpin') : t('overlay.pin')" @click.stop="$emit('togglePin')">
            <svg width="14" height="14" viewBox="0 0 24 24" :fill="pinned ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 17v5"/><path d="M9 10.76a2 2 0 01-1.11 1.79l-1.78.9A2 2 0 005 15.24V17h14v-1.76a2 2 0 00-1.11-1.79l-1.78-.9A2 2 0 0115 10.76V7a1 1 0 011-1 2 2 0 000-4H8a2 2 0 000 4 1 1 0 011 1z"/></svg>
          </button>
        </div>
      </div>

      <!-- Search Bar -->
      <div class="panel-search">
        <svg class="search-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
        <input
          class="search-input"
          v-model="searchQuery"
          :placeholder="t('dashboard.searchNotes') || '搜索便签...'"
          @click.stop
        />
        <button v-if="searchQuery" class="search-clear" @click.stop="searchQuery = ''">
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>

      <!-- Tag Filter -->
      <div v-if="allTags.length > 0" class="tag-filter">
        <span
          v-for="tag in allTags"
          :key="tag"
          class="tag-pill"
          :class="{ active: activeTag === tag }"
          @click.stop="toggleTagFilter(tag)"
        >{{ tag }}</span>
      </div>

      <!-- Notes List -->
      <div class="notes-list" @click="expandedNoteId = null">

        <div
          v-for="note in filteredNotes"
          :key="String(note.id)"
          class="note-card-wrapper"
        >
          <!-- Note Card -->
          <div 
            class="note-card" 
            :style="getCardStyle(note)"
            :class="{ 'is-pinned': note.pinned, 'is-expanded': expandedNoteId === note.id }"
            @click.stop="expandedNoteId = expandedNoteId === note.id ? null : note.id"
          >
            <!-- Header (Title & Actions) -->
            <div class="card-header">
              <input
                class="note-title-input"
                v-model="note.title"
                :placeholder="t('dashboard.noteTitle')"
                @change="$emit('updateNote', note)"
                @blur="$emit('updateNote', note)"
                @click.stop="expandedNoteId = expandedNoteId === note.id ? null : note.id"
              />
              
              <!-- Card Pin & Trash -->
              <div class="card-actions">

              <button 
                class="card-action-btn pin-btn" 
                :class="{ active: note.pinned }" 
                :title="t('dashboard.pin')"
                @click.stop="$emit('togglePinNote', note)"
              >
                <svg width="11" height="11" viewBox="0 0 24 24" :fill="note.pinned ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 17v5"/><path d="M9 10.76a2 2 0 01-1.11 1.79l-1.78.9A2 2 0 005 15.24V17h14v-1.76a2 2 0 00-1.11-1.79l-1.78-.9A2 2 0 0115 10.76V7a1 1 0 011-1 2 2 0 000-4H8a2 2 0 000 4 1 1 0 011 1z"/></svg>
              </button>
              <button 
                class="card-action-btn float-btn" 
                title="独立悬浮"
                @click.stop="$emit('floatNote', note.id!)"
              >
                <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
              </button>
              <button 
                class="card-action-btn delete-btn" 
                title="Delete"
                @click.stop="$emit('deleteNote', note)"
              >
                <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></svg>
              </button>
            </div>
            </div>

            <!-- Content Area (Only visible when expanded) -->
            <textarea
              v-if="expandedNoteId === note.id"
              class="note-content-input"
              v-model="note.content"
              :placeholder="t('dashboard.noteContent')"
              @change="$emit('updateNote', note)"
              @blur="$emit('updateNote', note)"
              @click.stop
            ></textarea>

            <!-- Tags -->
            <div class="note-tags" @click.stop>
              <span
                v-for="tag in getNoteTags(note)"
                :key="tag"
                class="tag-chip"
              >
                {{ tag }}
                <button class="tag-remove" @click.stop="$emit('removeTag', note.id!, tag)">×</button>
              </span>
              <button v-if="!addingTagFor.has(note.id!)" class="tag-add-btn" @click.stop="startAddTag(note.id!)">+</button>
              <input
                v-if="addingTagFor.has(note.id!)"
                class="tag-input"
                v-model="newTagText"
                placeholder="标签"
                @keydown.enter="confirmAddTag(note.id!)"
                @keydown.escape="cancelAddTag"
                @blur="confirmAddTag(note.id!)"
                @click.stop
              />
            </div>

            <!-- Bottom palette selector -->
            <div class="card-bottom">
              <div class="color-palette">
                <span 
                  v-for="color in NOTE_COLORS" 
                  :key="color"
                  class="color-dot"
                  :style="{ background: color }"
                  :class="{ selected: note.color === color }"
                  @click.stop="changeNoteColor(note, color)"
                />
              </div>
              <span class="note-time">{{ formatTime(note.updatedAt) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { t } from '../shared/i18n'
import type { NoteItem } from './types'
import { NOTE_COLORS } from './composables/useNote'

const props = defineProps<{
  show: boolean
  pinned: boolean
  notes: NoteItem[]
  theme: 'day' | 'night'
  noteTags?: Record<number, string[]>
  allTags?: string[]
}>()

const emit = defineEmits<{
  addNote: []
  togglePin: []
  updateNote: [note: NoteItem]
  togglePinNote: [note: NoteItem]
  deleteNote: [note: NoteItem]
  addTag: [noteId: number, tag: string]
  removeTag: [noteId: number, tag: string]
  floatNote: [noteId: number]
}>()

// Search & Expand
const searchQuery = ref('')
const activeTag = ref<string | null>(null)
const expandedNoteId = ref<number | null>(null)

const filteredNotes = computed(() => {
  let result = props.notes
  if (activeTag.value && props.noteTags) {
    result = result.filter(n => {
      const tags = props.noteTags?.[n.id!] || []
      return tags.includes(activeTag.value!)
    })
  }
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(n =>
      n.title.toLowerCase().includes(q) || n.content.toLowerCase().includes(q)
    )
  }
  return result
})

const allTags = computed(() => props.allTags || [])

function toggleTagFilter(tag: string) {
  activeTag.value = activeTag.value === tag ? null : tag
}

// Tag management
const addingTagFor = ref(new Set<number>())
const newTagText = ref('')

function getNoteTags(note: NoteItem): string[] {
  return props.noteTags?.[note.id!] || []
}

function startAddTag(noteId: number) {
  addingTagFor.value.add(noteId)
  newTagText.value = ''
}

function cancelAddTag() {
  addingTagFor.value.clear()
  newTagText.value = ''
}

function confirmAddTag(noteId: number) {
  const tag = newTagText.value.trim()
  if (tag) {
    emit('addTag', noteId, tag)
  }
  cancelAddTag()
}

function changeNoteColor(note: NoteItem, color: string) {
  if (note.color !== color) {
    note.color = color
    emit('updateNote', note)
  }
}

// 采用高档马卡龙配色以及半透明模糊(Glassmorphic)处理
function getCardStyle(note: NoteItem) {
  const isNight = props.theme === 'night'
  
  // 基于选定的马卡龙色生成精美贴纸底色
  const hex = note.color || '#fef3c7'
  
  // 白天模式：柔和半透明
  // 黑夜模式：深邃暗色，带彩色微光边框和半透明彩色底色
  if (isNight) {
    // 转换 HEX 颜色为 RGB 并降低透明度以实现深色护眼
    const rgb = hexToRgb(hex)
    return {
      background: rgb ? `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.12)` : 'rgba(255,255,255,0.06)',
      borderColor: hex,
      boxShadow: `0 4px 12px rgba(0,0,0,0.3), inset 0 1px 0 rgba(255,255,255,0.05)`,
    }
  } else {
    const rgb = hexToRgb(hex)
    return {
      background: rgb ? `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.65)` : 'rgba(255,255,255,0.85)',
      borderColor: 'rgba(0,0,0,0.05)',
      boxShadow: `0 4px 10px rgba(0,0,0,0.04), inset 0 1px 0 rgba(255,255,255,0.3)`,
    }
  }
}

// 时间格式化
function formatTime(ts: number): string {
  if (!ts) return ''
  const d = new Date(ts * 1000)
  const hour = d.getHours().toString().padStart(2, '0')
  const min = d.getMinutes().toString().padStart(2, '0')
  return `${hour}:${min}`
}

// Helper to convert hex to rgb
function hexToRgb(hex: string) {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex)
  return result ? {
    r: parseInt(result[1], 16),
    g: parseInt(result[2], 16),
    b: parseInt(result[3], 16)
  } : null
}
</script>

<style scoped>
.notes-panel {
  --panel-bg: rgba(247, 247, 245, 0.95);
  --panel-border: rgba(0, 0, 0, 0.08);
  --text-primary: #1a1a1a;
  --text-secondary: #5a5a5a;
  --text-muted: #9a9a9a;
  --surface: rgba(0, 0, 0, 0.04);
  --surface-hover: rgba(0, 0, 0, 0.07);
  --track: rgba(0, 0, 0, 0.08);

  width: 100%;
  max-height: 70vh;
  background: transparent;
  border: none;
  border-top: 1px solid var(--track);
  border-radius: 0;
  padding: 8px 0 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  box-sizing: border-box;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
}

/* ─── Night theme ─── */
.notes-panel.night {
  --panel-bg: rgba(26, 26, 26, 0.92);
  --panel-border: rgba(255, 255, 255, 0.08);
  --text-primary: #e0e0e0;
  --text-secondary: #a0a0a0;
  --text-muted: #606060;
  --surface: rgba(255, 255, 255, 0.06);
  --surface-hover: rgba(255, 255, 255, 0.1);
  --track: rgba(255, 255, 255, 0.08);
}

/* ─── Panel Header ─── */
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 2px;
}

.panel-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.01em;
}

.title-icon {
  color: #c47a5a;
}
.night .title-icon {
  color: #d4906e;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.action-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--surface);
  border: 1px solid transparent;
  border-radius: 6px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s ease;
  padding: 0;
}
.action-btn:hover {
  background: var(--surface-hover);
  color: var(--text-primary);
}
.action-btn.active {
  background: rgba(196, 122, 90, 0.12);
  color: #c47a5a;
  border-color: rgba(196, 122, 90, 0.2);
}
.notes-panel.night .action-btn.active {
  background: rgba(196, 122, 90, 0.2);
  color: #d4906e;
  border-color: rgba(196, 122, 90, 0.3);
}

/* ─── Search Bar ─── */
.panel-search {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--surface);
  border-radius: 8px;
  padding: 5px 8px;
}
.search-icon {
  flex-shrink: 0;
  color: var(--text-muted);
}
.search-input {
  flex: 1;
  border: none;
  background: transparent;
  font-family: 'Outfit', sans-serif;
  font-size: 11px;
  color: var(--text-primary);
  outline: none;
  padding: 0;
  min-width: 0;
}
.search-input::placeholder {
  color: var(--text-muted);
}
.search-clear {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-muted);
  padding: 2px;
  display: flex;
  align-items: center;
  border-radius: 3px;
}
.search-clear:hover {
  background: var(--surface-hover);
  color: var(--text-secondary);
}

/* ─── Tag Filter ─── */
.tag-filter {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}
.tag-pill {
  font-size: 9px;
  padding: 2px 7px;
  border-radius: 10px;
  background: var(--surface);
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s;
  border: 1px solid transparent;
  user-select: none;
}
.tag-pill:hover {
  background: var(--surface-hover);
  color: var(--text-secondary);
}
.tag-pill.active {
  background: rgba(196, 122, 90, 0.15);
  color: #c47a5a;
  border-color: rgba(196, 122, 90, 0.3);
}
.night .tag-pill.active {
  background: rgba(196, 122, 90, 0.2);
  color: #d4906e;
  border-color: rgba(196, 122, 90, 0.35);
}

/* ─── Note Tags (per-card) ─── */
.note-tags {
  display: flex;
  gap: 3px;
  flex-wrap: wrap;
  align-items: center;
}
.tag-chip {
  font-size: 8.5px;
  padding: 1px 6px;
  border-radius: 8px;
  background: rgba(196, 122, 90, 0.1);
  color: #c47a5a;
  display: flex;
  align-items: center;
  gap: 2px;
}
.night .tag-chip {
  background: rgba(196, 122, 90, 0.15);
  color: #d4906e;
}
.tag-remove {
  background: none;
  border: none;
  cursor: pointer;
  color: inherit;
  font-size: 10px;
  padding: 0;
  line-height: 1;
  opacity: 0.6;
}
.tag-remove:hover {
  opacity: 1;
}
.tag-add-btn {
  background: none;
  border: 1px dashed var(--text-muted);
  cursor: pointer;
  color: var(--text-muted);
  font-size: 10px;
  width: 16px;
  height: 16px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  line-height: 1;
}
.tag-add-btn:hover {
  border-color: var(--text-secondary);
  color: var(--text-secondary);
}
.tag-input {
  width: 50px;
  border: none;
  border-bottom: 1px solid var(--text-muted);
  background: transparent;
  font-family: 'Outfit', sans-serif;
  font-size: 9px;
  color: var(--text-primary);
  outline: none;
  padding: 1px 2px;
}

/* ─── Notes List ─── */
.notes-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 10px;
  overflow-y: auto;
  padding-right: 2px;
}

/* 滚动条 */
.notes-list::-webkit-scrollbar {
  width: 3px;
}
.notes-list::-webkit-scrollbar-track {
  background: transparent;
}
.notes-list::-webkit-scrollbar-thumb {
  background: var(--track);
  border-radius: 1.5px;
}

/* ─── Empty state ─── */
.notes-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: var(--text-muted);
  border: 1.5px dashed var(--panel-border);
  border-radius: 16px;
  cursor: pointer;
  padding: 20px 10px;
  transition: all 0.2s ease;
  user-select: none;
}
.notes-empty:hover {
  background: var(--surface);
  color: var(--text-secondary);
  border-color: var(--text-muted);
}
.empty-icon {
  margin-bottom: 2px;
  opacity: 0.7;
}
.empty-sub {
  font-size: 10px;
  opacity: 0.6;
}

/* ─── Note Card ─── */
.note-card-wrapper {
  perspective: 1000px;
}

.note-card {
  position: relative;
  border-radius: 14px;
  padding: 8px 10px;
  border: 1px solid transparent;
  display: flex;
  flex-direction: column;
  gap: 6px;
  transition: all 0.2s cubic-bezier(0.2, 0.8, 0.2, 1);
  box-sizing: border-box;
  cursor: pointer;
}
.note-card:hover {
}
.notes-panel.night .note-card:hover {
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.35), 0 0 8px rgba(255,255,255,0.02) !important;
}

.note-card.is-pinned {
  /* 置顶卡片带有精致发光感 */
  box-shadow: 0 0 0 1px rgba(196, 122, 90, 0.2) !important;
}

/* Card actions inside header */
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
}

.card-actions {
  display: flex;
  gap: 3px;
  opacity: 0.6;
  transition: all 0.15s ease;
  flex-shrink: 0;
}
.note-card:hover .card-actions,
.note-card.is-expanded .card-actions {
  opacity: 1;
}

.card-action-btn {
  width: 16px;
  height: 16px;
  border: none;
  background: rgba(255, 255, 255, 0.5);
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #333;
  cursor: pointer;
  padding: 0;
  transition: all 0.15s;
}
.night .card-action-btn {
  background: rgba(0, 0, 0, 0.4);
  color: #ccc;
}
.card-action-btn:hover {
  background: #fff;
  color: #000;
}
.night .card-action-btn:hover {
  background: #222;
  color: #fff;
}
.card-action-btn.pin-btn.active {
  color: #c47a5a;
  background: #fff;
}
.night .card-action-btn.pin-btn.active {
  color: #d4906e;
  background: #222;
}

.delete-btn:hover {
  color: #e74c3c !important;
}

/* Card Title Input */
.note-title-input {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  font-family: 'Outfit', sans-serif;
  font-size: 11.5px;
  font-weight: 600;
  color: #111;
  outline: none;
  padding: 0;
  letter-spacing: -0.01em;
}
.night .note-title-input {
  color: #e8e8e8;
}
.note-title-input::placeholder {
  color: rgba(0,0,0,0.3);
  font-weight: 500;
}
.night .note-title-input::placeholder {
  color: rgba(255,255,255,0.25);
}

/* Card Content Textarea */
.note-content-input {
  width: 100%;
  height: 48px;
  border: none;
  background: transparent;
  font-family: 'Outfit', sans-serif;
  font-size: 10.5px;
  line-height: 1.4;
  color: #444;
  outline: none;
  resize: none;
  padding: 0;
}
.night .note-content-input {
  color: #b5b5b5;
}
.note-content-input::placeholder {
  color: rgba(0,0,0,0.25);
}
.night .note-content-input::placeholder {
  color: rgba(255,255,255,0.2);
}

/* Card Bottom */
.card-bottom {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 4px;
  border-top: 1px solid rgba(0,0,0,0.04);
}
.night .card-bottom {
  border-top-color: rgba(255,255,255,0.03);
}

.color-palette {
  display: flex;
  gap: 4px;
  align-items: center;
}

.color-dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  cursor: pointer;
  border: 1px solid rgba(0, 0, 0, 0.05);
  transition: transform 0.15s;
}
.color-dot:hover {
  transform: scale(1.3);
}
.color-dot.selected {
  transform: scale(1.2);
  box-shadow: 0 0 0 1px rgba(0,0,0,0.3);
}
.night .color-dot.selected {
  box-shadow: 0 0 0 1px rgba(255,255,255,0.6);
}

.note-time {
  font-family: 'JetBrains Mono', monospace;
  font-size: 8.5px;
  color: rgba(0,0,0,0.3);
}
.night .note-time {
  color: rgba(255,255,255,0.3);
}

/* Note Content Visibility (Click to expand) */
.note-content-input,
.note-tags,
.card-bottom {
  display: none;
}
.note-card.is-expanded .note-content-input,
.note-card.is-expanded .note-tags,
.note-card.is-expanded .card-bottom {
  display: flex;
}
.note-card.is-expanded .note-content-input {
  display: block; /* textarea needs block or inline-block, not flex */
}

/* ───── Transition Animations ───── */
.slide-left-enter-active {
  transition: opacity 0.3s cubic-bezier(0.22, 1, 0.36, 1),
              transform 0.3s cubic-bezier(0.22, 1, 0.36, 1);
}
.slide-left-leave-active {
  transition: opacity 0.2s ease,
              transform 0.2s ease;
}
.slide-left-enter-from, .slide-left-leave-to {
  opacity: 0;
  transform: translateX(-15px);
}
.slide-left-enter-to, .slide-left-leave-from {
  opacity: 1;
  transform: translateX(0);
}
</style>
