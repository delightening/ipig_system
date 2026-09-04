/**
 * Guest Demo Mode — 報表中心模組靜態範例資料（batch 2）
 *
 * 涵蓋報表中心可見頁：
 *   庫存彙總 / 庫存流水 / 進貨明細 / 銷貨明細 / 成本彙總 /
 *   血檢成本 / 血檢分析 / 進銷彙總 / 週報 / 會計報表 / 副產物月報。
 *
 * 全部為虛構示範資料，沿用專案既有 guest demo 風格
 * （`範例…` 名稱、`demo-` 前綴 UUID、`AUP-2025-001` / `IACUC-2025-001` 編號、2025 年日期）。
 * 產品 / 倉庫 / 動物命名與 `./erp`、`./animals` 對齊（範例主倉庫 W001、範例藥品 A 等）。
 *
 * 型別來源：`@/types/report`、`@/types/accounting`、`@/types/animal`、
 * `@/lib/api/weeklyMedicalReport`、`@/lib/api/byproductMonthlyReport`。
 */
import type {
  StockOnHandReport,
  StockLedgerReport,
  PurchaseLinesReport,
  ProtocolConsumptionReport,
  SalesLinesReport,
  CostSummaryReport,
  PurchaseSalesMonthlySummary,
  PurchaseSalesPartnerSummary,
  PurchaseSalesCategorySummary,
  ProfitLossSummary,
  BloodTestCostReport,
} from '@/types/report'
import type {
  TrialBalanceRow,
  JournalEntryResponse,
  ApAgingRow,
  ArAgingRow,
} from '@/types/accounting'
import type { BloodTestAnalysisRow } from '@/types/animal'
import type { MedicalTimelineEvent } from '@/lib/api/weeklyMedicalReport'
import type { ByproductMonthlyRow } from '@/lib/api/byproductMonthlyReport'

const WAREHOUSE = { id: 'demo-w001', code: 'W001', name: '範例主倉庫' }

// ============================================================
// 庫存現況報表 — GET /reports/stock-on-hand（StockOnHandReport[]）
// ============================================================

export const DEMO_STOCK_ON_HAND: StockOnHandReport[] = [
  {
    warehouse_id: WAREHOUSE.id, warehouse_code: WAREHOUSE.code, warehouse_name: WAREHOUSE.name,
    product_id: 'demo-prod3', product_sku: 'FED-001', product_name: '範例飼料 C',
    category_name: '飼料', base_uom: '袋',
    qty_on_hand: '38', avg_cost: '850.00', total_value: '32300.00',
    safety_stock: '20', reorder_point: '50',
  },
  {
    warehouse_id: WAREHOUSE.id, warehouse_code: WAREHOUSE.code, warehouse_name: WAREHOUSE.name,
    product_id: 'demo-prod1', product_sku: 'MED-001', product_name: '範例藥品 A',
    category_name: '藥品', base_uom: '盒',
    qty_on_hand: '28', avg_cost: '500.00', total_value: '14000.00',
    safety_stock: '10', reorder_point: '20',
  },
  {
    warehouse_id: WAREHOUSE.id, warehouse_code: WAREHOUSE.code, warehouse_name: WAREHOUSE.name,
    product_id: 'demo-prod2', product_sku: 'SUP-001', product_name: '範例耗材 B',
    category_name: '耗材', base_uom: '瓶',
    qty_on_hand: '22', avg_cost: '120.00', total_value: '2640.00',
    safety_stock: '5', reorder_point: '15',
  },
  {
    warehouse_id: WAREHOUSE.id, warehouse_code: WAREHOUSE.code, warehouse_name: WAREHOUSE.name,
    product_id: 'demo-prod4', product_sku: 'MED-002', product_name: '範例藥品 D',
    category_name: '藥品', base_uom: '瓶',
    qty_on_hand: '12', avg_cost: '300.00', total_value: '3600.00',
    safety_stock: '8', reorder_point: '16',
  },
  {
    warehouse_id: WAREHOUSE.id, warehouse_code: WAREHOUSE.code, warehouse_name: WAREHOUSE.name,
    product_id: 'demo-prod5', product_sku: 'SUP-002', product_name: '範例耗材 E',
    category_name: '耗材', base_uom: '組',
    qty_on_hand: '5', avg_cost: '640.00', total_value: '3200.00',
    safety_stock: '10', reorder_point: '20',
  },
]

