/**
 * Guest Demo Mode — 其餘可見頁面靜態範例資料（batch 3）
 *
 * 涵蓋 batch 1（GLP）、batch 2（報表）以外，guest 可見但先前空白的頁面：
 *   動物：巡場報告、血檢項目 / 組合 / 常用組合
 *   HR：特休管理（內部人員 / 特休額度 / 過期待補償）
 *   ERP / 庫存：庫存查詢、庫存流水、設備維護明細
 *   QAU：稽查排程
 *   Admin：藥物選單、設施管理（物種 / 設施 / 部門）、安全審計（登入 / Sessions / 警報 / 活動）
 *   AUP：我的變更申請
 *
 * 全部為虛構示範資料，沿用專案既有 guest demo 風格
 * （`範例…` 名稱、`demo-` 前綴 UUID、耳號 D-001、範例主倉庫 W001、範例藥品 A、
 *  AUP-2025-001 / IACUC-2025-001 編號、*@example.com、2025~2026 年日期）。
 */
import type { PaginatedResponse } from '@/types/common'
import type {
  BloodTestTemplate,
  BloodTestPanel,
  BloodTestPreset,
} from '@/types/animal'
import type {
  InventoryOnHand,
  StockLedgerDetail,
} from '@/types/erp'
import type {
  AnnualLeaveBalanceView,
  ExpiredLeaveReport,
  LoginEventWithUser,
  SessionWithUser,
  SecurityAlert,
} from '@/types/hr'
import type {
  Species,
  Facility,
  DepartmentWithManager,
} from '@/types/facility'
import type { TreatmentDrugOption } from '@/types/treatment-drug'
import type { QaAuditSchedule } from '@/lib/api/qaPlan'
import type { AuditLog } from '@/pages/admin/types/audit'
import type { MaintenanceRecordWithDetails } from '@/pages/admin/types'
import type { AmendmentListItem } from '@/types/amendment'

// ============================================================
// 巡場報告 — GET /vet-patrol-reports（ReportRow[]，plain array）
// ReportRow 型別定義於頁面內（非匯出），此處以等價本地介面表示。
// ============================================================

interface DemoVetPatrolReport {
  id: string
  patrol_date: string
  accompanying_personnel: string | null
  status: 'draft' | 'awaiting_acknowledgement' | 'awaiting_follow_up' | 'completed'
  created_by: string | null
  follow_up_user_id: string | null
  created_at: string
  submitted_at?: string | null
  updated_at: string
  last_message_at?: string
}

export const DEMO_VET_PATROL_REPORTS: DemoVetPatrolReport[] = [
  {
    id: 'demo-vpr-1', patrol_date: '2026-03-30',
    accompanying_personnel: '範例實驗員 A、範例技術員',
    status: 'completed', created_by: 'demo-vet', follow_up_user_id: null,
    created_at: '2026-03-30T09:00:00Z', submitted_at: '2026-03-30T11:00:00Z',
    updated_at: '2026-04-01T10:00:00Z', last_message_at: '2026-04-01T10:00:00Z',
  },
  {
    id: 'demo-vpr-2', patrol_date: '2026-03-23',
    accompanying_personnel: '範例實驗員 A',
    status: 'awaiting_follow_up', created_by: 'demo-vet', follow_up_user_id: 'demo-staff',
    created_at: '2026-03-23T09:30:00Z', submitted_at: '2026-03-23T10:30:00Z',
    updated_at: '2026-03-24T08:00:00Z', last_message_at: '2026-03-24T08:00:00Z',
  },
  {
    id: 'demo-vpr-3', patrol_date: '2026-03-16',
    accompanying_personnel: null,
    status: 'awaiting_acknowledgement', created_by: 'demo-vet', follow_up_user_id: null,
    created_at: '2026-03-16T09:00:00Z', submitted_at: '2026-03-16T09:45:00Z',
    updated_at: '2026-03-16T09:45:00Z',
  },
  {
    id: 'demo-vpr-4', patrol_date: '2026-04-06',
    accompanying_personnel: null,
    status: 'draft', created_by: 'demo-vet', follow_up_user_id: null,
    created_at: '2026-04-06T08:30:00Z', submitted_at: null,
    updated_at: '2026-04-06T08:30:00Z',
  },
]

// ============================================================
// 血檢項目 / 組合 / 常用組合
//   GET /blood-test-templates/all（BloodTestTemplate[]）
//   GET /blood-test-panels/all（BloodTestPanel[]，含 items 巢狀 template）
//   GET /blood-test-presets/all（BloodTestPreset[]）
// ============================================================

