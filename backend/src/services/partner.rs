use sqlx::PgPool;
use uuid::Uuid;
use validator;

use crate::{
    models::{CreatePartnerRequest, Partner, PartnerQuery, SupplierCategory, UpdatePartnerRequest},
    AppError, Result,
};

pub struct PartnerService;

impl PartnerService {
    /// 根據供應商類別生成代碼
    /// 格式：類型代碼 + {:03} 流水號
    /// 例如：藥001, 藥002, 耗001, 耗002, 飼001, 飼002, 儀001, 儀002
    pub async fn generate_code(pool: &PgPool, category: SupplierCategory) -> Result<String> {
        let prefix = match category {
            SupplierCategory::Drug => "藥",
            SupplierCategory::Consumable => "耗",
            SupplierCategory::Feed => "飼",
            SupplierCategory::Equipment => "儀",
        };

        // 查詢該類別的所有代碼
        let codes: Vec<String> = sqlx::query_scalar(
            "SELECT code FROM partners WHERE supplier_category = $1 AND code LIKE $2 ORDER BY code DESC"
        )
        .bind(category)
        .bind(format!("{}%", prefix))
        .fetch_all(pool)
        .await?;

        // 解析序號並找出最大值
        let max_seq = codes
            .iter()
            .filter_map(|code| {
                // 格式：藥001，提取最後的數字部分
                if code.starts_with(prefix) && code.len() > prefix.len() {
                    let num_str = &code[prefix.len()..];
                    num_str.parse::<i32>().ok()
                } else {
                    None
                }
            })
            .max();

        let seq = max_seq.map(|s| s + 1).unwrap_or(1);
        Ok(format!("{}{:03}", prefix, seq))
    }

    /// 建立夥伴（供應商/客戶）
    pub async fn create(pool: &PgPool, req: &CreatePartnerRequest) -> Result<Partner> {
        // 如果 code 為空且提供了 supplier_category，則自動生成
        let code = if req.code.is_none() || req.code.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()).is_none() {
            if let Some(category) = req.supplier_category {
                Self::generate_code(pool, category).await?
            } else {
                return Err(AppError::Validation("Code is required when supplier_category is not provided".to_string()));
            }
        } else {
            req.code.as_ref().unwrap().trim().to_string()
        };

