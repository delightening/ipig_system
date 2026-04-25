mod crud;
mod import;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{CreateProductRequest, Product, ProductUomConversion, ProductWithUom},
    repositories, AppError, Result,
};

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
    Ok(format_product_sku(category_code, subcategory_code, sequence))
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
    let product = sqlx::query_as::<_, Product>(
        r#"
        INSERT INTO products (
            id, sku, name, spec, category_code, subcategory_code, base_uom,
            pack_unit, pack_qty, track_batch, track_expiry, default_expiry_days,
            safety_stock, safety_stock_uom, reorder_point, reorder_point_uom,
            barcode, image_url, license_no, storage_condition, tags, remark,
            is_active, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, true, NOW(), NOW())
        RETURNING *
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(sku)
    .bind(&req.name)
    .bind(&req.spec)
    .bind(category_code)
    .bind(subcategory_code)
    .bind(&req.base_uom)
    .bind(&req.pack_unit)
    .bind(req.pack_qty)
    .bind(req.track_batch)
    .bind(req.track_expiry)
    .bind(req.default_expiry_days)
    .bind(req.safety_stock)
    .bind(&req.safety_stock_uom)
    .bind(req.reorder_point)
    .bind(&req.reorder_point_uom)
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
async fn insert_uom_conversions_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    product_id: Uuid,
    conversions: &[crate::models::UomConversionInput],
) -> Result<Vec<ProductUomConversion>> {
    let mut result = Vec::new();
    for conv in conversions {
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
    let subcategory_name =
        if let (Some(ref cat_code), Some(ref sub_code)) =
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

/// 同步單位換算：刪除既有後重新建立（tx 版本）。
async fn sync_uom_conversions_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    product_id: Uuid,
    conversions: &[crate::models::UomConversionInput],
) -> Result<()> {
    sqlx::query("DELETE FROM product_uom_conversions WHERE product_id = $1")
        .bind(product_id)
        .execute(&mut **tx)
        .await?;
    for conv in conversions {
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
