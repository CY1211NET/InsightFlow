<template>
  <div class="note-window" :class="{ night: isNight }" :style="windowStyle">
    <!-- Title bar -->
    <div class="title-bar" data-tauri-drag-region @dblclick="toggleCollapse">
      <div class="title-bar-left">
      </div>
      <div class="title-bar-right">
        <button class="tb-btn" title="New Note" @click.stop="addNote">
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        </button>
        <button class="tb-btn" title="Pin" @click.stop="togglePin">
          <svg width="11" height="11" viewBox="0 0 24 24" :fill="note?.pinned ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
        </button>
        <button class="tb-btn" title="Type" @click.stop="toggleType">
          <svg v-if="note?.noteType === 'checklist'" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
          <svg v-else width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 11 12 14 22 4"/><path d="M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11"/></svg>
        </button>
        <button class="tb-btn delete-btn" title="Delete" @click.stop="deleteNote">
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></svg>
        </button>
        <button class="tb-btn close-btn" title="Close" @click.stop="closeWindow">
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>
    </div>


    <!-- Content -->
    <div v-if="note && !isCollapsed" class="note-body">
      <input
        class="note-title"
        v-model="note.title"
        placeholder="Title"
        @change="saveNote"
        @blur="saveNote"
      />

      <!-- Markdown mode -->
      <template v-if="note.noteType !== 'checklist'">
        <textarea
          v-if="isEditing"
          ref="contentInputRef"
          class="note-content"
          v-model="note.content"
          placeholder="Write something..."
          @change="saveNote"
          @blur="onContentBlur"
        />
        <div 
          v-else 
          class="note-md-preview" 
          @click="startEditing"
          v-html="renderMarkdown(note.content)"
        ></div>
      </template>

      <!-- Checklist mode -->
      <template v-else>
        <ChecklistEditor
          :modelValue="parsedChecklist"
          :addPlaceholder="t('dashboard.addCheckItem')"
          @update:modelValue="onChecklistUpdate"
        />
      </template>

      <!-- Bottom Palette & Time -->
      <div class="card-bottom">
        <div class="color-dots">
          <span
            v-for="c in NOTE_COLORS"
            :key="c"
            class="color-dot"
            :style="{ background: c }"
            :class="{ selected: note?.color === c }"
            @click.stop="changeColor(c)"
          />
        </div>
        <span class="note-time">{{ formatTime(note.updatedAt) }}</span>
      </div>
    </div>
    
    <!-- Loading / error state when note is null -->
    <div v-if="!note" class="note-loading">
      <div v-if="loadError" class="note-load-error">
        <span>⚠️ {{ loadError }}</span>
      </div>
      <span v-else>正在加载便签...</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { LogicalSize } from '@tauri-apps/api/dpi'
import { marked } from 'marked'
import { t, loadLocale } from '../shared/i18n'
import type { NoteItem, ChecklistItem } from '../overlay/types'
import { NOTE_COLORS, parseChecklistItems } from '../shared/composables/useNoteCore'
import ChecklistEditor from '../shared/components/ChecklistEditor.vue'

const note = ref<NoteItem | null>(null)
const isNight = ref(false)
const isEditing = ref(false)
const contentInputRef = ref<HTMLTextAreaElement | null>(null)
const isCollapsed = ref(false)
const loadError = ref('')

let tauriWindow: any = null
let preCollapseHeight = 320
let loaded = false
let geometryTimer: ReturnType<typeof setTimeout> | null = null
let unlistenNoteId: (() => void) | null = null
let unlistenUpdate: (() => void) | null = null
const unlisteners: (() => void)[] = []

watch(() => note.value?.pinned, async (pinned) => {
  if (tauriWindow && typeof pinned === 'boolean') {
    try {
      await tauriWindow.setIgnoreCursorEvents(pinned)
      if (pinned) {
        document.body.classList.add('pinned-click-through')
      } else {
        document.body.classList.remove('pinned-click-through')
      }
    } catch (e) {
      console.warn('[Note] setIgnoreCursorEvents failed:', e)
    }
  }
})

