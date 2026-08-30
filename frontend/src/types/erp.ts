/**
 * ERP 型別（倉庫、產品、交易夥伴、單據、庫存、SKU）
 */

import type { PendingOwner } from './pendingOwner'

// 倉庫
export interface Warehouse {
    id: string
    code: string
    name: string
    address?: string
    is_active: boolean
    created_at: string
    updated_at: string
}

// 倉庫樹節點（含貨架）
export interface WarehouseTreeNode {
    id: string
    code: string
    name: string
    shelves: ShelfNode[]
}

// 貨架節點
export interface ShelfNode {
    id: string
    code: string
    name?: string
}

// 儲位/貨架/建築結構
export type StorageLocationType = 'shelf' | 'rack' | 'zone' | 'bin' | 'wall' | 'door' | 'window'

export interface StorageLocation {
    id: string
    warehouse_id: string
    code: string
    name?: string
    location_type: StorageLocationType
    row_index: number
    col_index: number
    width: number
    height: number
    capacity?: number
    current_count: number
    color?: string
    is_active: boolean
    config?: Record<string, unknown>
    created_at: string
    updated_at: string
}

export interface StorageLocationWithWarehouse extends StorageLocation {
    warehouse_code: string
    warehouse_name: string
}

export interface StorageLayoutItem {
    id: string
    row_index: number
    col_index: number
    width: number
    height: number
}

export interface UpdateStorageLayoutRequest {
    items: StorageLayoutItem[]
}

export const storageLocationTypeNames: Record<StorageLocationType, string> = {
    shelf: '貨架',
    rack: '儲物架',
    zone: '區域',
    bin: '儲物格',
    wall: '牆壁',
    door: '門',
    window: '窗戶',
}

export interface StorageLocationInventoryItem {
    id: string
    storage_location_id: string
    product_id: string
    product_sku: string
    product_name: string
    on_hand_qty: string
    base_uom: string
    batch_no?: string
    expiry_date?: string
    updated_at: string
}

export interface UpdateStorageLocationInventoryItemRequest {
    on_hand_qty: string
}

// 倉庫報表
export interface StorageLocationWithInventory {
    id: string
    code: string
    name?: string
    location_type: string
    row_index: number
    col_index: number
    width: number
    height: number
    capacity?: number
    current_count: number
    color?: string
    is_active: boolean
    inventory: StorageLocationInventoryItem[]
}

export interface WarehouseReportSummary {
    total_locations: number
    active_locations: number
    total_capacity: number
    total_current_count: number
    total_inventory_items: number
    /** R35-3 (redo on R35-16): 後端 Decimal → 字串以避免 JS 浮點誤差；缺 selling_price 產品不計入 */
    total_inventory_value: string
}

export interface WarehouseReportData {
    warehouse: Warehouse
    summary: WarehouseReportSummary
    locations: StorageLocationWithInventory[]
    generated_at: string
}

// 產品
export interface Product {
    id: string
    sku: string
    name: string
    spec?: string
    category_id?: string
    base_uom: string
    track_batch: boolean
    track_expiry: boolean
    safety_stock?: string
    reorder_point?: string
    is_active: boolean
    created_at: string
    updated_at: string
}

// 交易夥伴
export interface Partner {
    id: string
    partner_type: 'supplier' | 'customer'
    code: string
    name: string
    customer_category?: 'internal' | 'external' | 'research' | 'other'
    tax_id?: string
    phone?: string
    phone_ext?: string
    email?: string
    address?: string
    payment_terms?: string
    is_active: boolean
    created_at: string
    updated_at: string
}

// 單據
// R84-13：'SR'（銷貨退貨）/ 'RTN'（退貨單）已從型別移除——業務上不存在銷貨退貨，
// 後端 DocType 已封鎖建立這兩種單據（見 backend/src/models/document.rs）。
export type DocType = 'PO' | 'GRN' | 'PR' | 'SO' | 'TR' | 'STK' | 'ADJ'
export type DocStatus = 'draft' | 'submitted' | 'approved' | 'cancelled'

