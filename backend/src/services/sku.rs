use chrono::{Datelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{
        CategoriesResponse, CategoryOption, CreateProductWithSkuRequest, GenerateSkuRequest,
        GenerateSkuResponse, Product, ProductWithUom, SkuCategory, SkuPreviewRequest,
        SkuPreviewResponse, SkuSegment, SkuSubcategory, SubcategoriesResponse, ValidateSkuRequest,
        ValidateSkuResponse,
    },
    AppError, Result,
};

pub struct SkuService;

impl SkuService {
    /// 取得所有 SKU 類別
    pub async fn get_categories(pool: &PgPool) -> Result<CategoriesResponse> {
        let categories: Vec<SkuCategory> = sqlx::query_as(
            "SELECT * FROM sku_categories WHERE is_active = true ORDER BY sort_order"
        )
        .fetch_all(pool)
        .await?;

        Ok(CategoriesResponse {
            categories: categories
                .into_iter()
                .map(|c| CategoryOption {
                    code: c.code,
                    name: c.name,
                })
                .collect(),
        })
    }

    /// 取得指定類別的子類別
    pub async fn get_subcategories(pool: &PgPool, category_code: &str) -> Result<SubcategoriesResponse> {
        // 查詢主類別
        let category: SkuCategory = sqlx::query_as(
            "SELECT * FROM sku_categories WHERE code = $1 AND is_active = true"
        )
        .bind(category_code)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

        // 查詢子類別
        let subcategories: Vec<SkuSubcategory> = sqlx::query_as(
            "SELECT * FROM sku_subcategories WHERE category_code = $1 AND is_active = true ORDER BY sort_order"
        )
        .bind(category_code)
        .fetch_all(pool)
        .await?;

        Ok(SubcategoriesResponse {
            category: CategoryOption {
                code: category.code,
                name: category.name,
            },
            subcategories: subcategories
                .into_iter()
                .map(|s| CategoryOption {
                    code: s.code,
                    name: s.name,
                })
                .collect(),
        })
    }

    /// 生成 SKU
    pub async fn generate(pool: &PgPool, req: &GenerateSkuRequest) -> Result<GenerateSkuResponse> {
        // 驗證類別
        let category: SkuCategory = sqlx::query_as(
            "SELECT * FROM sku_categories WHERE code = $1 AND is_active = true"
        )
        .bind(&req.category)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

        // 驗證子類別
        let subcategory: SkuSubcategory = sqlx::query_as(
            "SELECT * FROM sku_subcategories WHERE category_code = $1 AND code = $2 AND is_active = true"
        )
        .bind(&req.category)
        .bind(&req.subcategory)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Subcategory not found".to_string()))?;

        // 取得下一個流水號
        let sequence = Self::get_next_simple_sequence(pool, &req.category, &req.subcategory).await?;

        // 組合 SKU
        let sku = format!("{}-{}-{:03}", req.category, req.subcategory, sequence);

        Ok(GenerateSkuResponse {
            sku,
            category: CategoryOption {
                code: category.code,
                name: category.name,
            },
            subcategory: CategoryOption {
                code: subcategory.code,
                name: subcategory.name,
            },
            sequence,
        })
    }

