/**
 * Guest Demo Mode — URL → demo data 路由映射表
 *
 * 當 guest 使用者發出 API 請求時，axios interceptor 會根據此映射表
 * 回傳靜態 demo data，完全不打後端 API。
 *
 * 未匹配的 GET → 通用空分頁回應 { data: [], total: 0, ... }
 * 非 GET → 不攔截（後端 guest_guard 擋寫入）
 */

import i18n from '@/lib/i18n'
import {
  DEMO_ANIMALS_PAGINATED, DEMO_ANIMALS_ALL, DEMO_ANIMAL_STATS, DEMO_ANIMALS_BY_PEN,
  DEMO_BUILDINGS, DEMO_ZONES, DEMO_PENS,
  DEMO_ANIMAL_OBSERVATIONS, DEMO_ANIMAL_WEIGHTS, DEMO_ANIMAL_VACCINATIONS,
  DEMO_ANIMAL_SURGERIES, DEMO_ANIMAL_PAIN_ASSESSMENTS, DEMO_ANIMAL_SOURCES,
  DEMO_AVAILABLE_PIGS,
} from './animals'
import {
  DEMO_PROTOCOLS,
  DEMO_PROTOCOL_DETAIL_P1, DEMO_PROTOCOL_DETAIL_P2, DEMO_PROTOCOL_DETAIL_P3,
} from './protocols'
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
import {
  DEMO_USERS, DEMO_ROLES, DEMO_PERMISSIONS, DEMO_AUDIT_LOGS,
  DEMO_QAU_INSPECTIONS, DEMO_QAU_NCS, DEMO_QAU_SOPS,
  DEMO_TRAINING_RECORDS,
} from './admin'
import { DEMO_THREAD_LIST, DEMO_THREAD_DETAILS } from './messaging'
import {
  DEMO_CONTROLLED_DOCUMENTS, DEMO_CHANGE_REQUESTS, DEMO_RISKS,
  DEMO_MANAGEMENT_REVIEWS, DEMO_FORMULATION_RECORDS, DEMO_COMPETENCY_ASSESSMENTS,
  DEMO_STUDY_REPORTS, DEMO_MONITORING_POINTS, DEMO_ENVIRONMENT_READINGS,
} from './glp'
import {
  DEMO_LOW_STOCK_ALERTS, DEMO_VET_COMMENTS,
  DEMO_STAFF_ATTENDANCE_TODAY, DEMO_CALENDAR_EVENTS, DEMO_MY_PROJECTS_WIDGET,
  DEMO_NOTIFICATIONS_UNREAD, DEMO_NOTIFICATIONS_ACTION_REQUIRED, DEMO_AMENDMENTS_PENDING,
} from './dashboard'
import {
  DEMO_STOCK_ON_HAND, DEMO_STOCK_LEDGER, DEMO_PURCHASE_LINES, DEMO_SALES_LINES,
  DEMO_PROTOCOL_CONSUMPTION,
  DEMO_COST_SUMMARY, DEMO_PS_MONTHLY, DEMO_PS_BY_PARTNER, DEMO_PS_BY_CATEGORY,
  DEMO_BLOOD_TEST_COST, DEMO_BLOOD_TEST_ANALYSIS,
  DEMO_TRIAL_BALANCE, DEMO_JOURNAL_ENTRIES, DEMO_AP_AGING, DEMO_AR_AGING, DEMO_PROFIT_LOSS,
  DEMO_WEEKLY_MEDICAL, DEMO_BYPRODUCT_MONTHLY,
} from './reports'
import {
  DEMO_VET_PATROL_REPORTS,
  DEMO_BLOOD_TEST_TEMPLATES, DEMO_BLOOD_TEST_PANELS, DEMO_BLOOD_TEST_PRESETS,
  DEMO_HR_INTERNAL_USERS, DEMO_ANNUAL_LEAVE_BALANCE, DEMO_EXPIRED_COMPENSATION,
  DEMO_INVENTORY_ON_HAND, DEMO_INVENTORY_LEDGER, DEMO_MAINTENANCE_RECORDS,
  DEMO_QA_SCHEDULES, DEMO_TREATMENT_DRUGS,
  DEMO_FACILITIES, DEMO_SPECIES, DEMO_DEPARTMENTS,
  DEMO_LOGIN_EVENTS, DEMO_SESSIONS, DEMO_SECURITY_ALERTS, DEMO_SECURITY_AUDIT_LOGS,
  DEMO_MY_AMENDMENTS,
} from './misc'
import {
  DEMO_COMMITTEE_MEMBERS, DEMO_DEPARTMENT_MEMBERS, DEMO_IP_BLOCKLIST,
  DEMO_RESERVATION_PLANNING, DEMO_RESERVABLE_ANIMALS,
  DEMO_MESSAGE_RECIPIENTS, DEMO_VET_PATROL_REPORT_DETAILS,
  DEMO_DASHBOARD_CALENDAR, DEMO_HR_STAFF, DEMO_LEAVE_CALENDAR,
  DEMO_UNASSIGNED_INVENTORY, DEMO_EQUIPMENT_SUPPLIER_SUMMARY,
} from './fixes'

