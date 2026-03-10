use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{AnimalSource, CreateAnimalSourceRequest, UpdateAnimalSourceRequest},
    Result,
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

    /// 建立動物來源
    pub async fn create_source(
        pool: &PgPool,
        req: &CreateAnimalSourceRequest,
    ) -> Result<AnimalSource> {
        let source = sqlx::query_as::<_, AnimalSource>(
            r#"
            INSERT INTO animal_sources (id, code, name, address, contact, phone, is_active, sort_order, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, true, 0, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&req.code)
        .bind(&req.name)
        .bind(&req.address)
        .bind(&req.contact)
        .bind(&req.phone)
        .fetch_one(pool)
        .await?;

        Ok(source)
    }

    /// 更新動物來源
    pub async fn update_source(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateAnimalSourceRequest,
    ) -> Result<AnimalSource> {
        let source = sqlx::query_as::<_, AnimalSource>(
            r#"
            UPDATE animal_sources SET
                name = COALESCE($2, name),
                address = COALESCE($3, address),
                contact = COALESCE($4, contact),
                phone = COALESCE($5, phone),
                is_active = COALESCE($6, is_active),
                sort_order = COALESCE($7, sort_order),
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
        .bind(req.is_active)
        .bind(req.sort_order)
        .fetch_one(pool)
        .await?;

        Ok(source)
    }

    /// 刪除（停用）動物來源
    pub async fn delete_source(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE animal_sources SET is_active = false, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
