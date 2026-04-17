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
  DEMO_BUILDINGS, DEMO_ZONES, DEMO_PENS,
} from './animals'
import { DEMO_PROTOCOLS } from './protocols'
import {
  DEMO_BALANCE_SUMMARY, DEMO_LEAVES, DEMO_ATTENDANCE, DEMO_OVERTIME,
} from './hr'
import {
  DEMO_PRODUCTS, DEMO_DOCUMENTS, DEMO_PARTNERS,
  DEMO_WAREHOUSES, DEMO_WAREHOUSE_TREE, DEMO_STORAGE_LOCATIONS,
  DEMO_SHELF_FA_INVENTORY, DEMO_RACK_M1_INVENTORY, DEMO_SHELF_SA_INVENTORY,
} from './erp'
import {
  DEMO_EQUIPMENT_PAGINATED, DEMO_CALIBRATIONS, DEMO_ANNUAL_PLANS,
} from './equipment'
import { DEMO_QAU_DASHBOARD } from './qau'
import { DEMO_USERS, DEMO_ROLES, DEMO_AUDIT_LOGS } from './admin'
import {
  DEMO_LOW_STOCK_ALERTS, DEMO_VET_COMMENTS,
  DEMO_STAFF_ATTENDANCE_TODAY, DEMO_CALENDAR_EVENTS, DEMO_MY_PROJECTS_WIDGET,
  DEMO_NOTIFICATIONS_UNREAD, DEMO_AMENDMENTS_PENDING,
} from './dashboard'

// ============================================
// 通用空回應
// ============================================

const EMPTY_PAGINATED = { data: [], total: 0, page: 1, per_page: 20, total_pages: 0 }
const EMPTY_ARRAY: unknown[] = []
const EMPTY_OBJECT = {}

const GUEST_USER = {
  id: 'guest',
  display_name: '訪客',
  email: 'guest@guest.com',
  roles: ['GUEST'],
  permissions: [],
  is_active: true,
}

// ============================================
// 路由映射表（精確匹配 + 前綴匹配）
// ============================================