// ============================================
// 通用空回應
// ============================================

const EMPTY_PAGINATED = { data: [], total: 0, page: 1, per_page: 20, total_pages: 0 }
const EMPTY_ARRAY: unknown[] = []
const EMPTY_OBJECT = {}

// 報表查詢用 POST 傳 filter（語意為讀取，非寫入）→ 回 demo 陣列，
// 避免頁面對 { success: true } 呼叫 .map() 崩潰，且不顯示「無法儲存」toast。
const postReportRoutes: Record<string, unknown> = {
  '/reports/animal-medical/weekly': DEMO_WEEKLY_MEDICAL,
  '/reports/byproduct-monthly': DEMO_BYPRODUCT_MONTHLY,
}

const GUEST_USER = {
  id: 'guest',
  display_name: '訪客',
  email: 'guest@guest.com',
  roles: ['GUEST'],
  permissions: [],
  is_active: true,
}

// 變更申請 detail（訪客 demo）— 供 /protocols/amendments/:id 頁面與通知 deep-link，
// 避免 catch-all EMPTY_PAGINATED 落到 amendment object 造成欄位 undefined。
const DEMO_AMENDMENT_DETAIL = {
  id: 'demo-amd-1',
  protocol_id: 'demo-p1',
  amendment_no: 'AMD-2025-001',
  revision_number: 1,
  amendment_type: 'MAJOR',
  status: 'UNDER_REVIEW',
  title: '範例變更申請（示範資料）',
  description: '訪客模式範例：增加動物數量並調整實驗程序。',
  change_items: ['ANIMAL_COUNT', 'PROCEDURE'],
  created_by: 'demo-u1',
  created_at: '2025-01-10T00:00:00Z',
  updated_at: '2025-01-12T00:00:00Z',
  is_historical: false,
  version: 1,
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

  // 動物 detail 子資源（demo-a1 動物完整 timeline 範例）
  '/animals/demo-a1/observations': DEMO_ANIMAL_OBSERVATIONS.filter(o => o.animal_id === 'demo-a1'),
  '/animals/demo-a1/observations/with-recommendations': DEMO_ANIMAL_OBSERVATIONS.filter(o => o.animal_id === 'demo-a1'),
  '/animals/demo-a1/weights': DEMO_ANIMAL_WEIGHTS.filter(w => w.animal_id === 'demo-a1'),
  '/animals/demo-a1/vaccinations': DEMO_ANIMAL_VACCINATIONS.filter(v => v.animal_id === 'demo-a1'),
  '/animals/demo-a1/surgeries': DEMO_ANIMAL_SURGERIES.filter(s => s.animal_id === 'demo-a1'),
  '/animals/demo-a1/surgeries/with-recommendations': DEMO_ANIMAL_SURGERIES.filter(s => s.animal_id === 'demo-a1'),
  '/animals/demo-a1/care-records': DEMO_ANIMAL_PAIN_ASSESSMENTS,
  '/animals/demo-a2/observations': DEMO_ANIMAL_OBSERVATIONS.filter(o => o.animal_id === 'demo-a2'),
  // 手術 detail 對應 pain assessments
  '/surgeries/demo-surg-1/care-records': DEMO_ANIMAL_PAIN_ASSESSMENTS,

  // === 計畫書 / AUP ===
  '/protocols': { data: DEMO_PROTOCOLS, total: DEMO_PROTOCOLS.length, page: 1, per_page: 20, total_pages: 1 },
  '/protocols/demo-p1': DEMO_PROTOCOL_DETAIL_P1,
  '/protocols/demo-p2': DEMO_PROTOCOL_DETAIL_P2,
  '/protocols/demo-p3': DEMO_PROTOCOL_DETAIL_P3,
  '/protocols/demo-p1/animal-stats': { total: 2, in_experiment: 2, completed: 0 },
  '/protocols/demo-p2/animal-stats': { total: 0, in_experiment: 0, completed: 0 },
  '/protocols/demo-p3/animal-stats': { total: 0, in_experiment: 0, completed: 0 },
  '/protocols/demo-p1/versions': EMPTY_ARRAY,
  '/protocols/demo-p2/versions': EMPTY_ARRAY,
  '/protocols/demo-p3/versions': EMPTY_ARRAY,
  '/protocols/demo-p1/attachments': EMPTY_ARRAY,
  '/protocols/demo-p2/attachments': EMPTY_ARRAY,
  '/protocols/demo-p3/attachments': EMPTY_ARRAY,
  '/protocols/demo-p1/review-comments': EMPTY_ARRAY,
  '/protocols/demo-p2/review-comments': EMPTY_ARRAY,
  '/protocols/demo-p3/review-comments': EMPTY_ARRAY,
  '/protocols/demo-p1/reviews/assignments': EMPTY_ARRAY,
  '/protocols/demo-p2/reviews/assignments': EMPTY_ARRAY,
  '/protocols/demo-p3/reviews/assignments': EMPTY_ARRAY,
  '/my-projects': DEMO_MY_PROJECTS_WIDGET,
  // /amendments 回傳 AmendmentListItem[]（plain array）；MyAmendmentsPage 打此端點，客戶端過濾
  '/amendments': DEMO_MY_AMENDMENTS,
  '/my-amendments': EMPTY_PAGINATED,
  // 變更申請 detail 子資源（exact match 優先於 /amendments/ prefix catch-all）
  // demo-amd-1 用完整 detail；demo-amd-2/3 直接沿用列表項（AmendmentListItem 相容 Amendment），
  // 避免點列表列 deep-link 落到 /amendments/ 的 EMPTY_ARRAY catch-all（物件頁拿到陣列）。
  '/amendments/demo-amd-1': DEMO_AMENDMENT_DETAIL,
  '/amendments/demo-amd-1/assignments': EMPTY_ARRAY,
  '/amendments/demo-amd-1/history': EMPTY_ARRAY,
  '/amendments/demo-amd-2': DEMO_MY_AMENDMENTS.find(a => a.id === 'demo-amd-2'),
  '/amendments/demo-amd-2/assignments': EMPTY_ARRAY,
  '/amendments/demo-amd-2/history': EMPTY_ARRAY,
  '/amendments/demo-amd-3': DEMO_MY_AMENDMENTS.find(a => a.id === 'demo-amd-3'),
  '/amendments/demo-amd-3/assignments': EMPTY_ARRAY,
  '/amendments/demo-amd-3/history': EMPTY_ARRAY,

  // === HR ===
  '/hr/balances/summary': DEMO_BALANCE_SUMMARY,
  '/hr/leaves': DEMO_LEAVES,
  '/hr/my-leaves': DEMO_LEAVES,
  '/hr/attendance': DEMO_ATTENDANCE,
  '/hr/overtime': DEMO_OVERTIME,
  // These endpoints return plain arrays, not paginated objects
  '/hr/internal-users': DEMO_HR_INTERNAL_USERS,
  // /hr/staff 回傳 StaffInfo[]（plain array，員工清單 / 收件人 / 陪同人員下拉用）
  '/hr/staff': DEMO_HR_STAFF,
  '/hr/balances/annual': DEMO_ANNUAL_LEAVE_BALANCE,
  '/hr/balances/expired-compensation': DEMO_EXPIRED_COMPENSATION,
  // 首頁日曆 widget 期望 CalendarData 物件（今日請假 / 今日日程 / 近期請假）；
  // 與 Google 行事曆 widget 的 /hr/calendar/events（CalendarEvent[]）形狀不同，須分開。
  '/hr/dashboard/calendar': DEMO_DASHBOARD_CALENDAR,
  '/hr/calendar/events': DEMO_CALENDAR_EVENTS,
  '/hr/calendar/config': { is_connected: false, provider: null, calendar_id: null },
  '/hr/calendar/status': { last_synced_at: null, is_syncing: false, error: null },
  // 原生請假行事曆：形狀為 { entries, scope }，與上面三個 Google 端點都不同。
  '/hr/leaves/calendar': DEMO_LEAVE_CALENDAR,

  // === ERP ===
  '/products': DEMO_PRODUCTS,
  '/documents': DEMO_DOCUMENTS.data,  // backend returns plain Vec<DocumentListItem>, not paginated
  '/partners': DEMO_PARTNERS,
  // /inventory/on-hand and /inventory/ledger return plain arrays
  '/inventory/on-hand': DEMO_INVENTORY_ON_HAND,
  '/inventory/ledger': DEMO_INVENTORY_LEDGER,
  '/inventory/low-stock': DEMO_LOW_STOCK_ALERTS,
  '/inventory/unassigned': DEMO_UNASSIGNED_INVENTORY,
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
  '/equipment-maintenance': DEMO_MAINTENANCE_RECORDS,
  '/equipment-suppliers/summary': DEMO_EQUIPMENT_SUPPLIER_SUMMARY,
  // '/warehouses/with-shelves' handled in exactRoutes above

  // === Admin 系統管理 ===
  // /users returns plain User[] (not paginated)
  '/users': DEMO_USERS.data,
  // /roles and /permissions return plain arrays
  '/roles': DEMO_ROLES,
  '/permissions': DEMO_PERMISSIONS,
  // /admin/system-settings returns Record<string, string> object
  '/admin/system-settings': EMPTY_OBJECT,
  // 操作日誌頁（AuditLogsPage）打 /admin/audit/activities（PaginatedResponse<UserActivityLog>）
  '/admin/audit/activities': DEMO_AUDIT_LOGS,
  // 安全審計頁子分頁：登入端點實為 /admin/audit/logins（非 /login-events）
  '/admin/audit/logins': DEMO_LOGIN_EVENTS,
  '/admin/audit/sessions': DEMO_SESSIONS,
  '/admin/audit/alerts': DEMO_SECURITY_ALERTS,
  // 安全事件 tab（PaginatedResponse<UserActivityLog>）沿用操作日誌資料
  '/admin/audit/security-events': DEMO_AUDIT_LOGS,
  '/admin/audit/dashboard': {
    total_users: 4, active_sessions: 1, suspicious_events_24h: 0,
    failed_logins_24h: 0, recent_activities: [],
  },
  // 安全審計「活動記錄」tab 打 /audit-logs（較精簡的 AuditLog 型別）
  '/audit-logs': DEMO_SECURITY_AUDIT_LOGS,
  // IP 黑名單頁（IpBlocklistEntry[]，plain array）；exact 優先於 /admin/audit/ prefix
  '/admin/audit/ip-blocklist': DEMO_IP_BLOCKLIST,
  // notification-routing sub-routes return plain arrays
  '/admin/notification-routing': EMPTY_ARRAY,
  '/admin/notification-routing/event-types': EMPTY_ARRAY,
  '/admin/notification-routing/roles': EMPTY_ARRAY,
  '/admin/expiry-config': EMPTY_OBJECT,
  // /admin/treatment-drugs 回傳 plain TreatmentDrugOption[]（非分頁）；用 EMPTY_PAGINATED 會讓頁面對物件 .map() 崩潰
  '/admin/treatment-drugs': DEMO_TREATMENT_DRUGS,
  '/admin/invitations': EMPTY_PAGINATED,
  // /facilities (not /admin/facilities) returns plain Facility[]
  '/facilities': DEMO_FACILITIES,
  '/facilities/species': DEMO_SPECIES,
  '/facilities/buildings': DEMO_BUILDINGS,
  '/facilities/zones': DEMO_ZONES,
  '/facilities/pens': DEMO_PENS,
  '/facilities/departments': DEMO_DEPARTMENTS,
  // 組織圖：全部門成員（DepartmentMember[]，plain array；department_id 對齊 DEMO_DEPARTMENTS）
  '/facilities/department-members': DEMO_DEPARTMENT_MEMBERS,
  // IACUC 委員名冊（CommitteeMember[]）。刻意讓「範例獸醫」同時出現在獸醫部與
  // 這份名冊裡——那正是本功能存在的理由：委員會是跨部門的橫切面。
  '/facilities/committee-members': DEMO_COMMITTEE_MEMBERS,

  // === QAU ===
  // 後端路由 /qau/* (非 /admin/qau/*)；list 端點回傳 { "data": [...] }，前端取 res.data.data
  '/qau/inspections': DEMO_QAU_INSPECTIONS,
  '/qau/non-conformances': DEMO_QAU_NCS,
  '/qau/sop': DEMO_QAU_SOPS,
  // /qau/schedules 後端回 { data: [...] }，前端取 res.data.data
  '/qau/schedules': { data: DEMO_QA_SCHEDULES },
  // QAU Dashboard 使用 useGuestQuery 自帶 fallback，不依賴此 interceptor，仍設定以防萬一
  '/qau/dashboard': DEMO_QAU_DASHBOARD,
  // alerts/recent 回傳 SecurityAlert[]（plain array）
  '/admin/audit/alerts/recent': DEMO_SECURITY_ALERTS.data,

  // === GLP 合規（8 頁；list 端點回傳 { data: [...] }，前端取 res.data.data） ===
  '/admin/documents': { data: DEMO_CONTROLLED_DOCUMENTS },
  '/admin/change-requests': { data: DEMO_CHANGE_REQUESTS },
  '/admin/risks': { data: DEMO_RISKS },
  '/admin/management-reviews': { data: DEMO_MANAGEMENT_REVIEWS },
  '/admin/formulation-records': { data: DEMO_FORMULATION_RECORDS },
  '/admin/competency-assessments': { data: DEMO_COMPETENCY_ASSESSMENTS },
  '/admin/study-reports': { data: DEMO_STUDY_REPORTS },
  '/admin/env-monitoring/points': { data: DEMO_MONITORING_POINTS },
  '/admin/env-monitoring/readings': { data: DEMO_ENVIRONMENT_READINGS },

  // === Training ===
  '/training-records': DEMO_TRAINING_RECORDS,

  // === Vet patrol reports (R39) — 列表回傳 ReportRow[]，需 flat array 否則 .map() 炸 ===
  '/vet-patrol-reports': DEMO_VET_PATROL_REPORTS,

  // === R47 可用豬隻快速查詢 ===
  '/animals/available': DEMO_AVAILABLE_PIGS,
  // === 動物預約與試驗規劃（Phase 4）===
  // 兩者皆回 plain array；/animals/reservable exact 蓋過 /animals/ prefix。
  '/reservation-planning': DEMO_RESERVATION_PLANNING,
  '/animals/reservable': DEMO_RESERVABLE_ANIMALS,

  // === R40 站內信 ===
  '/messages/threads': DEMO_THREAD_LIST,
  '/messages/threads/demo-t1': DEMO_THREAD_DETAILS['demo-t1'],
  '/messages/threads/demo-t2': DEMO_THREAD_DETAILS['demo-t2'],
  '/messages/threads/demo-t3': DEMO_THREAD_DETAILS['demo-t3'],
  // 新訊息收件人（StaffOption[]，plain array）
  '/messages/recipients': DEMO_MESSAGE_RECIPIENTS,

  // === 計畫書審查 tab（依 protocol_id query 撈；interceptor 去 query 後回空陣列，
  //     避免頁面對 EMPTY_PAGINATED 物件 .filter/.map/.some 崩潰）===
  '/reviews/comments': EMPTY_ARRAY,
  '/reviews/assignments': EMPTY_ARRAY,

  // === 巡場報告詳情（VetPatrolReportView）— 回單一 PatrolReport 物件，
  //     photos 子路徑回空陣列（照片留空避免破圖）。exact 優先，列表 /vet-patrol-reports 不受影響 ===
  '/vet-patrol-reports/demo-vpr-1': DEMO_VET_PATROL_REPORT_DETAILS['demo-vpr-1'],
  '/vet-patrol-reports/demo-vpr-2': DEMO_VET_PATROL_REPORT_DETAILS['demo-vpr-2'],
  '/vet-patrol-reports/demo-vpr-3': DEMO_VET_PATROL_REPORT_DETAILS['demo-vpr-3'],
  '/vet-patrol-reports/demo-vpr-4': DEMO_VET_PATROL_REPORT_DETAILS['demo-vpr-4'],
  '/vet-patrol-reports/demo-vpr-1/photos': EMPTY_ARRAY,
  '/vet-patrol-reports/demo-vpr-2/photos': EMPTY_ARRAY,
  '/vet-patrol-reports/demo-vpr-3/photos': EMPTY_ARRAY,
  '/vet-patrol-reports/demo-vpr-4/photos': EMPTY_ARRAY,

  // === Dashboard widgets ===
  '/staff-attendance': DEMO_STAFF_ATTENDANCE_TODAY,
  '/notifications/unread-count': DEMO_NOTIFICATIONS_UNREAD,
  '/notifications/action-required-count': DEMO_NOTIFICATIONS_ACTION_REQUIRED,
  '/amendments/pending-count': DEMO_AMENDMENTS_PENDING,

  // === 報表中心（batch 2；ERP 報表回 plain array） ===
  '/reports/stock-on-hand': DEMO_STOCK_ON_HAND,
  '/reports/stock-ledger': DEMO_STOCK_LEDGER,
  '/reports/sales-lines': DEMO_SALES_LINES,
  '/reports/purchase-lines': DEMO_PURCHASE_LINES,
  '/reports/protocol-consumption': DEMO_PROTOCOL_CONSUMPTION,
  '/reports/cost-summary': DEMO_COST_SUMMARY,
  '/reports/blood-test-analysis': DEMO_BLOOD_TEST_ANALYSIS,
  '/reports/blood-test-cost': DEMO_BLOOD_TEST_COST,
  '/reports/purchase-sales-monthly': DEMO_PS_MONTHLY,
  '/reports/purchase-sales-by-partner': DEMO_PS_BY_PARTNER,
  '/reports/purchase-sales-by-category': DEMO_PS_BY_CATEGORY,
  // 會計報表：實際端點為 /accounting/*（非 /accounting-*）；帳齡/試算/傳票回陣列、損益表回物件
  '/accounting/trial-balance': DEMO_TRIAL_BALANCE,
  '/accounting/journal-entries': DEMO_JOURNAL_ENTRIES,
  '/accounting/ap-aging': DEMO_AP_AGING,
  '/accounting/ar-aging': DEMO_AR_AGING,
  '/accounting/profit-loss': DEMO_PROFIT_LOSS,

  // === 其他 ===
  '/animal-sources': DEMO_ANIMAL_SOURCES,
  // blood-test-templates returns plain array
  '/blood-test-templates': DEMO_BLOOD_TEST_TEMPLATES,
  '/blood-test-templates/all': DEMO_BLOOD_TEST_TEMPLATES,
  '/blood-test-panels': DEMO_BLOOD_TEST_PANELS,
  '/blood-test-panels/all': DEMO_BLOOD_TEST_PANELS,
  '/blood-test-presets': DEMO_BLOOD_TEST_PRESETS,
  '/blood-test-presets/all': DEMO_BLOOD_TEST_PRESETS,
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
  // 變更申請子資源 — assignments / history 回傳陣列（須在 /amendments/ catch-all 之前）
  ['/amendments/', EMPTY_ARRAY],
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

  // ── 報表查詢（POST 傳 filter，實為讀取）：回 demo 陣列 ───────────
  if (cleanPath in postReportRoutes) return postReportRoutes[cleanPath]

  // ── 寫入（POST / PUT / PATCH / DELETE）：模擬成功，不打後端 ───────
  // 標記 _guest_demo 旗標讓 client interceptor 顯示 toast 提示「示範模式無法儲存」
  return { success: true, _guest_demo: true }
}

