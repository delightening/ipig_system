use sqlx::PgPool;
use uuid::Uuid;

use crate::models::Pen;
use crate::Result;

pub struct PenRepository;

impl PenRepository {
    /// 依 ID 查詢欄位
    pub async fn find_pen_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Pen>> {
        let pen = sqlx::query_as::<_, Pen>("SELECT * FROM pens WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(pen)
    }

    /// 計算欄位中存活動物數量（排除終態）
    pub async fn count_active_animals_in_pen(pool: &PgPool, pen_id: Uuid) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM animals
            WHERE pen_id = $1
            AND deleted_at IS NULL
            AND status NOT IN ('euthanized', 'sudden_death', 'transferred')
            "#,
        )
        .bind(pen_id)
        .fetch_one(pool)
        .await?;
        Ok(count)
    }
}