        // 檢查 code 是否已存在
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM partners WHERE code = $1)"
        )
        .bind(&code)
        .fetch_one(pool)
        .await?;

        if exists {
            return Err(AppError::Conflict("Partner code already exists".to_string()));
        }

        // 將空字串轉換為 None，並驗證 email 格式（如果提供）
        let email = req.email.as_ref()
            .map(|e| e.trim())
            .filter(|e| !e.is_empty())
            .map(|e| {
                if !validator::validate_email(e) {
                    return Err(AppError::Validation("Invalid email format".to_string()));
                }
                Ok(e.to_string())
            })
            .transpose()?;

        let partner = sqlx::query_as::<_, Partner>(
            r#"
            INSERT INTO partners (
                id, partner_type, code, name, supplier_category, tax_id, phone, email, address, 
                payment_terms, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(&req.partner_type)
        .bind(&code)
        .bind(&req.name)
        .bind(&req.supplier_category)
        .bind(req.tax_id.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()))
        .bind(req.phone.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()))
        .bind(&email)
        .bind(req.address.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()))
        .bind(req.payment_terms.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()))
        .fetch_one(pool)
        .await?;

        Ok(partner)
    }

    /// 取得夥伴列表
    pub async fn list(pool: &PgPool, query: &PartnerQuery) -> Result<Vec<Partner>> {
        let partners = if let Some(ref kw) = query.keyword {
            let pattern = format!("%{}%", kw);
            if let Some(partner_type) = query.partner_type {
                if let Some(is_active) = query.is_active {
                    sqlx::query_as::<_, Partner>(
                        r#"
                        SELECT * FROM partners 
                        WHERE (code ILIKE $1 OR name ILIKE $1) 
                          AND partner_type = $2 
                          AND is_active = $3 
                        ORDER BY code
                        "#
                    )
                    .bind(&pattern)
                    .bind(partner_type)
                    .bind(is_active)
                    .fetch_all(pool)
                    .await?
                } else {
                    sqlx::query_as::<_, Partner>(
                        r#"
                        SELECT * FROM partners 
                        WHERE (code ILIKE $1 OR name ILIKE $1) 
                          AND partner_type = $2 
                        ORDER BY code
                        "#
                    )
                    .bind(&pattern)
                    .bind(partner_type)
                    .fetch_all(pool)
                    .await?
                }
            } else if let Some(is_active) = query.is_active {
                sqlx::query_as::<_, Partner>(
                    r#"
                    SELECT * FROM partners 
                    WHERE (code ILIKE $1 OR name ILIKE $1) 
                      AND is_active = $2 
                    ORDER BY code
                    "#
                )
                .bind(&pattern)
                .bind(is_active)
                .fetch_all(pool)
                .await?
            } else {
                sqlx::query_as::<_, Partner>(
                    "SELECT * FROM partners WHERE (code ILIKE $1 OR name ILIKE $1) ORDER BY code"
                )
                .bind(&pattern)
                .fetch_all(pool)
                .await?
            }
        } else if let Some(partner_type) = query.partner_type {
            if let Some(is_active) = query.is_active {
                sqlx::query_as::<_, Partner>(
                    "SELECT * FROM partners WHERE partner_type = $1 AND is_active = $2 ORDER BY code"
                )
                .bind(partner_type)
                .bind(is_active)
                .fetch_all(pool)
                .await?
            } else {
                sqlx::query_as::<_, Partner>(
                    "SELECT * FROM partners WHERE partner_type = $1 ORDER BY code"
                )
                .bind(partner_type)
                .fetch_all(pool)
                .await?
            }
        } else if let Some(is_active) = query.is_active {
            sqlx::query_as::<_, Partner>(
                "SELECT * FROM partners WHERE is_active = $1 ORDER BY code"
            )
            .bind(is_active)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, Partner>("SELECT * FROM partners ORDER BY code")
                .fetch_all(pool)
                .await?
        };

        Ok(partners)
    }

    /// 取得單一夥伴
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Partner> {
        let partner = sqlx::query_as::<_, Partner>(
            "SELECT * FROM partners WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Partner not found".to_string()))?;

        Ok(partner)
    }

    /// 更新夥伴
    pub async fn update(pool: &PgPool, id: Uuid, req: &UpdatePartnerRequest) -> Result<Partner> {
        // 處理 email：將空字串轉換為 None，並驗證格式（如果提供）
        let email = req.email.as_ref()
            .map(|e| e.trim())
            .filter(|e| !e.is_empty())
            .map(|e| {
                if !validator::validate_email(e) {
                    return Err(AppError::Validation("Invalid email format".to_string()));
                }
                Ok(e.to_string())
            })
            .transpose()?;

        let partner = sqlx::query_as::<_, Partner>(
            r#"
            UPDATE partners SET
                name = COALESCE($1, name),
                tax_id = COALESCE($2, tax_id),
                phone = COALESCE($3, phone),
                email = COALESCE($4, email),
                address = COALESCE($5, address),
                payment_terms = COALESCE($6, payment_terms),
                is_active = COALESCE($7, is_active),
                updated_at = NOW()
            WHERE id = $8
            RETURNING *
            "#
        )
        .bind(req.name.as_ref())
        .bind(req.tax_id.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()))
        .bind(req.phone.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()))
        .bind(&email)
        .bind(req.address.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()))
        .bind(req.payment_terms.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()))
        .bind(req.is_active)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Partner not found".to_string()))?;

        Ok(partner)
    }

    /// 刪除夥伴（軟刪除）
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        let result = sqlx::query(
            "UPDATE partners SET is_active = false, updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Partner not found".to_string()));
        }

        Ok(())
    }
}
