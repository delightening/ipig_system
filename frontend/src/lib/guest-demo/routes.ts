/**
 * Guest Demo Mode — URL → demo data 路由映射表
 *
 * 當 guest 使用者發出 API 請求時，axios interceptor 會根據此映射表
 * 回傳靜態 demo data，完全不打後端 API。
 *
 * 未匹配的 GET → 通用空分頁回應 { data: [], total: 0, ... }
 * 非 GET → 不攔截（後端 guest_guard 擋寫入）
 */

import {
  DEMO_ANIMALS_PAGINATED, DEMO_ANIMALS_ALL, DEMO_ANIMAL_STATS, DEMO_ANIMALS_BY_PEN,
} from './animals'
import { DEMO_PROTOCOLS } from './protocols'
import {
  DEMO_BALANCE_SUMMARY, DEMO_LEAVES, DEMO_ATTENDANCE, DEMO_OVERTIME,
} from './hr'
import { DEMO_PRODUCTS, DEMO_DOCUMENTS, DEMO_PARTNERS } from './erp'
import {
  DEMO_EQUIPMENT_PAGINATED, DEMO_EQUIPMENT_ALL, DEMO_CALIBRATIONS, DEMO_ANNUAL_PLANS,
} from './equipment'
import { DEMO_QAU_DASHBOARD } from './qau'
import { DEMO_USERS, DEMO_ROLES, DEMO_AUDIT_LOGS } from './admin'
import {
  DEMO_LOW_STOCK_ALERTS, DEMO_VET_COMMENTS, DEMO_ANIMALS_ON_MEDICATION,
  DEMO_STAFF_ATTENDANCE_TODAY, DEMO_CALENDAR_EVENTS, DEMO_MY_PROJECTS_WIDGET,
  DEMO_NOTIFICATIONS_UNREAD, DEMO_AMENDMENTS_PENDING,
} from './dashboard'

// ============================================
// 通用空回應
// ============================================

const EMPTY_PAGINATED = { data: [], total: 0, page: 1, per_page: 20, total_pages: 0 }
const EMPTY_ARRAY: unknown[] = []

// ============================================
// 路由映射表（精確匹配 + 前綴匹配）
// ============================================

/** 精確匹配：去除 query string 後的 path → demo data */
const exactRoutes: Record<string, unknown> = {
  // === 動物管理 ===
  '/animals': DEMO_ANIMALS_PAGINATED,
  '/animals/all': DEMO_ANIMALS_ALL,
  '/animals/stats': DEMO_ANIMAL_STATS,
  '/animals/by-pen': DEMO_ANIMALS_BY_PEN,
  '/animals/vet-comments': DEMO_VET_COMMENTS,

  // === 計畫書 / AUP ===
  '/protocols': { data: DEMO_PROTOCOLS, total: DEMO_PROTOCOLS.length, page: 1, per_page: 20, total_pages: 1 },
  '/my-projects': DEMO_MY_PROJECTS_WIDGET,
  '/my-amendments': EMPTY_PAGINATED,

  // === HR ===
  '/hr/balances/summary': DEMO_BALANCE_SUMMARY,
  '/hr/leaves': DEMO_LEAVES,
  '/hr/my-leaves': DEMO_LEAVES,
  '/hr/attendance': DEMO_ATTENDANCE,
  '/hr/overtime': DEMO_OVERTIME,
  '/hr/annual-leave': EMPTY_PAGINATED,
  '/hr/training-records': EMPTY_PAGINATED,
  '/hr/dashboard/calendar': DEMO_CALENDAR_EVENTS,
  '/hr/calendar/events': DEMO_CALENDAR_EVENTS,

  // === ERP ===
  '/products': DEMO_PRODUCTS,
  '/documents': DEMO_DOCUMENTS,
  '/partners': DEMO_PARTNERS,
  '/inventory': EMPTY_PAGINATED,
  '/inventory/low-stock': DEMO_LOW_STOCK_ALERTS,
  '/warehouses': EMPTY_PAGINATED,
  '/stock-balances': EMPTY_PAGINATED,

  // === 設備 ===
  '/equipment': DEMO_EQUIPMENT_PAGINATED,
  '/equipment-calibrations': DEMO_CALIBRATIONS,
  '/equipment-annual-plans': DEMO_ANNUAL_PLANS,
  '/equipment-maintenance': EMPTY_PAGINATED,

  // === Admin 系統管理 ===
  '/users': DEMO_USERS,
  '/roles': DEMO_ROLES,
  '/admin/audit/activities': DEMO_AUDIT_LOGS,
  '/admin/audit/login-events': EMPTY_PAGINATED,
  '/admin/audit/sessions': EMPTY_PAGINATED,
  '/admin/audit/alerts': EMPTY_PAGINATED,
  '/admin/audit/dashboard': {
    total_users: 4, active_sessions: 1, suspicious_events_24h: 0,
    failed_logins_24h: 0, recent_activities: [],
  },
  '/audit-logs': EMPTY_PAGINATED,
  '/admin/settings': {},
  '/admin/facilities': EMPTY_PAGINATED,
  '/admin/notification-routing': EMPTY_ARRAY,
  '/admin/treatment-drugs': EMPTY_PAGINATED,
  '/admin/invitations': EMPTY_PAGINATED,

  // === QAU ===
  '/admin/qau/dashboard': DEMO_QAU_DASHBOARD,
  '/admin/qau/inspections': EMPTY_PAGINATED,
  '/admin/qau/non-conformances': EMPTY_PAGINATED,
  '/admin/qau/sop': EMPTY_PAGINATED,
  '/admin/qau/schedules': EMPTY_PAGINATED,

  // === Dashboard widgets ===
  '/staff-attendance': DEMO_STAFF_ATTENDANCE_TODAY,
  '/notifications/unread-count': DEMO_NOTIFICATIONS_UNREAD,
  '/amendments/pending-count': DEMO_AMENDMENTS_PENDING,

  // === 報表 ===
  '/report-stock-on-hand': EMPTY_PAGINATED,
  '/report-stock-ledger': EMPTY_PAGINATED,
  '/report-sales-lines': EMPTY_PAGINATED,
  '/report-purchase-lines': EMPTY_PAGINATED,
  '/accounting-trial-balance': EMPTY_ARRAY,
  '/accounting-profit-loss': EMPTY_ARRAY,
  '/accounting-journal-entries': EMPTY_PAGINATED,

  // === 其他 ===
  '/animal-sources': EMPTY_PAGINATED,
  '/blood-test-templates': EMPTY_PAGINATED,
  '/blood-test-panels': EMPTY_ARRAY,
  '/controlled-documents': EMPTY_PAGINATED,
  '/pens': EMPTY_ARRAY,
  '/staff': EMPTY_ARRAY,
}

