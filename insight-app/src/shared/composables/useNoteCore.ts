import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { NoteItem, ChecklistItem } from '../../overlay/types'

export const NOTE_COLORS = [
  '#fef3c7', // yellow
  '#dcfce7', // green
  '#dbeafe', // blue
  '#f3e8ff', // purple
  '#ffe4e6', // pink
  '#fce7f3', // rose
  '#ccfbf1', // teal
  '#e0e7ff', // indigo
  '#fed7aa', // orange
  '#d1fae5', // emerald
  '#fae8ff', // magenta
  '#f1f5f9', // gray
]

export function parseChecklistItems(json: string): ChecklistItem[] {
  try {
    const parsed = JSON.parse(json)
    if (Array.isArray(parsed)) return parsed
  } catch {}
  return []
}

export function useNoteCore() {
  const notes = ref<NoteItem[]>([])
  const trashedNotes = ref<NoteItem[]>([])
  const noteTags = ref<Record<number, string[]>>({})
  const allTags = ref<string[]>([])

  const sortedNotes = computed(() => {
    return [...notes.value].sort((a, b) => {
      if (a.pinned !== b.pinned) return a.pinned ? -1 : 1
      return (a.sortOrder ?? 0) - (b.sortOrder ?? 0)
    })
  })

  function ensureParsed(note: NoteItem) {
    if (!note._parsedChecklist) {
      note._parsedChecklist = parseChecklistItems(note.checklistItems || '[]')
    }
    return note._parsedChecklist
  }

  async function loadAllTags() {
    try {
      allTags.value = await invoke<string[]>('list_all_tags')
    } catch (e) {
      console.warn('loadAllTags failed:', e)
    }
  }

  async function loadTagsForNote(noteId: number) {
    try {
      const tags = await invoke<string[]>('list_tags_for_note', { noteId })
      noteTags.value[noteId] = tags
    } catch (e) {
      console.warn(`loadTagsForNote failed for note ${noteId}:`, e)
    }
  }

  async function addTag(noteId: number, tag: string) {
    try {
      await invoke('add_note_tag', { noteId, tag })
      await loadTagsForNote(noteId)
      await loadAllTags()
    } catch (e) {
      console.warn(`addTag failed for note ${noteId}:`, e)
    }
  }

  async function removeTag(noteId: number, tag: string) {
    try {
      await invoke('remove_note_tag', { noteId, tag })
      await loadTagsForNote(noteId)
      await loadAllTags()
    } catch (e) {
      console.warn(`removeTag failed for note ${noteId}:`, e)
    }
  }

  async function loadNotes() {
    try {
      const loaded = await invoke<NoteItem[]>('list_notes')
      loaded.forEach(n => ensureParsed(n))
      notes.value = loaded
      
      // Load tags for all notes
      for (const n of loaded) {
        if (n.id) {
          await loadTagsForNote(n.id)
        }
      }
      await loadAllTags()
    } catch (e) {
      console.warn('loadNotes failed:', e)
    }
  }

  async function loadTrashedNotes() {
    try {
      trashedNotes.value = await invoke<NoteItem[]>('list_trashed_notes')
    } catch (e) {
      console.warn('loadTrashedNotes failed:', e)
    }
  }

  async function createNote(title: string, content: string, color: string, noteType?: string): Promise<NoteItem | undefined> {
    try {
      const item = await invoke<NoteItem>('create_note', { title, content, color, noteType: noteType || 'markdown' })
      notes.value.unshift(item)
      if (item && item.id) {
        noteTags.value[item.id] = []
      }
      return item
    } catch (e) {
      console.warn('create_note failed:', e)
    }
  }

  async function updateNote(note: NoteItem): Promise<void> {
    if (!note.id) return
    try {
      await invoke('update_note', {
        id: note.id,
        title: note.title,
        content: note.content,
        color: note.color,
        noteType: note.noteType || 'markdown',
        checklistItems: note.checklistItems || '[]',
      })
      note.updatedAt = Math.floor(Date.now() / 1000)
      const { emit } = await import('@tauri-apps/api/event')
      await emit('note-updated', note.id)
    } catch (e) {
      console.warn('update_note failed:', e)
    }
  }

  async function togglePinNote(note: NoteItem): Promise<void> {
    if (!note.id) return
    const nextPinned = !note.pinned
    try {
      await invoke('pin_note', { id: note.id, pinned: nextPinned })
      note.pinned = nextPinned
    } catch (e) {
      console.warn('pin_note failed:', e)
    }
  }

  async function trashNote(note: NoteItem): Promise<void> {
    if (!note.id) return
    try {
      await invoke('trash_note', { id: note.id })
      notes.value = notes.value.filter(n => n.id !== note.id)
    } catch (e) {
      console.warn('trash_note failed:', e)
    }
  }

  async function restoreNote(note: NoteItem): Promise<void> {
    if (!note.id) return
    try {
      await invoke('restore_note', { id: note.id })
      trashedNotes.value = trashedNotes.value.filter(n => n.id !== note.id)
      await loadNotes()
    } catch (e) {
      console.warn('restore_note failed:', e)
    }
  }

  async function purgeNote(note: NoteItem): Promise<void> {
    if (!note.id) return
    try {
      await invoke('purge_note', { id: note.id })
      trashedNotes.value = trashedNotes.value.filter(n => n.id !== note.id)
    } catch (e) {
      console.warn('purge_note failed:', e)
    }
  }

  async function emptyTrash(): Promise<void> {
    try {
      await invoke('empty_trash')
      trashedNotes.value = []
    } catch (e) {
      console.warn('empty_trash failed:', e)
    }
  }

  async function reorderNotes(idsInOrder: number[]): Promise<void> {
    try {
      await invoke('reorder_notes', { idsInOrder })
    } catch (e) {
      console.warn('reorder_notes failed:', e)
      await loadNotes()
    }
  }

  function startNoteSyncListener(onChange?: () => void) {
    listen('notes-changed', async () => {
      await loadNotes()
      onChange?.()
    })
    listen<number>('note-updated', async () => {
      await loadNotes()
      onChange?.()
    })
  }

  return {
    notes,
    trashedNotes,
    sortedNotes,
    noteTags,
    allTags,
    loadNotes,
    loadTrashedNotes,
    loadAllTags,
    loadTagsForNote,
    addTag,
    removeTag,
    createNote,
    updateNote,
    togglePinNote,
    trashNote,
    restoreNote,
    purgeNote,
    emptyTrash,
    reorderNotes,
    startNoteSyncListener,
    ensureParsed,
    parseChecklistItems,
  }
}