// ============================================================
// 庫存流水報表 — GET /reports/stock-ledger（StockLedgerReport[]）
// ============================================================

export const DEMO_STOCK_LEDGER: StockLedgerReport[] = [
  {
    doc_id: 'demo-doc1',
    trx_date: '2025-01-06T09:12:00Z', warehouse_code: WAREHOUSE.code, warehouse_name: WAREHOUSE.name,
    product_sku: 'FED-001', product_name: '範例飼料 C',
    doc_type: 'GRN', doc_no: 'GRN-2025-0001', direction: 'in',
    qty_base: '30', unit_cost: '850.00', batch_no: 'LOT-2025-001', expiry_date: '2026-06-30',
  },
  {
    doc_id: 'demo-doc2',
    trx_date: '2025-01-08T10:05:00Z', warehouse_code: WAREHOUSE.code, warehouse_name: WAREHOUSE.name,
    product_sku: 'MED-001', product_name: '範例藥品 A',
    doc_type: 'GRN', doc_no: 'GRN-2025-0002', direction: 'in',
    qty_base: '20', unit_cost: '500.00', batch_no: 'MED-LOT-2025-01', expiry_date: '2026-12-31',
  },
  {
    doc_id: 'demo-doc11',
    trx_date: '2025-01-15T14:20:00Z', warehouse_code: WAREHOUSE.code, warehouse_name: WAREHOUSE.name,
    product_sku: 'SUP-001', product_name: '範例耗材 B',
    doc_type: 'SO', doc_no: 'SO-2025-0001', direction: 'out',
    qty_base: '6', unit_cost: '120.00',
  },
  {
    doc_id: 'demo-doc12',
    trx_date: '2025-02-03T11:40:00Z', warehouse_code: WAREHOUSE.code, warehouse_name: WAREHOUSE.name,
    product_sku: 'FED-001', product_name: '範例飼料 C',
    doc_type: 'SO', doc_no: 'SO-2025-0002', direction: 'out',
    qty_base: '12', unit_cost: '850.00', batch_no: 'LOT-2025-001',
  },
  {
    doc_id: 'demo-doc5',
    trx_date: '2025-02-20T16:00:00Z', warehouse_code: WAREHOUSE.code, warehouse_name: WAREHOUSE.name,
    product_sku: 'MED-001', product_name: '範例藥品 A',
    doc_type: 'ADJ', doc_no: 'ADJ-2025-0001', direction: 'adjust_out',
    qty_base: '2', unit_cost: '500.00', batch_no: 'MED-LOT-2025-01',
  },
  {
    doc_id: 'demo-doc6',
    trx_date: '2025-03-05T08:55:00Z', warehouse_code: WAREHOUSE.code, warehouse_name: WAREHOUSE.name,
    product_sku: 'SUP-001', product_name: '範例耗材 B',
    doc_type: 'GRN', doc_no: 'GRN-2025-0003', direction: 'in',
    qty_base: '20', unit_cost: '120.00',
  },
]

// ============================================================
// 案件消耗報表 — GET /reports/protocol-consumption（ProtocolConsumptionReport[]）
// ============================================================
// 刻意做成「兩個案件 × 三個品項」但只有 5 組有值，讓交叉表分頁在 demo 裡
// 就能看到稀疏格（顯示「—」）而不是滿版數字。

