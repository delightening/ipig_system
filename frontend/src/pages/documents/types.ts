import type { DocType } from '@/lib/api'

export interface DocumentLine {
  /**
   * line 唯一 id；後端載入時為 UUID，前端新增 / 複製時由 generateLineId() 產生
   * （`temp-` 前綴，post 後不傳此欄位）。所有路徑都保證有值，不應 undefined。
   */
  id: string
  line_no: number
  product_id: string
  product_name?: string
  product_sku?: string
  /** 該品項的基本單位；單位下拉的第一個選項 */
  base_uom?: string
  /** 該品項換算表裡的其他單位（如「盒」）；與 base_uom 合成單位下拉的選項 */
  alt_uoms?: string[]
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
  /** SO 跨倉：該行儲位所屬倉（挑選儲位時一併記下，供倉別 chip 與批號過濾；
   *  server 端仍以儲位反推為準，此欄僅 UI 輔助） */
  warehouse_id?: string
  remark: string
}

export interface DocumentFormData {
  doc_type: DocType
  doc_date: string
  warehouse_id: string
  warehouse_from_id: string
  warehouse_to_id: string
  partner_id: string
  /** SO 直接關聯計畫 UUID */
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
  TR: '調撥單',
  STK: '盤點單',
  ADJ: '調整單',
}
