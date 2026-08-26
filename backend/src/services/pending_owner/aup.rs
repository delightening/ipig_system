//! AUP 計畫與變更申請的待處理人解析。
//!
//! 本檔涵蓋三種形狀，各自對應不同的 [`PendingOwnerKind`]：
//!
//! | 關卡 | 判準 | kind |
//! |---|---|---|
//! | 計畫待行政受理（`SUBMITTED` / `PRE_REVIEW` / `RESUBMITTED`） | `aup.protocol.change_status`（`handlers/protocol/crud.rs:418`） | Role |
//! | 計畫獸醫審查（`VET_REVIEW`） | `vet_review_assignments.vet_id`（`services/protocol/review.rs:209`） | Person |
//! | 計畫委員會審查（`UNDER_REVIEW`） | `review_assignments`（`review.rs:119`） | **Anonymous** |
//! | 計畫需修正（`*_REVISION_REQUIRED`） | 球在申請人身上 | Applicant |
//! | 變更申請待分類（`SUBMITTED` / `RESUBMITTED`） | `aup.amendment.classify`（`handlers/amendment.rs:235`） | Role |
//! | 變更申請已分類待送審（`CLASSIFIED`） | `aup.protocol.change_status`（`handlers/amendment.rs:347`、`:405`） | Role |
//! | 變更申請委員會審查（`UNDER_REVIEW`） | `amendment_review_assignments` | **Anonymous** |
//! | 變更申請需修正（`REVISION_REQUIRED`） | 球在申請人身上 | Applicant |
//!
//! # 為什麼委員會審查不列名
//!
//! 2026-08-26 使用者裁定：IACUC 審查委員的身分**對所有人一律不揭露**，只給人數。
//!
//! ⚠️ **2026-08-26 事實訂正**：本段原本寫「用狀態擋是因為系統反正已經到處洩漏委員身分
//! （`comment.rs:105` 把 `reviewer_name` / `reviewer_email` 回給含 PI 的所有 scope 內使用者）」
//! ——**那是錯的**。實查 `handlers/protocol/review.rs:242-248`，該端點在回傳前依
//! `aup.review.identity_view` 裁剪：沒有該權限者拿到的是 `reviewer_name = "審查者"`、
//! `reviewer_email = None`。正式庫實查該權限授予 `IACUC_CHAIR` / `IACUC_STAFF` /
//! `REVIEWER` / `VET`（＋ admin 短路），**PI 不在內**。
//! 另一條 `list_review_assignments`（`review.rs:88-138`）不裁剪，但靠 `:96-103` 的
//! 守衛擋在門外（需 `view_all` / admin / 該案審查委員本人），PI 一樣進不來。
//!
//! 也就是說**系統既有行為本來就是「委員身分只給審查方、不給申請方」**。本檔目前的
//! 一律不列名比它更嚴（連 IACUC 執行秘書也看不到名字），這一點待使用者裁定是否
//! 改為比照 `aup.review.identity_view`。

use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{PendingOwner, PendingOwnerKind};

use super::{resolve_single_stage, CandidateSource, PendingRow};

const STAGE_PROTOCOL_INTAKE: &str = "aup_protocol_intake";
const STAGE_PROTOCOL_VET_REVIEW: &str = "aup_protocol_vet_review";
const STAGE_PROTOCOL_UNDER_REVIEW: &str = "aup_protocol_under_review";
const STAGE_PROTOCOL_REVISION: &str = "aup_protocol_revision";
const STAGE_AMENDMENT_CLASSIFY: &str = "aup_amendment_classify";
const STAGE_AMENDMENT_TO_REVIEW: &str = "aup_amendment_to_review";
const STAGE_AMENDMENT_UNDER_REVIEW: &str = "aup_amendment_under_review";
const STAGE_AMENDMENT_REVISION: &str = "aup_amendment_revision";

const PERMISSION_CHANGE_STATUS: &str = "aup.protocol.change_status";

/// 委員會審查關對外顯示的角色（只給角色與人數，不給名字）。
const ROLE_REVIEWER: &str = "REVIEWER";

/// 計畫停在「等行政作業」的三個狀態。
///
/// `PRE_REVIEW`（行政預審中）也算——球仍在執行秘書手上，不在申請人手上。
///
/// ⚠️ **`PRE_REVIEW` 有第二條出口**：`handlers/protocol/ai_review.rs:163-177` 的
/// `staff_batch_return`（批次退回補件）也會離開這個狀態，而它的守衛是**角色**
/// （`IACUC_STAFF` / `IACUC_CHAIR` / `SYSTEM_ADMIN`，用 `roles.contains`）而非權限碼。
/// 本檔取 `change_protocol_status` 的權限判準，涵蓋範圍是前者的超集
/// （`aup.protocol.change_status` 授予 `IACUC_CHAIR` / `IACUC_STAFF` / `admin`），
/// 方向是「可能多列一兩個做不了某個特定動作的人」而非漏列，可接受。
///
/// ⚠️ 另註（超出本檔範圍，已回報）：`staff_batch_return` 比對的是
/// `ROLE_SYSTEM_ADMIN`＝`"SYSTEM_ADMIN"`，而 `roles` 表裡的管理員代碼是 `admin`
/// （2026-08-26 實查 15 個角色）。那條管理員放行實際上永遠不成立。
const PROTOCOL_INTAKE_STATUSES: &[&str] = &["SUBMITTED", "PRE_REVIEW", "RESUBMITTED"];