/** Guest 寫入操作攔截後的 toast 文案（已透過 i18n.t() 取值，可直接渲染） */
export interface GuestWriteToast {
  title: string
  description: string
}

/** 寫入時的友善提示文字（client interceptor 用），透過 i18n 取值支援多語系 */
export function getGuestWriteToast(method: string, path: string): GuestWriteToast | null {
  const m = method.toUpperCase()
  if (m === 'GET') return null

  // 報表查詢 / 匯出走 POST，屬讀取語意（非儲存）→ 不顯示「無法儲存」toast
  if (path.includes('/reports/')) return null

  // 將 HTTP method / path 對應到 i18n key（純語意對應，無語言相關字串）
  let actionKey: 'default' | 'delete' | 'post' | 'put' = 'default'
  if (m === 'DELETE') actionKey = 'delete'
  else if (m === 'POST') actionKey = 'post'
  else if (m === 'PUT' || m === 'PATCH') actionKey = 'put'

  // 順序：先匹配具體子資源，避免 /animals/X/observations 之類路徑誤落到 /animals
  let resourceKey = 'default'
  if (path.includes('/observations')) resourceKey = 'observations'
  else if (path.includes('/surgeries')) resourceKey = 'surgeries'
  else if (path.includes('/weights')) resourceKey = 'weights'
  else if (path.includes('/vaccinations')) resourceKey = 'vaccinations'
  else if (path.includes('/animals')) resourceKey = 'animals'
  else if (path.includes('/protocols')) resourceKey = 'protocols'
  else if (path.includes('/leaves')) resourceKey = 'leaves'
  else if (path.includes('/overtime')) resourceKey = 'overtime'
  else if (path.includes('/products')) resourceKey = 'products'
  else if (path.includes('/documents')) resourceKey = 'documents'

  return {
    title: i18n.t('guest.write.title'),
    description: i18n.t('guest.write.description', {
      action: i18n.t(`guest.write.actions.${actionKey}`),
      resource: i18n.t(`guest.write.resources.${resourceKey}`),
    }),
  }
}