export const DEMO_PROTOCOL_CONSUMPTION: ProtocolConsumptionReport[] = [
  {
    protocol_id: 'demo-proto1', protocol_no: 'P-2025-001',
    iacuc_no: 'IACUC-2025-001', protocol_title: '範例研究計畫 A',
    product_id: 'demo-prod1', product_sku: 'MED-001', product_name: '範例藥品 A',
    category_name: '藥品', base_uom: '盒', qty_base: '12', doc_count: 3,
    first_trx_date: '2025-01-10T02:00:00Z', last_trx_date: '2025-03-18T06:30:00Z',
    total_cost: '6000.00',
  },
  {
    protocol_id: 'demo-proto1', protocol_no: 'P-2025-001',
    iacuc_no: 'IACUC-2025-001', protocol_title: '範例研究計畫 A',
    product_id: 'demo-prod2', product_sku: 'SUP-001', product_name: '範例耗材 B',
    category_name: '耗材', base_uom: '包', qty_base: '40', doc_count: 5,
    first_trx_date: '2025-01-12T01:00:00Z', last_trx_date: '2025-04-02T08:15:00Z',
    total_cost: '4800.00',
  },
  {
    protocol_id: 'demo-proto1', protocol_no: 'P-2025-001',
    iacuc_no: 'IACUC-2025-001', protocol_title: '範例研究計畫 A',
    product_id: 'demo-prod3', product_sku: 'CON-GLV-001', product_name: '範例手套',
    category_name: '耗材', base_uom: '雙', qty_base: '250', doc_count: 4,
    first_trx_date: '2025-02-01T03:00:00Z', last_trx_date: '2025-04-20T05:00:00Z',
    total_cost: '1250.00',
  },
  {
    // 尚未取得核准編號的計畫：iacuc_no 為 null，畫面應顯示「—」而不是空白或崩掉
    protocol_id: 'demo-proto2', protocol_no: 'P-2025-002',
    iacuc_no: null, protocol_title: '範例研究計畫 B（審查中）',
    product_id: 'demo-prod2', product_sku: 'SUP-001', product_name: '範例耗材 B',
    category_name: '耗材', base_uom: '包', qty_base: '15', doc_count: 2,
    first_trx_date: '2025-03-05T02:30:00Z', last_trx_date: '2025-03-28T07:45:00Z',
    total_cost: '1800.00',
  },
  {
    protocol_id: 'demo-proto2', protocol_no: 'P-2025-002',
    iacuc_no: null, protocol_title: '範例研究計畫 B（審查中）',
    product_id: 'demo-prod3', product_sku: 'CON-GLV-001', product_name: '範例手套',
    category_name: '耗材', base_uom: '雙', qty_base: '80', doc_count: 1,
    first_trx_date: '2025-03-06T02:00:00Z', last_trx_date: '2025-03-06T02:00:00Z',
    total_cost: '400.00',
  },
]

// ============================================================
// 採購明細報表 — GET /reports/purchase-lines（PurchaseLinesReport[]）
// ============================================================

export const DEMO_PURCHASE_LINES: PurchaseLinesReport[] = [
  {
    doc_id: 'demo-doc7',
    doc_date: '2025-01-06', doc_no: 'PO-2025-0001', status: 'approved',
    partner_code: 'SUP-A', partner_name: '範例供應商 A', warehouse_name: WAREHOUSE.name,
    product_sku: 'FED-001', product_name: '範例飼料 C',
    qty: '30', uom: '袋', unit_price: '850.00', line_total: '25500.00',
    created_by_name: '範例採購員', approved_by_name: '範例主管',
  },
  {
    doc_id: 'demo-doc8',
    doc_date: '2025-01-08', doc_no: 'PO-2025-0002', status: 'approved',
    partner_code: 'SUP-A', partner_name: '範例供應商 A', warehouse_name: WAREHOUSE.name,
    product_sku: 'MED-001', product_name: '範例藥品 A',
    qty: '20', uom: '盒', unit_price: '500.00', line_total: '10000.00',
    created_by_name: '範例採購員', approved_by_name: '範例主管',
  },
  {
    doc_id: 'demo-doc9',
    doc_date: '2025-02-14', doc_no: 'PO-2025-0003', status: 'submitted',
    partner_code: 'SUP-B', partner_name: '範例供應商 B', warehouse_name: WAREHOUSE.name,
    product_sku: 'SUP-001', product_name: '範例耗材 B',
    qty: '20', uom: '瓶', unit_price: '120.00', line_total: '2400.00',
    created_by_name: '範例採購員',
  },
  {
    doc_id: 'demo-doc10',
    doc_date: '2025-03-05', doc_no: 'PO-2025-0004', status: 'draft',
    partner_code: 'SUP-B', partner_name: '範例供應商 B', warehouse_name: WAREHOUSE.name,
    product_sku: 'MED-002', product_name: '範例藥品 D',
    qty: '12', uom: '瓶', unit_price: '300.00', line_total: '3600.00',
    created_by_name: '範例採購員',
  },
]

// ============================================================
// 銷貨明細報表 — GET /reports/sales-lines（SalesLinesReport[]）
// ============================================================