    /// 驗證 SKU 格式
    pub async fn validate(pool: &PgPool, req: &ValidateSkuRequest) -> Result<ValidateSkuResponse> {
        // 驗證格式：XXX-XXX-NNN
        let re = regex::Regex::new(r"^([A-Z]{3})-([A-Z]{3})-(\d{3})$")
            .map_err(|e| AppError::Internal(format!("Regex error: {}", e)))?;

        let captures = match re.captures(&req.sku) {
            Some(c) => c,
            None => {
                return Ok(ValidateSkuResponse {
                    valid: false,
                    category: None,
                    subcategory: None,
                    sequence: None,
                    exists: false,
                    error: Some("Invalid SKU format. Expected: XXX-XXX-NNN".to_string()),
                });
            }
        };

        let cat_code = captures.get(1).unwrap().as_str();
        let sub_code = captures.get(2).unwrap().as_str();
        let seq_str = captures.get(3).unwrap().as_str();
        let sequence: i32 = seq_str.parse().unwrap_or(0);

        // 查詢類別
        let category: Option<SkuCategory> = sqlx::query_as(
            "SELECT * FROM sku_categories WHERE code = $1"
        )
        .bind(cat_code)
        .fetch_optional(pool)
        .await?;

        let category = match category {
            Some(c) => c,
            None => {
                return Ok(ValidateSkuResponse {
                    valid: false,
                    category: None,
                    subcategory: None,
                    sequence: Some(sequence),
                    exists: false,
                    error: Some(format!("Unknown category code: {}", cat_code)),
                });
            }
        };

        // 查詢子類別
        let subcategory: Option<SkuSubcategory> = sqlx::query_as(
            "SELECT * FROM sku_subcategories WHERE category_code = $1 AND code = $2"
        )
        .bind(cat_code)
        .bind(sub_code)
        .fetch_optional(pool)
        .await?;

        let subcategory = match subcategory {
            Some(s) => s,
            None => {
                return Ok(ValidateSkuResponse {
                    valid: false,
                    category: Some(CategoryOption {
                        code: category.code,
                        name: category.name,
                    }),
                    subcategory: None,
                    sequence: Some(sequence),
                    exists: false,
                    error: Some(format!("Unknown subcategory code: {}", sub_code)),
                });
            }
        };

        // 檢查 SKU 是否已存在
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM products WHERE sku = $1)"
        )
        .bind(&req.sku)
        .fetch_one(pool)
        .await?;

        Ok(ValidateSkuResponse {
            valid: true,
            category: Some(CategoryOption {
                code: category.code,
                name: category.name,
            }),
            subcategory: Some(CategoryOption {
                code: subcategory.code,
                name: subcategory.name,
            }),
            sequence: Some(sequence),
            exists,
            error: None,
        })
    }

    /// 取得簡單流水號（符合 skuSpec.md）
    async fn get_next_simple_sequence(
        pool: &PgPool,
        category: &str,
        subcategory: &str,
    ) -> Result<i32> {
        // 嘗試更新並取得序號
        let result = sqlx::query_scalar::<_, i32>(
            r#"
            INSERT INTO sku_sequences (category_code, subcategory_code, last_sequence)
            VALUES ($1, $2, 1)
            ON CONFLICT (category_code, subcategory_code)
            DO UPDATE SET last_sequence = sku_sequences.last_sequence + 1
            RETURNING last_sequence
            "#
        )
        .bind(category)
        .bind(subcategory)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    // ============================================
    // 以下是原有的複雜 SKU 生成邏輯（保留向後兼容）
    // ============================================

    /// 獲取當前週數 (ISO week)
    fn get_year_week() -> (i32, u32) {
        let now = Utc::now();
        let year = now.year() % 100; // 取後兩位
        let week = now.iso_week().week();
        (year, week)
    }

    /// 生成 ATTR 片段
    fn generate_attr_segment(
        cat: &str,
        attributes: &Option<std::collections::HashMap<String, serde_json::Value>>,
        spec: &Option<String>,
    ) -> String {
        // 如果是藥品類別，使用藥品規格生成
        if cat == "DRG" {
            if let Some(attrs) = attributes {
                let generic_name = attrs
                    .get("generic_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("UNK");
                let dose_value = attrs
                    .get("dose_value")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let dose_unit = attrs
                    .get("dose_unit")
                    .and_then(|v| v.as_str())
                    .unwrap_or("mg");
                let dosage_form = attrs
                    .get("dosage_form")
                    .and_then(|v| v.as_str())
                    .unwrap_or("oth");

                // 取學名前3字元
                let generic_abbr: String = generic_name
                    .chars()
                    .take(3)
                    .collect::<String>()
                    .to_uppercase();

                // 劑量
                let dose = format!("{}{}", dose_value as i32, dose_unit.to_uppercase());

                // 劑型縮寫
                let form = dosage_form.chars().take(2).collect::<String>().to_uppercase();

                return format!("{}{}{}", generic_abbr, dose, form);
            }
        }

        // 其他類別，使用規格摘要
        if let Some(s) = spec {
            s.chars()
                .filter(|c| !c.is_whitespace())
                .take(8)
                .collect::<String>()
                .to_uppercase()
        } else {
            "GEN".to_string()
        }
    }

    /// 生成 PACK 片段
    fn generate_pack_segment(uom: &str, qty: i32) -> String {
        format!("{}{:02}", uom.to_uppercase(), qty)
    }

    /// 生成 YYWW 片段
    fn generate_yyww_segment() -> String {
        let (year, week) = Self::get_year_week();
        format!("{:02}{:02}", year, week)
    }

    /// 計算檢查碼 (簡單的 Luhn 變體)
    fn calculate_check_digit(sku_without_check: &str) -> char {
        let sum: u32 = sku_without_check
            .chars()
            .filter(|c| c.is_alphanumeric())
            .enumerate()
            .map(|(i, c)| {
                let val = if c.is_ascii_digit() {
                    c.to_digit(10).unwrap_or(0)
                } else {
                    (c.to_ascii_uppercase() as u32) - ('A' as u32) + 10
                };
                if i % 2 == 0 {
                    val * 2
                } else {
                    val
                }
            })
            .sum();

        let check = (10 - (sum % 10)) % 10;
        char::from_digit(check, 10).unwrap_or('0')
    }

    /// 獲取下一個序號（複雜版）
    async fn get_next_sequence(pool: &PgPool, prefix: &str) -> Result<i32> {
        // 查詢當前最大序號
        let result: Option<String> = sqlx::query_scalar(
            "SELECT sku FROM products WHERE sku LIKE $1 ORDER BY sku DESC LIMIT 1",
        )
        .bind(format!("{}%", prefix))
        .fetch_optional(pool)
        .await?;

        if let Some(existing_sku) = result {
            // 嘗試解析序號部分
            let parts: Vec<&str> = existing_sku.split('-').collect();
            if parts.len() >= 8 {
                if let Ok(seq) = parts[7].parse::<i32>() {
                    return Ok(seq + 1);
                }
            }
        }

        Ok(1) // 從 1 開始
    }

    /// 預覽 SKU（不含序號和檢查碼）
    pub async fn preview(
        _pool: &PgPool,
        req: &SkuPreviewRequest,
    ) -> Result<SkuPreviewResponse> {
        let org = req.org.as_deref().unwrap_or("PMAT");
        let cat = &req.cat;
        let sub = &req.sub;
        let attr = Self::generate_attr_segment(cat, &req.attributes, &None);
        let pack = Self::generate_pack_segment(&req.pack.uom, req.pack.qty);
        let src = &req.source;
        let yyww = Self::generate_yyww_segment();

        let preview_sku = format!(
            "{}-{}-{}-{}-{}-{}-{}-XXX-X",
            org, cat, sub, attr, pack, src, yyww
        );

        let segments = vec![
            SkuSegment {
                code: "ORG".to_string(),
                label: "機構".to_string(),
                value: org.to_string(),
                source: "機構設定".to_string(),
            },
            SkuSegment {
                code: "CAT".to_string(),
                label: "品類".to_string(),
                value: cat.to_string(),
                source: "品類欄位".to_string(),
            },
            SkuSegment {
                code: "SUB".to_string(),
                label: "子類".to_string(),
                value: sub.to_string(),
                source: "子類欄位".to_string(),
            },
            SkuSegment {
                code: "ATTR".to_string(),
                label: "規格".to_string(),
                value: attr,
                source: if cat == "DRG" {
                    "學名＋劑量＋劑型".to_string()
                } else {
                    "規格摘要".to_string()
                },
            },
            SkuSegment {
                code: "PACK".to_string(),
                label: "包裝".to_string(),
                value: pack,
                source: "單位與包裝量".to_string(),
            },
            SkuSegment {
                code: "SRC".to_string(),
                label: "來源".to_string(),
                value: src.to_string(),
                source: "供應來源".to_string(),
            },
            SkuSegment {
                code: "YYWW".to_string(),
                label: "週期".to_string(),
                value: yyww,
                source: "系統日期".to_string(),
            },
            SkuSegment {
                code: "SEQ".to_string(),
                label: "序號".to_string(),
                value: "XXX".to_string(),
                source: "建立時分配".to_string(),
            },
            SkuSegment {
                code: "CHK".to_string(),
                label: "檢查碼".to_string(),
                value: "X".to_string(),
                source: "建立時計算".to_string(),
            },
        ];

        Ok(SkuPreviewResponse {
            preview_sku,
            segments,
            rule_version: "v1.0".to_string(),
            rule_updated_at: Some("2026-01-01".to_string()),
        })
    }

    /// 生成正式 SKU 並建立產品
    pub async fn create_product_with_sku(
        pool: &PgPool,
        req: &CreateProductWithSkuRequest,
    ) -> Result<ProductWithUom> {
        // 使用簡化的 SKU 生成（符合 skuSpec.md）
        let generate_req = GenerateSkuRequest {
            category: req.category_code.clone(),
            subcategory: req.subcategory_code.clone(),
        };
        
        let sku_result = Self::generate(pool, &generate_req).await?;
        let final_sku = sku_result.sku;

        // 檢查 SKU 是否已存在
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM products WHERE sku = $1)")
                .bind(&final_sku)
                .fetch_one(pool)
                .await?;

        if exists {
            return Err(AppError::Conflict("SKU already exists".to_string()));
        }

        // 生成產品名稱（如果未提供）
        let product_name = req.name.clone().unwrap_or_else(|| {
            req.spec.clone().unwrap_or_else(|| final_sku.clone())
        });

        // 建立產品
        let product = sqlx::query_as::<_, Product>(
            r#"
            INSERT INTO products (
                id, sku, name, spec, category_id, base_uom, pack_unit, pack_qty,
                track_batch, track_expiry, safety_stock, reorder_point,
                is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, NULL, $5, $6, $7, $8, $9, $10, $11, true, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&final_sku)
        .bind(&product_name)
        .bind(&req.spec)
        .bind(&req.base_uom)
        .bind(&req.pack_unit)
        .bind(req.pack_qty)
        .bind(req.track_batch)
        .bind(req.track_expiry)
        .bind(req.safety_stock)
        .bind(req.reorder_point)
        .fetch_one(pool)
        .await?;

        // 查詢類別名稱
        let category_name: Option<String> = sqlx::query_scalar(
            "SELECT name FROM sku_categories WHERE code = $1"
        )
        .bind(&req.category_code)
        .fetch_optional(pool)
        .await?;

        let subcategory_name: Option<String> = sqlx::query_scalar(
            "SELECT name FROM sku_subcategories WHERE category_code = $1 AND code = $2"
        )
        .bind(&req.category_code)
        .bind(&req.subcategory_code)
        .fetch_optional(pool)
        .await?;

        Ok(ProductWithUom {
            product,
            uom_conversions: Vec::new(),
            category_name,
            subcategory_name,
        })
    }
}