/// 計畫停在「等申請人補件」的三個狀態。三個都是把球踢回 PI，只是關卡不同。
const PROTOCOL_REVISION_STATUSES: &[&str] = &[
    "REVISION_REQUIRED",
    "PRE_REVIEW_REVISION_REQUIRED",
    "VET_REVISION_REQUIRED",
];

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

/// 卡在特定人 / 申請人身上的計畫列。
#[derive(Debug, sqlx::FromRow)]
struct ProtocolAssigneeRow {
    id: Uuid,
    status: String,
    /// 該關卡的負責人顯示名（獸醫 / 申請人）；查不到姓名時為 None。
    assignee_name: Option<String>,
    /// 委員會審查關的指派委員人數。
    reviewer_count: i64,
    since: Option<DateTime<Utc>>,
}

/// AUP 計畫：卡在行政受理 / 預審的那幾筆。
async fn resolve_protocol_intake(
    pool: &PgPool,
    protocol_ids: &[Uuid],
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
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

/// AUP 計畫：卡在獸醫 / 委員會 / 申請人身上的那幾筆。
///
/// 這三關的共通點是「負責人由資料決定、不是由權限決定」，所以走一支帶 JOIN 的查詢，
/// 不經過 [`resolve_single_stage`]（那支是給權限型用的）。
async fn resolve_protocol_assignees(
    pool: &PgPool,
    protocol_ids: &[Uuid],
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
    let revision_statuses: Vec<String> = PROTOCOL_REVISION_STATUSES
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    let rows = sqlx::query_as::<_, ProtocolAssigneeRow>(
        r#"
        SELECT p.id,
               p.status::text AS status,
               CASE
                   WHEN p.status::text = 'VET_REVIEW' THEN vu.display_name
                   WHEN p.status::text = ANY($2) THEN pi.display_name
                   ELSE NULL
               END AS assignee_name,
               (SELECT COUNT(*) FROM review_assignments ra
                 WHERE ra.protocol_id = p.id AND ra.completed_at IS NULL) AS reviewer_count,
               COALESCE(p.submitted_at, p.updated_at) AS since
        FROM protocols p
        LEFT JOIN vet_review_assignments vra ON vra.protocol_id = p.id
        LEFT JOIN users vu ON vu.id = vra.vet_id
        LEFT JOIN users pi ON pi.id = p.pi_user_id
        -- 同上：軟刪除靠 status='DELETED'，狀態白名單已排除。
        WHERE p.id = ANY($1)
          AND (p.status::text IN ('VET_REVIEW', 'UNDER_REVIEW') OR p.status::text = ANY($2))
        "#,
    )
    .bind(protocol_ids)
    .bind(&revision_statuses)
    .fetch_all(pool)
    .await?;

    let mut result = HashMap::with_capacity(rows.len());
    for row in rows {
        let owner = match row.status.as_str() {
            // 委員會審查：一律不列名，只給人數（2026-08-26 使用者裁定）。
            "UNDER_REVIEW" => PendingOwner::anonymous(
                STAGE_PROTOCOL_UNDER_REVIEW,
                ROLE_REVIEWER,
                row.reviewer_count,
                row.since,
            ),
            // 獸醫審查：指派給特定一位獸醫，列名。
            "VET_REVIEW" => PendingOwner::from_candidates(
                STAGE_PROTOCOL_VET_REVIEW,
                PendingOwnerKind::Person,
                None,
                row.assignee_name.into_iter().collect(),
                row.since,
            ),
            // 需修正 / 補件：球回到申請人身上。這一關「卡在誰」的答案是 PI 自己，
            // 顯示出來才不會讓人以為還在等審查。
            _ => PendingOwner::from_candidates(
                STAGE_PROTOCOL_REVISION,
                PendingOwnerKind::Applicant,
                None,
                row.assignee_name.into_iter().collect(),
                row.since,
            ),
        };
        result.insert(row.id, owner);
    }

    Ok(result)
}

/// AUP 計畫：一次算出多筆的待處理人（涵蓋行政受理、獸醫、委員會、需修正四種形狀）。
pub async fn resolve_for_protocols(
    pool: &PgPool,
    protocol_ids: &[Uuid],
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
    if protocol_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut result = resolve_protocol_intake(pool, protocol_ids).await?;
    result.extend(resolve_protocol_assignees(pool, protocol_ids).await?);
    Ok(result)
}

#[derive(Debug, sqlx::FromRow)]
struct AmendmentStageRow {
    id: Uuid,
    status: String,
    submitted_at: Option<DateTime<Utc>>,
    /// 分類完成的時間。`CLASSIFIED` 是**分類之後**才開始等送審，用 `submitted_at`
    /// 會把「送審到分類」那段也算進等待天數（CodeRabbit 於 #30 指出）。
    classified_at: Option<DateTime<Utc>>,
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
        SELECT id, status::text AS status, submitted_at, classified_at, updated_at
        FROM amendments
        WHERE id = ANY($1) AND status::text IN ('SUBMITTED', 'RESUBMITTED', 'CLASSIFIED')
        "#,
    )
    .bind(amendment_ids)
    .fetch_all(pool)
    .await?;

    // 「進入本關的時間」必須是**該關卡開始等待的時刻**，不是整件事的起點。
    // 兩關的起點不同，共用同一個算式會讓已分類的件把「送審→分類」那段也算成等待
    // （CodeRabbit 於 #30 指出；同樣的道理已套用在單據終審關與維修待驗收關）。
    let since_submitted = |r: &AmendmentStageRow| PendingRow {
        id: r.id,
        excluded: None,
        since: Some(r.submitted_at.unwrap_or(r.updated_at)),
    };
    let since_classified = |r: &AmendmentStageRow| PendingRow {
        id: r.id,
        excluded: None,
        since: Some(r.classified_at.unwrap_or(r.updated_at)),
    };

    let to_classify: Vec<PendingRow> = rows
        .iter()
        .filter(|r| r.status == "SUBMITTED" || r.status == "RESUBMITTED")
        .map(since_submitted)
        .collect();
    let to_review: Vec<PendingRow> = rows
        .iter()
        .filter(|r| r.status == "CLASSIFIED")
        .map(since_classified)
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

    result.extend(resolve_amendment_assignees(pool, amendment_ids).await?);

    Ok(result)
}

