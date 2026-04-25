use chrono::{Datelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, CategoriesResponse, CategoriesTreeResponse, CategoryForEdit,
        CategoryOption, CreateProductWithSkuRequest, CreateSkuSubcategoryRequest,
        GenerateSkuRequest, GenerateSkuResponse, Product, ProductWithUom, SkuCategory,
        SkuPreviewRequest, SkuPreviewResponse, SkuSegment, SkuSubcategory, SubcategoriesResponse,
        SubcategoryForEdit, UpdateSkuCategoryRequest, UpdateSkuSubcategoryRequest,
        ValidateSkuRequest, ValidateSkuResponse,
    },
    repositories,
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
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

    /// 取得完整品類樹（含停用項）供編輯分類使用
    /// 使用單一查詢 + 應用層分組，避免 N+1
    pub async fn get_categories_tree(pool: &PgPool) -> Result<CategoriesTreeResponse> {
        // 兩次查詢改為批量取回，再在應用層組裝
        let (categories, all_subcategories) = tokio::try_join!(
            sqlx::query_as::<_, SkuCategory>(
                "SELECT code, name, sort_order, is_active, created_at FROM sku_categories ORDER BY sort_order, code"
            )
            .fetch_all(pool),
            sqlx::query_as::<_, SkuSubcategory>(
                "SELECT id, category_code, code, name, sort_order, is_active, created_at FROM sku_subcategories ORDER BY sort_order, code"
            )
            .fetch_all(pool),
        )?;

        // 以 category_code 分組子類別
        let mut sub_map: std::collections::HashMap<String, Vec<SubcategoryForEdit>> =
            std::collections::HashMap::new();
        for s in all_subcategories {
            sub_map
                .entry(s.category_code.clone())
                .or_default()
                .push(SubcategoryForEdit {
                    id: s.id,
                    code: s.code,
                    name: s.name,
                    sort_order: s.sort_order,
                    is_active: s.is_active,
                });
        }

        let result = categories
            .into_iter()
            .map(|cat| {
                let subcategories = sub_map.remove(&cat.code).unwrap_or_default();
                CategoryForEdit {
                    code: cat.code,
                    name: cat.name,
                    sort_order: cat.sort_order,
                    is_active: cat.is_active,
                    subcategories,
                }
            })
            .collect();

        Ok(CategoriesTreeResponse {
            categories: result,
        })
    }

    /// 品類／子類名稱最大長度（與 DB VARCHAR(50) 一致）
    const NAME_MAX_LEN: usize = 50;
    /// 排序值合理範圍
    const SORT_ORDER_MIN: i32 = 0;
    const SORT_ORDER_MAX: i32 = 9999;

    /// 更新品類（名稱、排序、啟用狀態）— Service-driven audit
    pub async fn update_category(
        pool: &PgPool,
        actor: &ActorContext,
        code: &str,
        req: &UpdateSkuCategoryRequest,
    ) -> Result<SkuCategory> {
        let mut tx = pool.begin().await?;
        let result = Self::update_category_tx(&mut tx, actor, code, req).await?;
        tx.commit().await?;
        Ok(result)
    }

    /// Transaction 版本：更新 SKU 品類 — R26-3 Phase 2。
    pub async fn update_category_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        code: &str,
        req: &UpdateSkuCategoryRequest,
    ) -> Result<SkuCategory> {
        let _user = actor.require_user()?;

        let before: SkuCategory = sqlx::query_as(
            "SELECT code, name, sort_order, is_active, created_at FROM sku_categories WHERE code = $1 FOR UPDATE",
        )
        .bind(code)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

        let name = req.name.as_deref().unwrap_or(&before.name);
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::Validation("品類名稱為必填".to_string()));
        }
        if name.len() > Self::NAME_MAX_LEN {
            return Err(AppError::Validation(format!(
                "品類名稱不可超過 {} 字元",
                Self::NAME_MAX_LEN
            )));
        }
        let sort_order = req.sort_order.unwrap_or(before.sort_order);
        if !(Self::SORT_ORDER_MIN..=Self::SORT_ORDER_MAX).contains(&sort_order) {
            return Err(AppError::Validation(format!(
                "排序請介於 {} 與 {} 之間",
                Self::SORT_ORDER_MIN,
                Self::SORT_ORDER_MAX
            )));
        }
        let is_active = req.is_active.unwrap_or(before.is_active);

        let after: SkuCategory = sqlx::query_as(
            "UPDATE sku_categories SET name = $1, sort_order = $2, is_active = $3 WHERE code = $4 RETURNING code, name, sort_order, is_active, created_at",
        )
        .bind(name)
        .bind(sort_order)
        .bind(is_active)
        .bind(code)
        .fetch_one(&mut **tx)
        .await?;

        let display = format!("{} {}", after.code, after.name);
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "SKU_CATEGORY_UPDATE",
                entity: Some(AuditEntity::new(
                    "sku_category",
                    uuid::Uuid::nil(),
                    &display,
                )),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        Ok(after)
    }

    /// 取得單一品類（含停用）
    pub async fn get_category_by_code(pool: &PgPool, code: &str) -> Result<SkuCategory> {
        let category: SkuCategory = sqlx::query_as(
            "SELECT code, name, sort_order, is_active, created_at FROM sku_categories WHERE code = $1",
        )
        .bind(code)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;
        Ok(category)
    }

    /// 更新子類（名稱、排序、啟用狀態）— Service-driven audit
    pub async fn update_subcategory(
        pool: &PgPool,
        actor: &ActorContext,
        category_code: &str,
        code: &str,
        req: &UpdateSkuSubcategoryRequest,
    ) -> Result<SkuSubcategory> {
        let mut tx = pool.begin().await?;
        let result = Self::update_subcategory_tx(&mut tx, actor, category_code, code, req).await?;
        tx.commit().await?;
        Ok(result)
    }

    /// Transaction 版本：更新 SKU 子類 — R26-3 Phase 2。
    pub async fn update_subcategory_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        category_code: &str,
        code: &str,
        req: &UpdateSkuSubcategoryRequest,
    ) -> Result<SkuSubcategory> {
        let _user = actor.require_user()?;

        let before: SkuSubcategory = sqlx::query_as(
            "SELECT id, category_code, code, name, sort_order, is_active, created_at FROM sku_subcategories WHERE category_code = $1 AND code = $2 FOR UPDATE",
        )
        .bind(category_code)
        .bind(code)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Subcategory not found".to_string()))?;

        let name = req.name.as_deref().unwrap_or(&before.name);
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::Validation("子類名稱為必填".to_string()));
        }
        if name.len() > Self::NAME_MAX_LEN {
            return Err(AppError::Validation(format!(
                "子類名稱不可超過 {} 字元",
                Self::NAME_MAX_LEN
            )));
        }
        let sort_order = req.sort_order.unwrap_or(before.sort_order);
        if !(Self::SORT_ORDER_MIN..=Self::SORT_ORDER_MAX).contains(&sort_order) {
            return Err(AppError::Validation(format!(
                "排序請介於 {} 與 {} 之間",
                Self::SORT_ORDER_MIN,
                Self::SORT_ORDER_MAX
            )));
        }
        let is_active = req.is_active.unwrap_or(before.is_active);

        let after: SkuSubcategory = sqlx::query_as(
            "UPDATE sku_subcategories SET name = $1, sort_order = $2, is_active = $3 WHERE category_code = $4 AND code = $5 RETURNING id, category_code, code, name, sort_order, is_active, created_at",
        )
        .bind(name)
        .bind(sort_order)
        .bind(is_active)
        .bind(category_code)
        .bind(code)
        .fetch_one(&mut **tx)
        .await?;

        let display = format!("{}:{} {}", after.category_code, after.code, after.name);
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "SKU_SUBCATEGORY_UPDATE",
                entity: Some(AuditEntity::new(
                    "sku_subcategory",
                    uuid::Uuid::nil(),
                    &display,
                )),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        Ok(after)
    }

    /// 取得單一子類（含停用）
    pub async fn get_subcategory_by_codes(
        pool: &PgPool,
        category_code: &str,
        code: &str,
    ) -> Result<SkuSubcategory> {
        let sub: SkuSubcategory = sqlx::query_as(
            "SELECT id, category_code, code, name, sort_order, is_active, created_at FROM sku_subcategories WHERE category_code = $1 AND code = $2",
        )
        .bind(category_code)
        .bind(code)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Subcategory not found".to_string()))?;
        Ok(sub)
    }

    /// 新增子類 — Service-driven audit
    pub async fn create_subcategory(
        pool: &PgPool,
        actor: &ActorContext,
        category_code: &str,
        req: &CreateSkuSubcategoryRequest,
    ) -> Result<SkuSubcategory> {
        let mut tx = pool.begin().await?;
        let result = Self::create_subcategory_tx(&mut tx, actor, category_code, req).await?;
        tx.commit().await?;
        Ok(result)
    }

    /// Transaction 版本：建立 SKU 子類 — R26-3 Phase 2。
    pub async fn create_subcategory_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        category_code: &str,
        req: &CreateSkuSubcategoryRequest,
    ) -> Result<SkuSubcategory> {
        let _user = actor.require_user()?;

        let code = req.code.trim().to_uppercase();
        if code.len() != 3 {
            return Err(AppError::Validation("子類代碼須為 3 碼".to_string()));
        }
        if !code.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(AppError::Validation("子類代碼僅可為英文或數字".to_string()));
        }

        let name = req.name.trim();
        if name.is_empty() {
            return Err(AppError::Validation("子類名稱為必填".to_string()));
        }
        if name.len() > Self::NAME_MAX_LEN {
            return Err(AppError::Validation(format!(
                "子類名稱不可超過 {} 字元",
                Self::NAME_MAX_LEN
            )));
        }

        let sort_order = req.sort_order.unwrap_or(0);
        if !(Self::SORT_ORDER_MIN..=Self::SORT_ORDER_MAX).contains(&sort_order) {
            return Err(AppError::Validation(format!(
                "排序請介於 {} 與 {} 之間",
                Self::SORT_ORDER_MIN,
                Self::SORT_ORDER_MAX
            )));
        }
        let is_active = req.is_active.unwrap_or(true);

        // 品類必須存在（在 tx 內檢查以避免併發 race）
        let _parent: (String,) =
            sqlx::query_as("SELECT code FROM sku_categories WHERE code = $1")
                .bind(category_code)
                .fetch_optional(&mut **tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

        let after: SkuSubcategory = sqlx::query_as(
            "INSERT INTO sku_subcategories (category_code, code, name, sort_order, is_active, created_at)
             VALUES ($1, $2, $3, $4, $5, NOW())
             RETURNING id, category_code, code, name, sort_order, is_active, created_at",
        )
        .bind(category_code)
        .bind(&code)
        .bind(name)
        .bind(sort_order)
        .bind(is_active)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.constraint().is_some() {
                    return AppError::Conflict("該品類下已存在相同子類代碼".to_string());
                }
            }
            AppError::from(e)
        })?;

        let display = format!("{}:{} {}", after.category_code, after.code, after.name);
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "SKU_SUBCATEGORY_CREATE",
                entity: Some(AuditEntity::new(
                    "sku_subcategory",
                    uuid::Uuid::nil(),
                    &display,
                )),
                data_diff: Some(DataDiff::create_only(&after)),
                request_context: None,
            },
        )
        .await?;

        Ok(after)
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

        let cat_code = captures.get(1)
            .ok_or_else(|| AppError::Internal("regex capture group 1 missing".to_string()))?.as_str();
        let sub_code = captures.get(2)
            .ok_or_else(|| AppError::Internal("regex capture group 2 missing".to_string()))?.as_str();
        let seq_str = captures.get(3)
            .ok_or_else(|| AppError::Internal("regex capture group 3 missing".to_string()))?.as_str();
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
        let exists = repositories::product::exists_product_by_sku(pool, &req.sku).await?;

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

    /// 生成正式 SKU 並建立產品 — Service-driven audit
    pub async fn create_product_with_sku(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateProductWithSkuRequest,
    ) -> Result<ProductWithUom> {
        let _user = actor.require_user()?;

        // 使用簡化的 SKU 生成（符合 skuSpec.md）— pool-based (outside tx)
        let generate_req = GenerateSkuRequest {
            category: req.category_code.clone(),
            subcategory: req.subcategory_code.clone(),
        };
        let sku_result = Self::generate(pool, &generate_req).await?;
        let final_sku = sku_result.sku;

        let mut tx = pool.begin().await?;
        let product =
            Self::create_product_with_sku_tx(&mut tx, actor, req, &final_sku).await?;
        tx.commit().await?;

        // tx 外取類別/子類別顯示名稱（pool-based 讀取，rollback 時不受影響）
        let category_name =
            repositories::sku::find_category_name_by_code(pool, &req.category_code).await?;

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

    /// Transaction 版本：在既有 tx 內建立產品（含 SKU 驗證）— R26-3 Phase 2。
    /// 呼叫端需預先生成 final_sku（使用 `Self::generate` 於 pool 環境）。
    pub async fn create_product_with_sku_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        req: &CreateProductWithSkuRequest,
        final_sku: &str,
    ) -> Result<Product> {
        let _user = actor.require_user()?;

        // 生成產品名稱（如果未提供）
        let product_name = req.name.clone().unwrap_or_else(|| {
            req.spec.clone().unwrap_or_else(|| final_sku.to_string())
        });

        // 在 tx 內檢查 SKU 唯一性（防併發搶註冊；若失敗由 UNIQUE constraint 兜底）
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM products WHERE sku = $1)")
                .bind(final_sku)
                .fetch_one(&mut **tx)
                .await?;
        if exists {
            return Err(AppError::Conflict("SKU already exists".to_string()));
        }

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
        .bind(final_sku)
        .bind(&product_name)
        .bind(&req.spec)
        .bind(&req.base_uom)
        .bind(&req.pack_unit)
        .bind(req.pack_qty)
        .bind(req.track_batch)
        .bind(req.track_expiry)
        .bind(req.safety_stock)
        .bind(req.reorder_point)
        .fetch_one(&mut **tx)
        .await?;

        let display = format!("{} ({})", product.name, product.sku);
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "PRODUCT_CREATE_WITH_SKU",
                entity: Some(AuditEntity::new("product", product.id, &display)),
                data_diff: Some(DataDiff::create_only(&product)),
                request_context: None,
            },
        )
        .await?;

        Ok(product)
    }

    /// 刪除子類（僅在無產品使用該子類時允許；僅 admin 可呼叫，由 handler 檢查）
    /// Service-driven audit
    pub async fn delete_subcategory(
        pool: &PgPool,
        actor: &ActorContext,
        category_code: &str,
        code: &str,
    ) -> Result<()> {
        let mut tx = pool.begin().await?;
        Self::delete_subcategory_tx(&mut tx, actor, category_code, code).await?;
        tx.commit().await?;
        Ok(())
    }

    /// Transaction 版本：刪除 SKU 子類 — R26-3 Phase 2。
    pub async fn delete_subcategory_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        category_code: &str,
        code: &str,
    ) -> Result<()> {
        let _user = actor.require_user()?;

        let before: SkuSubcategory = sqlx::query_as(
            "SELECT id, category_code, code, name, sort_order, is_active, created_at FROM sku_subcategories WHERE category_code = $1 AND code = $2 FOR UPDATE",
        )
        .bind(category_code)
        .bind(code)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Subcategory not found".to_string()))?;

        // 若有產品使用此 category_code + subcategory_code（欄位或 SKU 前綴），不允許刪除
        let by_columns: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM products WHERE category_code = $1 AND subcategory_code = $2)",
        )
        .bind(category_code)
        .bind(code)
        .fetch_one(&mut **tx)
        .await?;
        if by_columns {
            return Err(AppError::BusinessRule(
                "尚有產品使用此子類，無法刪除".to_string(),
            ));
        }
        let sku_prefix = format!("{}-{}-", category_code, code);
        let by_sku: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM products WHERE sku LIKE $1)",)
            .bind(format!("{}%", sku_prefix))
            .fetch_one(&mut **tx)
            .await?;
        if by_sku {
            return Err(AppError::BusinessRule(
                "尚有產品使用此子類，無法刪除".to_string(),
            ));
        }

        sqlx::query("DELETE FROM sku_sequences WHERE category_code = $1 AND subcategory_code = $2")
            .bind(category_code)
            .bind(code)
            .execute(&mut **tx)
            .await?;
        let result = sqlx::query("DELETE FROM sku_subcategories WHERE category_code = $1 AND code = $2")
            .bind(category_code)
            .bind(code)
            .execute(&mut **tx)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Subcategory not found".to_string()));
        }

        let display = format!("{}:{} {}", before.category_code, before.code, before.name);
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "SKU_SUBCATEGORY_DELETE",
                entity: Some(AuditEntity::new(
                    "sku_subcategory",
                    uuid::Uuid::nil(),
                    &display,
                )),
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;

        Ok(())
    }

    /// 刪除品類（僅在無產品使用該品類時允許；僅 admin 可呼叫，由 handler 檢查）
    /// Service-driven audit
    pub async fn delete_category(pool: &PgPool, actor: &ActorContext, code: &str) -> Result<()> {
        let mut tx = pool.begin().await?;
        Self::delete_category_tx(&mut tx, actor, code).await?;
        tx.commit().await?;
        Ok(())
    }

    /// Transaction 版本：刪除 SKU 品類 — R26-3 Phase 2。
    pub async fn delete_category_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        code: &str,
    ) -> Result<()> {
        let _user = actor.require_user()?;

        let before: SkuCategory = sqlx::query_as(
            "SELECT code, name, sort_order, is_active, created_at FROM sku_categories WHERE code = $1 FOR UPDATE",
        )
        .bind(code)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

        let by_columns: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM products WHERE category_code = $1)",
        )
        .bind(code)
        .fetch_one(&mut **tx)
        .await?;
        if by_columns {
            return Err(AppError::BusinessRule(
                "尚有產品使用此品類，無法刪除".to_string(),
            ));
        }
        let sku_prefix = format!("{}-", code);
        let by_sku: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM products WHERE sku LIKE $1)")
            .bind(format!("{}%", sku_prefix))
            .fetch_one(&mut **tx)
            .await?;
        if by_sku {
            return Err(AppError::BusinessRule(
                "尚有產品使用此品類，無法刪除".to_string(),
            ));
        }

        sqlx::query("DELETE FROM sku_sequences WHERE category_code = $1")
            .bind(code)
            .execute(&mut **tx)
            .await?;
        let result = sqlx::query("DELETE FROM sku_categories WHERE code = $1")
            .bind(code)
            .execute(&mut **tx)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Category not found".to_string()));
        }

        let display = format!("{} {}", before.code, before.name);
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "SKU_CATEGORY_DELETE",
                entity: Some(AuditEntity::new(
                    "sku_category",
                    uuid::Uuid::nil(),
                    &display,
                )),
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;

        Ok(())
    }

    /// 解析 SKU 格式字串 (XXX-XXX-NNN)，供單元測試驗證格式邏輯
    #[cfg(test)]
    pub fn parse_sku_format(sku: &str) -> Option<(String, String, i32)> {
        let re = regex::Regex::new(r"^([A-Z]{3})-([A-Z]{3})-(\d{3})$").ok()?;
        let captures = re.captures(sku)?;
        let cat = captures.get(1)?.as_str().to_string();
        let sub = captures.get(2)?.as_str().to_string();
        let seq: i32 = captures.get(3)?.as_str().parse().ok()?;
        Some((cat, sub, seq))
    }
}

