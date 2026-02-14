use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{
    models::{DocumentLineInput, StocktakeScope},
    AppError, Result,
};

use super::DocumentService;

impl DocumentService {
    /// 根據盤點範圍生成盤點項目
    pub(crate) async fn generate_stocktake_lines(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        warehouse_id: Option<Uuid>,
        scope: &Option<serde_json::Value>,
    ) -> Result<Vec<DocumentLineInput>> {
        let warehouse_id = warehouse_id.ok_or_else(|| AppError::BusinessRule("Warehouse is required for stocktake".to_string()))?;

        // 解析範圍設定
        let scope: Option<StocktakeScope> = if let Some(ref scope_json) = scope {
            serde_json::from_value(scope_json.clone()).ok()
        } else {
            None
        };

        // 使用 StockService 查詢庫存現況
        use crate::models::InventoryQuery;
        let _query = InventoryQuery {
            warehouse_id: Some(warehouse_id),
            product_id: None,
            keyword: None,
            batch_no: None,
            low_stock_only: None,
        };
        
        // 由於我們在事務中，需要直接查詢
        let inventory_items: Vec<(Uuid, String, Decimal)> = sqlx::query_as(
            r#"
            SELECT 
                p.id as product_id,
                p.base_uom,
                COALESCE(SUM(
                    CASE 
                        WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
                        WHEN sl.direction IN ('out', 'transfer_out', 'adjust_out') THEN -sl.qty_base
                        ELSE 0
                    END
                ), 0) as qty_on_hand
            FROM products p
            LEFT JOIN stock_ledger sl ON p.id = sl.product_id AND sl.warehouse_id = $1
            WHERE p.is_active = true
            GROUP BY p.id, p.base_uom
            HAVING COALESCE(SUM(
                CASE 
                    WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
                    WHEN sl.direction IN ('out', 'transfer_out', 'adjust_out') THEN -sl.qty_base
                    ELSE 0
                END
            ), 0) != 0
            ORDER BY p.sku
            "#
        )
        .bind(warehouse_id)
        .fetch_all(&mut **tx)
        .await?;

        // 根據範圍篩選
        let filtered_items: Vec<(Uuid, String, Decimal)> = if let Some(ref scope) = scope {
            inventory_items
                .into_iter()
                .filter(|(product_id, _, _)| {
                    // 如果指定了產品ID列表，只包含這些產品
                    if let Some(ref product_ids) = scope.product_ids {
                        if !product_ids.is_empty() && !product_ids.contains(product_id) {
                            return false;
                        }
                    }
                    // 如果指定了類別，需要查詢產品類別（這裡簡化處理，實際應該查詢）
                    // TODO: 如果需要按類別篩選，需要額外查詢產品類別
                    true
                })
                .collect()
        } else {
            inventory_items
        };

        // 轉換為 DocumentLineInput
        let lines: Vec<DocumentLineInput> = filtered_items
            .into_iter()
            .map(|(product_id, uom, qty)| DocumentLineInput {
                product_id,
                qty, // 系統庫存數量作為初始值
                uom,
                unit_price: None,
                batch_no: None,
                expiry_date: None,
                remark: Some("系統庫存".to_string()),
                storage_location_id: None,
            })
            .collect();

        Ok(lines)
    }
}