/// 卡在委員會 / 申請人身上的變更申請列。
#[derive(Debug, sqlx::FromRow)]
struct AmendmentAssigneeRow {
    id: Uuid,
    status: String,
    /// 申請人顯示名（`REVISION_REQUIRED` 用）。
    applicant_name: Option<String>,
    /// 委員會審查關的指派委員人數。
    reviewer_count: i64,
    since: Option<DateTime<Utc>>,
}

/// 變更申請：委員會審查（不列名）與需修正（卡在申請人）。
async fn resolve_amendment_assignees(
    pool: &PgPool,
    amendment_ids: &[Uuid],
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
    let rows = sqlx::query_as::<_, AmendmentAssigneeRow>(
        r#"
        SELECT a.id,
               a.status::text AS status,
               COALESCE(su.display_name, cu.display_name) AS applicant_name,
               (SELECT COUNT(*) FROM amendment_review_assignments ara
                 WHERE ara.amendment_id = a.id AND ara.decided_at IS NULL) AS reviewer_count,
               COALESCE(a.submitted_at, a.updated_at) AS since
        FROM amendments a
        LEFT JOIN users su ON su.id = a.submitted_by
        LEFT JOIN users cu ON cu.id = a.created_by
        WHERE a.id = ANY($1)
          AND a.status::text IN ('UNDER_REVIEW', 'REVISION_REQUIRED')
        "#,
    )
    .bind(amendment_ids)
    .fetch_all(pool)
    .await?;

    let mut result = HashMap::with_capacity(rows.len());
    for row in rows {
        let owner = if row.status == "UNDER_REVIEW" {
            PendingOwner::anonymous(
                STAGE_AMENDMENT_UNDER_REVIEW,
                ROLE_REVIEWER,
                row.reviewer_count,
                row.since,
            )
        } else {
            PendingOwner::from_candidates(
                STAGE_AMENDMENT_REVISION,
                PendingOwnerKind::Applicant,
                None,
                row.applicant_name.into_iter().collect(),
                row.since,
            )
        };
        result.insert(row.id, owner);
    }

    Ok(result)
}

// PI 開通信（`pi_account_invites`）刻意不做。
//
// `handlers/protocol/pi_provision.rs:80` 讓**只有系統管理員**看得到那個列表，
// 而它的合法核准人是 `aup.pi_invite.approve` + 管理員短路——換句話說，
// 看得到的人本身就是能核准的人。tooltip 只會顯示「卡在系統管理員：你自己」，
// 零資訊量。要做也應該先讓非管理員看得到那個列表，那是另一件事。
