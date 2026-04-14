/**
 * Dashboard widget demo data
 * 各 widget 的範例資料，guest 模式下使用
 */

// === 低庫存警示 ===
export const DEMO_LOW_STOCK_ALERTS = [
  { product_id: 'demo-prod1', product_name: '範例藥品 A', sku: 'MED-001', on_hand: 8, safety_stock: 10, warehouse_name: '範例主倉庫' },
  { product_id: 'demo-prod3', product_name: '範例飼料 C', sku: 'FED-001', on_hand: 15, safety_stock: 20, warehouse_name: '範例主倉庫' },
]

// === 最近單據 ===
export const DEMO_RECENT_DOCUMENTS = [
  { id: 'demo-doc1', doc_no: 'PO-2026-0001', doc_type: 'PO', status: 'approved', created_at: '2026-03-15T08:00:00Z' },
  { id: 'demo-doc2', doc_no: 'GRN-2026-0001', doc_type: 'GRN', status: 'approved', created_at: '2026-03-20T08:00:00Z' },
  { id: 'demo-doc3', doc_no: 'SO-2026-0001', doc_type: 'SO', status: 'draft', created_at: '2026-03-28T08:00:00Z' },
]

// === 今日已核准單據數 ===
export const DEMO_TODAY_APPROVED_DOCS = 0

// === 獸醫評論 ===
export const DEMO_VET_COMMENTS = {
  data: [
    {
      id: 'demo-vc1', animal_id: 'demo-a2', ear_tag: 'D-002', species_name: '迷你豬',
      comment: '範例獸醫評論：持續觀察精神食慾', comment_type: 'observation',
      created_by_name: '範例獸醫', created_at: '2026-04-01T10:00:00Z',
    },
    {
      id: 'demo-vc2', animal_id: 'demo-a1', ear_tag: 'D-001', species_name: '迷你豬',
      comment: '範例獸醫評論：術後恢復良好', comment_type: 'follow_up',
      created_by_name: '範例獸醫', created_at: '2026-03-30T14:00:00Z',
    },
  ],
}

// === 用藥中動物 ===
export const DEMO_ANIMALS_ON_MEDICATION = {
  data: [
    {
      id: 'demo-a2', ear_tag: 'D-002', species_name: '迷你豬', pen_location: 'A-02',
      status: 'in_experiment', is_on_medication: true, iacuc_no: 'IACUC-2025-001',
    },
  ],
  total: 1, page: 1, per_page: 10, total_pages: 1,
}

// === 今日出勤（widget 用） ===
export const DEMO_STAFF_ATTENDANCE_TODAY = {
  data: [
    {
      id: 'demo-att-w1', user_id: 'demo-u1', user_email: 'staff@example.com',
      user_name: '範例員工 A', work_date: new Date().toISOString().slice(0, 10),
      clock_in_time: new Date().toISOString().replace(/T.*/, 'T08:05:00Z'),
      clock_out_time: null, status: 'present', remark: null, is_corrected: false,
    },
    {
      id: 'demo-att-w2', user_id: 'demo-u2', user_email: 'vet@example.com',
      user_name: '範例獸醫', work_date: new Date().toISOString().slice(0, 10),
      clock_in_time: new Date().toISOString().replace(/T.*/, 'T07:55:00Z'),
      clock_out_time: null, status: 'present', remark: null, is_corrected: false,
    },
  ],
  total: 2, page: 1, per_page: 50, total_pages: 1,
}

// === 日曆事件 ===
export const DEMO_CALENDAR_EVENTS = [
  {
    id: 'demo-cal-e1', title: '範例：IACUC 月例會議',
    start: new Date().toISOString().replace(/T.*/, 'T09:00:00Z'),
    end: new Date().toISOString().replace(/T.*/, 'T10:30:00Z'),
    type: 'meeting',
  },
  {
    id: 'demo-cal-e2', title: '範例：動物健康檢查',
    start: new Date().toISOString().replace(/T.*/, 'T14:00:00Z'),
    end: new Date().toISOString().replace(/T.*/, 'T15:00:00Z'),
    type: 'health_check',
  },
]

// === 我的計畫（widget 用） ===
export const DEMO_MY_PROJECTS_WIDGET = [
  {
    id: 'demo-p1', protocol_no: 'AUP-2025-001', iacuc_no: 'IACUC-2025-001',
    title: '範例研究計畫：心血管藥物安全性評估', status: 'APPROVED',
    pi_name: '範例研究員 A', start_date: '2025-06-01', end_date: '2026-12-31',
  },
]

// === 通知 / 修正 ===
export const DEMO_NOTIFICATIONS_UNREAD = { count: 0 }
export const DEMO_AMENDMENTS_PENDING = { count: 0 }

// === 整合匯出（向後相容） ===
export const DEMO_DASHBOARD_WIDGETS = {
  lowStockAlerts: DEMO_LOW_STOCK_ALERTS,
  recentDocuments: DEMO_RECENT_DOCUMENTS,
  todayApprovedDocs: DEMO_TODAY_APPROVED_DOCS,
}
