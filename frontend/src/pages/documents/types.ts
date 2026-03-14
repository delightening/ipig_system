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
  /** 儲位 ID (入庫 GRN, 銷貨 SO, 調整 ADJ 使用) */
  storage_location_id?: string
  /** 調撥來源儲位 ID (TR 使用) */
  storage_location_from_id?: string
  /** 調撥目標儲位 ID (TR 使用) */
  storage_location_to_id?: string
  remark: string
}

export interface DocumentFormData {
  doc_type: DocType
  doc_date: string
  warehouse_id: string
  warehouse_from_id: string
  warehouse_to_id: string
  partner_id: string
  /** SO/DO 直接關聯計畫 UUID */
  protocol_id?: string
  protocol_no?: string
  source_doc_id?: string
  remark: string
  lines: DocumentLine[]
}

export const DOC_TYPE_NAMES: Record<DocType, string> = {
  PO: '採購單',
  GRN: '採購入庫',
  PR: '採購退貨',
  SO: '銷貨單',
  DO: '銷貨出庫', // 雖然計畫中建議簡化到 SO，但保留舊單據查詢相容性
  TR: '調撥單',
  STK: '盤點單',
  ADJ: '調整單',
  RM: '退料單', // 同上，保留系統現有型別相容性，但新增頁面可隱藏
}
