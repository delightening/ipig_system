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

/// 盤點範圍的內部一致性檢查。
///
/// 呼叫時機在 JSON 解析成功之後——形狀對了不代表意思對。這裡擋的是**形狀合法但自相矛盾**
/// 的範圍，理由與上游那段「解析失敗不可 `.ok()` 靜默降級」完全相同：
/// 使用者拿到的底稿與他要求的範圍不一致，而畫面上不會有任何跡象，盤完才發現。
///
/// 四條規則，各自對應一種「靜默做出使用者沒要求的事」：
///
/// 1. `scope_type` 只能是 `full` / `partial`。看似多餘（下面兩條才是實質檢查），
///    但少了它，第 2 條就有洞：`"fulll"` 這種 typo 會落進「不是 full」而被當成
///    `partial` 放行，使用者本意是全盤、實際拿到篩選過的底稿。
/// 2. `full` 不得帶任何非空篩選。`generate_stocktake_lines` 從來不讀 `scope_type`
///    ——它只看 `product_ids` / `category_codes`——所以
///    `{"scope_type":"full","category_codes":["DRG"]}` 目前會成功且產生**部分**盤點。
///    這是 CodeRabbit 在 PR #37 指出的矛盾（Minor），成立。
///    修法選「拒絕」而非「full 時忽略篩選」：後者等於把矛盾靜默解讀成全盤，
///    正是本檔要消滅的那種行為。
/// 3. `warehouse_ids` 非空一律拒絕。這個欄位**從未被實作**：底稿只依函式參數的
///    `warehouse_id` 產生，帶了會被完全忽略。與其讓呼叫端以為自己指定了跨倉範圍、
///    拿到的卻是單倉底稿，不如明說尚未支援。
/// 4. `partial` 必須真的給出篩選。第 2 條的鏡像：宣告「只盤一部分」卻一個條件都沒給，
///    底稿會是全盤——同樣是宣告與結果不符，只是方向相反。前端的 `buildStocktakeScope`
///    保證空清單一定送 `full`，所以這條擋的是直接打 API 的呼叫端。
fn validate_scope(scope: &StocktakeScope) -> Result<()> {
    const FULL: &str = "full";
    const PARTIAL: &str = "partial";

    if scope.scope_type != FULL && scope.scope_type != PARTIAL {
        return Err(AppError::Validation(format!(
            "盤點範圍格式錯誤：scope_type 只能是 {FULL} 或 {PARTIAL}，收到「{}」",
            scope.scope_type
        )));
    }

    let non_empty = |v: &Option<Vec<String>>| v.as_ref().is_some_and(|x| !x.is_empty());
    let non_empty_uuid = |v: &Option<Vec<Uuid>>| v.as_ref().is_some_and(|x| !x.is_empty());

    if non_empty_uuid(&scope.warehouse_ids) {
        return Err(AppError::Validation(
            "盤點範圍格式錯誤：warehouse_ids 尚未支援，底稿只會依單據本身的倉庫產生；\
             請改用單據的 warehouse_id，不要在範圍裡指定倉庫。"
                .to_string(),
        ));
    }

    let has_filter = non_empty(&scope.category_codes) || non_empty_uuid(&scope.product_ids);

    if scope.scope_type == FULL && has_filter {
        return Err(AppError::Validation(
            "盤點範圍格式錯誤：scope_type=full（全盤）不可同時指定 category_codes 或 product_ids。\
             要限定範圍請改用 scope_type=partial，要全盤請把篩選清空。"
                .to_string(),
        ));
    }

    if scope.scope_type == PARTIAL && !has_filter {
        return Err(AppError::Validation(
            "盤點範圍格式錯誤：scope_type=partial（循環盤點）必須指定 category_codes 或 product_ids，\
             否則底稿會是全盤——與「只盤一部分」的宣告不符。要全盤請改用 scope_type=full。"
                .to_string(),
        ));
    }

    Ok(())
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
    /// 解析並驗證盤點範圍，回傳解析後的結果（未指定時為 `None`）。
    ///
    /// **必須在「要不要自動產生底稿」那個分支之前呼叫。** 盤點單允許呼叫端自帶明細，
    /// 那條路徑不會經過 `generate_stocktake_lines`，但 `stocktake_scope` 仍會被原樣
    /// 寫進單據（`crud.rs` 的 INSERT）——驗證掛在產生底稿裡的話，
    /// `{"lines":[…],"stocktake_scope":1}` 就會安靜地把一個形狀非法的值存進資料庫，
    /// 而同一個欄位走另一條路是會報錯的。同一份資料不該因為走哪條路而有兩套規則。
    /// （CodeRabbit 於 PR #37 指出，Minor。）
    ///
    /// `None` 與 JSON `null` 都是「不限範圍」，不是錯誤。
    pub(crate) fn parse_and_validate_stocktake_scope(
        scope: &Option<serde_json::Value>,
    ) -> Result<Option<StocktakeScope>> {
        // 解析失敗一律報錯，**不可 `.ok()` 靜默降級成全盤**：盤點範圍是「這張單要盤什麼」的
        // 唯一依據，形狀寫錯而默默全盤，使用者只會看到一份比預期長的底稿，
        // 不會知道自己的篩選被丟掉了——盤完才發現等於白盤一次。
        let parsed: Option<StocktakeScope> = match scope {
            None => None,
            Some(v) if v.is_null() => None,
            Some(v) => Some(
                serde_json::from_value(v.clone())
                    .map_err(|e| AppError::Validation(format!("盤點範圍格式錯誤：{e}")))?,
            ),
        };

        if let Some(ref s) = parsed {
            validate_scope(s)?;
        }

        Ok(parsed)
    }

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

        // `crud.rs` 在分支前已經驗過一次；這裡再解析一次是為了讓本函式自身完備
        // （驗證是純函式、無副作用，重跑不影響結果），不依賴呼叫端記得先驗。
        let scope = Self::parse_and_validate_stocktake_scope(scope)?;

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