/** 前綴匹配：若 path 以某前綴開頭，回傳對應 demo data */
const prefixRoutes: [string, unknown][] = [
  // 單筆查詢 — /animals/:id, /protocols/:id 等
  ['/animals/', DEMO_ANIMALS_PAGINATED.data[0]],
  ['/protocols/', DEMO_PROTOCOLS[0]],
  ['/equipment/', DEMO_EQUIPMENT_PAGINATED.data[0]],
  ['/products/', DEMO_PRODUCTS.data[0]],
  ['/documents/', DEMO_DOCUMENTS.data[0]],
  ['/users/', DEMO_USERS.data[0]],
  // Admin sub-routes
  ['/admin/qau/', EMPTY_PAGINATED],
  ['/admin/audit/', EMPTY_PAGINATED],
  ['/erp/reports', EMPTY_PAGINATED],
]

// ============================================
// 不攔截清單（讓請求正常到達後端）
// ============================================

const PASSTHROUGH_PREFIXES = [
  '/auth/',      // 登入/登出/refresh
  '/me',         // 當前用戶 profile + preferences
]

function isPassthrough(path: string): boolean {
  return PASSTHROUGH_PREFIXES.some(p => path === p || path.startsWith(p))
}

// ============================================
// 主入口
// ============================================

/**
 * 根據 URL 和 HTTP method 取得 guest demo data。
 * 回傳 undefined 表示不攔截（讓請求正常發出）。
 */
export function getGuestDemoData(url: string, method: string): unknown | undefined {
  // 只攔截 GET
  if (method !== 'GET') return undefined

  // 去除 baseURL 前綴（interceptor 中 url 可能帶 /api/v1）
  let path = url
  if (path.startsWith('/api/v1')) path = path.slice(7)
  else if (path.startsWith('/api')) path = path.slice(4)

  // 不攔截 auth / me 相關請求
  if (isPassthrough(path)) return undefined

  // 去除 query string
  const qIdx = path.indexOf('?')
  const cleanPath = qIdx >= 0 ? path.slice(0, qIdx) : path

  // 精確匹配
  if (cleanPath in exactRoutes) return exactRoutes[cleanPath]

  // 前綴匹配
  for (const [prefix, data] of prefixRoutes) {
    if (cleanPath.startsWith(prefix)) return data
  }

  // 通用 fallback — 未映射的 GET 回傳空分頁
  return EMPTY_PAGINATED
}
