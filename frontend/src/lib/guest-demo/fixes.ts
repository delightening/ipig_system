/**
 * Guest Demo Mode — crash / 空白 / 資料不一致 修補批次
 *
 * 這些端點多數期望「plain array」或「特定形狀物件」，但未匹配時 interceptor
 * 會落到通用 EMPTY_PAGINATED（物件），造成頁面對物件 .filter/.map/for...of 崩潰，
 * 或 widget 拿到錯形狀而空白。本檔集中補上正確形狀的虛構 demo data。
 *
 * 全部為虛構示範資料，沿用專案既有 guest demo 風格
 * （`範例…` 名稱、`demo-` 前綴 id、耳號 D-001、AUP-2025-001 / IACUC-2025-001 編號、
 *  *@example.com、192.168.x / 203.0.113.x 測試網段、2025~2026 年日期）。
 */
import type { CommitteeMember, DepartmentMember } from '@/types/facility'
import type { IpBlocklistEntry } from '@/lib/api/ipBlocklist'
import type {
  ReservationPlanningGroup,
  ReservableAnimalRow,
} from '@/lib/api/reservationPlanning'
import type { LeaveCalendarResponse, StaffInfo } from '@/types/hr'
import type { UnassignedInventoryItem } from '@/types/erp'

// ============================================================
// A1 組織圖 — GET /facilities/department-members（DepartmentMember[]，plain array）
//   department_id 對齊 misc.ts 的 DEMO_DEPARTMENTS（RD/VET/QA/ADM）；
//   成員 id / display_name / email 對齊 admin.ts 的 DEMO_USERS。
// ============================================================

export const DEMO_DEPARTMENT_MEMBERS: DepartmentMember[] = [
  // 研究部（RD）
  // 研究員刻意設為外部人員：PI 是外部角色，也順便讓 demo 呈現得出
  // 「部門裡可以有外部成員」這個真實情境（IACUC 外聘委員即為此類）。
  { id: 'demo-pi', display_name: '範例研究員', email: 'pi@example.com', department_id: 'demo-dept-research', is_internal: false },
  { id: 'demo-staff', display_name: '範例實驗員', email: 'staff@example.com', department_id: 'demo-dept-research', is_internal: true },
  // 獸醫部（VET）
  { id: 'demo-vet', display_name: '範例獸醫', email: 'vet@example.com', department_id: 'demo-dept-vet', is_internal: true },
  // 品保部（QA）
  { id: 'demo-admin', display_name: '範例管理員', email: 'admin@example.com', department_id: 'demo-dept-qa', is_internal: true },
  // 行政部（ADM）
  { id: 'demo-admin', display_name: '範例管理員', email: 'admin@example.com', department_id: 'demo-dept-admin', is_internal: true },
]

// ============================================================
// A2 IP 黑名單 — GET /admin/audit/ip-blocklist（IpBlocklistEntry[]，plain array）
// ============================================================

export const DEMO_IP_BLOCKLIST: IpBlocklistEntry[] = [
  {
    id: 'demo-ipb-1', ip_address: '203.0.113.45',
    reason: '暴力登入嘗試（5 分鐘內登入失敗 5 次）', source: 'honeypot',
    alert_id: 'demo-alert-1',
    blocked_at: '2026-04-02T03:16:00Z', blocked_until: '2026-04-03T03:16:00Z',
    blocked_by: 'demo-admin', hit_count: 12, last_hit_at: '2026-04-02T03:20:00Z',
    unblocked_at: null, unblocked_by: null, unblocked_reason: null,
  },
  {
    id: 'demo-ipb-2', ip_address: '203.0.113.88',
    reason: 'IDOR 探測：連續存取未授權資源', source: 'R22-6_idor',
    alert_id: null,
    blocked_at: '2026-03-28T22:05:00Z', blocked_until: null,
    blocked_by: null, hit_count: 5, last_hit_at: '2026-03-28T22:07:00Z',
    unblocked_at: null, unblocked_by: null, unblocked_reason: null,
  },
  {
    id: 'demo-ipb-3', ip_address: '192.168.50.77',
    reason: '地理圍籬 403 誤觸自動封鎖', source: 'geofence_autoblock',
    alert_id: null,
    blocked_at: '2026-03-20T09:30:00Z', blocked_until: '2026-03-20T10:30:00Z',
    blocked_by: null, hit_count: 1, last_hit_at: '2026-03-20T09:30:00Z',
    unblocked_at: '2026-03-20T09:45:00Z', unblocked_by: 'demo-admin',
    unblocked_reason: '確認為員工室內 GPS 飄移誤判，已解除封鎖',
  },
]