const bloodTestTemplate = (
  id: string, code: string, name: string,
  default_unit: string, reference_range: string,
  default_price: number, sort_order: number,
): BloodTestTemplate => ({
  id, code, name, default_unit, reference_range, default_price, sort_order,
  is_active: true, created_at: '2025-01-01T08:00:00Z', updated_at: '2025-06-01T08:00:00Z',
})

const TPL_WBC = bloodTestTemplate('demo-bt-wbc', 'WBC', '白血球 (WBC)', '10^3/µL', '6.0-16.0', 120, 1)
const TPL_RBC = bloodTestTemplate('demo-bt-rbc', 'RBC', '紅血球 (RBC)', '10^6/µL', '5.0-8.0', 120, 2)
const TPL_HGB = bloodTestTemplate('demo-bt-hgb', 'HGB', '血紅素 (HGB)', 'g/dL', '10.0-16.0', 100, 3)
const TPL_PLT = bloodTestTemplate('demo-bt-plt', 'PLT', '血小板 (PLT)', '10^3/µL', '150-450', 100, 4)
const TPL_ALT = bloodTestTemplate('demo-bt-alt', 'ALT', '丙胺酸轉胺酶 (ALT)', 'U/L', '20-80', 150, 5)
const TPL_AST = bloodTestTemplate('demo-bt-ast', 'AST', '天門冬胺酸轉胺酶 (AST)', 'U/L', '15-55', 150, 6)
const TPL_BUN = bloodTestTemplate('demo-bt-bun', 'BUN', '尿素氮 (BUN)', 'mg/dL', '8-24', 130, 7)
const TPL_CREA = bloodTestTemplate('demo-bt-crea', 'CREA', '肌酸酐 (CREA)', 'mg/dL', '0.8-2.3', 130, 8)

export const DEMO_BLOOD_TEST_TEMPLATES: BloodTestTemplate[] = [
  TPL_WBC, TPL_RBC, TPL_HGB, TPL_PLT, TPL_ALT, TPL_AST, TPL_BUN, TPL_CREA,
]

export const DEMO_BLOOD_TEST_PANELS: BloodTestPanel[] = [
  {
    id: 'demo-btp-cbc', key: 'cbc', name: '全血球計數 (CBC)', icon: 'droplet',
    sort_order: 1, is_active: true,
    items: [TPL_WBC, TPL_RBC, TPL_HGB, TPL_PLT],
    created_at: '2025-01-01T08:00:00Z', updated_at: '2025-06-01T08:00:00Z',
  },
  {
    id: 'demo-btp-lft', key: 'lft', name: '肝功能 (LFT)', icon: 'activity',
    sort_order: 2, is_active: true,
    items: [TPL_ALT, TPL_AST],
    created_at: '2025-01-01T08:00:00Z', updated_at: '2025-06-01T08:00:00Z',
  },
  {
    id: 'demo-btp-rft', key: 'rft', name: '腎功能 (RFT)', icon: 'filter',
    sort_order: 3, is_active: true,
    items: [TPL_BUN, TPL_CREA],
    created_at: '2025-01-01T08:00:00Z', updated_at: '2025-06-01T08:00:00Z',
  },
]

export const DEMO_BLOOD_TEST_PRESETS: BloodTestPreset[] = [
  {
    id: 'demo-btpr-preop', name: '術前常規', icon: 'clipboard-check',
    panel_keys: ['cbc', 'lft'], sort_order: 1, is_active: true,
    created_at: '2025-01-01T08:00:00Z', updated_at: '2025-06-01T08:00:00Z',
  },
  {
    id: 'demo-btpr-organ', name: '肝腎功能追蹤', icon: 'stethoscope',
    panel_keys: ['lft', 'rft'], sort_order: 2, is_active: true,
    created_at: '2025-01-01T08:00:00Z', updated_at: '2025-06-01T08:00:00Z',
  },
  {
    id: 'demo-btpr-full', name: '完整體檢', icon: 'list-checks',
    panel_keys: ['cbc', 'lft', 'rft'], sort_order: 3, is_active: true,
    created_at: '2025-01-01T08:00:00Z', updated_at: '2025-06-01T08:00:00Z',
  },
]

// ============================================================
// 特休管理 — HrAnnualLeavePage
//   GET /hr/internal-users（內部人員 plain array）
//   GET /hr/balances/annual?user_id=…（AnnualLeaveBalanceView[]，選人後才觸發）
//   GET /hr/balances/expired-compensation（ExpiredLeaveReport[]）
// ============================================================

interface DemoInternalUser {
  id: string
  email: string
  display_name: string
  is_active: boolean
  is_internal: boolean
}

