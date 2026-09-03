mod crud;
mod import;
pub(crate) mod uom;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{CreateProductRequest, Product, ProductUomConversion, ProductWithUom},
    repositories, AppError, Result,
};

pub(crate) use uom::{canonical_uom, canonical_uom_opt, derive_pack_conversion};

/// 允許的產品狀態值（與 DB chk_product_status 一致）
const ALLOWED_STATUSES: [&str; 3] = ["active", "inactive", "discontinued"];

/// 格式化產品 SKU（分類-子分類-三位流水號），供建立與單元測試使用。
pub fn format_product_sku(category_code: &str, subcategory_code: &str, sequence: i32) -> String {
    format!("{}-{}-{:03}", category_code, subcategory_code, sequence)
}

/// 驗證並正規化產品狀態，與 DB chk_product_status 一致。
pub fn validate_product_status(status: &str) -> std::result::Result<String, String> {
    let s = status.trim().to_lowercase();
    if ALLOWED_STATUSES.contains(&s.as_str()) {
        Ok(s)
    } else {
        Err(format!("status 必須為: {}", ALLOWED_STATUSES.join(", ")))
    }
}

/// 從 request 中取得分類代碼，若未提供則使用預設值。
fn resolve_category_codes(req: &CreateProductRequest) -> (String, String) {
    let cat = req
        .category_code
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "GEN".to_string());
    let sub = req
        .subcategory_code
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "OTH".to_string());
    (cat, sub)
}

/// 解析 SKU：若請求有提供且非空則使用（並檢查唯一），否則自動生成。
/// tx 版本：在 tx 內執行 SELECT，避免在 tx 外讀出後 SKU 被併發搶註冊。
async fn resolve_sku_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    req_sku: Option<&str>,
    category_code: &str,
    subcategory_code: &str,
) -> Result<String> {
    if let Some(s) = req_sku.map(str::trim) {
        if !s.is_empty() {
            let exists: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM products WHERE sku = $1)")
                    .bind(s)
                    .fetch_one(&mut **tx)
                    .await?;
            if exists {
                return Err(AppError::Conflict("SKU already exists".to_string()));
            }
            return Ok(s.to_string());
        }
    }
    let sequence = get_next_sequence_tx(tx, category_code, subcategory_code).await?;
    Ok(format_product_sku(
        category_code,
        subcategory_code,
        sequence,
    ))
}

/// 取得下一個 SKU 流水號（tx 版本）。
pub(super) async fn get_next_sequence_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    category_code: &str,
    subcategory_code: &str,
) -> Result<i32> {
    let pattern = format!("{}-{}-___", category_code, subcategory_code);
    let max_seq: Option<i32> = sqlx::query_scalar(
        r#"
        SELECT MAX(CAST(SUBSTRING(sku FROM '\d{3}$') AS INTEGER))
        FROM products
        WHERE sku LIKE $1
        "#,
    )
    .bind(&pattern)
    .fetch_optional(&mut **tx)
    .await?
    .flatten();
    Ok(max_seq.unwrap_or(0) + 1)
}

