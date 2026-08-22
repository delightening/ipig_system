use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{
    models::{DocumentLineInput, StocktakeScope},
    AppError, Result,
};

use super::DocumentService;

/// 盤點底稿：某貨架上某品項（含批號/效期維度）的系統現存量。
#[derive(sqlx::FromRow)]
struct StocktakeShelfRow {
    storage_location_id: Uuid,
    product_id: Uuid,
    base_uom: String,
    batch_no: Option<String>,
    expiry_date: Option<NaiveDate>,
    on_hand_qty: Decimal,
}

impl DocumentService {
    /// 根據盤點範圍生成盤點項目（**貨架層級**）。
    ///
    /// 每個 (儲位 × 品項 × 批號 × 效期) 在 `storage_location_inventory` 有現存量者
    /// 各產生一行，`storage_location_id` 帶該貨架、`qty` 帶系統現存量，供現場逐貨架點數。
    /// 貨架層級確保盤點差異 ADJ 能綁儲位、不產生倉庫層級未分配 drift（杜絕未分配政策）。
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

        let has_product_ids = product_ids.as_ref().is_some_and(|ids| !ids.is_empty());
        let has_category_codes = category_codes
            .as_ref()
            .is_some_and(|codes| !codes.is_empty());

        // 逐貨架現存量（含批號/效期維度），作為盤點點數底稿
        let rows: Vec<StocktakeShelfRow> = sqlx::query_as(
            r#"
            SELECT
                sli.storage_location_id,
                p.id as product_id,
                p.base_uom,
                sli.batch_no,
                sli.expiry_date,
                sli.on_hand_qty
            FROM storage_location_inventory sli
            JOIN storage_locations sl ON sli.storage_location_id = sl.id
            JOIN products p ON sli.product_id = p.id
            WHERE sl.warehouse_id = $1
              AND sl.is_active = true
              AND p.is_active = true
              AND sli.on_hand_qty > 0
              AND ($2::bool = false OR p.id = ANY($3))
              AND ($4::bool = false OR p.category_code = ANY($5))
            ORDER BY sl.code, p.sku, sli.batch_no
            "#,
        )
        .bind(warehouse_id)
        .bind(has_product_ids)
        .bind(product_ids.unwrap_or_default().as_slice())
        .bind(has_category_codes)
        .bind(category_codes.unwrap_or_default().as_slice())
        .fetch_all(&mut **tx)
        .await?;

        let lines: Vec<DocumentLineInput> = rows
            .into_iter()
            .map(|r| DocumentLineInput {
                product_id: r.product_id,
                qty: r.on_hand_qty,
                uom: r.base_uom,
                unit_price: None,
                batch_no: r.batch_no,
                expiry_date: r.expiry_date,
                remark: Some("系統庫存".to_string()),
                storage_location_id: Some(r.storage_location_id),
                // STK 不涉及 transfer，from/to 永遠 None（migration 069）
                storage_location_from_id: None,
                storage_location_to_id: None,
            })
            .collect();

        Ok(lines)
    }
}