/** 精確匹配：去除 query string 後的 path → demo data */
const exactRoutes: Record<string, unknown> = {
  // === 當前用戶（攔截 /me，避免訪客模式打後端） ===
  '/me': GUEST_USER,
  '/me/preferences/nav_order': { key: 'nav_order', value: null },

  // === 動物管理 ===
  '/animals': DEMO_ANIMALS_PAGINATED,
  '/animals/all': DEMO_ANIMALS_ALL,
  '/animals/stats': DEMO_ANIMAL_STATS,
  '/animals/by-pen': DEMO_ANIMALS_BY_PEN,
  '/animals/vet-comments': DEMO_VET_COMMENTS,

  // === 計畫書 / AUP ===
  '/protocols': { data: DEMO_PROTOCOLS, total: DEMO_PROTOCOLS.length, page: 1, per_page: 20, total_pages: 1 },
  '/my-projects': DEMO_MY_PROJECTS_WIDGET,
  // /amendments 回傳 AmendmentListItem[]（plain array）；/my-amendments 為 sidebar badge 用
  '/amendments': EMPTY_ARRAY,
  '/my-amendments': EMPTY_PAGINATED,

  // === HR ===
  '/hr/balances/summary': DEMO_BALANCE_SUMMARY,
  '/hr/leaves': DEMO_LEAVES,
  '/hr/my-leaves': DEMO_LEAVES,
  '/hr/attendance': DEMO_ATTENDANCE,
  '/hr/overtime': DEMO_OVERTIME,
  // These endpoints return plain arrays, not paginated objects
  '/hr/internal-users': EMPTY_ARRAY,
  '/hr/staff': EMPTY_ARRAY,
  '/hr/balances/annual': EMPTY_ARRAY,
  '/hr/balances/expired-compensation': EMPTY_ARRAY,
  '/hr/dashboard/calendar': DEMO_CALENDAR_EVENTS,
  '/hr/calendar/events': DEMO_CALENDAR_EVENTS,
  '/hr/calendar/config': { is_connected: false, provider: null, calendar_id: null },
  '/hr/calendar/status': { last_synced_at: null, is_syncing: false, error: null },

  // === ERP ===
  '/products': DEMO_PRODUCTS,
  '/documents': DEMO_DOCUMENTS.data,  // backend returns plain Vec<DocumentListItem>, not paginated
  '/partners': DEMO_PARTNERS,
  // /inventory/on-hand and /inventory/ledger return plain arrays
  '/inventory/on-hand': EMPTY_ARRAY,
  '/inventory/ledger': EMPTY_ARRAY,
  '/inventory/low-stock': DEMO_LOW_STOCK_ALERTS,
  '/inventory/unassigned': EMPTY_ARRAY,
  // /warehouses returns plain array (not paginated)
  '/warehouses': DEMO_WAREHOUSES,
  '/warehouses/with-shelves': DEMO_WAREHOUSE_TREE,
  '/storage-locations': DEMO_STORAGE_LOCATIONS,
  // 各儲位庫存（exact match 優先於 prefixRoutes 中的 catch-all）
  '/storage-locations/demo-loc-shelf-fa/inventory': DEMO_SHELF_FA_INVENTORY,
  '/storage-locations/demo-loc-rack-m1/inventory': DEMO_RACK_M1_INVENTORY,
  '/storage-locations/demo-loc-shelf-sa/inventory': DEMO_SHELF_SA_INVENTORY,
  '/stock-balances': EMPTY_PAGINATED,

  // === 設備 ===
  '/equipment': DEMO_EQUIPMENT_PAGINATED,
  '/equipment-calibrations': DEMO_CALIBRATIONS,
  '/equipment-annual-plans': DEMO_ANNUAL_PLANS,
  '/equipment-maintenance': EMPTY_PAGINATED,
  '/equipment-suppliers/summary': EMPTY_ARRAY,
  // '/warehouses/with-shelves' handled in exactRoutes above

  // === Admin 系統管理 ===
  // /users returns plain User[] (not paginated)
  '/users': DEMO_USERS.data,
  // /roles and /permissions return plain arrays
  '/roles': DEMO_ROLES,
  '/permissions': EMPTY_ARRAY,
  // /admin/system-settings returns Record<string, string> object
  '/admin/system-settings': EMPTY_OBJECT,
  '/admin/audit/activities': DEMO_AUDIT_LOGS,
  '/admin/audit/login-events': EMPTY_PAGINATED,
  '/admin/audit/sessions': EMPTY_PAGINATED,
  '/admin/audit/alerts': EMPTY_PAGINATED,
  '/admin/audit/dashboard': {
    total_users: 4, active_sessions: 1, suspicious_events_24h: 0,
    failed_logins_24h: 0, recent_activities: [],
  },
  '/audit-logs': EMPTY_PAGINATED,
  // notification-routing sub-routes return plain arrays
  '/admin/notification-routing': EMPTY_ARRAY,
  '/admin/notification-routing/event-types': EMPTY_ARRAY,
  '/admin/notification-routing/roles': EMPTY_ARRAY,
  '/admin/expiry-config': EMPTY_OBJECT,
  '/admin/treatment-drugs': EMPTY_PAGINATED,
  '/admin/invitations': EMPTY_PAGINATED,
  // /facilities (not /admin/facilities) returns plain Facility[]
  '/facilities': EMPTY_ARRAY,
  '/facilities/species': EMPTY_ARRAY,
  '/facilities/buildings': DEMO_BUILDINGS,
  '/facilities/zones': DEMO_ZONES,
  '/facilities/pens': DEMO_PENS,
  '/facilities/departments': EMPTY_ARRAY,

  // === QAU ===
  // 後端路由 /qau/* (非 /admin/qau/*)；list 端點回傳 { "data": [...] }，前端取 res.data.data
  '/qau/inspections': EMPTY_PAGINATED,
  '/qau/non-conformances': EMPTY_PAGINATED,
  '/qau/sop': EMPTY_PAGINATED,
  '/qau/schedules': EMPTY_PAGINATED,
  // QAU Dashboard 使用 useGuestQuery 自帶 fallback，不依賴此 interceptor，仍設定以防萬一
  '/qau/dashboard': DEMO_QAU_DASHBOARD,
  // alerts/recent 回傳 SecurityAlert[]（plain array）
  '/admin/audit/alerts/recent': EMPTY_ARRAY,

  // === Training ===
  '/training-records': EMPTY_PAGINATED,

  // === Dashboard widgets ===
  '/staff-attendance': DEMO_STAFF_ATTENDANCE_TODAY,
  '/notifications/unread-count': DEMO_NOTIFICATIONS_UNREAD,
  '/amendments/pending-count': DEMO_AMENDMENTS_PENDING,

  // === 報表（全部回傳 plain array） ===
  '/reports/stock-on-hand': EMPTY_ARRAY,
  '/reports/stock-ledger': EMPTY_ARRAY,
  '/reports/sales-lines': EMPTY_ARRAY,
  '/reports/purchase-lines': EMPTY_ARRAY,
  '/reports/cost-summary': EMPTY_ARRAY,
  '/reports/blood-test-analysis': EMPTY_ARRAY,
  '/reports/blood-test-cost': EMPTY_ARRAY,
  '/reports/purchase-sales-monthly': EMPTY_ARRAY,
  '/reports/purchase-sales-by-partner': EMPTY_ARRAY,
  '/reports/purchase-sales-by-category': EMPTY_ARRAY,
  '/accounting-trial-balance': EMPTY_ARRAY,
  '/accounting-profit-loss': EMPTY_ARRAY,
  '/accounting-journal-entries': EMPTY_PAGINATED,

  // === 其他 ===
  '/animal-sources': EMPTY_ARRAY,
  // blood-test-templates returns plain array
  '/blood-test-templates': EMPTY_ARRAY,
  '/blood-test-templates/all': EMPTY_ARRAY,
  '/blood-test-panels': EMPTY_ARRAY,
  '/blood-test-panels/all': EMPTY_ARRAY,
  '/blood-test-presets': EMPTY_ARRAY,
  '/blood-test-presets/all': EMPTY_ARRAY,
  '/controlled-documents': EMPTY_PAGINATED,
  '/pens': EMPTY_ARRAY,
  '/staff': EMPTY_ARRAY,
}