// ============================================================
// A3 預約與試驗規劃
//   GET /reservation-planning（ReservationPlanningGroup[]，plain array）
//   GET /animals/reservable（ReservableAnimalRow[]，plain array）
// animals[] id / 耳號對齊 animals.ts 的 DEMO_ANIMALS（demo-a1~a5、D-001~D-005）。
// ============================================================

export const DEMO_RESERVATION_PLANNING: ReservationPlanningGroup[] = [
  {
    group_type: 'approved',
    id: 'demo-p1', iacuc_no: 'IACUC-2025-001', unit: '範例研究單位',
    pi_name: '範例研究員', description: '心血管藥物安全性評估',
    demand: 4, reserved_count: 1, in_experiment_count: 2, completed_count: 0,
    animals: [
      {
        id: 'demo-a1', ear_tag: 'D-001', animal_no: null, gender: 'female',
        birth_date: '2025-06-15', latest_weight_kg: '28.5', weight_measured_at: '2026-03-28',
        remark: '主受試動物，性情溫馴', status: 'in_experiment', relation: 'in_experiment',
      },
      {
        id: 'demo-a2', ear_tag: 'D-002', animal_no: null, gender: 'male',
        birth_date: '2025-05-15', latest_weight_kg: '32.1', weight_measured_at: '2026-03-28',
        remark: '同窩 D-001，長期投藥中', status: 'in_experiment', relation: 'in_experiment',
      },
      {
        id: 'demo-a3', ear_tag: 'D-003', animal_no: null, gender: 'female',
        birth_date: '2025-09-10', latest_weight_kg: '28.0', weight_measured_at: '2026-03-25',
        remark: '已預約，待分配至試驗', status: 'unassigned', relation: 'reserved',
      },
    ],
  },
  {
    group_type: 'orphan',
    id: 'demo-orphan-1', iacuc_no: null, unit: '範例待分配',
    pi_name: null, description: '已預約但尚未對應計畫的動物',
    demand: 0, reserved_count: 1, in_experiment_count: 0, completed_count: 0,
    animals: [
      {
        id: 'demo-a5', ear_tag: 'D-005', animal_no: null, gender: 'female',
        birth_date: '2025-09-25', latest_weight_kg: '22.8', weight_measured_at: '2026-03-30',
        remark: '預約中，計畫待確認', status: 'unassigned', relation: 'reserved',
      },
    ],
  },
]

export const DEMO_RESERVABLE_ANIMALS: ReservableAnimalRow[] = [
  {
    id: 'demo-a3', ear_tag: 'D-003', animal_no: null, gender: 'female',
    birth_date: '2025-09-10', age_months: 6, pen_location: 'B-03',
    latest_weight_kg: '28.0', weight_measured_at: '2026-03-25',
    remark: '尚未指派計畫，隔離觀察中',
  },
  {
    id: 'demo-ap-1', ear_tag: 'A-101', animal_no: 'AP-2026-001', gender: 'male',
    birth_date: '2024-01-15', age_months: 28, pen_location: 'A-03',
    latest_weight_kg: '32.5', weight_measured_at: '2026-05-08',
    remark: null,
  },
  {
    id: 'demo-ap-2', ear_tag: 'A-102', animal_no: 'AP-2026-002', gender: 'female',
    birth_date: '2024-02-20', age_months: 27, pen_location: 'A-03',
    latest_weight_kg: '28.7', weight_measured_at: '2026-05-08',
    remark: null,
  },
  {
    id: 'demo-ap-7', ear_tag: 'C-401', animal_no: 'AP-2026-007', gender: 'male',
    birth_date: '2024-06-18', age_months: 23, pen_location: 'C-05',
    latest_weight_kg: '38.5', weight_measured_at: '2026-05-11',
    remark: null,
  },
]