const parsedChecklist = computed<ChecklistItem[]>(() => {
  if (!note.value) return []
  return note.value._parsedChecklist || parseChecklistItems(note.value.checklistItems || '[]')
})

const windowStyle = computed(() => {
  const hex = note.value?.color || '#fef3c7'
  const rgb = hexToRgb(hex)
  
  if (isNight.value) {
    const bg = rgb ? `rgba(${Math.round(rgb.r * 0.2 + 25)}, ${Math.round(rgb.g * 0.2 + 25)}, ${Math.round(rgb.b * 0.2 + 25)}, 0.85)` : 'rgba(40,40,40,0.85)'
    return { 
      background: bg, 
      boxShadow: `0 8px 30px rgba(0,0,0,0.4), inset 0 1px 0 rgba(255,255,255,0.05)`,
      backdropFilter: 'blur(20px)',
      border: `1px solid ${hex}`
    }
  }
  const bg = rgb ? `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.85)` : 'rgba(255,255,255,0.85)'
  return { 
    background: bg,
    boxShadow: `0 8px 30px rgba(0,0,0,0.1), inset 0 1px 0 rgba(255,255,255,0.3)`,
    backdropFilter: 'blur(20px)'
  }
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
  try { return marked.parse(content, { breaks: true }) as string }
  catch { return content }
}

function formatTime(ts: number | undefined): string {
  if (!ts) return ''
  const d = new Date(ts * 1000)
  const hour = d.getHours().toString().padStart(2, '0')
  const min = d.getMinutes().toString().padStart(2, '0')
  return `${hour}:${min}`
}

/** Extract note ID from multiple sources */
function getNoteId(): number {
  // Strategy 1: window label (format: note_{id})
  try {
    const label = tauriWindow?.label
    console.log('[Note] window label =', label)
    if (label && label.startsWith('note_')) {
      const id = Number(label.split('_')[1])
      if (id > 0) return id
    }
  } catch (e) {
    console.warn('[Note] failed to read window label:', e)
  }

  // Strategy 2: URL query params
  try {
    const params = new URLSearchParams(window.location.search)
    const id = Number(params.get('id'))
    if (id > 0) return id
  } catch (e) {
    console.warn('[Note] failed to parse URL params:', e)
  }

  return 0
}

async function fetchNoteById(id: number): Promise<boolean> {
  if (id <= 0) return false
  try {
    console.log('[Note] fetching note id =', id)
    const item = await invoke<NoteItem>('get_note', { id })
    if (item) {
      note.value = item
      note.value._parsedChecklist = parseChecklistItems(item.checklistItems || '[]')
      try {
        const size = await tauriWindow.innerSize()
        const factor = await tauriWindow.scaleFactor()
        preCollapseHeight = item.height || Math.round(size.height / factor)
      } catch { /* ignore geometry errors */ }
      console.log('[Note] loaded note successfully:', item.title)
      loadError.value = ''
      return true
    }
  } catch (e: any) {
    console.warn('[Note] fetchNoteById failed:', e)
    loadError.value = `加载失败: ${e?.message || e}`
  }
  return false
}

async function loadNote() {
  const id = getNoteId()

  if (id > 0) {
    await fetchNoteById(id)
  } else {
    console.warn('[Note] no note ID found from label or URL')
    loadError.value = '未找到便签ID (label: ' + (tauriWindow?.label || 'unknown') + ')'
  }

  setTimeout(() => { loaded = true }, 1000)
}

let saveTimer: ReturnType<typeof setTimeout> | null = null

function saveNote() {
  if (!note.value || !note.value.id) return
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(async () => {
    try {
      await invoke('update_note', {
        id: note.value!.id,
        title: note.value!.title,
        content: note.value!.content,
        color: note.value!.color,
        noteType: note.value!.noteType || 'markdown',
        checklistItems: note.value!.checklistItems || '[]',
      })
      const { emit } = await import('@tauri-apps/api/event')
      await emit('note-updated', note.value!.id)
    } catch (e) {
      console.warn('[Note] failed to save note:', e)
    }
  }, 300)
}

function startEditing() {
  if (note.value?.pinned) return // 穿透模式下无法点击，但以防万一
  isEditing.value = true
  nextTick(() => {
    contentInputRef.value?.focus()
  })
}

function onContentBlur() {
  isEditing.value = false
  saveNote()
}

async function changeColor(color: string) {
  if (!note.value) return
  note.value.color = color
  saveNote()
}

async function togglePin() {
  if (!note.value || !note.value.id) return
  try {
    await invoke('pin_note', { id: note.value.id, pinned: !note.value.pinned })
    note.value.pinned = !note.value.pinned
  } catch (e) {
    console.warn('pin_note failed:', e)
  }
}

async function toggleType() {
  if (!note.value) return
  note.value.noteType = note.value.noteType === 'checklist' ? 'markdown' : 'checklist'
  if (note.value.noteType === 'checklist' && !note.value._parsedChecklist) {
    note.value._parsedChecklist = []
  }
  saveNote()
}

function onChecklistUpdate(items: ChecklistItem[]) {
  if (!note.value) return
  note.value._parsedChecklist = items
  note.value.checklistItems = JSON.stringify(items)
  saveNote()
}

async function closeWindow() {
  try { await getCurrentWindow().close() }
  catch (e) { console.warn('close failed:', e) }
}

async function addNote() {
  try {
    const item = await invoke<NoteItem>('create_note', {
      title: '',
      content: '',
      color: note.value?.color || NOTE_COLORS[0],
      noteType: note.value?.noteType || 'markdown',
    })
    if (item && item.id) {
      await invoke('open_note_window', { noteId: item.id })
    }
  } catch (e) {
    console.warn('addNote failed:', e)
  }
}

async function deleteNote() {
  if (!note.value || !note.value.id) return
  try {
    await invoke('trash_note', { id: note.value.id })
    await closeWindow()
  } catch (e) {
    console.warn('deleteNote failed:', e)
  }
}

async function toggleCollapse() {
  if (!note.value) return
  try {
    const size = await tauriWindow.innerSize()
    const factor = await tauriWindow.scaleFactor()
    const w = Math.round(size.width / factor)
    
    if (isCollapsed.value) {
      isCollapsed.value = false
      const h = preCollapseHeight || 320
      await tauriWindow.setSize(new LogicalSize(w, h))
    } else {
      preCollapseHeight = Math.round(size.height / factor)
      isCollapsed.value = true
      await tauriWindow.setSize(new LogicalSize(w, 32))
    }
  } catch (e) {
    console.warn('toggleCollapse failed:', e)
  }
}

async function saveGeometry() {
  if (!loaded || isCollapsed.value || !note.value || !note.value.id) return
  try {
    const factor = await tauriWindow.scaleFactor()
    const pSize = await tauriWindow.innerSize()
    const pPos = await tauriWindow.innerPosition()
    
    const width = Math.round(pSize.width / factor)
    const height = Math.round(pSize.height / factor)
    const x = Math.round(pPos.x / factor)
    const y = Math.round(pPos.y / factor)

    if (geometryTimer) clearTimeout(geometryTimer)
    geometryTimer = setTimeout(async () => {
      try {
        await invoke('update_note_geometry', { id: note.value!.id, x, y, width, height })
      } catch (e) {
        console.warn('update_note_geometry failed:', e)
      }
    }, 500)
  } catch (e) {
    console.warn('saveGeometry error:', e)
  }
}

let unlistenMoved: (() => void) | null = null
let unlistenResized: (() => void) | null = null

onMounted(async () => {
  try {
    tauriWindow = getCurrentWindow()
    console.log('[Note] getCurrentWindow() OK, label =', tauriWindow?.label)
  } catch (e: any) {
    console.error('[Note] getCurrentWindow() failed:', e)
    loadError.value = `Tauri窗口初始化失败: ${e?.message || e}`
    return
  }

  try {
    await loadLocale()
  } catch (e) {
    console.warn('[Note] loadLocale failed:', e)
  }

  isNight.value = document.documentElement.classList.contains('night') || localStorage.getItem('insight_theme') === 'night'

  // Strategy A: try loading note from window label / URL params
  await loadNote()

  // Strategy B: if note is still null, listen for the Rust-emitted event
  if (!note.value) {
    console.log('[Note] note not loaded yet, listening for load-note-id event...')
    try {
      unlistenNoteId = await listen<number>('load-note-id', async (event) => {
        console.log('[Note] received load-note-id event, payload =', event.payload)
        if (!note.value && event.payload) {
          const success = await fetchNoteById(event.payload)
          if (success && unlistenNoteId) {
            unlistenNoteId()
            unlistenNoteId = null
          }
        }
      })
    } catch (e) {
      console.warn('[Note] listen for load-note-id failed:', e)
    }

    // Strategy C: retry with delays as final fallback
    for (const delay of [300, 700, 1500]) {
      if (note.value) break
      await new Promise(r => setTimeout(r, delay))
      const id = getNoteId()
      if (id > 0) {
        const success = await fetchNoteById(id)
        if (success) break
      }
    }
  }

  // Listen for sync updates
  try {
    unlistenUpdate = await listen<number>('note-updated', (event) => {
      const updatedId = event.payload
      if (note.value && note.value.id === updatedId) {
        fetchNoteById(updatedId)
      }
    })
  } catch (e) {
    console.warn('[Note] listen for note-updated failed:', e)
  }

  // Set up geometry listeners
  try {
    unlistenMoved = await tauriWindow.onMoved(() => { saveGeometry() })
    unlistenResized = await tauriWindow.onResized(() => { saveGeometry() })
  } catch (e) {
    console.warn('[Note] geometry listeners failed:', e)
  }

  // Set up auto-resize based on content height
  const noteWindowEl = document.querySelector('.note-window') as HTMLElement
  if (noteWindowEl) {
    let resizeTimer: any = null
    const observer = new ResizeObserver((entries) => {
      if (resizeTimer) clearTimeout(resizeTimer)
      resizeTimer = setTimeout(async () => {
        const contentH = entries[0].borderBoxSize ? entries[0].borderBoxSize[0].blockSize : entries[0].contentRect.height
        const size = await tauriWindow.innerSize()
        const factor = await tauriWindow.scaleFactor()
        const w = Math.round(size.width / factor)
        // Clamp height between 40px and 800px, add 16px safe buffer for transparent window to prevent clipping
        const newH = Math.min(Math.max(Math.ceil(contentH) + 16, 40), 800)
        
        // Prevent infinite loops or unnecessary updates
        if (Math.abs(Math.round(size.height / factor) - newH) > 2) {
          await tauriWindow.setSize(new LogicalSize(w, newH))
        }
      }, 50)
    })
    observer.observe(noteWindowEl)
    unlisteners.push(() => observer.disconnect())
  }
})

onUnmounted(() => {
  unlisteners.forEach(fn => fn())
  if (unlistenMoved) unlistenMoved()
  if (unlistenResized) unlistenResized()
  if (unlistenNoteId) unlistenNoteId()
  if (unlistenUpdate) unlistenUpdate()
  if (geometryTimer) clearTimeout(geometryTimer)
  if (saveTimer) clearTimeout(saveTimer)
})
</script>

<style>
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

html, body, #app {
  width: 100%;
  overflow: hidden;
  font-family: 'Outfit', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  -webkit-font-smoothing: antialiased;
  /* CRITICAL: provide fallback background so transparent window is never invisible */
  background: transparent;
}