export const DEMO_SALES_LINES: SalesLinesReport[] = [
  {
    doc_id: 'demo-doc11',
    doc_date: '2025-02-10', doc_no: 'SO-2025-0001', status: 'approved',
    partner_code: 'CUS-A', partner_name: '範例客戶甲', customer_category: 'external',
    warehouse_name: WAREHOUSE.name, product_sku: 'FED-001', product_name: '範例飼料 C',
    qty: '5', uom: '袋', unit_price: '1000.00', line_total: '5000.00',
    created_by_name: '範例業務', approved_by_name: '範例主管',
  },
  {
    doc_id: 'demo-doc12',
    doc_date: '2025-02-18', doc_no: 'SO-2025-0002', status: 'approved',
    partner_code: 'CUS-B', partner_name: '範例研究單位乙', customer_category: 'research',
    warehouse_name: WAREHOUSE.name, product_sku: 'MED-001', product_name: '範例藥品 A',
    qty: '3', uom: '盒', unit_price: '650.00', line_total: '1950.00',
    created_by_name: '範例業務', approved_by_name: '範例主管',
  },
  {
    doc_id: 'demo-doc13',
    doc_date: '2025-03-01', doc_no: 'SO-2025-0003', status: 'submitted',
    partner_code: 'CUS-C', partner_name: '範例內部單位丙', customer_category: 'internal',
    warehouse_name: WAREHOUSE.name, product_sku: 'SUP-001', product_name: '範例耗材 B',
    qty: '4', uom: '瓶', unit_price: '180.00', line_total: '720.00',
    created_by_name: '範例業務',
  },
  {
    doc_id: 'demo-doc14',
    doc_date: '2025-03-12', doc_no: 'SO-2025-0004', status: 'approved',
    partner_code: 'CUS-A', partner_name: '範例客戶甲', customer_category: 'external',
    warehouse_name: WAREHOUSE.name, product_sku: 'FED-001', product_name: '範例飼料 C',
    qty: '8', uom: '袋', unit_price: '1000.00', line_total: '8000.00',
    created_by_name: '範例業務', approved_by_name: '範例主管',
  },
]

// ============================================================
// 成本摘要報表 — GET /reports/cost-summary（CostSummaryReport[]）
// ============================================================

export const DEMO_COST_SUMMARY: CostSummaryReport[] = [
  {
    warehouse_id: WAREHOUSE.id, warehouse_code: WAREHOUSE.code, warehouse_name: WAREHOUSE.name,
    product_id: 'demo-prod3', product_sku: 'FED-001', product_name: '範例飼料 C',
    category_name: '飼料', qty_on_hand: '38', avg_cost: '850.00', total_value: '32300.00',
  },
  {
    warehouse_id: WAREHOUSE.id, warehouse_code: WAREHOUSE.code, warehouse_name: WAREHOUSE.name,
    product_id: 'demo-prod1', product_sku: 'MED-001', product_name: '範例藥品 A',
    category_name: '藥品', qty_on_hand: '28', avg_cost: '500.00', total_value: '14000.00',
  },
  {
    warehouse_id: WAREHOUSE.id, warehouse_code: WAREHOUSE.code, warehouse_name: WAREHOUSE.name,
    product_id: 'demo-prod2', product_sku: 'SUP-001', product_name: '範例耗材 B',
    category_name: '耗材', qty_on_hand: '22', avg_cost: '120.00', total_value: '2640.00',
  },
  {
    warehouse_id: WAREHOUSE.id, warehouse_code: WAREHOUSE.code, warehouse_name: WAREHOUSE.name,
    product_id: 'demo-prod4', product_sku: 'MED-002', product_name: '範例藥品 D',
    category_name: '藥品', qty_on_hand: '12', avg_cost: '300.00', total_value: '3600.00',
  },
  {
    warehouse_id: WAREHOUSE.id, warehouse_code: WAREHOUSE.code, warehouse_name: WAREHOUSE.name,
    product_id: 'demo-prod5', product_sku: 'SUP-002', product_name: '範例耗材 E',
    category_name: '耗材', qty_on_hand: '5', avg_cost: '640.00', total_value: '3200.00',
  },
]

// ============================================================
// 進銷貨彙總報表 — 三個 tab
//   GET /reports/purchase-sales-monthly       （PurchaseSalesMonthlySummary[]）
//   GET /reports/purchase-sales-by-partner     （PurchaseSalesPartnerSummary[]）
//   GET /reports/purchase-sales-by-category    （PurchaseSalesCategorySummary[]）
// ============================================================

