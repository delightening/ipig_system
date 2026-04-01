/**
 * Dashboard widget demo data
 * 各 widget 的範例資料，guest 模式下使用
 */

export const DEMO_DASHBOARD_WIDGETS = {
  // 低庫存警示
  lowStockAlerts: [] as { product_name: string; sku: string; on_hand: number; safety_stock: number }[],

  // 最近單據
  recentDocuments: [] as { id: string; doc_no: string; doc_type: string; status: string; created_at: string }[],

  // 今日已核准單據數
  todayApprovedDocs: 0,
}
