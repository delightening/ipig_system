//! 犧牲補登工具（`services::animal::sacrifice_backfill`）專用查詢。
//!
//! 唯一的寫入是 `recount_pen`：`upsert_sacrifice` 把動物改成 euthanized 時不會重算
//! `pens.current_count`（既有行為，本工具不去改它），所以補登後由本工具重算受影響的那一欄。
//! 算法與 `services/animal/core/update.rs` 重算 current_count 的 SQL 完全相同。

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::Result;

/// 以「耳號 + 出生日期」找到的候選動物（未刪除）。
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AnimalCandidate {
    pub id: Uuid,
    pub status: String,
    pub iacuc_no: Option<String>,
    pub pen_id: Option<Uuid>,
    pub version: i32,
    /// 是否已有犧牲紀錄（冪等判斷用）。
    pub has_sacrifice: bool,
}

pub struct SacrificeBackfillRepository;

impl SacrificeBackfillRepository {
    pub async fn find_candidates(
        pool: &PgPool,
        ear_tag: &str,
        birth_date: NaiveDate,
    ) -> Result<Vec<AnimalCandidate>> {
        let rows = sqlx::query_as::<_, AnimalCandidate>(
            r#"
            SELECT a.id, a.status::text AS status, a.iacuc_no, a.pen_id, a.version,
                   EXISTS (SELECT 1 FROM animal_sacrifices s WHERE s.animal_id = a.id)
                       AS has_sacrifice
            FROM animals a
            WHERE a.deleted_at IS NULL
              AND a.ear_tag = $1
              AND a.birth_date = $2
            ORDER BY a.id
            "#,
        )
        .bind(ear_tag)
        .bind(birth_date)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    /// 取動物目前的版本號（IACUC 補寫後版本會變，補登前重讀）。
    pub async fn current_version(pool: &PgPool, animal_id: Uuid) -> Result<Option<i32>> {
        let v = sqlx::query_scalar::<_, i32>(
            "SELECT version FROM animals WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(animal_id)
        .fetch_optional(pool)
        .await?;
        Ok(v)
    }

    /// 重算單一欄位的 current_count（只動 `pens.current_count` 這個快取欄）。
    pub async fn recount_pen(pool: &PgPool, pen_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE pens SET current_count = (SELECT COUNT(*) FROM animals WHERE pen_id = $1 AND deleted_at IS NULL AND status NOT IN ('euthanized', 'sudden_death', 'transferred')) WHERE id = $1",
        )
        .bind(pen_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// 操作者的真實角色代碼（授權檢查用，不偽造）。
    pub async fn user_role_codes(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT r.code FROM user_roles ur JOIN roles r ON r.id = ur.role_id WHERE ur.user_id = $1",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