// 涵蓋所有訓練紀錄出現過的 user_id（demo-vet / demo-staff / demo-pi / demo-admin），
// 讓「人員訓練」篩選下拉的人 = 紀錄裡的人，避免選了人卻無紀錄的資料不一致。
export const DEMO_HR_INTERNAL_USERS: DemoInternalUser[] = [
  { id: 'demo-vet', email: 'vet@example.com', display_name: '範例獸醫', is_active: true, is_internal: true },
  { id: 'demo-staff', email: 'staff@example.com', display_name: '範例實驗員', is_active: true, is_internal: true },
  { id: 'demo-pi', email: 'pi@example.com', display_name: '範例研究員', is_active: true, is_internal: true },
  { id: 'demo-admin', email: 'admin@example.com', display_name: '範例管理員', is_active: true, is_internal: true },
]

export const DEMO_ANNUAL_LEAVE_BALANCE: AnnualLeaveBalanceView[] = [
  {
    entitlement_year: 2026, entitled_days: 14, used_days: 5, remaining_days: 9,
    expires_at: '2027-03-31T00:00:00Z', days_until_expiry: 300, is_expired: false,
  },
  {
    entitlement_year: 2025, entitled_days: 10, used_days: 8, remaining_days: 2,
    expires_at: '2026-03-31T00:00:00Z', days_until_expiry: 20, is_expired: false,
  },
]

export const DEMO_EXPIRED_COMPENSATION: ExpiredLeaveReport[] = [
  {
    user_id: 'demo-staff', user_name: '範例實驗員', user_email: 'staff@example.com',
    entitlement_year: 2024, entitled_days: 10, used_days: 8, remaining_days: 2,
    expires_at: '2025-12-31T00:00:00Z',
  },
]

// ============================================================
// 庫存查詢 — GET /inventory/on-hand（InventoryOnHand[]，plain array）
// 產品 / 倉庫 / 儲位命名對齊 ./erp（W001 範例主倉庫、範例藥品 A 等）。
// ============================================================

export const DEMO_INVENTORY_ON_HAND: InventoryOnHand[] = [
  {
    warehouse_id: 'demo-w001', warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    storage_location_id: 'demo-loc-shelf-fa', storage_location_code: 'F-A', storage_location_name: '飼料架A',
    product_id: 'demo-prod3', product_sku: 'FED-001', product_name: '範例飼料 C',
    base_uom: '袋', qty_on_hand: '30', avg_cost: '850.00',
    batch_no: 'LOT-2026-001', expiry_date: '2027-01-31',
    safety_stock: '20', reorder_point: '50', last_updated_at: '2026-04-01T08:00:00Z',
  },
  {
    warehouse_id: 'demo-w001', warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    storage_location_id: 'demo-loc-shelf-fa', storage_location_code: 'F-A', storage_location_name: '飼料架A',
    product_id: 'demo-prod3', product_sku: 'FED-001', product_name: '範例飼料 C',
    base_uom: '袋', qty_on_hand: '8', avg_cost: '860.00',
    batch_no: 'LOT-2026-002', expiry_date: '2027-03-31',
    safety_stock: '20', reorder_point: '50', last_updated_at: '2026-04-10T08:00:00Z',
  },
  {
    warehouse_id: 'demo-w001', warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    storage_location_id: 'demo-loc-rack-m1', storage_location_code: 'M-1', storage_location_name: '藥品架1',
    product_id: 'demo-prod1', product_sku: 'MED-001', product_name: '範例藥品 A',
    base_uom: '盒', qty_on_hand: '8', avg_cost: '500.00',
    batch_no: 'MED-LOT-2025-05', expiry_date: '2026-07-31',
    safety_stock: '10', reorder_point: '20', last_updated_at: '2026-03-20T08:00:00Z',
  },
  {
    warehouse_id: 'demo-w001', warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    storage_location_id: 'demo-loc-rack-m1', storage_location_code: 'M-1', storage_location_name: '藥品架1',
    product_id: 'demo-prod1', product_sku: 'MED-001', product_name: '範例藥品 A',
    base_uom: '盒', qty_on_hand: '20', avg_cost: '500.00',
    batch_no: 'MED-LOT-2026-01', expiry_date: '2027-06-30',
    safety_stock: '10', reorder_point: '20', last_updated_at: '2026-04-05T08:00:00Z',
  },
  {
    warehouse_id: 'demo-w001', warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    storage_location_id: 'demo-loc-shelf-sa', storage_location_code: 'S-A', storage_location_name: '耗材架A',
    product_id: 'demo-prod2', product_sku: 'SUP-001', product_name: '範例耗材 B',
    base_uom: '瓶', qty_on_hand: '22', avg_cost: '120.00',
    expiry_date: '2027-12-31',
    safety_stock: '5', reorder_point: '15', last_updated_at: '2026-04-10T08:00:00Z',
  },
]

// ============================================================
// 庫存流水 — GET /inventory/ledger（StockLedgerDetail[]，plain array）
// ============================================================