export const DEMO_PS_MONTHLY: PurchaseSalesMonthlySummary[] = [
  {
    year_month: '2025-01',
    purchase_total: '35500.00', purchase_return: '0.00', net_purchase: '35500.00',
    sales_total: '0.00', sales_return: '0.00', net_sales: '0.00',
    cogs_total: '0.00', gross_profit: '0.00',
  },
  {
    year_month: '2025-02',
    purchase_total: '2400.00', purchase_return: '0.00', net_purchase: '2400.00',
    sales_total: '6950.00', sales_return: '0.00', net_sales: '6950.00',
    cogs_total: '4200.00', gross_profit: '2750.00',
  },
  {
    year_month: '2025-03',
    purchase_total: '3600.00', purchase_return: '0.00', net_purchase: '3600.00',
    sales_total: '8720.00', sales_return: '0.00', net_sales: '8720.00',
    cogs_total: '5600.00', gross_profit: '3120.00',
  },
]

export const DEMO_PS_BY_PARTNER: PurchaseSalesPartnerSummary[] = [
  {
    partner_id: 'demo-partner1', partner_code: 'SUP-A', partner_name: '範例供應商 A',
    partner_type: 'supplier', total_amount: '35500.00', return_amount: '0.00',
    net_amount: '35500.00', doc_count: 2,
  },
  {
    partner_id: 'demo-partner2', partner_code: 'SUP-B', partner_name: '範例供應商 B',
    partner_type: 'supplier', total_amount: '6000.00', return_amount: '0.00',
    net_amount: '6000.00', doc_count: 2,
  },
  {
    partner_id: 'demo-cus-a', partner_code: 'CUS-A', partner_name: '範例客戶甲',
    partner_type: 'customer', total_amount: '13000.00', return_amount: '0.00',
    net_amount: '13000.00', doc_count: 2,
  },
  {
    partner_id: 'demo-cus-b', partner_code: 'CUS-B', partner_name: '範例研究單位乙',
    partner_type: 'customer', total_amount: '1950.00', return_amount: '0.00',
    net_amount: '1950.00', doc_count: 1,
  },
]

export const DEMO_PS_BY_CATEGORY: PurchaseSalesCategorySummary[] = [
  {
    category_name: '飼料', purchase_amount: '25500.00', sales_amount: '13000.00',
    cogs_amount: '8500.00', gross_profit: '4500.00',
  },
  {
    category_name: '藥品', purchase_amount: '13600.00', sales_amount: '1950.00',
    cogs_amount: '1200.00', gross_profit: '750.00',
  },
  {
    category_name: '耗材', purchase_amount: '2400.00', sales_amount: '720.00',
    cogs_amount: '480.00', gross_profit: '240.00',
  },
]

// ============================================================
// 血液檢查費用報表 — GET /reports/blood-test-cost（BloodTestCostReport[]）
// ============================================================

export const DEMO_BLOOD_TEST_COST: BloodTestCostReport[] = [
  {
    iacuc_no: 'IACUC-2025-001', ear_tag: 'D-001', animal_id: 'demo-a1',
    test_date: '2025-02-14', lab_name: '範例檢驗中心', item_count: 8,
    total_cost: '3200.00', created_by_name: '範例獸醫', created_at: '2025-02-14T09:00:00Z',
  },
  {
    iacuc_no: 'IACUC-2025-001', ear_tag: 'D-002', animal_id: 'demo-a2',
    test_date: '2025-02-14', lab_name: '範例檢驗中心', item_count: 6,
    total_cost: '2400.00', created_by_name: '範例獸醫', created_at: '2025-02-14T09:05:00Z',
  },
  {
    iacuc_no: 'IACUC-2025-003', ear_tag: 'D-005', animal_id: 'demo-a5',
    test_date: '2025-03-05', lab_name: '範例動物醫院', item_count: 5,
    total_cost: '1800.00', created_by_name: '範例獸醫', created_at: '2025-03-05T10:30:00Z',
  },
  {
    iacuc_no: null, ear_tag: 'D-003', animal_id: 'demo-a3',
    test_date: '2025-03-20', lab_name: '範例檢驗中心', item_count: 4,
    total_cost: '1200.00', created_by_name: '範例技術員', created_at: '2025-03-20T13:15:00Z',
  },
]

// ============================================================
// 血液檢查結果分析 — GET /reports/blood-test-analysis（BloodTestAnalysisRow[]）
// 僅在使用者輸入篩選條件後才觸發（enabled: hasFilter）；含正常與異常值、跨兩次日期供趨勢圖。
// ============================================================

