import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { TodoItem } from '../types'

export function useTodo() {
  const showTodo = ref(false)
  const todos = ref<TodoItem[]>([])
  const newTodoText = ref('')

  const todayStartTs = computed(() => {
    const d = new Date()
    d.setHours(0, 0, 0, 0)
    return Math.floor(d.getTime() / 1000)
  })

  // 只显示未完成的：待完成（今天）+ 未完成（过去滚入）
  const activeTodos = computed(() =>
    todos.value
      .filter(t => !t.done)
      .sort((a, b) => {
        // overdue 排前面
        const aOver = (a.targetDate ?? 0) < todayStartTs.value
        const bOver = (b.targetDate ?? 0) < todayStartTs.value
        if (aOver !== bOver) return aOver ? -1 : 1
        return (a.sortOrder ?? 0) - (b.sortOrder ?? 0)
      })
  )

  function togglePanel() {
    showTodo.value = !showTodo.value
  }

  async function loadTodos() {
    try {
      await invoke('rollover_todos')
      await invoke('generate_recurring')
      todos.value = await invoke<TodoItem[]>('list_todos')
    } catch (e) {
      console.warn('list_todos failed:', e)
    }
  }

  async function addTodo() {
    const text = newTodoText.value.trim()
    if (!text) return
    try {
      const item = await invoke<TodoItem>('add_todo', { text, dueDate: null, targetDate: null })
      todos.value.push(item)
      newTodoText.value = ''
    } catch (e) {
      console.warn('add_todo failed:', e)
    }
  }

  async function toggleTodo(todo: TodoItem) {
    const done = !todo.done
    try {
      await invoke('toggle_todo', { id: todo.id, done })
      if (done) {
        // 完成后从活跃列表移除
        todos.value = todos.value.filter(t => t.id !== todo.id)
      } else {
        todo.done = false
      }
    } catch (e) {
      console.warn('toggle_todo failed:', e)
    }
  }

  async function removeTodo(todo: TodoItem) {
    try {
      await invoke('delete_todo', { id: todo.id })
      todos.value = todos.value.filter(t => t.id !== todo.id)
    } catch (e) {
      console.warn('delete_todo failed:', e)
    }
  }

  // 监听跨窗口同步事件
  listen('todos-changed', () => {
    loadTodos()
  })

  return {
    showTodo,
    activeTodos,
    newTodoText,
    togglePanel,
    loadTodos,
    addTodo,
    toggleTodo,
    removeTodo,
  }
}
