import type { PaginatedResponse } from '@/types/common'

export interface DemoProduct {
  id: string
  sku: string
  name: string
  spec?: string
  base_uom: string
  track_batch: boolean
  track_expiry: boolean
  safety_stock?: string
  reorder_point?: string
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface DemoDocumentListItem {
  id: string
  doc_type: string
  doc_no: string
  status: string
  warehouse_name?: string
  partner_name?: string
  doc_date: string
  created_by_name: string
  approved_by_name?: string
  created_at: string
  approved_at?: string
  line_count: number
  total_amount?: string
  receipt_status?: string
  has_journal_entry: boolean
}

export interface DemoPartner {
  id: string
  name: string
  partner_type: string
  contact_person?: string
  phone?: string
  email?: string
  address?: string
  is_active: boolean
  created_at: string
}

export const DEMO_PRODUCTS: PaginatedResponse<DemoProduct> = {
  data: [
    {
      id: 'demo-prod1', sku: 'MED-001', name: '範例藥品 A',
      spec: '100mg/tab', base_uom: '盒', track_batch: true, track_expiry: true,
      safety_stock: '10', reorder_point: '20', is_active: true,
      created_at: '2025-06-01T08:00:00Z', updated_at: '2026-03-01T08:00:00Z',
    },
    {
      id: 'demo-prod2', sku: 'SUP-001', name: '範例耗材 B',
      spec: '50ml', base_uom: '瓶', track_batch: false, track_expiry: true,
      safety_stock: '5', reorder_point: '15', is_active: true,
      created_at: '2025-06-01T08:00:00Z', updated_at: '2026-02-15T08:00:00Z',
    },
    {
      id: 'demo-prod3', sku: 'FED-001', name: '範例飼料 C',
      spec: '25kg/袋', base_uom: '袋', track_batch: true, track_expiry: true,
      safety_stock: '20', reorder_point: '50', is_active: true,
      created_at: '2025-08-01T08:00:00Z', updated_at: '2026-03-20T08:00:00Z',
    },
  ],
  total: 3, page: 1, per_page: 20, total_pages: 1,
}

export const DEMO_DOCUMENTS: PaginatedResponse<DemoDocumentListItem> = {
  data: [
    {
      id: 'demo-doc1', doc_type: 'PO', doc_no: 'PO-2026-0001', status: 'approved',
      warehouse_name: '範例主倉庫', partner_name: '範例供應商 A',
      doc_date: '2026-03-15', created_by_name: '範例採購員',
      approved_by_name: '範例主管', created_at: '2026-03-15T08:00:00Z',
      approved_at: '2026-03-16T10:00:00Z', line_count: 3, total_amount: '15000',
      has_journal_entry: true,
    },
    {
      id: 'demo-doc2', doc_type: 'GRN', doc_no: 'GRN-2026-0001', status: 'approved',
      warehouse_name: '範例主倉庫', partner_name: '範例供應商 A',
      doc_date: '2026-03-20', created_by_name: '範例倉管員',
      created_at: '2026-03-20T08:00:00Z', line_count: 3,
      has_journal_entry: true,
    },
    {
      id: 'demo-doc3', doc_type: 'SO', doc_no: 'SO-2026-0001', status: 'draft',
      warehouse_name: '範例主倉庫',
      doc_date: '2026-03-28', created_by_name: '範例員工',
      created_at: '2026-03-28T08:00:00Z', line_count: 2,
      has_journal_entry: false,
    },
  ],
  total: 3, page: 1, per_page: 20, total_pages: 1,
}

// ============================================
// 倉庫佈局 Demo Data
// ============================================

export const DEMO_WAREHOUSES = [
  {
    id: 'demo-w001', code: 'W001', name: '範例主倉庫',
    address: '動物中心 B 棟 1F', is_active: true,
    created_at: '2025-01-01T08:00:00Z', updated_at: '2026-01-01T08:00:00Z',
  },
]

// 10 欄 × 8 列的佈局
// 結構: 北牆(row0), 南牆+門(row7), 西牆(col0 rows1-6), 東牆(col9 rows1-6)
// 儲位: 上排(rows1-2), 下排(rows4-5)，row3/row6 為走道
export const DEMO_STORAGE_LOCATIONS = [
  // === 結構 ===
  {
    id: 'demo-loc-wall-top', warehouse_id: 'demo-w001', code: 'WALL-N', name: '北牆',
    location_type: 'wall', row_index: 0, col_index: 0, width: 10, height: 1,
    current_count: 0, color: '#475569', is_active: true,
    warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    created_at: '2025-01-01T08:00:00Z', updated_at: '2025-01-01T08:00:00Z',
  },
  {
    id: 'demo-loc-wall-left', warehouse_id: 'demo-w001', code: 'WALL-W', name: '西牆',
    location_type: 'wall', row_index: 1, col_index: 0, width: 1, height: 6,
    current_count: 0, color: '#475569', is_active: true,
    warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    created_at: '2025-01-01T08:00:00Z', updated_at: '2025-01-01T08:00:00Z',
  },
  {
    id: 'demo-loc-wall-right', warehouse_id: 'demo-w001', code: 'WALL-E', name: '東牆',
    location_type: 'wall', row_index: 1, col_index: 9, width: 1, height: 6,
    current_count: 0, color: '#475569', is_active: true,
    warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    created_at: '2025-01-01T08:00:00Z', updated_at: '2025-01-01T08:00:00Z',
  },
  {
    id: 'demo-loc-wall-bot-l', warehouse_id: 'demo-w001', code: 'WALL-S-L', name: '南牆左',
    location_type: 'wall', row_index: 7, col_index: 0, width: 3, height: 1,
    current_count: 0, color: '#475569', is_active: true,
    warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    created_at: '2025-01-01T08:00:00Z', updated_at: '2025-01-01T08:00:00Z',
  },
  {
    id: 'demo-loc-door', warehouse_id: 'demo-w001', code: 'DOOR-1', name: '出入口',
    location_type: 'door', row_index: 7, col_index: 3, width: 2, height: 1,
    current_count: 0, color: '#94a3b8', is_active: true,
    warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    created_at: '2025-01-01T08:00:00Z', updated_at: '2025-01-01T08:00:00Z',
  },
  {
    id: 'demo-loc-wall-bot-r', warehouse_id: 'demo-w001', code: 'WALL-S-R', name: '南牆右',
    location_type: 'wall', row_index: 7, col_index: 5, width: 5, height: 1,
    current_count: 0, color: '#475569', is_active: true,
    warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    created_at: '2025-01-01T08:00:00Z', updated_at: '2025-01-01T08:00:00Z',
  },

  // === 上排儲位 (rows 1-2) ===
  {
    id: 'demo-loc-shelf-fa', warehouse_id: 'demo-w001', code: 'F-A', name: '飼料架A',
    location_type: 'shelf', row_index: 1, col_index: 1, width: 2, height: 2,
    capacity: 50, current_count: 38, color: '#3b82f6', is_active: true,
    warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    created_at: '2025-01-01T08:00:00Z', updated_at: '2026-04-01T08:00:00Z',
  },
  {
    id: 'demo-loc-shelf-fb', warehouse_id: 'demo-w001', code: 'F-B', name: '飼料架B',
    location_type: 'shelf', row_index: 1, col_index: 3, width: 2, height: 2,
    capacity: 50, current_count: 15, color: '#3b82f6', is_active: true,
    warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    created_at: '2025-01-01T08:00:00Z', updated_at: '2026-04-01T08:00:00Z',
  },
  {
    id: 'demo-loc-rack-m1', warehouse_id: 'demo-w001', code: 'M-1', name: '藥品架1',
    location_type: 'rack', row_index: 1, col_index: 6, width: 1, height: 2,
    capacity: 30, current_count: 28, color: '#10b981', is_active: true,
    warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    created_at: '2025-01-01T08:00:00Z', updated_at: '2026-04-01T08:00:00Z',
  },
  {
    id: 'demo-loc-rack-m2', warehouse_id: 'demo-w001', code: 'M-2', name: '藥品架2',
    location_type: 'rack', row_index: 1, col_index: 7, width: 2, height: 2,
    capacity: 30, current_count: 12, color: '#10b981', is_active: true,
    warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    created_at: '2025-01-01T08:00:00Z', updated_at: '2026-04-01T08:00:00Z',
  },

  // === 下排儲位 (rows 4-5) ===
  {
    id: 'demo-loc-shelf-sa', warehouse_id: 'demo-w001', code: 'S-A', name: '耗材架A',
    location_type: 'shelf', row_index: 4, col_index: 1, width: 2, height: 2,
    capacity: 40, current_count: 22, color: '#3b82f6', is_active: true,
    warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    created_at: '2025-01-01T08:00:00Z', updated_at: '2026-04-10T08:00:00Z',
  },
  {
    id: 'demo-loc-shelf-sb', warehouse_id: 'demo-w001', code: 'S-B', name: '耗材架B',
    location_type: 'shelf', row_index: 4, col_index: 3, width: 2, height: 2,
    capacity: 40, current_count: 8, color: '#3b82f6', is_active: true,
    warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    created_at: '2025-01-01T08:00:00Z', updated_at: '2026-04-10T08:00:00Z',
  },
  {
    id: 'demo-loc-bin-b1', warehouse_id: 'demo-w001', code: 'B-1', name: '備品格',
    location_type: 'bin', row_index: 4, col_index: 6, width: 2, height: 2,
    capacity: 20, current_count: 5, color: '#6366f1', is_active: true,
    warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    created_at: '2025-01-01T08:00:00Z', updated_at: '2026-04-10T08:00:00Z',
  },
  {
    id: 'demo-loc-rack-c1', warehouse_id: 'demo-w001', code: 'C-1', name: '冷藏暫放架',
    location_type: 'rack', row_index: 4, col_index: 8, width: 1, height: 2,
    capacity: 15, current_count: 10, color: '#0ea5e9', is_active: true,
    warehouse_code: 'W001', warehouse_name: '範例主倉庫',
    created_at: '2025-01-01T08:00:00Z', updated_at: '2026-04-10T08:00:00Z',
  },
]

// /warehouses/with-shelves 用：WarehouseTreeNode[]（含 shelves 欄位）
export const DEMO_WAREHOUSE_TREE = [
  {
    id: 'demo-w001', code: 'W001', name: '範例主倉庫',
    shelves: [
      { id: 'demo-loc-shelf-fa', code: 'F-A', name: '飼料架A' },
      { id: 'demo-loc-shelf-fb', code: 'F-B', name: '飼料架B' },
      { id: 'demo-loc-rack-m1', code: 'M-1', name: '藥品架1' },
      { id: 'demo-loc-rack-m2', code: 'M-2', name: '藥品架2' },
      { id: 'demo-loc-shelf-sa', code: 'S-A', name: '耗材架A' },
      { id: 'demo-loc-shelf-sb', code: 'S-B', name: '耗材架B' },
      { id: 'demo-loc-bin-b1',  code: 'B-1', name: '備品格' },
      { id: 'demo-loc-rack-c1', code: 'C-1', name: '冷藏暫放架' },
    ],
  },
]

// 飼料架A 庫存（FED-001 兩批次）
export const DEMO_SHELF_FA_INVENTORY = [
  {
    id: 'demo-inv-fa-1', storage_location_id: 'demo-loc-shelf-fa',
    product_id: 'demo-prod3', product_sku: 'FED-001', product_name: '範例飼料 C',
    on_hand_qty: '30', base_uom: '袋',
    batch_no: 'LOT-2026-001', expiry_date: '2027-01-31',
    updated_at: '2026-04-01T08:00:00Z',
  },
  {
    id: 'demo-inv-fa-2', storage_location_id: 'demo-loc-shelf-fa',
    product_id: 'demo-prod3', product_sku: 'FED-001', product_name: '範例飼料 C',
    on_hand_qty: '8', base_uom: '袋',
    batch_no: 'LOT-2026-002', expiry_date: '2027-03-31',
    updated_at: '2026-04-10T08:00:00Z',
  },
]

// 藥品架1 庫存（MED-001 兩批次，其中一批次即將到期）
export const DEMO_RACK_M1_INVENTORY = [
  {
    id: 'demo-inv-m1-1', storage_location_id: 'demo-loc-rack-m1',
    product_id: 'demo-prod1', product_sku: 'MED-001', product_name: '範例藥品 A',
    on_hand_qty: '8', base_uom: '盒',
    batch_no: 'MED-LOT-2025-05', expiry_date: '2026-07-31',
    updated_at: '2026-03-20T08:00:00Z',
  },
  {
    id: 'demo-inv-m1-2', storage_location_id: 'demo-loc-rack-m1',
    product_id: 'demo-prod1', product_sku: 'MED-001', product_name: '範例藥品 A',
    on_hand_qty: '20', base_uom: '盒',
    batch_no: 'MED-LOT-2026-01', expiry_date: '2027-06-30',
    updated_at: '2026-04-05T08:00:00Z',
  },
]

// 耗材架A 庫存（SUP-001）
export const DEMO_SHELF_SA_INVENTORY = [
  {
    id: 'demo-inv-sa-1', storage_location_id: 'demo-loc-shelf-sa',
    product_id: 'demo-prod2', product_sku: 'SUP-001', product_name: '範例耗材 B',
    on_hand_qty: '22', base_uom: '瓶',
    batch_no: undefined, expiry_date: '2027-12-31',
    updated_at: '2026-04-10T08:00:00Z',
  },
]

export const DEMO_PARTNERS: DemoPartner[] = [
  {
    id: 'demo-partner1', name: '範例供應商 A', partner_type: 'supplier',
    contact_person: '範例聯絡人', phone: '02-1234-5678',
    email: 'demo@example.com', is_active: true, created_at: '2025-01-01T08:00:00Z',
  },
  {
    id: 'demo-partner2', name: '範例供應商 B', partner_type: 'supplier',
    contact_person: '範例聯絡人 B', is_active: true, created_at: '2025-03-01T08:00:00Z',
  },
]