export const DEMO_INVENTORY_LEDGER: StockLedgerDetail[] = [
  {
    id: 'demo-sld-1', warehouse_id: 'demo-w001', warehouse_name: '範例主倉庫',
    product_id: 'demo-prod3', product_sku: 'FED-001', product_name: '範例飼料 C',
    trx_date: '2026-01-06T09:12:00Z', doc_type: 'GRN', doc_id: 'demo-doc2', doc_no: 'GRN-2026-0001',
    direction: 'in', qty_base: '30', unit_cost: '850.00',
    batch_no: 'LOT-2026-001', expiry_date: '2027-01-31',
  },
  {
    id: 'demo-sld-2', warehouse_id: 'demo-w001', warehouse_name: '範例主倉庫',
    product_id: 'demo-prod1', product_sku: 'MED-001', product_name: '範例藥品 A',
    trx_date: '2026-01-08T10:05:00Z', doc_type: 'GRN', doc_id: 'demo-doc4', doc_no: 'GRN-2026-0002',
    direction: 'in', qty_base: '20', unit_cost: '500.00',
    batch_no: 'MED-LOT-2026-01', expiry_date: '2027-06-30',
  },
  {
    id: 'demo-sld-3', warehouse_id: 'demo-w001', warehouse_name: '範例主倉庫',
    product_id: 'demo-prod2', product_sku: 'SUP-001', product_name: '範例耗材 B',
    trx_date: '2026-02-03T11:40:00Z', doc_type: 'SO', doc_id: 'demo-doc5', doc_no: 'SO-2026-0001',
    direction: 'out', qty_base: '6', unit_cost: '120.00',
    iacuc_no: 'IACUC-2025-001',
  },
  {
    id: 'demo-sld-4', warehouse_id: 'demo-w001', warehouse_name: '範例主倉庫',
    product_id: 'demo-prod3', product_sku: 'FED-001', product_name: '範例飼料 C',
    trx_date: '2026-02-20T16:00:00Z', doc_type: 'SO', doc_id: 'demo-doc6', doc_no: 'SO-2026-0002',
    direction: 'out', qty_base: '12', unit_cost: '850.00', batch_no: 'LOT-2026-001',
  },
  {
    id: 'demo-sld-5', warehouse_id: 'demo-w001', warehouse_name: '範例主倉庫',
    product_id: 'demo-prod1', product_sku: 'MED-001', product_name: '範例藥品 A',
    trx_date: '2026-03-05T08:55:00Z', doc_type: 'ADJ', doc_id: 'demo-doc7', doc_no: 'ADJ-2026-0001',
    direction: 'adjust_out', qty_base: '2', unit_cost: '500.00', batch_no: 'MED-LOT-2025-05',
  },
]

// ============================================================
// 設備維護明細 — GET /equipment-maintenance
//   PaginatedResponse<MaintenanceRecordWithDetails>
// 設備命名對齊 ./equipment（範例氣體麻醉機 EQ-002 等）。
// ============================================================

export const DEMO_MAINTENANCE_RECORDS: PaginatedResponse<MaintenanceRecordWithDetails> = {
  data: [
    {
      id: 'demo-mnt-1', equipment_id: 'demo-eq2', equipment_name: '範例氣體麻醉機',
      maintenance_type: 'repair', status: 'completed',
      reported_at: '2026-02-10T09:00:00Z', completed_at: '2026-02-14T16:00:00Z',
      problem_description: 'SpO₂ 探頭讀值不穩，偶有掉訊',
      repair_content: '更換 SpO₂ 探頭與延長線，校正後讀值恢復正常',
      repair_partner_id: 'demo-partner1', repair_partner_name: '範例維修廠商',
      maintenance_items: 'SpO₂ 探頭 ×1、延長線 ×1', performed_by: '範例維修工程師',
      notes: '已回歸使用', created_by: 'demo-vet',
      reviewed_by: 'demo-admin', reviewer_name: '範例主管',
      reviewed_at: '2026-02-15T09:00:00Z', review_notes: '維修完成，同意結案',
      created_at: '2026-02-10T09:00:00Z',
    },
    {
      id: 'demo-mnt-2', equipment_id: 'demo-eq1', equipment_name: '範例電子秤',
      maintenance_type: 'maintenance', status: 'in_progress',
      reported_at: '2026-03-28T10:00:00Z', completed_at: null,
      problem_description: '定期保養：清潔秤盤、檢查水平',
      repair_content: null,
      repair_partner_id: null, repair_partner_name: null,
      maintenance_items: null, performed_by: '範例內部技術員',
      notes: '例行季度保養', created_by: 'demo-staff',
      reviewed_by: null, reviewer_name: null,
      reviewed_at: null, review_notes: null,
      created_at: '2026-03-28T10:00:00Z',
    },
    {
      id: 'demo-mnt-3', equipment_id: 'demo-eq3', equipment_name: '範例低溫冷凍櫃',
      maintenance_type: 'repair', status: 'pending',
      reported_at: '2026-04-05T14:30:00Z', completed_at: null,
      problem_description: '溫度上升至 -60°C，疑似壓縮機異常',
      repair_content: null,
      repair_partner_id: null, repair_partner_name: null,
      maintenance_items: null, performed_by: null,
      notes: '已改由備用冷凍櫃暫存樣本', created_by: 'demo-vet',
      reviewed_by: null, reviewer_name: null,
      reviewed_at: null, review_notes: null,
      created_at: '2026-04-05T14:30:00Z',
    },
    {
      id: 'demo-mnt-4', equipment_id: 'demo-eq4', equipment_name: '範例動物磅秤',
      maintenance_type: 'repair', status: 'pending_review',
      reported_at: '2026-03-15T08:00:00Z', completed_at: '2026-03-18T11:00:00Z',
      problem_description: '歸零漂移，量測值不準',
      repair_content: '重新校正並更換荷重元，量測值恢復合格',
      repair_partner_id: 'demo-partner1', repair_partner_name: '範例維修廠商',
      maintenance_items: '荷重元 ×1', performed_by: '範例維修工程師',
      notes: '待主管覆核', created_by: 'demo-staff',
      reviewed_by: null, reviewer_name: null,
      reviewed_at: null, review_notes: null,
      created_at: '2026-03-15T08:00:00Z',
    },
  ],
  total: 4, page: 1, per_page: 20, total_pages: 1,
}

