import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/** 系統統一使用台灣時間 (Asia/Taipei) 顯示，可供元件內聯日期格式使用 */
export const TAIWAN_TIMEZONE = 'Asia/Taipei'

export function formatDate(date: string | Date, options?: { weekday?: boolean }) {
  return new Date(date).toLocaleDateString('zh-TW', {
    timeZone: TAIWAN_TIMEZONE,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    ...(options?.weekday && { weekday: 'long' as const }),
  })
}

export function formatDateTime(date: string | Date) {
  return new Date(date).toLocaleString('zh-TW', {
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

export function truncateText(text: string | null, maxLength: number): string {
  if (!text) return ''
  return text.length <= maxLength ? text : `${text.slice(0, maxLength)}...`
}

export function formatFileSize(bytes: number) {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
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

export function formatTime(dateStr: string | null): string {
  if (!dateStr) return '-'
  return new Date(dateStr).toLocaleTimeString('zh-TW', { timeZone: TAIWAN_TIMEZONE, hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false })
}