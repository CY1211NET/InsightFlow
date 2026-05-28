<template>
  <div class="checklist-editor">
    <div
      v-for="(item, idx) in items"
      :key="item.id"
      class="checklist-item"
      :class="{ done: item.done, dragging: draggedIdx === idx }"
      draggable="true"
      @dragstart="onDragStart($event, idx)"
      @dragover.prevent
      @dragenter="onDragEnter(idx)"
      @dragend="onDragEnd"
    >
      <input
        type="checkbox"
        class="check-box"
        :checked="item.done"
        @change="toggleDone(idx)"
      />
      <input
        class="check-text"
        :class="{ 'line-through': item.done }"
        :value="item.text"
        @input="updateText(idx, ($event.target as HTMLInputElement).value)"
        @keydown.enter.prevent="addItemAfter(idx)"
        @keydown.backspace="onBackspace(idx, $event)"
        @blur="onBlurItem(idx)"
      />
      <button class="check-delete" @click="removeItem(idx)" title="Delete">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
    </div>
    <div class="checklist-add" @click="addItem">
      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
      <span>{{ addPlaceholder }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import type { ChecklistItem } from '../../overlay/types'

const props = defineProps<{
  modelValue: ChecklistItem[]
  addPlaceholder?: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', items: ChecklistItem[]): void
}>()

const items = ref<ChecklistItem[]>([...props.modelValue])

watch(() => props.modelValue, (val) => {
  items.value = [...val]
}, { deep: true })

let draggedIdx: number | null = null
const draggedIdxRef = ref<number | null>(null)

function emitUpdate() {
  emit('update:modelValue', [...items.value])
}

function toggleDone(idx: number) {
  items.value[idx].done = !items.value[idx].done
  emitUpdate()
}

function updateText(idx: number, text: string) {
  items.value[idx].text = text
  emitUpdate()
}

function addItem() {
  const newItem: ChecklistItem = {
    id: crypto.randomUUID(),
    text: '',
    done: false,
    sortOrder: items.value.length + 1,
  }
  items.value.push(newItem)
  emitUpdate()
}

function addItemAfter(idx: number) {
  if (items.value[idx].text.trim() === '') return
  const newItem: ChecklistItem = {
    id: crypto.randomUUID(),
    text: '',
    done: false,
    sortOrder: idx + 2,
  }
  items.value.splice(idx + 1, 0, newItem)
  emitUpdate()
}

function removeItem(idx: number) {
  items.value.splice(idx, 1)
  emitUpdate()
}

function onBackspace(idx: number, e: KeyboardEvent) {
  if (items.value[idx].text === '' && items.value.length > 1) {
    e.preventDefault()
    items.value.splice(idx, 1)
    emitUpdate()
  }
}

function onBlurItem(idx: number) {
  if (items.value[idx].text.trim() === '' && items.value.length > 1) {
    items.value.splice(idx, 1)
    emitUpdate()
  }
}

function onDragStart(e: DragEvent, idx: number) {
  draggedIdx = idx
  draggedIdxRef.value = idx
  if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move'
}

function onDragEnter(idx: number) {
  if (draggedIdx === null || draggedIdx === idx) return
  const [moved] = items.value.splice(draggedIdx, 1)
  items.value.splice(idx, 0, moved)
  draggedIdx = idx
  draggedIdxRef.value = idx
}

function onDragEnd() {
  draggedIdx = null
  draggedIdxRef.value = null
  items.value.forEach((item: ChecklistItem, i: number) => { item.sortOrder = i + 1 })
  emitUpdate()
}
</script>

<style scoped>
.checklist-editor {
  display: flex;
  flex-direction: column;
  gap: 2px;
  width: 100%;
}
.checklist-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 3px 4px;
  border-radius: 4px;
  transition: background 0.15s;
}
.checklist-item:hover {
  background: rgba(0,0,0,0.03);
}
.night .checklist-item:hover {
  background: rgba(255,255,255,0.04);
}
.checklist-item.dragging {
  opacity: 0.4;
}
.check-box {
  width: 14px;
  height: 14px;
  accent-color: var(--accent, #c47a5a);
  cursor: pointer;
  flex-shrink: 0;
}
.check-text {
  flex: 1;
  border: none;
  background: transparent;
  font-size: 11px;
  font-family: 'Outfit', sans-serif;
  color: var(--text-primary, #1a1a1a);
  outline: none;
  padding: 0;
  line-height: 1.4;
}
.night .check-text {
  color: var(--text-primary, #e0e0e0);
}
.check-text.line-through {
  text-decoration: line-through;
  opacity: 0.45;
}
.check-delete {
  opacity: 0;
  border: none;
  background: none;
  cursor: pointer;
  color: var(--text-muted, #9a9a9a);
  padding: 2px;
  display: flex;
  align-items: center;
}
.checklist-item:hover .check-delete {
  opacity: 1;
}
.checklist-add {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 4px;
  color: var(--text-muted, #9a9a9a);
  cursor: pointer;
  font-size: 11px;
  border-radius: 4px;
}
.checklist-add:hover {
  background: rgba(0,0,0,0.03);
}
.night .checklist-add:hover {
  background: rgba(255,255,255,0.04);
}
</style>
