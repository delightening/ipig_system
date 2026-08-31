//! 獸醫巡場報告的待處理人解析。
//!
//! 送出後的兩個在途狀態都卡在**同一個特定人**——獸醫送出時指派的追蹤者
//! （`vet_patrol_reports.follow_up_user_id`）：
//!
//! | 狀態 | 在等什麼 | 誰能做 |
//! |---|---|---|
//! | `awaiting_acknowledgement` | 追蹤者按「確認收到」 | 追蹤者本人（`services/animal/vet_patrol.rs:1293` 前置條件 + `:504` 的可見性查詢） |
//! | `awaiting_follow_up` | 追蹤者回覆追蹤改善並完成 | 同上（`:1385`） |
//!
//! 因為只有被指派的那一位做得到，`kind` 是 `Person` 而非 `Role`——顯示人名、不提角色。

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{PendingOwner, PendingOwnerKind};

/// 待追蹤者確認收到。
const STAGE_AWAITING_ACK: &str = "vet_patrol_acknowledge";
/// 待追蹤者回覆追蹤改善。
const STAGE_AWAITING_FOLLOW_UP: &str = "vet_patrol_follow_up";

#[derive(Debug, sqlx::FromRow)]
struct VetPatrolStageRow {
    id: Uuid,
    status: String,
    follower_name: Option<String>,
    /// 進入本關的時間：確認收到之後看 `acknowledged_at`，否則看送出時的 `updated_at`。
    since: Option<DateTime<Utc>>,
}

/// 一次算出多份巡場報告的待處理人。草稿與已完成的 id 不會出現在結果裡。
pub async fn resolve_for_vet_patrol_reports(
    pool: &PgPool,
    report_ids: &[Uuid],
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
    if report_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as::<_, VetPatrolStageRow>(
        r#"
        SELECT r.id,
               r.status,
               u.display_name AS follower_name,
               COALESCE(r.acknowledged_at, r.updated_at) AS since
        FROM vet_patrol_reports r
        LEFT JOIN users u ON u.id = r.follow_up_user_id
        WHERE r.id = ANY($1)
          AND r.deleted_at IS NULL
          AND r.status IN ('awaiting_acknowledgement', 'awaiting_follow_up')
        "#,
    )
    .bind(report_ids)
    .fetch_all(pool)
    .await?;

    let mut result = HashMap::with_capacity(rows.len());
    for row in rows {
        let stage = if row.status == "awaiting_acknowledgement" {
            STAGE_AWAITING_ACK
        } else {
            STAGE_AWAITING_FOLLOW_UP
        };
        result.insert(
            row.id,
            PendingOwner::from_candidates(
                stage,
                PendingOwnerKind::Person,
                None,
                row.follower_name.into_iter().collect(),
                row.since,
            ),
        );
    }

    Ok(result)
}