// ============================================================
// A4 站內信收件人 — GET /messages/recipients（StaffOption[]：{ id, display_name, email }）
// StaffOption 結構與 StaffInfo 相同，直接沿用。對齊 admin.ts 的 DEMO_USERS。
// ============================================================

export const DEMO_MESSAGE_RECIPIENTS: StaffInfo[] = [
  { id: 'demo-admin', display_name: '範例管理員', email: 'admin@example.com' },
  { id: 'demo-vet', display_name: '範例獸醫', email: 'vet@example.com' },
  { id: 'demo-staff', display_name: '範例實驗員', email: 'staff@example.com' },
  { id: 'demo-pi', display_name: '範例研究員', email: 'pi@example.com' },
]

// ============================================================
// A4 巡場報告詳情 — GET /vet-patrol-reports/:id（PatrolReport 物件）
//   照片子路徑 GET /vet-patrol-reports/:id/photos（陣列）
// 對齊 misc.ts 的 DEMO_VET_PATROL_REPORTS（demo-vpr-1~4，patrol_date / status）。
// 照片一律留空，避免 guest 模式 img src 404 顯示破圖。
// ============================================================

interface DemoPatrolEntry {
  id: string
  category: 'pig_condition' | 'epidemic_prevention' | 'case_record' | 'other'
  ear_tags: string[]
  observation: string
  suggestion: string
  follow_up: string
  sort_order: number
}

interface DemoPatrolReportDetail {
  id: string
  patrol_date: string
  accompanying_personnel: string | null
  status: 'draft' | 'awaiting_acknowledgement' | 'awaiting_follow_up' | 'completed'
  entries: DemoPatrolEntry[]
  photos: unknown[]
  entry_photos: unknown[]
}

export const DEMO_VET_PATROL_REPORT_DETAILS: Record<string, DemoPatrolReportDetail> = {
  'demo-vpr-1': {
    id: 'demo-vpr-1', patrol_date: '2026-03-30',
    accompanying_personnel: '範例實驗員 A、範例技術員', status: 'completed',
    entries: [
      {
        id: 'demo-vpe-1a', category: 'pig_condition', ear_tags: ['D-001'],
        observation: '左眼術後恢復良好，傷口癒合，無紅腫滲液',
        suggestion: '持續每日觀察 3 天，如有異常立即回報',
        follow_up: '已連續觀察 3 日，狀況穩定，恢復正常活動', sort_order: 0,
      },
      {
        id: 'demo-vpe-1b', category: 'epidemic_prevention', ear_tags: [],
        observation: '全場定期清洗消毒（每週一次，週三）',
        suggestion: '維持現行消毒頻率',
        follow_up: '已完成本週清消', sort_order: 1,
      },
      {
        id: 'demo-vpe-1c', category: 'other', ear_tags: [],
        observation: '動物房溫度 24°C、濕度 60%，帆布狀況良好',
        suggestion: '', follow_up: '', sort_order: 2,
      },
    ],
    photos: [], entry_photos: [],
  },
  'demo-vpr-2': {
    id: 'demo-vpr-2', patrol_date: '2026-03-23',
    accompanying_personnel: '範例實驗員 A', status: 'awaiting_follow_up',
    entries: [
      {
        id: 'demo-vpe-2a', category: 'pig_condition', ear_tags: ['D-002'],
        observation: '輕微跛行（左後肢），疑似昨日採血保定所致',
        suggestion: '休養觀察，避免劇烈保定，必要時投予止痛',
        follow_up: '', sort_order: 0,
      },
    ],
    photos: [], entry_photos: [],
  },
  'demo-vpr-3': {
    id: 'demo-vpr-3', patrol_date: '2026-03-16',
    accompanying_personnel: null, status: 'awaiting_acknowledgement',
    entries: [
      {
        id: 'demo-vpe-3a', category: 'case_record', ear_tags: ['D-001'],
        observation: '例行病歷檢視，無異常',
        suggestion: '', follow_up: '', sort_order: 0,
      },
    ],
    photos: [], entry_photos: [],
  },
  'demo-vpr-4': {
    id: 'demo-vpr-4', patrol_date: '2026-04-06',
    accompanying_personnel: null, status: 'draft',
    entries: [
      {
        id: 'demo-vpe-4a', category: 'pig_condition', ear_tags: [],
        observation: '（草稿）巡場中，尚未完成填寫',
        suggestion: '', follow_up: '', sort_order: 0,
      },
    ],
    photos: [], entry_photos: [],
  },
}