#[cfg(test)]
mod tests {
    use super::SkuService;

    #[test]
    fn test_parse_sku_format_valid() {
        let r = SkuService::parse_sku_format("ABC-XYZ-001");
        assert!(r.is_some());
        let (cat, sub, seq) = r.expect("valid format");
        assert_eq!(cat, "ABC");
        assert_eq!(sub, "XYZ");
        assert_eq!(seq, 1);
    }

    #[test]
    fn test_parse_sku_format_valid_999() {
        let r = SkuService::parse_sku_format("MED-DRU-999");
        assert!(r.is_some());
        let (_, _, seq) = r.expect("valid format");
        assert_eq!(seq, 999);
    }

    #[test]
    fn test_parse_sku_format_lowercase_invalid() {
        assert!(SkuService::parse_sku_format("abc-xyz-001").is_none());
    }

    #[test]
    fn test_parse_sku_format_wrong_length() {
        assert!(SkuService::parse_sku_format("AB-XYZ-001").is_none());
        assert!(SkuService::parse_sku_format("ABCD-XYZ-001").is_none());
    }

    #[test]
    fn test_parse_sku_format_invalid_digits() {
        assert!(SkuService::parse_sku_format("ABC-XYZ-01").is_none());
        assert!(SkuService::parse_sku_format("ABC-XYZ-1000").is_none());
    }

    #[test]
    fn test_parse_sku_format_empty() {
        assert!(SkuService::parse_sku_format("").is_none());
    }

    #[test]
    fn test_parse_sku_format_wrong_separator() {
        assert!(SkuService::parse_sku_format("ABC_XYZ_001").is_none());
    }

    #[test]
    fn test_parse_sku_format_leading_trailing_whitespace() {
        // 格式解析不應接受前後空白
        assert!(SkuService::parse_sku_format(" ABC-XYZ-001").is_none());
        assert!(SkuService::parse_sku_format("ABC-XYZ-001 ").is_none());
    }

    #[test]
    fn test_parse_sku_format_hyphen_in_segment() {
        // 類別與子類別不應含額外連字號
        assert!(SkuService::parse_sku_format("AB-C-XYZ-001").is_none());
    }
}
