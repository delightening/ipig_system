//! 欄位對齊工具（`services::animal::pen_alignment`）專用的唯讀查詢。
//!
//! 「在場」定義與 `pens.current_count` 的維護程式一致：未刪除、
//! status 不在 euthanized / sudden_death / transferred
//! （見 `services/animal/core/update.rs` 重算 current_count 的 SQL）。
//! 本檔只有 SELECT；寫入一律走 `AnimalService::update`。

use sqlx::PgPool;
use uuid::Uuid;

use crate::Result;

/// 在場動物快照（只含對齊所需欄位）。
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AnimalSnap {
    pub id: Uuid,
    pub ear_tag: String,
    pub pen_id: Option<Uuid>,
    pub pen_location: Option<String>,
    /// 樂觀鎖版本；更新時帶回去，期間被別人改過就會 Conflict 而不是蓋掉。
    pub version: i32,
}

/// 欄位快照。`live_count` 是即時數出來的在場隻數，`current_count` 是欄位表上的快取值，
/// 兩者可能不一致（既有現況，不是本工具造成的）。
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PenSnap {
    pub id: Uuid,
    pub code: String,
    pub capacity: i32,
    pub current_count: i32,
    pub status: String,
    pub is_active: bool,
    pub live_count: i64,
}

pub struct PenAlignmentRepository;

impl PenAlignmentRepository {
    /// 全部在場（未刪除、非終態）動物。
    pub async fn list_live_animals(pool: &PgPool) -> Result<Vec<AnimalSnap>> {
        let rows = sqlx::query_as::<_, AnimalSnap>(
            r#"
            SELECT id, ear_tag, pen_id, pen_location, version
            FROM animals
            WHERE deleted_at IS NULL
              AND status NOT IN ('euthanized', 'sudden_death', 'transferred')
            ORDER BY ear_tag, id
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    /// 欄位快照；`only_ids` 為 `None` 取全部，否則只取指定的欄。
    pub async fn list_pens(pool: &PgPool, only_ids: Option<&[Uuid]>) -> Result<Vec<PenSnap>> {
        let rows = sqlx::query_as::<_, PenSnap>(
            r#"
            SELECT p.id, p.code, p.capacity, p.current_count, p.status, p.is_active,
                   (SELECT COUNT(*) FROM animals a
                     WHERE a.pen_id = p.id
                       AND a.deleted_at IS NULL
                       AND a.status NOT IN ('euthanized', 'sudden_death', 'transferred')
                   ) AS live_count
            FROM pens p
            WHERE ($1::uuid[] IS NULL OR p.id = ANY($1))
            ORDER BY p.code, p.id
            "#,
        )
        .bind(only_ids)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
