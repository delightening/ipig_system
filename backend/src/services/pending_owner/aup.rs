//! AUP 計畫、變更申請、PI 開通信的待處理人解析（權限型關卡）。
//!
//! ⚠️ **本檔只涵蓋「行政受理 / 分類」那幾關**。委員會審查（`UNDER_REVIEW`）與獸醫審查
//! （`VET_REVIEW`）卡在**被指派的特定人**、需修正類（`*_REVISION_REQUIRED`）卡在申請人，
//! 三者的形狀都不是「權限型」，留給後續處理。
//!
//! | 關卡 | 判準 |
//! |---|---|
//! | 計畫待行政受理（`SUBMITTED` / `PRE_REVIEW` / `RESUBMITTED`） | `aup.protocol.change_status`（`handlers/protocol/crud.rs:418`） |
//! | 變更申請待分類（`SUBMITTED` / `RESUBMITTED`） | `aup.amendment.classify`（`handlers/amendment.rs:235`） |
//! | 變更申請已分類待送審（`CLASSIFIED`） | `aup.protocol.change_status`（`handlers/amendment.rs:347`、`:405`） |

use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::PendingOwner;

use super::{resolve_single_stage, CandidateSource, PendingRow};

const STAGE_PROTOCOL_INTAKE: &str = "aup_protocol_intake";
const STAGE_AMENDMENT_CLASSIFY: &str = "aup_amendment_classify";
const STAGE_AMENDMENT_TO_REVIEW: &str = "aup_amendment_to_review";

const PERMISSION_CHANGE_STATUS: &str = "aup.protocol.change_status";

/// 計畫停在「等行政作業」的三個狀態。
///
/// `PRE_REVIEW`（行政預審中）也算——球仍在執行秘書手上，不在申請人手上。
const PROTOCOL_INTAKE_STATUSES: &[&str] = &["SUBMITTED", "PRE_REVIEW", "RESUBMITTED"];

#[derive(Debug, sqlx::FromRow)]
struct SubmittedStageRow {
    id: Uuid,
    /// ⚠️ `protocols.submitted_at` 是 **DATE** 不是 TIMESTAMPTZ（2026-08-26 實查）。
    /// 宣告成 `DateTime<Utc>` 會在解列時噴 `ColumnDecode ... mismatched types`。
    /// 精度只到「日」對本用途完全夠：前端只拿它算已等待幾天。
    submitted_at: Option<NaiveDate>,
    updated_at: DateTime<Utc>,
}

impl SubmittedStageRow {
    /// 進入本關的時間：優先用送審日，沒有就退回最後一次寫入。
    ///
    /// 送審日轉時間戳時取當日 00:00 UTC——前端 `getWaitingDays` 會換算回台灣時區的
    /// 日期鍵，08:00+08:00 仍是同一天，不會差一天。
    fn to_pending_row(&self) -> PendingRow {
        let since = self
            .submitted_at
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|dt| dt.and_utc())
            .unwrap_or(self.updated_at);
        PendingRow {
            id: self.id,
            excluded: None,
            since: Some(since),
        }
    }
}

/// AUP 計畫：卡在行政受理 / 預審的那幾筆。
pub async fn resolve_for_protocols(
    pool: &PgPool,
    protocol_ids: &[Uuid],
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
    if protocol_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let statuses: Vec<String> = PROTOCOL_INTAKE_STATUSES
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    let rows = sqlx::query_as::<_, SubmittedStageRow>(
        r#"
        -- protocols 沒有 deleted_at 欄位：軟刪除是把 status 設成 'DELETED'
        -- （2026-08-26 實查 16 個 protocol_status 值）。狀態白名單本身已排除它。
        SELECT id, submitted_at, updated_at
        FROM protocols
        WHERE id = ANY($1) AND status::text = ANY($2)
        "#,
    )
    .bind(protocol_ids)
    .bind(&statuses)
    .fetch_all(pool)
    .await?;

    let pending: Vec<PendingRow> = rows.iter().map(SubmittedStageRow::to_pending_row).collect();
    resolve_single_stage(
        pool,
        STAGE_PROTOCOL_INTAKE,
        None,
        CandidateSource::Permission(PERMISSION_CHANGE_STATUS),
        &pending,
    )
    .await
}

#[derive(Debug, sqlx::FromRow)]
struct AmendmentStageRow {
    id: Uuid,
    status: String,
    submitted_at: Option<DateTime<Utc>>,
    updated_at: DateTime<Utc>,
}

/// 變更申請：待分類與已分類待送審兩關。
pub async fn resolve_for_amendments(
    pool: &PgPool,
    amendment_ids: &[Uuid],
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
    if amendment_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as::<_, AmendmentStageRow>(
        r#"
        SELECT id, status::text AS status, submitted_at, updated_at
        FROM amendments
        WHERE id = ANY($1) AND status::text IN ('SUBMITTED', 'RESUBMITTED', 'CLASSIFIED')
        "#,
    )
    .bind(amendment_ids)
    .fetch_all(pool)
    .await?;

    let to_pending_row = |r: &AmendmentStageRow| PendingRow {
        id: r.id,
        excluded: None,
        since: Some(r.submitted_at.unwrap_or(r.updated_at)),
    };

    let to_classify: Vec<PendingRow> = rows
        .iter()
        .filter(|r| r.status == "SUBMITTED" || r.status == "RESUBMITTED")
        .map(to_pending_row)
        .collect();
    let to_review: Vec<PendingRow> = rows
        .iter()
        .filter(|r| r.status == "CLASSIFIED")
        .map(to_pending_row)
        .collect();

    let mut result = resolve_single_stage(
        pool,
        STAGE_AMENDMENT_CLASSIFY,
        None,
        CandidateSource::Permission("aup.amendment.classify"),
        &to_classify,
    )
    .await?;

    result.extend(
        resolve_single_stage(
            pool,
            STAGE_AMENDMENT_TO_REVIEW,
            None,
            CandidateSource::Permission(PERMISSION_CHANGE_STATUS),
            &to_review,
        )
        .await?,
    );

    Ok(result)
}

// PI 開通信（`pi_account_invites`）刻意不做。
//
// `handlers/protocol/pi_provision.rs:80` 讓**只有系統管理員**看得到那個列表，
// 而它的合法核准人是 `aup.pi_invite.approve` + 管理員短路——換句話說，
// 看得到的人本身就是能核准的人。tooltip 只會顯示「卡在系統管理員：你自己」，
// 零資訊量。要做也應該先讓非管理員看得到那個列表，那是另一件事。