// ============================================================
// 稽查排程 — GET /qau/schedules
//   後端回 { data: QaAuditSchedule[] }，前端 listSchedules 取 res.data.data
// ============================================================

export const DEMO_QA_SCHEDULES: QaAuditSchedule[] = [
  {
    id: 'demo-qas-1', year: 2026, title: '2026 年度品保稽查計畫',
    schedule_type: 'annual', description: '涵蓋計畫書、設施、設備三大類年度稽查',
    status: 'in_progress', created_by: 'demo-admin',
    created_at: '2026-01-05T08:00:00Z', updated_at: '2026-03-01T08:00:00Z',
  },
  {
    id: 'demo-qas-2', year: 2026, title: '動物房環境季度稽查',
    schedule_type: 'periodic', description: '每季一次的環境溫濕度與清潔稽查',
    status: 'planned', created_by: 'demo-admin',
    created_at: '2026-01-10T08:00:00Z', updated_at: '2026-01-10T08:00:00Z',
  },
  {
    id: 'demo-qas-3', year: 2025, title: '2025 年度品保稽查計畫',
    schedule_type: 'annual', description: '2025 年度已完成之稽查計畫',
    status: 'completed', created_by: 'demo-admin',
    created_at: '2025-01-05T08:00:00Z', updated_at: '2025-12-20T08:00:00Z',
  },
]

// ============================================================
// 藥物選單 — GET /admin/treatment-drugs（TreatmentDrugOption[]，plain array）
// ============================================================

export const DEMO_TREATMENT_DRUGS: TreatmentDrugOption[] = [
  {
    id: 'demo-drug-1', name: 'Meloxicam', display_name: 'Meloxicam（美洛昔康）',
    default_dosage_unit: 'mg/kg', available_units: ['mg/kg', 'ml'], default_dosage_value: '0.4',
    erp_product_id: null, category: '止痛', sort_order: 1, is_active: true,
    created_by: 'demo-vet', created_at: '2025-01-01T08:00:00Z', updated_at: '2025-06-01T08:00:00Z',
  },
  {
    id: 'demo-drug-2', name: 'Cefazolin', display_name: 'Cefazolin（頭孢唑林）',
    default_dosage_unit: 'mg/kg', available_units: ['mg/kg'], default_dosage_value: '20',
    erp_product_id: 'demo-prod1', category: '抗生素', sort_order: 2, is_active: true,
    created_by: 'demo-vet', created_at: '2025-01-01T08:00:00Z', updated_at: '2025-06-01T08:00:00Z',
  },
  {
    id: 'demo-drug-3', name: 'Ivermectin', display_name: 'Ivermectin（伊維菌素）',
    default_dosage_unit: 'mg/kg', available_units: ['mg/kg'], default_dosage_value: '0.3',
    erp_product_id: null, category: '驅蟲', sort_order: 3, is_active: true,
    created_by: 'demo-vet', created_at: '2025-01-01T08:00:00Z', updated_at: '2025-06-01T08:00:00Z',
  },
  {
    id: 'demo-drug-4', name: 'Atropine', display_name: null,
    default_dosage_unit: 'ml/kg', available_units: ['ml/kg', 'mg'], default_dosage_value: '0.05',
    erp_product_id: null, category: '麻醉', sort_order: 4, is_active: true,
    created_by: 'demo-vet', created_at: '2025-01-01T08:00:00Z', updated_at: '2025-06-01T08:00:00Z',
  },
  {
    id: 'demo-drug-5', name: 'Ketamine', display_name: 'Ketamine（K 他命）',
    default_dosage_unit: 'mg/kg', available_units: ['mg/kg'], default_dosage_value: '10',
    erp_product_id: null, category: '麻醉', sort_order: 5, is_active: false,
    created_by: 'demo-vet', created_at: '2025-01-01T08:00:00Z', updated_at: '2025-08-01T08:00:00Z',
  },
]

