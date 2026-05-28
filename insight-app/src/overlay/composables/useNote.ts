import { useNoteCore, NOTE_COLORS } from '../../shared/composables/useNoteCore'

export { NOTE_COLORS }

export function useNote() {
  const core = useNoteCore()

  async function addNote(colorCode?: string) {
    const color = colorCode || NOTE_COLORS[0]
    const item = await core.createNote('', '', color)
    return item
  }

  return {
    addNote
  }
}
