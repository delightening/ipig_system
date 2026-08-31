use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{
    models::{DocumentLineInput, StocktakeScope},
    AppError, Result,
};

use super::DocumentService;

/// 盤點底稿：某貨架上某品項（含批號/效期維度）的系統現存量。
///
/// `on_hand_qty` 的單位恆為 `base_uom`（storage_location_inventory 一律以基本單位存放）。
/// `pack_unit` / `pack_factor` 供 `counting_uom` 決定底稿要以哪個單位呈現。
#[derive(sqlx::FromRow)]
struct StocktakeShelfRow {
    storage_location_id: Uuid,
    product_id: Uuid,
    base_uom: String,
    pack_unit: Option<String>,
    pack_factor: Option<Decimal>,
    batch_no: Option<String>,
    expiry_date: Option<NaiveDate>,
    on_hand_qty: Decimal,
}

/// 決定盤點底稿的呈現單位與數量。
///
/// 現場點數的痛點是「一盒五十雙」：整盒未拆時數盒最快，拆過的那盒只能數雙。
/// 因此規則是**整除才用包裝單位**——`on_hand_qty` 是 `pack_factor` 的整數倍時以
/// 盒呈現（162 雙不會變成 3.24 盒），否則退回 base_uom 逐一點數。
///
/// 回傳的 uom 一定是 base_uom 或該品項換算表已定義的 `pack_unit`，因此產生的盤點單
/// 通得過 `assert_lines_uom_defined`，核准時也換算得回 base（見 `uom::to_base_lines`）。
///
/// 🔴 `pack_unit == base_uom` 必須排除，否則會開出一張把貨架清空的盤虧 ADJ。
/// 該品項若又有一列 `uom = base_uom` 且 `factor_to_base = 50` 的壞換算資料：
/// 本函式會把 150 雙除成底稿上的「3 雙」，而核准時 `ProductUomTable::factor` 對
/// base_uom 恆回 1（它先判斷 `uom == base_uom`，不看換算表），於是 3 被當成 3 雙
/// 去減系統存量 150 雙，差異 −147。這正是本檔與 `workflow.rs` 註解警告的那個災難，
/// 只是觸發條件不是「缺換算列」而是「同名列」。
/// 兩端對 base_uom 的處理本來就不對稱（`factor()` 短路、SQL 不會），此處直接不進包裝分支。
fn counting_uom(row: &StocktakeShelfRow) -> (Decimal, String) {
    match (&row.pack_unit, row.pack_factor) {
        (Some(pack_unit), Some(factor))
            if pack_unit != &row.base_uom
                && factor > Decimal::ONE
                && (row.on_hand_qty % factor).is_zero() =>
        {
            (row.on_hand_qty / factor, pack_unit.clone())
        }
        _ => (row.on_hand_qty, row.base_uom.clone()),
    }
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
                p.pack_unit,
                c.factor_to_base as pack_factor,
                sli.batch_no,
                sli.expiry_date,
                sli.on_hand_qty
            FROM storage_location_inventory sli
            JOIN storage_locations sl ON sli.storage_location_id = sl.id
            JOIN products p ON sli.product_id = p.id
            -- 包裝單位須在換算表裡有 factor 才能用來點數；只填 products.pack_unit
            -- 而沒建換算列的品項，pack_factor 為 NULL → counting_uom 退回 base_uom。
            LEFT JOIN product_uom_conversions c
                   ON c.product_id = p.id AND c.uom = p.pack_unit
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
            .map(|r| {
                let (qty, uom) = counting_uom(&r);
                DocumentLineInput {
                    product_id: r.product_id,
                    qty,
                    uom,
                    unit_price: None,
                    batch_no: r.batch_no,
                    expiry_date: r.expiry_date,
                    remark: Some("系統庫存".to_string()),
                    storage_location_id: Some(r.storage_location_id),
                    // STK 不涉及 transfer，from/to 永遠 None（migration 069）
                    storage_location_from_id: None,
                    storage_location_to_id: None,
                }
            })
            .collect();

        Ok(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(on_hand: i64, pack_unit: Option<&str>, factor: Option<i64>) -> StocktakeShelfRow {
        StocktakeShelfRow {
            storage_location_id: Uuid::nil(),
            product_id: Uuid::nil(),
            base_uom: "雙".to_string(),
            pack_unit: pack_unit.map(str::to_string),
            pack_factor: factor.map(Decimal::from),
            batch_no: None,
            expiry_date: None,
            on_hand_qty: Decimal::from(on_hand),
        }
    }

    #[test]
    fn whole_boxes_are_counted_in_pack_unit() {
        // 150 雙 = 3 盒，整除 → 現場數 3 盒
        assert_eq!(
            counting_uom(&row(150, Some("盒"), Some(50))),
            (Decimal::from(3), "盒".to_string())
        );
    }

    #[test]
    fn partial_box_falls_back_to_base_uom() {
        // 162 雙不是 50 的倍數，不可呈現成 3.24 盒
        assert_eq!(
            counting_uom(&row(162, Some("盒"), Some(50))),
            (Decimal::from(162), "雙".to_string())
        );
    }

    #[test]
    fn pack_unit_without_a_conversion_row_falls_back_to_base_uom() {
        assert_eq!(
            counting_uom(&row(150, Some("盒"), None)),
            (Decimal::from(150), "雙".to_string())
        );
    }

    #[test]
    fn factor_of_one_is_not_treated_as_a_pack() {
        // factor = 1 的「包裝」等於沒有包裝，用它換算只會多一個名字不同的相同數字
        assert_eq!(
            counting_uom(&row(150, Some("個"), Some(1))),
            (Decimal::from(150), "雙".to_string())
        );
    }

    #[test]
    fn pack_unit_equal_to_base_uom_never_divides() {
        // 🔴 壞資料：pack_unit 與 base_uom 同名，且換算表給了 factor 50。
        // 若照除下去，底稿會寫「3 雙」；核准時 ProductUomTable::factor 對 base_uom
        // 恆回 1（先判斷 uom == base_uom，不看換算表），3 就被當 3 雙去減 150 雙，
        // 差異 −147 —— 一張把整個貨架清空的盤虧 ADJ。必須原封不動退回 base_uom。
        assert_eq!(
            counting_uom(&row(150, Some("雙"), Some(50))),
            (Decimal::from(150), "雙".to_string())
        );
    }
}