// ============================================================
// 設施管理 — FacilitiesPage（各 tab plain array）
//   GET /facilities（Facility[]）
//   GET /facilities/species（Species[]）
//   GET /facilities/departments（DepartmentWithManager[]）
// 建物 / 區 / 欄已於 ./animals 提供；此處補物種 / 設施 / 部門。
// 設施 id / code 對齊 ./animals（demo-facility-1 / DEMO）。
// ============================================================

export const DEMO_FACILITIES: Facility[] = [
  {
    id: 'demo-facility-1', code: 'DEMO', name: '範例動物實驗設施',
    address: '範例市範例區範例路 1 號', phone: '02-1234-5678', contact_person: '範例設施管理員',
    is_active: true, config: null,
    created_at: '2024-01-01T08:00:00Z', updated_at: '2025-01-01T08:00:00Z',
  },
  {
    id: 'demo-facility-2', code: 'DEMO-Q', name: '範例檢疫隔離設施',
    address: '範例市範例區範例路 3 號', phone: '02-1234-5679', contact_person: '範例檢疫獸醫',
    is_active: true, config: null,
    created_at: '2024-06-01T08:00:00Z', updated_at: '2025-01-01T08:00:00Z',
  },
]

export const DEMO_SPECIES: Species[] = [
  {
    id: 'demo-species-minipig', code: 'MINIPIG', name: '迷你豬', name_en: 'Minipig',
    icon: 'pig', is_active: true, config: null, parent_id: null, sort_order: 1,
    created_at: '2024-01-01T08:00:00Z', updated_at: '2024-01-01T08:00:00Z',
  },
  {
    id: 'demo-species-white', code: 'WHITE', name: '白豬', name_en: 'White Pig',
    icon: 'pig', is_active: true, config: null, parent_id: null, sort_order: 2,
    created_at: '2024-01-01T08:00:00Z', updated_at: '2024-01-01T08:00:00Z',
  },
  {
    id: 'demo-species-lyd', code: 'LYD', name: 'LYD 三品種豬', name_en: 'LYD',
    icon: 'pig', is_active: true, config: null, parent_id: null, sort_order: 3,
    created_at: '2024-01-01T08:00:00Z', updated_at: '2024-01-01T08:00:00Z',
  },
]

export const DEMO_DEPARTMENTS: DepartmentWithManager[] = [
  {
    id: 'demo-dept-research', code: 'RD', name: '研究部',
    parent_id: null, parent_name: null,
    manager_id: 'demo-pi', manager_name: '範例研究員',
    is_active: true, sort_order: 1,
  },
  {
    id: 'demo-dept-vet', code: 'VET', name: '獸醫部',
    parent_id: null, parent_name: null,
    manager_id: 'demo-vet', manager_name: '範例獸醫',
    is_active: true, sort_order: 2,
  },
  {
    id: 'demo-dept-qa', code: 'QA', name: '品保部',
    parent_id: null, parent_name: null,
    manager_id: 'demo-admin', manager_name: '範例品保主管',
    is_active: true, sort_order: 3,
  },
  {
    id: 'demo-dept-admin', code: 'ADM', name: '行政部',
    parent_id: null, parent_name: null,
    manager_id: null, manager_name: null,
    is_active: true, sort_order: 4,
  },
]

// ============================================================
// 安全審計 — AdminAuditPage 子分頁
//   GET /admin/audit/logins（PaginatedResponse<LoginEventWithUser>）
//   GET /admin/audit/sessions（PaginatedResponse<SessionWithUser>）
//   GET /admin/audit/alerts（PaginatedResponse<SecurityAlert>）
//   GET /audit-logs（PaginatedResponse<AuditLog>，「活動記錄」tab）
// ============================================================

