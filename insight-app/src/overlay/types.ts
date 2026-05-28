export interface AppUsage {
  appName: string
  durationSecs: number
}

export interface ModuleConfig {
  id: string
  name: string
  color: string
  appKeywords: string[]
  siteDomains: string[]
}

export interface ModuleProgress {
  category: string
  actualSecs: number
  goalSecs: number
  goalPct: number
  topApps: AppUsage[]
}

export interface OverlayData {
  currentApp: string
  category: string
  sessionSecs: number
  focusSecs: number
  goalPct: number
  categorySecs: number
  aiHint: string
}

export interface TodoItem {
  id: number | null
  text: string
  done: boolean
  sortOrder: number
  createdAt: number
  updatedAt: number
  source: string
  groupId: string | null
  dueDate: number | null
  targetDate: number | null
  doneDate: number | null
}

export interface ChecklistItem {
  id: string
  text: string
  done: boolean
  sortOrder: number
}

export interface NoteItem {
  id: number | null
  title: string
  content: string
  color: string
  pinned: boolean
  noteType: string
  checklistItems: string
  sortOrder: number
  x: number | null
  y: number | null
  width: number | null
  height: number | null
  trashed: boolean
  trashedAt: number | null
  createdAt: number
  updatedAt: number
  _parsedChecklist?: ChecklistItem[]
}