export const DEMO_BLOOD_TEST_ANALYSIS: BloodTestAnalysisRow[] = [
  {
    animal_id: 'demo-a1', ear_tag: 'D-001', iacuc_no: 'IACUC-2025-001',
    test_date: '2025-02-14', lab_name: '範例檢驗中心',
    item_name: '白血球 (WBC)', template_code: 'WBC', result_value: '12.5',
    result_unit: '10^3/µL', reference_range: '6.0-16.0', is_abnormal: false,
  },
  {
    animal_id: 'demo-a1', ear_tag: 'D-001', iacuc_no: 'IACUC-2025-001',
    test_date: '2025-02-14', lab_name: '範例檢驗中心',
    item_name: '紅血球 (RBC)', template_code: 'RBC', result_value: '6.8',
    result_unit: '10^6/µL', reference_range: '5.0-8.0', is_abnormal: false,
  },
  {
    animal_id: 'demo-a1', ear_tag: 'D-001', iacuc_no: 'IACUC-2025-001',
    test_date: '2025-02-14', lab_name: '範例檢驗中心',
    item_name: '血紅素 (HGB)', template_code: 'HGB', result_value: '11.2',
    result_unit: 'g/dL', reference_range: '10.0-16.0', is_abnormal: false,
  },
  {
    animal_id: 'demo-a1', ear_tag: 'D-001', iacuc_no: 'IACUC-2025-001',
    test_date: '2025-02-14', lab_name: '範例檢驗中心',
    item_name: '丙胺酸轉胺酶 (ALT)', template_code: 'ALT', result_value: '92',
    result_unit: 'U/L', reference_range: '20-80', is_abnormal: true,
  },
  {
    animal_id: 'demo-a1', ear_tag: 'D-001', iacuc_no: 'IACUC-2025-001',
    test_date: '2025-03-14', lab_name: '範例檢驗中心',
    item_name: '白血球 (WBC)', template_code: 'WBC', result_value: '10.1',
    result_unit: '10^3/µL', reference_range: '6.0-16.0', is_abnormal: false,
  },
  {
    animal_id: 'demo-a1', ear_tag: 'D-001', iacuc_no: 'IACUC-2025-001',
    test_date: '2025-03-14', lab_name: '範例檢驗中心',
    item_name: '丙胺酸轉胺酶 (ALT)', template_code: 'ALT', result_value: '78',
    result_unit: 'U/L', reference_range: '20-80', is_abnormal: false,
  },
  {
    animal_id: 'demo-a2', ear_tag: 'D-002', iacuc_no: 'IACUC-2025-001',
    test_date: '2025-02-14', lab_name: '範例檢驗中心',
    item_name: '白血球 (WBC)', template_code: 'WBC', result_value: '18.3',
    result_unit: '10^3/µL', reference_range: '6.0-16.0', is_abnormal: true,
  },
  {
    animal_id: 'demo-a2', ear_tag: 'D-002', iacuc_no: 'IACUC-2025-001',
    test_date: '2025-02-14', lab_name: '範例檢驗中心',
    item_name: '血紅素 (HGB)', template_code: 'HGB', result_value: '9.1',
    result_unit: 'g/dL', reference_range: '10.0-16.0', is_abnormal: true,
  },
  {
    animal_id: 'demo-a5', ear_tag: 'D-005', iacuc_no: 'IACUC-2025-003',
    test_date: '2025-03-05', lab_name: '範例動物醫院',
    item_name: '白血球 (WBC)', template_code: 'WBC', result_value: '8.4',
    result_unit: '10^3/µL', reference_range: '6.0-16.0', is_abnormal: false,
  },
  {
    animal_id: 'demo-a5', ear_tag: 'D-005', iacuc_no: 'IACUC-2025-003',
    test_date: '2025-03-05', lab_name: '範例動物醫院',
    item_name: '丙胺酸轉胺酶 (ALT)', template_code: 'ALT', result_value: '45',
    result_unit: 'U/L', reference_range: '20-80', is_abnormal: false,
  },
]