export interface DocumentLine {
    id: string
    document_id: string
    line_no: number
    product_id: string
    product_sku: string
    product_name: string
    qty: string
    uom: string
    unit_price?: string
    batch_no?: string
    expiry_date?: string
    storage_location_id?: string
    storage_location_from_id?: string
    storage_location_to_id?: string
    /** SO 多倉銷貨（migration 136）：該行倉庫＝儲位所屬倉，server 端反推回填 */
    warehouse_id?: string
    remark?: string
}

export interface Document {
    id: string
    doc_type: DocType
    doc_no: string
    status: DocStatus
    warehouse_id?: string
    warehouse_from_id?: string
    warehouse_to_id?: string
    partner_id?: string
    /** SO 直接關聯計畫 ID（取代手動建立客戶） */
    protocol_id?: string
    source_doc_id?: string
    /** PO 入庫狀態：pending | partial | complete */
    receipt_status?: 'pending' | 'partial' | 'complete'
    doc_date: string
    remark?: string
    created_by: string
    approved_by?: string
    created_at: string
    updated_at: string
    approved_at?: string
    lines: DocumentLine[]
    warehouse_name?: string
    warehouse_from_name?: string
    warehouse_to_name?: string
    partner_name?: string
    /** 對應 protocol_id 的計畫編號（唯讀） */
    protocol_no?: string
    created_by_name: string
    approved_by_name?: string
    /**
     * R84-5 沖銷關聯（本單是沖銷單時）：本單沖銷了哪一張原單。
     * `reverses_doc_id` 為 UUID、`reverses_doc_no` 為可顯示的單號。
     */
    reverses_doc_id?: string
    reverses_doc_no?: string
    /** R84-5 沖銷關聯（本單被沖銷時）：沖銷本單的那張沖銷單（後端反向查詢後回傳）。 */
    reversed_by_doc_id?: string
    reversed_by_doc_no?: string
    /** 沖銷生效時間（沖銷單的 approved_at）；沖銷單尚未核准時為 undefined。 */
    reversed_at?: string
    /** 大金額 ADJ 兩級審批欄位 */
    requires_manager_approval?: boolean
    scrap_total_amount?: string
    /** pending | wm_approved | approved | rejected */
    manager_approval_status?: string
    manager_approved_by?: string
    manager_approved_at?: string
    manager_reject_reason?: string
    /** 這張單現在卡在誰手上；僅 status='submitted' 有值（後端 services/pending_owner.rs） */
    pending_owner?: PendingOwner
}

export interface DocumentListItem {
    id: string
    doc_type: DocType
    doc_no: string
    status: DocStatus
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
    /** 這張單現在卡在誰手上；僅 status='submitted' 有值（後端 services/pending_owner.rs） */
    pending_owner?: PendingOwner
}

// 庫存
export interface InventoryOnHand {
    warehouse_id: string
    warehouse_code: string
    warehouse_name: string
    storage_location_id?: string
    storage_location_code?: string
    storage_location_name?: string
    product_id: string
    product_sku: string
    product_name: string
    base_uom: string
    qty_on_hand: string
    avg_cost?: string
    batch_no?: string
    expiry_date?: string
    safety_stock?: string
    reorder_point?: string
    last_updated_at?: string
}

export interface StockLedgerDetail {
    id: string
    warehouse_id: string
    warehouse_name: string
    product_id: string
    product_sku: string
    product_name: string
    trx_date: string
    doc_type: DocType
    doc_id: string
    doc_no: string
    direction: string
    qty_base: string
    unit_cost?: string
    batch_no?: string
    expiry_date?: string
    iacuc_no?: string
}

/** 批號時間軸單筆紀錄（R84-6，跨倉彙總） */
export interface LotMovement {
    id: string
    warehouse_id: string
    warehouse_name: string
    trx_date: string
    doc_type: DocType
    doc_id: string
    doc_no: string
    direction: string
    qty_base: string
}

/**
 * 批號對帳分級：批號層級不平不一定代表數量出錯。2026-05-20 (migration 069) 之前
 * 的異動只寫 stock_ledger 不寫儲位庫存，之後的歷史補帳把品項總量補平了卻未帶批號，
 * 因此先看品項總量再判定嚴重度。
 */