// ============================================================
// B1 首頁日曆 widget — GET /hr/dashboard/calendar
//   CalendarData 物件 { today_leaves, today_events, upcoming_leaves }
// （與 /hr/calendar/events 的 CalendarEvent[] 形狀不同，須各自獨立。）
// ============================================================

interface DemoCalendarData {
  today_leaves: Array<{
    user_name: string
    leave_type: string
    start_time: string
    end_time: string
    is_all_day: boolean
  }>
  today_events: Array<{
    id: string
    summary: string
    start: string
    end: string
    all_day: boolean
    location?: string
  }>
  upcoming_leaves: Array<{
    user_name: string
    start_date: string
    end_date: string
  }>
}

const todayDate = () => new Date().toISOString().slice(0, 10)
const todayAt = (hhmmss: string) => `${todayDate()}T${hhmmss}Z`
const daysFromNow = (n: number) =>
  new Date(Date.now() + n * 86400000).toISOString().slice(0, 10)

export const DEMO_DASHBOARD_CALENDAR: DemoCalendarData = {
  today_leaves: [
    {
      user_name: '範例實驗員', leave_type: 'PERSONAL',
      start_time: todayAt('13:00:00'), end_time: todayAt('17:00:00'), is_all_day: false,
    },
    {
      user_name: '範例技術員', leave_type: 'SICK',
      start_time: todayAt('00:00:00'), end_time: todayAt('23:59:00'), is_all_day: true,
    },
  ],
  today_events: [
    {
      id: 'demo-cev-1', summary: '範例：IACUC 月例會議',
      start: todayAt('09:00:00'), end: todayAt('10:30:00'), all_day: false, location: '會議室 A',
    },
    {
      id: 'demo-cev-2', summary: '範例：動物房環境週檢',
      start: todayAt('14:00:00'), end: todayAt('15:00:00'), all_day: false,
    },
  ],
  upcoming_leaves: [
    { user_name: '範例獸醫', start_date: daysFromNow(3), end_date: daysFromNow(4) },
    { user_name: '範例管理員', start_date: daysFromNow(7), end_date: daysFromNow(7) },
  ],
}

// ============================================================
// B1-2 請假行事曆 — GET /hr/leaves/calendar
//   { entries, scope }，與上面三個 Google 端點形狀皆不同。
//   刻意含一筆「審核中且代理人未確認」，讓 demo 也看得到虛線框與「尚未確認」標示。
// ============================================================

export const DEMO_LEAVE_CALENDAR: LeaveCalendarResponse = {
  scope: 'all',
  entries: [
    {
      id: 'demo-lc-1',
      user_id: 'demo-staff',
      user_name: '範例實驗員',
      department_name: '試驗部',
      leave_type: 'ANNUAL',
      leave_type_display: '特休假',
      status: 'APPROVED',
      pending_stage_display: null,
      start_date: daysFromNow(2),
      end_date: daysFromNow(3),
      start_time: null,
      end_time: null,
      is_all_day: true,
      is_partial_day: false,
      total_days: '2.00',
      total_hours: null,
      proxy_user_id: 'demo-admin',
      proxy_user_name: '範例管理員',
      proxy_confirmed: true,
      is_mine: false,
      is_my_proxy_duty: false,
    },
    {
      id: 'demo-lc-2',
      user_id: 'demo-vet',
      user_name: '範例獸醫',
      department_name: '試驗部',
      leave_type: 'SICK',
      leave_type_display: '病假',
      status: 'PENDING_PROXY',
      pending_stage_display: '待代理確認',
      start_date: daysFromNow(5),
      end_date: daysFromNow(5),
      start_time: '09:00:00',
      end_time: '13:00:00',
      is_all_day: false,
      is_partial_day: true,
      total_days: '0.50',
      total_hours: '4.00',
      proxy_user_id: 'demo-staff',
      proxy_user_name: '範例實驗員',
      proxy_confirmed: false,
      is_mine: false,
      is_my_proxy_duty: true,
    },
  ],
}

