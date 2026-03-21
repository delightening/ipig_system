// 記錄內容查詢（用於簽章雜湊計算）

use sqlx::PgPool;
use uuid::Uuid;

use crate::{AppError, Result};

use super::SignatureService;

impl SignatureService {
    /// 取得犧牲記錄內容（用於生成簽章雜湊）
    pub async fn fetch_sacrifice_content(pool: &PgPool, id: Uuid) -> Result<String> {
        sqlx::query_scalar(
            r#"SELECT CONCAT(
                'sacrifice_id:', id::text,
                ',animal_id:', animal_id::text,
                ',date:', COALESCE(sacrifice_date::text, ''),
                ',confirmed:', confirmed_sacrifice::text
            ) FROM animal_sacrifices WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到犧牲記錄".into()))
    }

    /// 取得觀察記錄內容（用於生成簽章雜湊）
    pub async fn fetch_observation_content(pool: &PgPool, id: i32) -> Result<String> {
        sqlx::query_scalar(
            r#"SELECT CONCAT(
                'observation_id:', id::text,
                ',animal_id:', animal_id::text,
                ',date:', event_date::text,
                ',content:', content
            ) FROM animal_observations WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到觀察記錄".into()))
    }

    /// 取得安樂死單據內容（用於生成簽章雜湊）
    pub async fn fetch_euthanasia_content(pool: &PgPool, id: Uuid) -> Result<String> {
        sqlx::query_scalar(
            r#"SELECT CONCAT(
                'euthanasia_id:', id::text,
                ',animal_id:', animal_id::text,
                ',reason:', reason,
                ',status:', status
            ) FROM euthanasia_orders WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到安樂死單據".into()))
    }

    /// 取得轉讓記錄內容（用於生成簽章雜湊）
    pub async fn fetch_transfer_content(pool: &PgPool, id: Uuid) -> Result<String> {
        sqlx::query_scalar(
            r#"SELECT CONCAT(
                'transfer_id:', id::text,
                ',animal_id:', animal_id::text,
                ',from_iacuc:', from_iacuc_no,
                ',status:', status
            ) FROM animal_transfers WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到轉讓記錄".into()))
    }

    /// 取得計畫書內容（用於生成簽章雜湊）
    pub async fn fetch_protocol_content(pool: &PgPool, id: Uuid) -> Result<String> {
        sqlx::query_scalar(
            r#"SELECT CONCAT(
                'protocol_id:', id::text,
                ',title:', title,
                ',status:', status
            ) FROM protocols WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到計劃書".into()))
    }
}