// ============================================================
// 會計報表 — 五個 tab
//   GET /accounting/trial-balance    （TrialBalanceRow[]）
//   GET /accounting/journal-entries  （JournalEntryResponse[]）
//   GET /accounting/ap-aging          （ApAgingRow[]）
//   GET /accounting/ar-aging          （ArAgingRow[]）
//   GET /accounting/profit-loss       （ProfitLossSummary 物件）
// ============================================================

export const DEMO_TRIAL_BALANCE: TrialBalanceRow[] = [
  { account_id: 'demo-acc-1101', account_code: '1101', account_name: '現金', account_type: 'asset', debit_balance: '50000.00', credit_balance: '0.00' },
  { account_id: 'demo-acc-1201', account_code: '1201', account_name: '應收帳款', account_type: 'asset', debit_balance: '6950.00', credit_balance: '0.00' },
  { account_id: 'demo-acc-1301', account_code: '1301', account_name: '存貨', account_type: 'asset', debit_balance: '55740.00', credit_balance: '0.00' },
  { account_id: 'demo-acc-2101', account_code: '2101', account_name: '應付帳款', account_type: 'liability', debit_balance: '0.00', credit_balance: '11500.00' },
  { account_id: 'demo-acc-4101', account_code: '4101', account_name: '銷貨收入', account_type: 'revenue', debit_balance: '0.00', credit_balance: '15670.00' },
  { account_id: 'demo-acc-5101', account_code: '5101', account_name: '銷貨成本', account_type: 'expense', debit_balance: '9800.00', credit_balance: '0.00' },
]

export const DEMO_JOURNAL_ENTRIES: JournalEntryResponse[] = [
  {
    entry: {
      id: 'demo-je-1', entry_no: 'JE-2025-0001', entry_date: '2025-01-06',
      description: '採購入庫 GRN-2025-0001', source_entity_type: 'GRN', source_entity_id: 'demo-doc2',
    },
    lines: [
      { line_id: 'demo-jel-1a', line_no: 1, account_code: '1301', account_name: '存貨', debit_amount: '25500.00', credit_amount: '0.00', description: '範例飼料 C 入庫' },
      { line_id: 'demo-jel-1b', line_no: 2, account_code: '2101', account_name: '應付帳款', debit_amount: '0.00', credit_amount: '25500.00', description: '應付範例供應商 A' },
    ],
  },
  {
    entry: {
      id: 'demo-je-2', entry_no: 'JE-2025-0002', entry_date: '2025-02-10',
      description: '銷貨 SO-2025-0001', source_entity_type: 'SO', source_entity_id: 'demo-doc11',
    },
    lines: [
      { line_id: 'demo-jel-2a', line_no: 1, account_code: '1201', account_name: '應收帳款', debit_amount: '5000.00', credit_amount: '0.00', description: '應收範例客戶甲' },
      { line_id: 'demo-jel-2b', line_no: 2, account_code: '4101', account_name: '銷貨收入', debit_amount: '0.00', credit_amount: '5000.00', description: '範例飼料 C 銷貨' },
    ],
  },
]

export const DEMO_AP_AGING: ApAgingRow[] = [
  { partner_id: 'demo-partner1', partner_code: 'SUP-A', partner_name: '範例供應商 A', total_payable: '35500.00', total_paid: '30000.00', balance: '5500.00' },
  { partner_id: 'demo-partner2', partner_code: 'SUP-B', partner_name: '範例供應商 B', total_payable: '6000.00', total_paid: '0.00', balance: '6000.00' },
]

export const DEMO_AR_AGING: ArAgingRow[] = [
  { partner_id: 'demo-cus-a', partner_code: 'CUS-A', partner_name: '範例客戶甲', total_receivable: '13000.00', total_received: '8000.00', balance: '5000.00' },
  { partner_id: 'demo-cus-b', partner_code: 'CUS-B', partner_name: '範例研究單位乙', total_receivable: '1950.00', total_received: '0.00', balance: '1950.00' },
]

export const DEMO_PROFIT_LOSS: ProfitLossSummary = {
  rows: [
    { account_code: '4101', account_name: '銷貨收入', account_type: 'revenue', amount: '15670.00' },
    { account_code: '4102', account_name: '其他收入', account_type: 'revenue', amount: '2000.00' },
    { account_code: '5101', account_name: '銷貨成本', account_type: 'expense', amount: '9800.00' },
    { account_code: '6201', account_name: '管理費用', account_type: 'expense', amount: '3200.00' },
    { account_code: '6202', account_name: '用品耗材', account_type: 'expense', amount: '1500.00' },
  ],
  total_revenue: '17670.00',
  total_expense: '14500.00',
  net_income: '3170.00',
}

