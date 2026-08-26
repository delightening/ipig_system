//! 設備報廢 / 閒置 / 維修的待處理人解析。
//!
//! 三者的授權出處：
//!
//! | 關卡 | 判準 | 職務分離 |
//! |---|---|---|
//! | 報廢待核准 | `equipment.disposal.approve`（`services/equipment/disposal.rs:325`） | 申請人不得自核（`:340`） |
//! | 閒置待核准 | `equipment.idle.approve`（`services/equipment/idle.rs:162`） | 申請人不得自核（`:196`） |
//! | 維修待處理 | `equipment.maintenance.manage`（`services/equipment/maintenance.rs:345`） | 無——這關是「等人去修」不是「等人核准」 |
//! | 維修待驗收 | `equipment.maintenance.review` **或** `equipment.manage`（`maintenance.rs:444-446`） | 登錄者不得自驗（`:466`） |

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::PendingOwner;

use super::{resolve_single_stage, CandidateSource, PendingRow};

const STAGE_DISPOSAL: &str = "equipment_disposal_approve";
const STAGE_IDLE: &str = "equipment_idle_approve";
const STAGE_MAINTENANCE_START: &str = "equipment_maintenance_start";
const STAGE_MAINTENANCE_REVIEW: &str = "equipment_maintenance_review";

/// 驗收關是「這個權限**或**那個權限」，只查其中一個會漏列另一半的人。
const MAINTENANCE_REVIEW_PERMISSIONS: &[&str] =
    &["equipment.maintenance.review", "equipment.manage"];

/// 報廢與閒置的資料形狀相同（同樣的 `applied_by` / `applied_at` / `status`）。
#[derive(Debug, sqlx::FromRow)]
struct AppliedStageRow {
    id: Uuid,
    applied_by: Uuid,
    applied_at: Option<DateTime<Utc>>,
}

impl AppliedStageRow {
    fn to_pending_row(&self) -> PendingRow {
        PendingRow {
            id: self.id,
            excluded: Some(self.applied_by),
            since: self.applied_at,
        }
    }
}

/// 設備報廢申請：待核准者。
pub async fn resolve_for_disposals(
    pool: &PgPool,
    disposal_ids: &[Uuid],
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
    if disposal_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as::<_, AppliedStageRow>(
        r#"
        SELECT id, applied_by, applied_at
        FROM equipment_disposals
        WHERE id = ANY($1) AND status::text = 'pending'
        "#,
    )
    .bind(disposal_ids)
    .fetch_all(pool)
    .await?;

    let pending: Vec<PendingRow> = rows.iter().map(AppliedStageRow::to_pending_row).collect();
    resolve_single_stage(
        pool,
        STAGE_DISPOSAL,
        None,
        CandidateSource::Permission("equipment.disposal.approve"),
        &pending,
    )
    .await
}

/// 設備閒置申請：待核准者。
pub async fn resolve_for_idle_requests(
    pool: &PgPool,
    request_ids: &[Uuid],
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
    if request_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as::<_, AppliedStageRow>(
        r#"
        SELECT id, applied_by, applied_at
        FROM equipment_idle_requests
        WHERE id = ANY($1) AND status::text = 'pending'
        "#,
    )
    .bind(request_ids)
    .fetch_all(pool)
    .await?;

    let pending: Vec<PendingRow> = rows.iter().map(AppliedStageRow::to_pending_row).collect();
    resolve_single_stage(
        pool,
        STAGE_IDLE,
        None,
        CandidateSource::Permission("equipment.idle.approve"),
        &pending,
    )
    .await
}

#[derive(Debug, sqlx::FromRow)]
struct MaintenanceStageRow {
    id: Uuid,
    status: String,
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// 設備維修保養：待處理（等人去修）與待驗收（等人簽收）兩關。
pub async fn resolve_for_maintenance(
    pool: &PgPool,
    record_ids: &[Uuid],
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
    if record_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as::<_, MaintenanceStageRow>(
        r#"
        SELECT id, status::text AS status, created_by, created_at, updated_at
        FROM equipment_maintenance_records
        WHERE id = ANY($1) AND status::text IN ('pending', 'pending_review')
        "#,
    )
    .bind(record_ids)
    .fetch_all(pool)
    .await?;

    // 待處理：等人開工，沒有「自己不能做自己的」這回事——登錄者本來就常是修的人。
    let to_start: Vec<PendingRow> = rows
        .iter()
        .filter(|r| r.status == "pending")
        .map(|r| PendingRow {
            id: r.id,
            excluded: None,
            since: Some(r.created_at),
        })
        .collect();

    // 待驗收：登錄者不得自驗（`maintenance.rs:466`）。進入本關的時間取
    // `updated_at`——送驗那一刻的寫入，`created_at` 是登錄維修單的時間、早得多。
    let to_review: Vec<PendingRow> = rows
        .iter()
        .filter(|r| r.status == "pending_review")
        .map(|r| PendingRow {
            id: r.id,
            excluded: r.created_by,
            since: Some(r.updated_at),
        })
        .collect();

    let mut result = resolve_single_stage(
        pool,
        STAGE_MAINTENANCE_START,
        None,
        CandidateSource::Permission("equipment.maintenance.manage"),
        &to_start,
    )
    .await?;

    result.extend(
        resolve_single_stage(
            pool,
            STAGE_MAINTENANCE_REVIEW,
            None,
            CandidateSource::AnyPermission(MAINTENANCE_REVIEW_PERMISSIONS),
            &to_review,
        )
        .await?,
    );

    Ok(result)
}