export const DEMO_LOGIN_EVENTS: PaginatedResponse<LoginEventWithUser> = {
  data: [
    {
      id: 'demo-le-1', user_id: 'demo-admin', email: 'admin@example.com', user_name: '範例管理員',
      event_type: 'login_success', ip_address: '192.168.1.100', user_agent: 'Mozilla/5.0',
      device_type: 'desktop', browser: 'Chrome', os: 'Windows',
      is_unusual_time: false, is_unusual_location: false, is_new_device: false, is_mass_login: false,
      failure_reason: null, created_at: '2026-04-01T08:00:00Z',
    },
    {
      id: 'demo-le-2', user_id: 'demo-vet', email: 'vet@example.com', user_name: '範例獸醫',
      event_type: 'login_success', ip_address: '192.168.1.102', user_agent: 'Mozilla/5.0',
      device_type: 'mobile', browser: 'Safari', os: 'iOS',
      is_unusual_time: false, is_unusual_location: false, is_new_device: true, is_mass_login: false,
      failure_reason: null, created_at: '2026-03-31T09:00:00Z',
    },
    {
      id: 'demo-le-3', user_id: null, email: 'admin@example.com', user_name: null,
      event_type: 'login_failure', ip_address: '203.0.113.45', user_agent: 'curl/7.81',
      device_type: null, browser: null, os: null,
      is_unusual_time: true, is_unusual_location: true, is_new_device: true, is_mass_login: true,
      failure_reason: 'invalid_credentials', created_at: '2026-04-02T03:14:00Z',
    },
    {
      id: 'demo-le-4', user_id: 'demo-admin', email: 'admin@example.com', user_name: '範例管理員',
      event_type: 'logout', ip_address: '192.168.1.100', user_agent: 'Mozilla/5.0',
      device_type: 'desktop', browser: 'Chrome', os: 'Windows',
      is_unusual_time: false, is_unusual_location: false, is_new_device: false, is_mass_login: false,
      failure_reason: null, created_at: '2026-04-01T18:30:00Z',
    },
  ],
  total: 4, page: 1, per_page: 20, total_pages: 1,
}

export const DEMO_SESSIONS: PaginatedResponse<SessionWithUser> = {
  data: [
    {
      id: 'demo-sess-1', user_id: 'demo-admin', user_email: 'admin@example.com', user_name: '範例管理員',
      started_at: '2026-04-01T08:00:00Z', ended_at: null, last_activity_at: '2026-04-01T11:45:00Z',
      ip_address: '192.168.1.100', user_agent: 'Mozilla/5.0',
      page_view_count: 42, action_count: 15, is_active: true, ended_reason: null,
    },
    {
      id: 'demo-sess-2', user_id: 'demo-vet', user_email: 'vet@example.com', user_name: '範例獸醫',
      started_at: '2026-03-31T09:00:00Z', ended_at: '2026-03-31T17:30:00Z', last_activity_at: '2026-03-31T17:28:00Z',
      ip_address: '192.168.1.102', user_agent: 'Mozilla/5.0',
      page_view_count: 88, action_count: 34, is_active: false, ended_reason: 'logout',
    },
    {
      id: 'demo-sess-3', user_id: 'demo-staff', user_email: 'staff@example.com', user_name: '範例實驗員',
      started_at: '2026-03-30T08:30:00Z', ended_at: '2026-03-30T18:00:00Z', last_activity_at: '2026-03-30T17:15:00Z',
      ip_address: '192.168.1.101', user_agent: 'Mozilla/5.0',
      page_view_count: 61, action_count: 20, is_active: false, ended_reason: 'idle_timeout',
    },
  ],
  total: 3, page: 1, per_page: 20, total_pages: 1,
}

export const DEMO_SECURITY_ALERTS: PaginatedResponse<SecurityAlert> = {
  data: [
    {
      id: 'demo-alert-1', alert_type: 'brute_force', severity: 'high',
      title: '偵測到疑似暴力登入嘗試', description: '同一 IP 於 5 分鐘內登入失敗 5 次',
      user_id: null, context_data: { ip: '203.0.113.45', attempts: 5 },
      status: 'open', resolved_by: null, resolved_at: null, resolution_notes: null,
      created_at: '2026-04-02T03:15:00Z',
    },
    {
      id: 'demo-alert-2', alert_type: 'unusual_login_time', severity: 'warning',
      title: '非常規時段登入', description: '使用者於凌晨 03:14 登入，與日常作息不符',
      user_id: 'demo-vet', context_data: null,
      status: 'acknowledged', resolved_by: null, resolved_at: null, resolution_notes: null,
      created_at: '2026-03-28T03:14:00Z',
    },
    {
      id: 'demo-alert-3', alert_type: 'new_device', severity: 'info',
      title: '新裝置登入', description: '偵測到自新行動裝置登入',
      user_id: 'demo-vet', context_data: { device: 'iPhone' },
      status: 'resolved', resolved_by: 'demo-admin', resolved_at: '2026-03-31T10:00:00Z',
      resolution_notes: '已與本人確認為公務手機，屬正常操作',
      created_at: '2026-03-31T09:00:00Z',
    },
  ],
  total: 3, page: 1, per_page: 20, total_pages: 1,
}