export type LotReconciliationStatus = 'balanced' | 'attribution_only' | 'unbalanced'

/** 批號數量對帳摘要：見 ERP流程.md §6.2.2 */
export interface LotReconciliation {
    received: string
    customer_returned: string
    internal_consumed: string
    returned_to_supplier: string
    adjusted_net: string
    remaining: string
    derived_remaining: string
    balanced: boolean
    status: LotReconciliationStatus
    product_derived_total: string
    product_remaining_total: string
    unattributed_adjust_net: string
}

export interface LotMovementsResponse {
    movements: LotMovement[]
    reconciliation: LotReconciliation
}

/** 低庫存品項在單一倉庫的分布（LowStockTotal 展開明細用） */
export interface LowStockWarehouseQty {
    warehouse_id: string
    warehouse_code: string
    warehouse_name: string
    qty_on_hand: string
}

/**
 * 低庫存彙總（全公司總量 vs 公司預設安全庫存；一品項一筆）
 * - total_on_hand：跨所有倉庫加總，用以判斷是否低於安全庫存（避免逐倉重複虛報）
 * - warehouse_breakdown：各倉分布，供展開查看
 */
export interface LowStockTotal {
    product_id: string
    product_sku: string
    product_name: string
    base_uom: string
    total_on_hand: string
    safety_stock?: string | null
    reorder_point?: string | null
    stock_status: string
    warehouse_breakdown: LowStockWarehouseQty[]
}

/** R35-17: 效期預警（從 v_expiry_alerts view） */
export interface ExpiryAlert {
    product_id: string
    sku: string
    product_name: string
    spec?: string | null
    category_code?: string | null
    warehouse_id: string
    warehouse_code: string
    warehouse_name: string
    batch_no?: string | null
    expiry_date: string
    on_hand_qty: string
    base_uom: string
    /** 距到期天數（負值代表已過期 N 天） */
    days_until_expiry: number
    /** 'expired' | 'critical' | 'warning' | 'soon' 等狀態 string */
    expiry_status: string
    /** 同品項同倉庫所有批號合計 */
    total_qty: string
}

export interface UnassignedInventoryItem {
    warehouse_id: string
    warehouse_name: string
    product_id: string
    product_sku: string
    product_name: string
    base_uom: string
    qty_on_warehouse: string
    qty_on_shelves: string
    qty_unassigned: string
}

/** 造成未分配的來源 GRN 明細（追溯：這批未分配是哪張採購入庫單造成的） */
export interface UnassignedSourceDoc {
    document_id: string
    doc_no: string
    doc_date: string
    line_no: number
    partner_name: string | null
    batch_no: string | null
    expiry_date: string | null
    remaining_unshelved: string
}

// SKU
export interface SkuSegment {
    code: string
    label: string
    value: string
    source: string
}

export interface SkuPreviewRequest {
    org?: string
    cat: string
    sub: string
    attributes?: {
        generic_name?: string
        dose_value?: number
        dose_unit?: string
        dosage_form?: string
        sterile?: boolean
        [key: string]: unknown
    }
    pack: {
        uom: string
        qty: number
    }
    source: string
    rule_version_hint?: string
}

export interface SkuPreviewResponse {
    preview_sku: string
    segments: SkuSegment[]
    rule_version: string
    rule_updated_at?: string
}

export interface SkuPreviewError {
    code: 'E1' | 'E2' | 'E3' | 'E4' | 'E5'
    message: string
    suggestion?: string
    field?: string
}

export interface CreateProductWithSkuRequest {
    name?: string
    spec?: string
    base_uom: string
    track_batch?: boolean
    track_expiry?: boolean
    safety_stock?: number | null
    reorder_point?: number | null
    category_code: string
    subcategory_code: string
    source_code: string
    pack_unit: string
    pack_qty: number
    attributes?: {
        generic_name?: string
        dose_value?: number
        dose_unit?: string
        dosage_form?: string
        sterile?: boolean
        [key: string]: unknown
    } | null
}
