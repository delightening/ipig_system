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

    /// 依「正規化後」代碼查詢啟用中的欄位。
    ///
    /// 比對兩邊都做同一種正規化：大寫、去掉 `-`、去前後空白（呼叫端須先把輸入正規化）。
    /// 欄位代碼只在「同一區、啟用中」唯一，不同區理論上可有相同代碼；最多回 2 筆，
    /// 足以讓呼叫端判斷「找不到 / 唯一 / 有歧義」，不需要把全部列出。
    pub async fn find_active_pens_by_normalized_code(
        pool: &PgPool,
        normalized_code: &str,
    ) -> Result<Vec<Pen>> {
        let pens = sqlx::query_as::<_, Pen>(
            r#"
            SELECT * FROM pens
            WHERE is_active
              AND upper(replace(btrim(code), '-', '')) = $1
            ORDER BY id
            LIMIT 2
            "#,
        )
        .bind(normalized_code)
        .fetch_all(pool)
        .await?;
        Ok(pens)
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