// /audit-logs（安全審計「活動記錄」tab）— 較精簡的 AuditLog 型別（非 UserActivityLog）
export const DEMO_SECURITY_AUDIT_LOGS: PaginatedResponse<AuditLog> = {
  data: [
    {
      id: 'demo-sal-1', actor_user_id: 'demo-staff', actor_email: 'staff@example.com', actor_name: '範例實驗員',
      action: 'CREATE', entity_type: 'animal_weight', entity_id: 'demo-w-1',
      entity_name: 'D-001 體重 28.5 kg',
      after_data: { weight: '28.5', measure_date: '2026-03-28' },
      created_at: '2026-03-28T10:00:00Z',
    },
    {
      id: 'demo-sal-2', actor_user_id: 'demo-vet', actor_email: 'vet@example.com', actor_name: '範例獸醫',
      action: 'CREATE', entity_type: 'animal_observation', entity_id: 'demo-obs-6',
      entity_name: 'D-002 異常觀察',
      after_data: { record_type: 'abnormal' },
      created_at: '2026-03-25T14:00:00Z',
    },
    {
      id: 'demo-sal-3', actor_user_id: 'demo-admin', actor_email: 'admin@example.com', actor_name: '範例管理員',
      action: 'UPDATE', entity_type: 'protocol', entity_id: 'demo-p1',
      entity_name: 'AUP-2025-001 心血管藥物安全性評估',
      before_data: { status: 'UNDER_REVIEW' }, after_data: { status: 'APPROVED' },
      created_at: '2025-11-01T14:00:00Z',
    },
  ],
  total: 3, page: 1, per_page: 50, total_pages: 1,
}

// ============================================================
// 我的變更申請 — GET /amendments（AmendmentListItem[]，plain array）
// demo-amd-1 與 routes.ts 內 DEMO_AMENDMENT_DETAIL 對齊，供 detail deep-link。
// ============================================================

export const DEMO_MY_AMENDMENTS: AmendmentListItem[] = [
  {
    id: 'demo-amd-1', protocol_id: 'demo-p1', amendment_no: 'AMD-2025-001',
    revision_number: 1, amendment_type: 'MAJOR', status: 'UNDER_REVIEW',
    title: '範例變更申請：增加動物數量並調整實驗程序',
    description: '因初步數據需擴大樣本數，申請增加 4 隻迷你豬並調整採血頻率。',
    change_items: ['ANIMAL_COUNT', 'PROCEDURE'],
    created_by: 'demo-pi', created_at: '2025-01-10T00:00:00Z', updated_at: '2025-01-12T00:00:00Z',
    version: 1, is_historical: false,
    submitted_by: 'demo-pi', submitted_at: '2025-01-11T00:00:00Z',
    protocol_iacuc_no: 'IACUC-2025-001', protocol_title: '心血管藥物安全性評估',
    submitted_by_name: '範例研究員',
  },
  {
    id: 'demo-amd-2', protocol_id: 'demo-p1', amendment_no: 'AMD-2025-002',
    revision_number: 1, amendment_type: 'MINOR', status: 'APPROVED',
    title: '範例變更申請：更換共同主持人',
    description: '原共同主持人離職，申請由範例研究員接任。',
    change_items: ['PERSONNEL'],
    created_by: 'demo-pi', created_at: '2025-03-01T00:00:00Z', updated_at: '2025-03-05T00:00:00Z',
    version: 1, is_historical: false,
    submitted_by: 'demo-pi', submitted_at: '2025-03-01T00:00:00Z',
    classified_by: 'demo-admin', classified_at: '2025-03-02T00:00:00Z',
    effective_from: '2025-03-05T00:00:00Z',
    protocol_iacuc_no: 'IACUC-2025-001', protocol_title: '心血管藥物安全性評估',
    submitted_by_name: '範例研究員', classified_by_name: '範例秘書',
  },
  {
    id: 'demo-amd-3', protocol_id: 'demo-p3', amendment_no: 'AMD-2025-003',
    revision_number: 1, amendment_type: 'PENDING', status: 'DRAFT',
    title: '範例變更申請：延長計畫執行期限（草稿）',
    description: '因設備校正延誤，擬申請延長 3 個月。',
    change_items: ['TIMELINE'],
    created_by: 'demo-pi', created_at: '2026-04-01T00:00:00Z', updated_at: '2026-04-01T00:00:00Z',
    version: 1, is_historical: false,
    protocol_iacuc_no: 'IACUC-2025-003', protocol_title: '新型疫苗免疫反應研究',
  },
]
