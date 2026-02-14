/**
 * ERP 型別（倉庫、產品、交易夥伴、單據、庫存、SKU）
 */

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

// 儲位/貨架
export type StorageLocationType = 'shelf' | 'rack' | 'zone' | 'bin'

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
    email?: string
    address?: string
    payment_terms?: string
    is_active: boolean
    created_at: string
    updated_at: string
}

// 單據
export type DocType = 'PO' | 'GRN' | 'PR' | 'SO' | 'DO' | 'TR' | 'STK' | 'ADJ' | 'RM'
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
    created_by_name: string
    approved_by_name?: string
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
}

// 庫存
export interface InventoryOnHand {
    warehouse_id: string
    warehouse_code: string
    warehouse_name: string
    product_id: string
    product_sku: string
    product_name: string
    base_uom: string
    qty_on_hand: string
    avg_cost?: string
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
}

export interface LowStockAlert {
    warehouse_id: string
    warehouse_name: string
    product_id: string
    product_sku: string
    product_name: string
    qty_on_hand: string
    safety_stock: string
    reorder_point: string
    shortage: string
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