// ============================================================
// 豬隻病歷彙整週報 — POST /reports/animal-medical/weekly（MedicalTimelineEvent[]）
// 頁面以 POST 傳 filter（讀取語意），僅在點「查詢」後觸發。
// ============================================================

export const DEMO_WEEKLY_MEDICAL: MedicalTimelineEvent[] = [
  {
    animal_id: 'demo-a1', ear_tag: 'D-001', iacuc_no: 'IACUC-2025-001',
    event_date: '2025-02-10', event_type: 'OBSERVATION',
    summary: '每日觀察', details: '精神食慾正常，切口乾燥無紅腫。',
    actor_name: '範例獸醫', source_id: 'demo-ev-1', created_at: '2025-02-10T08:00:00Z',
    birth_date: '2024-09-01', latest_weight: 26.5,
    sponsor_name: '範例委託廠商 A',
    equipment_used: null, anesthesia_start: null, anesthesia_end: null,
  },
  {
    animal_id: 'demo-a1', ear_tag: 'D-001', iacuc_no: 'IACUC-2025-001',
    event_date: '2025-02-14', event_type: 'SURGERY',
    summary: '心導管置放術', details: '手術順利完成，術中生命徵象穩定。',
    actor_name: '範例外科醫師', source_id: 'demo-ev-2', created_at: '2025-02-14T12:00:00Z',
    birth_date: '2024-09-01', latest_weight: 27.8,
    sponsor_name: '範例委託廠商 A',
    equipment_used: '血管攝影機、生理監視器',
    anesthesia_start: '2025-02-14T09:00:00Z', anesthesia_end: '2025-02-14T11:30:00Z',
  },
  {
    animal_id: 'demo-a2', ear_tag: 'D-002', iacuc_no: 'IACUC-2025-001',
    event_date: '2025-02-14', event_type: 'BLOOD_TEST',
    summary: '術前血液檢查', details: 'CBC + 生化組合，白血球略偏高。',
    actor_name: '範例獸醫', source_id: 'demo-ev-3', created_at: '2025-02-14T08:30:00Z',
    birth_date: '2024-09-05', latest_weight: 25.0,
    sponsor_name: '範例委託廠商 A',
    equipment_used: null, anesthesia_start: null, anesthesia_end: null,
  },
  {
    animal_id: 'demo-a5', ear_tag: 'D-005', iacuc_no: 'IACUC-2025-003',
    event_date: '2025-03-05', event_type: 'OBSERVATION',
    summary: '每日觀察', details: '活動力佳，採食正常。',
    actor_name: '範例獸醫', source_id: 'demo-ev-4', created_at: '2025-03-05T08:10:00Z',
    birth_date: '2024-11-12', latest_weight: 22.4,
    sponsor_name: '範例委託廠商 B',
    equipment_used: null, anesthesia_start: null, anesthesia_end: null,
  },
]

// ============================================================
// 廢棄物再利用月結報表 — POST /reports/byproduct-monthly（ByproductMonthlyRow[]）
// 頁面以 POST 傳 filter（讀取語意），僅在點「查詢」後觸發。
// ============================================================

export const DEMO_BYPRODUCT_MONTHLY: ByproductMonthlyRow[] = [
  {
    id: 'demo-bp-1', sampled_at: '2025-03-05T10:00:00Z',
    iacuc_no: 'IACUC-2024-012', protocol_title: '範例研究計畫：組織再利用評估',
    ear_tag: 'D-004', requester_display: '範例研究單位（陳博士）',
    sample_content: '心臟組織切片', collector_name: '範例技術員',
  },
  {
    id: 'demo-bp-2', sampled_at: '2025-03-12T14:30:00Z',
    iacuc_no: 'IACUC-2025-001', protocol_title: '範例研究計畫：心血管藥物安全性評估',
    ear_tag: 'D-001', requester_display: '範例大學實驗室',
    sample_content: '主動脈血管段', collector_name: '範例獸醫',
  },
  {
    id: 'demo-bp-3', sampled_at: '2025-03-20T09:15:00Z',
    iacuc_no: null, protocol_title: null,
    ear_tag: 'D-003', requester_display: '範例生技公司',
    sample_content: '皮膚樣本', collector_name: '範例技術員',
  },
]
