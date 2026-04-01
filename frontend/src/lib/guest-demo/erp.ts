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
