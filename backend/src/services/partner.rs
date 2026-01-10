use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{CreatePartnerRequest, Partner, PartnerQuery, UpdatePartnerRequest},
    AppError, Result,
};

pub struct PartnerService;

impl PartnerService {
    /// 建立夥伴（供應商/客戶）
    pub async fn create(pool: &PgPool, req: &CreatePartnerRequest) -> Result<Partner> {
        // 檢查 code 是否已存在
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM partners WHERE code = $1)"
        )
        .bind(&req.code)
        .fetch_one(pool)
        .await?;

        if exists {
            return Err(AppError::Conflict("Partner code already exists".to_string()));
        }

        let partner = sqlx::query_as::<_, Partner>(
            r#"
            INSERT INTO partners (
                id, partner_type, code, name, tax_id, phone, email, address, 
                payment_terms, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(&req.partner_type)
        .bind(&req.code)
        .bind(&req.name)
        .bind(&req.tax_id)
        .bind(&req.phone)
        .bind(&req.email)
        .bind(&req.address)
        .bind(&req.payment_terms)
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
        .bind(&req.name)
        .bind(&req.tax_id)
        .bind(&req.phone)
        .bind(&req.email)
        .bind(&req.address)
        .bind(&req.payment_terms)
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
