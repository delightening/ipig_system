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

/// 盤點範圍的內部一致性檢查。
///
/// 呼叫時機在 JSON 解析成功之後——形狀對了不代表意思對。這裡擋的是**形狀合法但自相矛盾**
/// 的範圍，理由與上游那段「解析失敗不可 `.ok()` 靜默降級」完全相同：
/// 使用者拿到的底稿與他要求的範圍不一致，而畫面上不會有任何跡象，盤完才發現。
///
/// 三條規則，各自對應一種「靜默做出使用者沒要求的事」：
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

    if scope.scope_type == FULL
        && (non_empty(&scope.category_codes) || non_empty_uuid(&scope.product_ids))
    {
        return Err(AppError::Validation(
            "盤點範圍格式錯誤：scope_type=full（全盤）不可同時指定 category_codes 或 product_ids。\
             要限定範圍請改用 scope_type=partial，要全盤請把篩選清空。"
                .to_string(),
        ));
    }

    Ok(())
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

        // 解析失敗一律報錯，**不可 `.ok()` 靜默降級成全盤**：盤點範圍是「這張單要盤什麼」的
        // 唯一依據，形狀寫錯而默默全盤，使用者只會看到一份比預期長的底稿，
        // 不會知道自己的篩選被丟掉了——盤完才發現等於白盤一次。
        // `null` 與整個欄位缺席仍視為未指定（全盤），那是明確的「不限範圍」意圖。
        let scope: Option<StocktakeScope> = match scope {
            None => None,
            Some(v) if v.is_null() => None,
            Some(v) => Some(
                serde_json::from_value(v.clone())
                    .map_err(|e| AppError::Validation(format!("盤點範圍格式錯誤：{e}")))?,
            ),
        };

        if let Some(ref s) = scope {
            validate_scope(s)?;
        }

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