body.pinned-click-through {
  /* Visual cue that it's click-through */
  opacity: 0.9;
}

.note-window {
  width: 100%;
  height: max-content;
  min-height: 40px;
  display: flex;
  flex-direction: column;
  border-radius: 16px;
  border: 1px solid rgba(0,0,0,0.08);
  overflow: hidden;
}
.note-window.night {
  border-color: rgba(255,255,255,0.08);
}

.title-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  flex-shrink: 0;
  cursor: grab;
}
.title-bar-left {
  display: flex;
  align-items: center;
  gap: 4px;
  flex: 1;
  min-width: 0;
}
.title-bar-right {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
}
.color-dots {
  display: flex;
  gap: 3px;
  flex-wrap: nowrap;
  overflow: hidden;
}
.color-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  cursor: pointer;
  border: 1.5px solid transparent;
  flex-shrink: 0;
  transition: border-color 0.15s;
}
.color-dot.selected {
  border-color: rgba(0,0,0,0.4);
}
.night .color-dot.selected {
  border-color: rgba(255,255,255,0.5);
}
.tb-btn {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-secondary, #5a5a5a);
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
}
.tb-btn:hover {
  background: rgba(0,0,0,0.06);
}
.night .tb-btn:hover {
  background: rgba(255,255,255,0.08);
}
.delete-btn:hover, .close-btn:hover {
  background: rgba(220,50,50,0.15);
  color: #dc3232;
}