/** 前綴匹配：若 path 以某前綴開頭，回傳對應 demo data */
const prefixRoutes: [string, unknown][] = [
  // /me/preferences/* 回傳通用空值（不打後端）
  ['/me/', { key: 'preference', value: null }],
  // 動物子資源（/animals/:id/weights 等）— 必須在 /animals/ catch-all 之前，避免回傳 animal object 給 array endpoint
  ['/animals/demo-a1/', EMPTY_ARRAY],
  ['/animals/demo-a2/', EMPTY_ARRAY],
  ['/animals/demo-a3/', EMPTY_ARRAY],
  ['/animals/demo-a4/', EMPTY_ARRAY],
  ['/animals/demo-a5/', EMPTY_ARRAY],
  // 儲位子資源（inventory 等）— 無 exact match 時回傳空陣列
  ['/storage-locations/', EMPTY_ARRAY],
  // 單筆查詢 — /animals/:id, /protocols/:id 等
  ['/animals/', DEMO_ANIMALS_PAGINATED.data[0]],
  ['/protocols/', DEMO_PROTOCOLS[0]],
  ['/equipment/', DEMO_EQUIPMENT_PAGINATED.data[0]],
  ['/products/', DEMO_PRODUCTS.data[0]],
  ['/documents/', DEMO_DOCUMENTS.data[0]],
  ['/users/', (DEMO_USERS.data as { id: string }[])[0]],
  // QAU 單筆查詢 — /qau/non-conformances/:id, /qau/inspections/:id 等
  ['/qau/', EMPTY_PAGINATED],
  // Admin audit sub-routes（alerts/recent 已有 exact match，其他回傳分頁空）
  ['/admin/audit/', EMPTY_PAGINATED],
  ['/erp/reports', EMPTY_PAGINATED],
]

// ============================================
// 不攔截清單（讓請求正常到達後端）
// ============================================

// 注意：/me 已從 passthrough 移除，改由 exactRoutes / prefixRoutes 攔截
const PASSTHROUGH_PREFIXES = [
  '/auth/',      // 登入/登出/refresh（不攔截，由 logout() 在 store 層處理）
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
  // 去除 baseURL 前綴（interceptor 中 url 可能帶 /api/v1）
  let path = url
  if (path.startsWith('/api/v1')) path = path.slice(7)
  else if (path.startsWith('/api')) path = path.slice(4)

  // auth 相關不攔截（/auth/logout 等由 store 層處理）
  if (isPassthrough(path)) return undefined

  // 去除 query string
  const qIdx = path.indexOf('?')
  const cleanPath = qIdx >= 0 ? path.slice(0, qIdx) : path

  // ── GET：回傳 demo 靜態資料 ─────────────────────────────────────
  if (method === 'GET') {
    if (cleanPath in exactRoutes) return exactRoutes[cleanPath]
    for (const [prefix, data] of prefixRoutes) {
      if (cleanPath.startsWith(prefix)) return data
    }
    return EMPTY_PAGINATED
  }

  // ── 寫入（POST / PUT / PATCH / DELETE）：模擬成功，不打後端 ───────
  return { success: true }
}