/// 插入產品記錄至 DB（tx 版本）。
async fn insert_product_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    sku: &str,
    req: &CreateProductRequest,
    category_code: &str,
    subcategory_code: &str,
) -> Result<Product> {
    // 單位一律以正規形式落地（見 `uom` 模組）。前端的單位選單送的是 `EA`/`BX` 這類代碼，
    // 而既有品項的 base_uom 是中文；不在寫入點收斂，新品項會持續產出第二套慣例，
    // 之後任何依字串相等比對單位的 SQL（GRN 收貨對帳、盤點 JOIN）都會分裂成兩組。
    let base_uom = canonical_uom(&req.base_uom);
    if base_uom.is_empty() {
        return Err(AppError::Validation("base_uom 不得為空".to_string()));
    }
    let pack_unit = canonical_uom_opt(req.pack_unit.as_deref());

    let product = sqlx::query_as::<_, Product>(
        r#"
        INSERT INTO products (
            id, sku, name, spec, category_code, subcategory_code, base_uom,
            pack_unit, pack_qty, track_batch, track_expiry, default_expiry_days,
            safety_stock, safety_stock_uom, reorder_point, reorder_point_uom,
            cost_price, selling_price,
            barcode, image_url, license_no, storage_condition, tags, remark,
            is_active, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, true, NOW(), NOW())
        RETURNING *
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(sku)
    .bind(&req.name)
    .bind(&req.spec)
    .bind(category_code)
    .bind(subcategory_code)
    .bind(&base_uom)
    .bind(&pack_unit)
    .bind(req.pack_qty)
    .bind(req.track_batch)
    .bind(req.track_expiry)
    .bind(req.default_expiry_days)
    .bind(req.safety_stock)
    .bind(&req.safety_stock_uom)
    .bind(req.reorder_point)
    .bind(&req.reorder_point_uom)
    .bind(req.cost_price)
    .bind(req.selling_price)
    .bind(&req.barcode)
    .bind(&req.image_url)
    .bind(&req.license_no)
    .bind(&req.storage_condition)
    .bind(&req.tags)
    .bind(&req.remark)
    .fetch_one(&mut **tx)
    .await?;
    Ok(product)
}

/// 批次建立單位換算記錄（tx 版本）。
///
/// **驗證寫在函式內部而不是呼叫端**：換算表的髒資料（同名列、非正數 factor）代價是
/// 盤點開出清空貨架的盤虧 ADJ、GRN 把正確收貨判成超收，不能靠每個呼叫端記得先驗。
/// 一併把包裝關係推導出的那一列併進來，讓 `products.pack_unit/pack_qty` 與換算表
/// 在寫入的當下就一致（見 `uom::merge_for_write`）。
pub(crate) async fn insert_uom_conversions_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    product_id: Uuid,
    base_uom: &str,
    explicit: &[crate::models::UomConversionInput],
    derived: Option<crate::models::UomConversionInput>,
) -> Result<Vec<ProductUomConversion>> {
    let conversions =
        uom::merge_for_write(base_uom, explicit, derived).map_err(AppError::Validation)?;
    let mut result = Vec::new();
    for conv in &conversions {
        let uom = sqlx::query_as::<_, ProductUomConversion>(
            r#"
            INSERT INTO product_uom_conversions (id, product_id, uom, factor_to_base)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(product_id)
        .bind(&conv.uom)
        .bind(conv.factor_to_base)
        .fetch_one(&mut **tx)
        .await?;
        result.push(uom);
    }
    Ok(result)
}

/// 組合產品與分類名稱為 `ProductWithUom`。
async fn build_product_with_uom(
    pool: &PgPool,
    product: Product,
    uom_conversions: Vec<ProductUomConversion>,
) -> Result<ProductWithUom> {
    let category_name = match product.category_code.as_deref() {
        Some(cat_code) => repositories::sku::find_category_name_by_code(pool, cat_code).await?,
        None => None,
    };
    let subcategory_name = if let (Some(ref cat_code), Some(ref sub_code)) =
        (&product.category_code, &product.subcategory_code)
    {
        repositories::product::find_subcategory_name(pool, cat_code, sub_code).await?
    } else {
        None
    };
    Ok(ProductWithUom {
        product,
        uom_conversions,
        category_name,
        subcategory_name,
    })
}

/// 同步單位換算：刪除既有後重新建立（tx 版本）。驗證與併入推導列同 `insert_uom_conversions_tx`。
async fn sync_uom_conversions_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    product_id: Uuid,
    base_uom: &str,
    explicit: &[crate::models::UomConversionInput],
    derived: Option<crate::models::UomConversionInput>,
) -> Result<()> {
    let conversions =
        uom::merge_for_write(base_uom, explicit, derived).map_err(AppError::Validation)?;
    sqlx::query("DELETE FROM product_uom_conversions WHERE product_id = $1")
        .bind(product_id)
        .execute(&mut **tx)
        .await?;
    for conv in &conversions {
        sqlx::query(
            r#"
            INSERT INTO product_uom_conversions (id, product_id, uom, factor_to_base)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(product_id)
        .bind(&conv.uom)
        .bind(conv.factor_to_base)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

/// 只調整「由包裝關係推導出來的那一列」，不動其他換算列（tx 版本）。
///
/// 用於**沒有明確帶 `uom_conversions` 的更新**——這是實務上最常見的路徑：產品編輯表單
/// 只送 `pack_unit` / `pack_qty`，從不送 `uom_conversions`。在本函式之前，這種更新會讓
/// 換算表停留在舊的換算率，而盤點底稿的除數取自換算表
/// （`document/stocktake.rs` 的 `LEFT JOIN product_uom_conversions ... ON c.uom = p.pack_unit`）
/// ——把 `pack_qty` 由 50 改成 24，底稿仍然照 50 除，**而且不會有任何錯誤訊息**。
///
/// 保守做法：舊的推導列只有在「值仍與舊推導完全一致」時才刪，避免刪掉人工調整過的列；
/// 新的推導列用 upsert，避免與既有同名列衝突（DB 有 `UNIQUE(product_id, uom)`）。
async fn reconcile_derived_pack_conversion_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    product_id: Uuid,
    before: &Product,
    after: &Product,
) -> Result<()> {
    let plan = uom::plan_pack_conversion_change(
        &before.base_uom,
        before.pack_unit.as_deref(),
        before.pack_qty,
        &after.base_uom,
        after.pack_unit.as_deref(),
        after.pack_qty,
    );

    // 別名列（舊寫法 `BX`）要先判定：值一致才刪；被人工改過就擋下來，不能放著讓它與
    // 正規列（`盒`）並存——兩者在單據的單位下拉裡長得一模一樣，換算率卻不同。
    if let Some(alias) = plan.remove_alias.as_ref() {
        let existing: Option<rust_decimal::Decimal> = sqlx::query_scalar(
            "SELECT factor_to_base FROM product_uom_conversions \
             WHERE product_id = $1 AND uom = $2",
        )
        .bind(product_id)
        .bind(&alias.uom)
        .fetch_optional(&mut **tx)
        .await?;

        let resolution = uom::resolve_alias(
            Some(alias),
            plan.upsert.as_ref().map(|u| u.uom.as_str()),
            existing,
        );
        if let Some(msg) = resolution.conflict_message() {
            return Err(AppError::Validation(msg));
        }
    }

    // WHERE 帶 factor：值不符即代表這一列已被人工改過，交給人負責，不代為刪除。
    // （別名列走到這裡時已由上面確認值一致，或根本不存在。）
    for row in [plan.remove, plan.remove_alias].into_iter().flatten() {
        sqlx::query(
            "DELETE FROM product_uom_conversions \
             WHERE product_id = $1 AND uom = $2 AND factor_to_base = $3",
        )
        .bind(product_id)
        .bind(&row.uom)
        .bind(row.factor_to_base)
        .execute(&mut **tx)
        .await?;
    }

    // ⚠️ 這裡**刻意**覆寫既有正規列的換算率，即使它曾被人工改成與 `pack_qty` 不同的值
    // （2026-09-03 使用者裁定；CodeRabbit 曾建議改成「不符就回 400」）。三個理由：
    //
    // 1. `pack_qty` 是推導列的真相來源。本模組存在的理由之一就是「改了 `pack_qty` 卻不同步
    //    換算表，盤點底稿照舊除數而且不報錯」。使用者改 `pack_qty` 就是對「1 盒 = 幾支」
    //    下最新的明確陳述，讓它蓋掉一個與之矛盾的舊值，方向是對的。
    // 2. 擋下來沒有出口：產品編輯表單從不送 `uom_conversions`，系統也沒有編輯單位換算的 UI。
    //    回 400 等於要使用者去修一個他看不到的欄位，那個品項將永遠無法從 UI 存檔。
    // 3. 與別名列（上面那段回 400）的不對稱是刻意的：別名列若不擋，結果是單位下拉出現兩個
    //    都顯示「盒」的選項（`formatUom` 對 `BX` 與 `盒` 輸出相同），選錯會靜默寫進差一倍的
    //    數量；正規列被覆寫只是較新的明確陳述取代較舊的，不產生歧義選項。
    if let Some(new) = plan.upsert {
        sqlx::query(
            "INSERT INTO product_uom_conversions (id, product_id, uom, factor_to_base) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (product_id, uom) DO UPDATE SET factor_to_base = EXCLUDED.factor_to_base",
        )
        .bind(Uuid::new_v4())
        .bind(product_id)
        .bind(&new.uom)
        .bind(new.factor_to_base)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

// build_list_sql / bind_list_params 已搬至 repositories::product::list_products

pub struct ProductService;

#[cfg(test)]
mod tests {
    use super::{format_product_sku, validate_product_status};
    use crate::services::product_parser::parse_bool;

    #[test]
    fn test_format_product_sku_normal() {
        assert_eq!(format_product_sku("DRG", "OTH", 1), "DRG-OTH-001");
        assert_eq!(format_product_sku("GEN", "OTH", 42), "GEN-OTH-042");
    }

    #[test]
    fn test_format_product_sku_zero_pad() {
        assert_eq!(format_product_sku("CAT", "SUB", 0), "CAT-SUB-000");
        assert_eq!(format_product_sku("A", "B", 999), "A-B-999");
    }

    #[test]
    fn test_format_product_sku_large_sequence() {
        assert_eq!(format_product_sku("X", "Y", 1000), "X-Y-1000");
    }

    #[test]
    fn test_validate_product_status_allowed() {
        assert_eq!(validate_product_status("active").expect("valid"), "active");
        assert_eq!(
            validate_product_status("inactive").expect("valid"),
            "inactive"
        );
        assert_eq!(
            validate_product_status("discontinued").expect("valid"),
            "discontinued"
        );
        assert_eq!(
            validate_product_status("  ACTIVE  ").expect("valid"),
            "active"
        );
    }

    #[test]
    fn test_validate_product_status_invalid() {
        assert!(validate_product_status("pending").is_err());
        assert!(validate_product_status("").is_err());
        assert!(validate_product_status("ActiveX").is_err());
    }

    #[test]
    fn test_validate_product_status_error_message() {
        let msg = validate_product_status("x").expect_err("invalid status");
        assert!(msg.contains("active"));
        assert!(msg.contains("inactive"));
        assert!(msg.contains("discontinued"));
    }

    #[test]
    fn test_parse_bool_true_variants() {
        assert!(parse_bool("true"));
        assert!(parse_bool("1"));
        assert!(parse_bool("yes"));
        assert!(parse_bool("是"));
        assert!(parse_bool("y"));
        assert!(parse_bool("  YES  "));
    }

    #[test]
    fn test_parse_bool_false() {
        assert!(!parse_bool("false"));
        assert!(!parse_bool("0"));
        assert!(!parse_bool("no"));
        assert!(!parse_bool(""));
        assert!(!parse_bool("n"));
        assert!(!parse_bool("other"));
    }
}