.note-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 0 10px 10px;
  overflow: hidden;
  position: relative;
}

.note-title {
  border: none;
  background: transparent;
  font-size: 14px;
  font-weight: 600;
  font-family: 'Outfit', sans-serif;
  color: var(--text-primary, #1a1a1a);
  outline: none;
  padding: 0 0 6px;
  flex-shrink: 0;
}
.night .note-title {
  color: var(--text-primary, #e0e0e0);
}
.note-title::placeholder {
  color: rgba(0,0,0,0.2);
}
.night .note-title::placeholder {
  color: rgba(255,255,255,0.2);
}

.note-content {
  flex: 1;
  border: none;
  background: transparent;
  font-size: 12px;
  font-family: 'Outfit', sans-serif;
  line-height: 1.5;
  color: var(--text-secondary, #5a5a5a);
  outline: none;
  resize: none;
  word-break: break-word;
  white-space: pre-wrap;
}
.night .note-content {
  color: var(--text-secondary, #a0a0a0);
}

.note-md-preview {
  flex: 1;
  overflow-y: auto;
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-secondary, #5a5a5a);
  word-break: break-word;
  cursor: text;
}
.note-md-preview :deep(p) { margin: 0 0 4px; }
.note-md-preview :deep(h1),
.note-md-preview :deep(h2),
.note-md-preview :deep(h3) { margin: 6px 0 3px; font-size: 13px; font-weight: 600; }
.note-md-preview :deep(ul),
.note-md-preview :deep(ol) { padding-left: 16px; margin: 3px 0; }
.note-md-preview :deep(code) {
  background: rgba(0,0,0,0.06);
  border-radius: 3px;
  padding: 0 3px;
  font-size: 11px;
}
.night .note-md-preview :deep(code) {
  background: rgba(255,255,255,0.08);
}
.note-md-preview :deep(pre) {
  background: rgba(0,0,0,0.04);
  border-radius: 4px;
  padding: 6px 8px;
  overflow-x: auto;
  margin: 4px 0;
}
.night .note-md-preview :deep(pre) {
  background: rgba(255,255,255,0.06);
}
.note-md-preview :deep(a) { color: var(--accent, #c47a5a); }
.note-md-preview :deep(blockquote) {
  border-left: 2px solid var(--accent, #c47a5a);
  padding-left: 8px;
  margin: 4px 0;
  opacity: 0.8;
}

.note-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 8px;
  font-size: 11px;
  color: var(--text-secondary, #5a5a5a);
}
.night .note-loading {
  color: var(--text-secondary, #a0a0a0);
}
.note-load-error {
  padding: 8px 12px;
  background: rgba(185, 28, 28, 0.08);
  border: 1px solid rgba(185, 28, 28, 0.2);
  border-radius: 6px;
  color: #b91c1c;
  font-size: 10px;
  max-width: 260px;
  text-align: center;
  word-break: break-word;
}
.night .note-load-error {
  background: rgba(248, 113, 113, 0.1);
  border-color: rgba(248, 113, 113, 0.2);
  color: #f87171;
}

/* Card Bottom */
.card-bottom {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 6px;
  border-top: 1px solid rgba(0,0,0,0.04);
  margin-top: auto;
}
.night .card-bottom {
  border-top-color: rgba(255,255,255,0.03);
}
.note-time {
  font-family: 'JetBrains Mono', monospace;
  font-size: 9px;
  color: rgba(0,0,0,0.3);
}
.night .note-time {
  color: rgba(255,255,255,0.3);
}
</style>
