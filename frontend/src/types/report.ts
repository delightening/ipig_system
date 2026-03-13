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
