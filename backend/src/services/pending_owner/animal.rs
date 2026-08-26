//! 動物欄位更正申請的待處理人解析。
//!
//! 單一關卡、單一權限碼：`handlers/animal/field_correction.rs:44` 與 `:57`
//! （核准與駁回兩個端點）都只要 `animal.field_correction.review`。
//!
//! ⚠️ **這一關沒有職務分離**：`services/animal/field_correction.rs` 全檔沒有
//! `assert_not_self_approval`，申請人若同時具備 review 權限，實際上是可以自核的。
//! 候選名單照實反映現況——這裡不是加守衛的地方，若要補 SoD 請改 service 並同步本檔。

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::PendingOwner;

use super::{resolve_single_stage, CandidateSource, PendingRow};

const STAGE: &str = "animal_field_correction_review";
const PERMISSION: &str = "animal.field_correction.review";

#[derive(Debug, sqlx::FromRow)]
struct FieldCorrectionStageRow {
    id: Uuid,
    created_at: DateTime<Utc>,
}

/// 一次算出多筆欄位更正申請的待處理人。非 `pending` 的 id 不會出現在結果裡。
pub async fn resolve_for_field_corrections(
    pool: &PgPool,
    request_ids: &[Uuid],
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
    if request_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as::<_, FieldCorrectionStageRow>(
        r#"
        SELECT id, created_at
        FROM animal_field_correction_requests
        WHERE id = ANY($1) AND status = 'pending'
        "#,
    )
    .bind(request_ids)
    .fetch_all(pool)
    .await?;

    let pending: Vec<PendingRow> = rows
        .iter()
        .map(|r| PendingRow {
            id: r.id,
            excluded: None,
            since: Some(r.created_at),
        })
        .collect();

    // role_code = None：這一關綁的是權限碼不是單一角色，硬指一個角色會誤導
    // （持有 review 權的可能不只一種角色）。
    resolve_single_stage(
        pool,
        STAGE,
        None,
        CandidateSource::Permission(PERMISSION),
        &pending,
    )
    .await
}