// ============================================================
// IACUC 委員名冊 — GET /facilities/committee-members
//   依角色列出，與部門編制無關。刻意讓「範例獸醫」同時屬於獸醫部又在名冊裡，
//   demo 才呈現得出「委員會是跨部門橫切面」這件事。
// ============================================================

export const DEMO_COMMITTEE_MEMBERS: CommitteeMember[] = [
  { id: 'demo-vet', display_name: '範例獸醫', department_name: '獸醫部', is_internal: true },
  { id: 'demo-admin', display_name: '範例管理員', department_name: '行政部', is_internal: true },
  { id: 'demo-reviewer', display_name: '範例外聘審查委員', department_name: null, is_internal: false },
]

// ============================================================
// B2 員工清單 — GET /hr/staff（StaffInfo[]，plain array）；對齊 DEMO_USERS。
// ============================================================

export const DEMO_HR_STAFF: StaffInfo[] = [
  { id: 'demo-admin', display_name: '範例管理員', email: 'admin@example.com' },
  { id: 'demo-vet', display_name: '範例獸醫', email: 'vet@example.com' },
  { id: 'demo-staff', display_name: '範例實驗員', email: 'staff@example.com' },
  { id: 'demo-pi', display_name: '範例研究員', email: 'pi@example.com' },
]

// ============================================================
// B2 未分配庫存 — GET /inventory/unassigned（UnassignedInventoryItem[]，plain array）
// 產品 / 倉庫命名對齊 misc.ts 的 DEMO_INVENTORY_ON_HAND（W001 範例主倉庫、範例藥品 A 等）。
// ============================================================

export const DEMO_UNASSIGNED_INVENTORY: UnassignedInventoryItem[] = [
  {
    warehouse_id: 'demo-w001', warehouse_name: '範例主倉庫',
    product_id: 'demo-prod1', product_sku: 'MED-001', product_name: '範例藥品 A',
    base_uom: '盒', qty_on_warehouse: '28', qty_on_shelves: '20', qty_unassigned: '8',
  },
  {
    warehouse_id: 'demo-w001', warehouse_name: '範例主倉庫',
    product_id: 'demo-prod2', product_sku: 'SUP-001', product_name: '範例耗材 B',
    base_uom: '瓶', qty_on_warehouse: '30', qty_on_shelves: '22', qty_unassigned: '8',
  },
  {
    warehouse_id: 'demo-w001', warehouse_name: '範例主倉庫',
    product_id: 'demo-prod3', product_sku: 'FED-001', product_name: '範例飼料 C',
    base_uom: '袋', qty_on_warehouse: '38', qty_on_shelves: '38', qty_unassigned: '0',
  },
]

// ============================================================
// B2 設備廠商彙總 — GET /equipment-suppliers/summary
//   plain array，每列 { equipment_id, partner_name }。
// equipment_id 對齊 misc.ts 維護紀錄用的 demo-eq1~eq4。
// ============================================================

interface DemoSupplierSummaryRow {
  equipment_id: string
  partner_name: string
}

export const DEMO_EQUIPMENT_SUPPLIER_SUMMARY: DemoSupplierSummaryRow[] = [
  { equipment_id: 'demo-eq1', partner_name: '範例維修廠商' },
  { equipment_id: 'demo-eq2', partner_name: '範例維修廠商' },
  { equipment_id: 'demo-eq2', partner_name: '範例儀器供應商' },
  { equipment_id: 'demo-eq4', partner_name: '範例維修廠商' },
]
