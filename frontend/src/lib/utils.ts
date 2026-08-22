import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'
import { enUS, zhTW } from 'date-fns/locale'
import type { Locale } from 'date-fns'
import i18n from './i18n'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/** 目前 UI 語系（BCP-47），供 Intl `toLocale*` 用；預設 zh-TW。
 *  在 render 中呼叫；語系切換時元件重繪即取得新值。 */
export function uiLocale(): string {
  return i18n.language || 'zh-TW'
}

/** 目前 UI 語系對應的 date-fns locale 物件（en* → enUS，其餘 → zhTW）。 */
export function getDateFnsLocale(): Locale {
  return uiLocale().startsWith('en') ? enUS : zhTW
}

/** LOW-02: 耳號格式化（從 api/client.ts 移入，符合單一職責原則）
 *  若為純數字且 < 100，補零至 3 位數（e.g. "5" → "005"）
 */
export function formatEarTag(earTag: string): string {
  if (!earTag) return earTag
  if (/^\d+$/.test(earTag)) {
    const num = parseInt(earTag, 10)
    if (num < 100) {
      return earTag.padStart(3, '0')
    }
  }
  return earTag
}

/** 系統統一使用台灣時間 (Asia/Taipei) 顯示，可供元件內聯日期格式使用 */
export const TAIWAN_TIMEZONE = 'Asia/Taipei'

export function formatDate(date: string | Date, options?: { weekday?: boolean }) {
  return new Date(date).toLocaleDateString(uiLocale(), {
    timeZone: TAIWAN_TIMEZONE,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    ...(options?.weekday && { weekday: 'long' as const }),
  })
}

export function formatDateTime(date: string | Date) {
  return new Date(date).toLocaleString(uiLocale(), {
    timeZone: TAIWAN_TIMEZONE,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

export function formatNumber(num: number | string, decimals = 2) {
  const value = typeof num === 'string' ? parseFloat(num) : num
  return value.toLocaleString('zh-TW', {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  })
}

export function formatCurrency(num: number | string) {
  const value = typeof num === 'string' ? parseFloat(num) : num
  return value.toLocaleString('zh-TW', {
    style: 'currency',
    currency: 'TWD',
    minimumFractionDigits: 0,
  })
}

export function formatFileSize(bytes: number) {
  if (bytes === 0) return '0 Bytes'
  if (bytes < 1024) return `${bytes} Bytes`

  const kb = bytes / 1024
  if (kb < 1000) return `${Math.round(kb)} KB`

  const mb = kb / 1024
  if (mb < 1000) return `${parseFloat(mb.toFixed(1))} MB`

  const gb = mb / 1024
  return `${parseFloat(gb.toFixed(1))} GB`
}

/**
 * Format quantity as integer (whole number)
 */
export function formatQuantity(value: string | number): string {
  const num = typeof value === 'string' ? parseFloat(value) : value
  if (isNaN(num)) return ''
  // Return as integer (no decimals)
  return Math.round(num).toString()
}

/**
 * Format unit price as integer if possible, otherwise 2 decimal places
 */
export function formatUnitPrice(value: string | number): string {
  const num = typeof value === 'string' ? parseFloat(value) : value
  if (isNaN(num)) return ''
  // Check if it's a whole number
  if (num % 1 === 0) {
    return Math.round(num).toString()
  }
  // Otherwise format to 2 decimal places
  return num.toFixed(2)
}

/**
 * 庫存單位代碼對照表（英文代碼 → 中文顯示）
 * 涵蓋 CreateProductPage UNITS、編輯產品包裝單位及單據顯示用。
 */
export const UOM_MAP: Record<string, string> = {
  // 計數／個體
  'EA': '個',
  'pcs': '個',
  'PC': '支',
  'PR': '雙',
  // 藥品／劑型
  'TB': '錠',
  'CP': '膠囊',
  'BT': '瓶',
  'AMP': '安瓿',
  'VIA': '小瓶',
  // 包裝
  'BX': '盒',
  'BOX': '箱',
  'CTN': '箱',
  'PK': '包',
  'CASE': '件',
  'RL': '卷',
  'SET': '組',
  // 重量
  'G': 'g',
  'KG': 'kg',
  'MG': 'mg',
  // 體積／容量
  'ML': 'mL',
  'L': 'L',
}

/**
 * 將庫存單位代碼轉換為中文顯示
 */
export function formatUom(uom: string): string {
  return UOM_MAP[uom] || uom
}

export function sanitizeDecimalInput(value: string): string {
  const numericValue = value.replace(/[^\d.]/g, '')
  const parts = numericValue.split('.')
  return parts.length > 2 ? parts[0] + '.' + parts.slice(1).join('') : numericValue
}

export function parseDecimal(value: string | number | null | undefined): number {
  if (value == null) return 0
  const num = typeof value === 'string' ? parseFloat(value) : value
  return isNaN(num) ? 0 : num
}

/**
 * 拆分「多值」自由文字欄位（例：聯絡人 / email 以 / ; , 或換行分隔多筆）。
 * 用於顯示時一行一個。回傳已 trim、去空白的陣列；無值回空陣列。
 */
export function splitMultiValue(value: string | null | undefined): string[] {
  if (!value) return []
  return value
    .split(/[/;,\n]+/)
    .map((s) => s.trim())
    .filter((s) => s.length > 0)
}

export function formatTime(dateStr: string | null): string {
  if (!dateStr) return '-'
  return new Date(dateStr).toLocaleTimeString(uiLocale(), { timeZone: TAIWAN_TIMEZONE, hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false })
}

/**
 * 短時間格式（HH:mm，不含秒）— 供 dashboard widgets / 行事曆共用（R73-3 去重）。
 * 統一走 uiLocale() + TAIWAN_TIMEZONE + 24 小時制，對齊 formatTime；空值/解析失敗回 fallback。
 */
export function formatTimeShort(dateStr: string | null | undefined, fallback = '-'): string {
  if (!dateStr) return fallback
  try {
    return new Date(dateStr).toLocaleTimeString(uiLocale(), {
      timeZone: TAIWAN_TIMEZONE,
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
    })
  } catch {
    return fallback
  }
}