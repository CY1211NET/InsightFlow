/**
 * 分类常量 — 与 Rust 端 Category 枚举保持一致
 * 消灭前端魔法字符串
 */

export const CATEGORY = {
  DEV: 'dev',
  PRODUCTIVITY: 'productivity',
  ENTERTAINMENT: 'entertainment',
  SOCIAL: 'social',
  BROWSER: 'browser',
  AFK: 'afk',
  OTHER: 'other',
  UNCATEGORIZED: 'uncategorized',
} as const

export type CategoryId = (typeof CATEGORY)[keyof typeof CATEGORY]

/** 是否为专注分类 */
export function isFocus(cat: string): boolean {
  return cat === CATEGORY.DEV || cat === CATEGORY.PRODUCTIVITY
}

/** 是否为干扰分类 */
export function isDistraction(cat: string): boolean {
  return cat === CATEGORY.ENTERTAINMENT || cat === CATEGORY.SOCIAL
}

/** 是否为空闲状态 */
export function isAfk(cat: string): boolean {
  return cat === CATEGORY.AFK
}
