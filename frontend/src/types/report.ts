/**
 * 報表型別
 */

export interface StockOnHandReport {
    warehouse_id: string
    warehouse_code: string
    warehouse_name: string
    product_id: string
    product_sku: string
    product_name: string
    category_name?: string
    base_uom: string
    qty_on_hand: string
    avg_cost?: string
    total_value?: string
    safety_stock?: string
    reorder_point?: string
}

export interface StockLedgerReport {
    doc_id: string
    trx_date: string
    warehouse_code: string
    warehouse_name: string
    product_sku: string
    product_name: string
    doc_type: string
    doc_no: string
    direction: string
    qty_base: string
    unit_cost?: string
    batch_no?: string
    expiry_date?: string
}

export interface PurchaseLinesReport {
    doc_id: string
    doc_date: string
    doc_no: string
    status: string
    partner_code?: string
    partner_name?: string
    warehouse_name?: string
    product_sku: string
    product_name: string
    qty: string
    uom: string
    unit_price?: string
    line_total?: string
    created_by_name: string
    approved_by_name?: string
}

export interface SalesLinesReport {
    doc_id: string
    doc_date: string
    doc_no: string
    status: string
    partner_code?: string
    partner_name?: string
    customer_category?: string
    warehouse_name?: string
    product_sku: string
    product_name: string
    qty: string
    uom: string
    unit_price?: string
    line_total?: string
    created_by_name: string
    approved_by_name?: string
}

export interface CostSummaryReport {
    warehouse_id: string
    warehouse_code: string
    warehouse_name: string
    product_id: string
    product_sku: string
    product_name: string
    category_name?: string
    qty_on_hand: string
    avg_cost?: string
    total_value?: string
}

// 進銷貨彙總 — 月份
export interface PurchaseSalesMonthlySummary {
    year_month: string
    purchase_total: string
    purchase_return: string
    net_purchase: string
    sales_total: string
    sales_return: string
    net_sales: string
    cogs_total: string
    gross_profit: string
}

// 進銷貨彙總 — 供應商/客戶
export interface PurchaseSalesPartnerSummary {
    partner_id: string
    partner_code: string
    partner_name: string
    partner_type: string
    total_amount: string
    return_amount: string
    net_amount: string
    doc_count: number
}

// 進銷貨彙總 — 產品類別
export interface PurchaseSalesCategorySummary {
    category_name: string
    purchase_amount: string
    sales_amount: string
    cogs_amount: string
    gross_profit: string
}

// 損益表
export interface ProfitLossRow {
    account_code: string
    account_name: string
    account_type: string
    amount: string
}

export interface ProfitLossSummary {
    rows: ProfitLossRow[]
    total_revenue: string
    total_expense: string
    net_income: string
}

/**
 * 案件消耗報表：一列 = 一個案件 × 一個品項的淨消耗。
 *
 * 數量以品項的 `base_uom`（領用單位）計，不做包裝換算——倉庫側論箱盒是
 * 庫存現況報表的事（2026-09-04 裁定）。
 */
export interface ProtocolConsumptionReport {
    protocol_id: string
    protocol_no: string
    /** 核准編號。DRAFT 階段的計畫尚未取得，可為 null */
    iacuc_no: string | null
    protocol_title: string | null
    product_id: string
    product_sku: string
    product_name: string
    category_name: string | null
    base_uom: string
    /** 淨消耗量（已扣除沖銷），以 base_uom 計 */
    qty_base: string
    doc_count: number
    first_trx_date: string
    last_trx_date: string
    total_cost: string | null
}

export interface BloodTestCostReport {
    iacuc_no: string | null
    ear_tag: string
    animal_id: string
    test_date: string
    lab_name: string | null
    item_count: number
    total_cost: string | null
    created_by_name: string | null
    created_at: string
}
