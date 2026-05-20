<template>
  <Transition name="expand">
    <div v-if="show" class="row-todo">
      <div class="todo-add">
        <input
          class="todo-input"
          :value="newTodoText"
          @input="$emit('update:newTodoText', ($event.target as HTMLInputElement).value)"
          :placeholder="t('dashboard.todoPlaceholder')"
          @keydown.enter="$emit('add')"
          @click.stop
        />
        <button class="todo-add-btn" @click.stop="$emit('add')">+</button>
      </div>
      <div class="todo-items">
        <div v-if="todos.length === 0" class="todo-empty">{{ t('dashboard.noTodos') }}</div>
        <div
          v-for="todo in sortedTodos"
          :key="String(todo.id)"
          class="todo-row"
          :class="{ done: todo.done }"
        >
          <input
            type="checkbox"
            class="todo-cb"
            :checked="todo.done"
            @change="$emit('toggle', todo)"
            @click.stop
          />
          <span class="todo-text">{{ todo.text }}</span>
          <span v-if="isOverdue(todo)" class="todo-due-tag overdue">{{ overdueLabel(todo) }}</span>
          <span v-else-if="todo.dueDate" class="todo-due-tag" :class="dueDateClass(todo.dueDate)">{{ dueDateLabel(todo.dueDate) }}</span>
          <button class="todo-del" @click.stop="$emit('remove', todo)" aria-label="Delete">×</button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { t } from '../shared/i18n'
import type { TodoItem } from './types'

const props = defineProps<{
  show: boolean
  todos: TodoItem[]
  newTodoText: string
}>()

defineEmits<{
  'update:newTodoText': [value: string]
  add: []
  toggle: [todo: TodoItem]
  remove: [todo: TodoItem]
}>()

function dueDateLabel(ts: number | null): string {
  if (!ts) return ''
  const now = new Date()
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const due = new Date(ts * 1000)
  const dueDay = new Date(due.getFullYear(), due.getMonth(), due.getDate())
  const diff = Math.round((dueDay.getTime() - today.getTime()) / 86400000)
  if (diff < 0) return `${-diff}d`
  if (diff === 0) return 'today'
  if (diff === 1) return 'tmr'
  return `${diff}d`
}

function dueDateClass(ts: number | null): string {
  if (!ts) return ''
  const now = new Date()
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const due = new Date(ts * 1000)
  const dueDay = new Date(due.getFullYear(), due.getMonth(), due.getDate())
  const diff = Math.round((dueDay.getTime() - today.getTime()) / 86400000)
  if (diff < 0) return 'overdue'
  if (diff === 0) return 'due-today'
  return 'due-future'
}

function todayStartTs(): number {
  const d = new Date()
  d.setHours(0, 0, 0, 0)
  return Math.floor(d.getTime() / 1000)
}

function isOverdue(todo: TodoItem): boolean {
  return !todo.done && (todo.targetDate ?? 0) < todayStartTs()
}

function overdueLabel(todo: TodoItem): string {
  const td = todo.targetDate
  if (!td) return ''
  const diff = Math.round((todayStartTs() - td) / 86400)
  if (diff <= 0) return ''
  return `${diff}d`
}

const sortedTodos = computed(() =>
  [...props.todos].sort((a, b) => {
    const aOver = isOverdue(a)
    const bOver = isOverdue(b)
    if (aOver !== bOver) return aOver ? -1 : 1
    return (a.sortOrder ?? 0) - (b.sortOrder ?? 0)
  })
)
</script>

<style scoped>
.row-todo {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-top: 6px;
  border-top: 1px solid var(--track);
}

.todo-add {
  display: flex;
  gap: 4px;
}

.todo-input {
  flex: 1;
  height: 24px;
  background: var(--surface);
  border: 1px solid var(--track);
  border-radius: 6px;
  color: var(--text-primary);
  font-family: 'Outfit', sans-serif;
  font-size: 11px;
  padding: 0 6px;
  outline: none;
  transition: border-color 0.15s;
}
.todo-input:focus {
  border-color: var(--text-muted);
}
.todo-input::placeholder {
  color: var(--text-dim);
}

.todo-add-btn {
  width: 24px;
  height: 24px;
  background: var(--surface);
  border: 1px solid var(--track);
  border-radius: 6px;
  color: var(--text-secondary);
  font-size: 14px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s;
  flex-shrink: 0;
}
.todo-add-btn:hover {
  background: var(--track);
}

.todo-items {
  display: flex;
  flex-direction: column;
  gap: 2px;
  max-height: 150px;
  overflow-y: auto;
}

.todo-empty {
  font-size: 10px;
  color: var(--text-dim);
  text-align: center;
  padding: 4px 0;
}

.todo-row {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 2px 4px;
  border-radius: 4px;
  transition: background 0.1s;
}
.todo-row:hover {
  background: var(--surface);
}
.todo-row.done .todo-text {
  text-decoration: line-through;
  color: var(--text-dim);
}

.todo-cb {
  width: 12px;
  height: 12px;
  accent-color: var(--accent);
  cursor: pointer;
  flex-shrink: 0;
}

.todo-text {
  flex: 1;
  font-size: 11px;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.todo-del {
  width: 16px;
  height: 16px;
  background: none;
  border: none;
  color: var(--text-dim);
  font-size: 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
  opacity: 0;
  transition: opacity 0.15s, color 0.15s;
  flex-shrink: 0;
}
.todo-row:hover .todo-del {
  opacity: 1;
}
.todo-del:hover {
  color: #e74c3c;
}

.todo-due-tag {
  font-size: 9px;
  padding: 1px 4px;
  border-radius: 3px;
  white-space: nowrap;
  flex-shrink: 0;
  font-weight: 500;
}
.todo-due-tag.overdue {
  background: rgba(231, 76, 60, 0.15);
  color: #e74c3c;
}
.todo-due-tag.due-today {
  background: var(--accent-soft);
  color: var(--accent);
}
.todo-due-tag.due-future {
  background: var(--surface-06);
  color: var(--text-muted);
}
</style>
