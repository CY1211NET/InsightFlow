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
