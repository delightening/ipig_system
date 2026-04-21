use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{audit_diff::DataDiff, AnimalSource, CreateAnimalSourceRequest, UpdateAnimalSourceRequest},
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

pub struct AnimalSourceService;

impl AnimalSourceService {
    // ============================================
    // 動物來源管理
    // ============================================

    /// 取得動物來源列表
    pub async fn list_sources(pool: &PgPool) -> Result<Vec<AnimalSource>> {
        let sources = sqlx::query_as::<_, AnimalSource>(
            "SELECT * FROM animal_sources WHERE is_active = true ORDER BY sort_order",
        )
        .fetch_all(pool)
        .await?;

        Ok(sources)
    }

    /// 建立動物來源（Service-driven audit）
    pub async fn create_source(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateAnimalSourceRequest,
    ) -> Result<AnimalSource> {
        let _user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let source = sqlx::query_as::<_, AnimalSource>(
            r#"
            INSERT INTO animal_sources (id, code, name, address, contact, phone, phone_ext, is_active, sort_order, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, true, 0, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&req.code)
        .bind(&req.name)
        .bind(&req.address)
        .bind(&req.contact)
        .bind(&req.phone)
        .bind(&req.phone_ext)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry::create(
                "ANIMAL",
                "SOURCE_CREATE",
                AuditEntity::new("animal_source", source.id, &source.name),
                &source,
            ),
        )
        .await?;

        tx.commit().await?;
        Ok(source)
    }

    /// 更新動物來源（Service-driven audit）
    pub async fn update_source(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateAnimalSourceRequest,
    ) -> Result<AnimalSource> {
        let _user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        // 讀 before（含列鎖）
        let before = sqlx::query_as::<_, AnimalSource>(
            "SELECT * FROM animal_sources WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Animal source not found".to_string()))?;

        let after = sqlx::query_as::<_, AnimalSource>(
            r#"
            UPDATE animal_sources SET
                name = COALESCE($2, name),
                address = COALESCE($3, address),
                contact = COALESCE($4, contact),
                phone = COALESCE($5, phone),
                phone_ext = COALESCE($6, phone_ext),
                is_active = COALESCE($7, is_active),
                sort_order = COALESCE($8, sort_order),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&req.address)
        .bind(&req.contact)
        .bind(&req.phone)
        .bind(&req.phone_ext)
        .bind(req.is_active)
        .bind(req.sort_order)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry::update(
                "ANIMAL",
                "SOURCE_UPDATE",
                AuditEntity::new("animal_source", after.id, &after.name),
                DataDiff::compute(Some(&before), Some(&after)),
            ),
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    /// 刪除（停用）動物來源（Service-driven audit）
    pub async fn delete_source(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        let _user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, AnimalSource>(
            "SELECT * FROM animal_sources WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Animal source not found".to_string()))?;

        sqlx::query(
            "UPDATE animal_sources SET is_active = false, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry::delete(
                "ANIMAL",
                "SOURCE_DELETE",
                AuditEntity::new("animal_source", before.id, &before.name),
                &before,
            ),
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
