import type { DocType } from '@/lib/api'

export interface DocumentLine {
  id?: string
  line_no: number
  product_id: string
  product_name?: string
  product_sku?: string
  qty: string
  uom: string
  unit_price: string
  batch_no: string
  expiry_date: string
  remark: string
}

export interface DocumentFormData {
  doc_type: DocType
  doc_date: string
  warehouse_id: string
  warehouse_from_id: string
  warehouse_to_id: string
  partner_id: string
  source_doc_id?: string
  remark: string
  lines: DocumentLine[]
}

export const DOC_TYPE_NAMES: Record<DocType, string> = {
  PO: '採購單',
  GRN: '採購入庫',
  PR: '採購退貨',
  SO: '銷貨單',
  DO: '銷貨出庫',
  TR: '調撥單',
  STK: '盤點單',
  ADJ: '調整單',
  RM: '退料單',
}
