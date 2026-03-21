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
        let warehouse_id = warehouse_id.ok_or_else(|| {
            AppError::BusinessRule("Warehouse is required for stocktake".to_string())
        })?;

        let scope: Option<StocktakeScope> = if let Some(ref scope_json) = scope {
            serde_json::from_value(scope_json.clone()).ok()
        } else {
            None
        };

        let (product_ids, category_codes) = match scope {
            Some(ref s) => (s.product_ids.clone(), s.category_codes.clone()),
            None => (None, None),
        };

        let has_product_ids =
            product_ids.as_ref().is_some_and(|ids| !ids.is_empty());
        let has_category_codes =
            category_codes.as_ref().is_some_and(|codes| !codes.is_empty());

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
              AND ($2::bool = false OR p.id = ANY($3))
              AND ($4::bool = false OR p.category_code = ANY($5))
            GROUP BY p.id, p.base_uom
            HAVING COALESCE(SUM(
                CASE
                    WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
                    WHEN sl.direction IN ('out', 'transfer_out', 'adjust_out') THEN -sl.qty_base
                    ELSE 0
                END
            ), 0) != 0
            ORDER BY p.sku
            "#,
        )
        .bind(warehouse_id)
        .bind(has_product_ids)
        .bind(product_ids.unwrap_or_default().as_slice())
        .bind(has_category_codes)
        .bind(category_codes.unwrap_or_default().as_slice())
        .fetch_all(&mut **tx)
        .await?;

        let lines: Vec<DocumentLineInput> = inventory_items
            .into_iter()
            .map(|(product_id, uom, qty)| DocumentLineInput {
                product_id,
                qty,
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
