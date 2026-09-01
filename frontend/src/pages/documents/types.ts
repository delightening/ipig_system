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

/**
 * 盤點範圍（僅盤點單 STK 使用，且只在**建立**時生效）。
 *
 * 後端 `generate_stocktake_lines` 在建單且未帶明細時，依本設定過濾底稿；
 * 空的 `category_codes` 等同全盤。改單時明細已存在、不會重新產生，故編輯畫面不顯示。
 */
export interface StocktakeScope {
  /** 'full' 全盤 / 'partial' 循環盤點；對應後端 StocktakeScope.scope_type */
  scope_type: 'full' | 'partial'
  /** 只盤這些品類（例：準備室只盤 DRG 藥品），空/未給即不限 */
  category_codes?: string[]
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
  /** 盤點範圍（僅 STK 建立時使用） */
  stocktake_scope?: StocktakeScope
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
