//! AUP 計畫與變更申請的待處理人解析。
//!
//! 本檔涵蓋三種形狀，各自對應不同的 [`PendingOwnerKind`]：
//!
//! | 關卡 | 判準 | kind |
//! |---|---|---|
//! | 計畫待行政受理（`SUBMITTED` / `PRE_REVIEW` / `RESUBMITTED`） | `aup.protocol.change_status`（`handlers/protocol/crud.rs:418`） | Role |
//! | 計畫獸醫審查（`VET_REVIEW`） | `vet_review_assignments.vet_id`（`services/protocol/review.rs:209`） | Person |
//! | 計畫委員會審查（`UNDER_REVIEW`） | `review_assignments`（`review.rs:119`）；**僅 IACUC 行政方可見** | Role |
//! | 計畫需修正（`*_REVISION_REQUIRED`） | 球在申請人身上 | Applicant |
//! | 變更申請待分類（`SUBMITTED` / `RESUBMITTED`） | `aup.amendment.classify`（`handlers/amendment.rs:235`） | Role |
//! | 變更申請已分類待送審（`CLASSIFIED`） | `aup.protocol.change_status`（`handlers/amendment.rs:347`、`:405`） | Role |
//! | 變更申請委員會審查（`UNDER_REVIEW`） | `amendment_review_assignments`；**僅 IACUC 行政方可見** | Role |
//! | 變更申請需修正（`REVISION_REQUIRED`） | 球在申請人身上 | Applicant |
//!
//! # 委員會審查的委員身分：只給 IACUC 行政方，其餘人完全沒有 tooltip
//!
//! 2026-08-26 使用者裁定（分兩次收斂）：
//!
//! | 誰在看 | 委員會審查關的 tooltip |
//! |---|---|
//! | IACUC 執行秘書 / 主席 | **列出委員姓名** |
//! | 獸醫 / 審查委員 / PI / 其餘所有人 | **完全不顯示**（不是顯示人數，是整個不出現） |
//!
//! 判準用 `aup.protocol.change_status`。正式庫實查該權限授予
//! `IACUC_CHAIR` / `IACUC_STAFF` / `admin`——與裁定精確吻合。
//!
//! ⚠️ **刻意不用 `aup.review.identity_view`**（那是我最初的提案）。實查該權限
//! 還授予 `REVIEWER` 與 `VET`，用它會讓獸醫與審查委員看得到委員名單，
//! **與裁定相反**。兩個權限碼在「誰算審查方」上的語意不同：
//! `identity_view` 管的是「審查流程內部彼此可見」，本 tooltip 是列表頁的旁註、
//! 受眾更廣，所以收緊到行政方。
//!
//! ⚠️ `admin` 有 `change_status` 但**沒有** `identity_view`，所以他在這裡看得到、
//! 在計畫審查意見頁反而看不到名字。這個不一致是既有的，本檔不處理——
//! 管理員本來就查得到資料庫，擋他沒有實際意義。
//!
//! ⚠️ **一個曾經寫錯的理由**：本段原本寫「用狀態擋是因為系統反正已經到處洩漏委員身分
//! （`comment.rs:105`）」——那是錯的。實查 `handlers/protocol/review.rs:242-248`，
//! 該端點在回傳前依 `aup.review.identity_view` 裁剪：沒有該權限者拿到的是
//! `reviewer_name = "審查者"`、`reviewer_email = None`，PI 不在授予名單內。
//! 另一條 `list_review_assignments` 不裁剪，但靠 `:96-103` 的守衛把 PI 擋在門外。
//! **系統既有行為本來就是「委員身分只給審查方、不給申請方」**，本檔與它一致。

use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::middleware::CurrentUser;
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

/// 委員會審查關顯示的角色前綴（「審查委員：王大明、李小華」）。
const ROLE_REVIEWER: &str = "REVIEWER";

/// 看得到委員姓名的人。與檔頭表格同源，改這裡要一併改那張表。
fn can_see_reviewer_identities(viewer: &CurrentUser) -> bool {
    viewer.has_permission(PERMISSION_CHANGE_STATUS)
}

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
    /// 委員會審查關的指派委員姓名。查詢只在檢視者有權限時才把 `UNDER_REVIEW`
    /// 納入狀態過濾，所以無權者這裡恆為空——**姓名根本不會離開資料庫**。
    reviewer_names: Vec<String>,
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
    viewer: &CurrentUser,
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
    let revision_statuses: Vec<String> = PROTOCOL_REVISION_STATUSES
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    // 無權檢視委員身分者，`UNDER_REVIEW` 根本不進狀態過濾——那幾筆不會有結果列，
    // 前端收到 `null` 就不包 tooltip。**這比「查出來再丟掉」好**：姓名不離開資料庫，
    // 也不會有人日後在中間加一個 log 或 debug 輸出而把它漏出去。
    let mut assignee_statuses = vec!["VET_REVIEW".to_string()];
    if can_see_reviewer_identities(viewer) {
        assignee_statuses.push("UNDER_REVIEW".to_string());
    }

    let rows = sqlx::query_as::<_, ProtocolAssigneeRow>(
        r#"
        SELECT p.id,
               p.status::text AS status,
               CASE
                   WHEN p.status::text = 'VET_REVIEW' THEN vu.display_name
                   WHEN p.status::text = ANY($2) THEN pi.display_name
                   ELSE NULL
               END AS assignee_name,
               (SELECT COALESCE(array_agg(ru.display_name ORDER BY ru.display_name), '{}')
                  FROM review_assignments ra
                  JOIN users ru ON ru.id = ra.reviewer_id
                 WHERE ra.protocol_id = p.id AND ra.completed_at IS NULL) AS reviewer_names,
               -- CodeRabbit #31：同 `resolve_amendment_assignees` 那條——`since` 要算
               -- 「進入目前這一關的時間」。一份計畫可能在獸醫審查 / 委員會審查 / 補件
               -- 之間繞好幾圈，`submitted_at` 是最初送審日，從頭到尾不變，拿它當
               -- VET_REVIEW / UNDER_REVIEW / 需修正三關共用的起算點，等於每繞一圈就
               -- 多算一圈的等待時間。改成「最後一次轉進目前 status 的時間」
               -- （`record_status_change_tx` 是 `protocols.status` 唯一寫入路徑，
               -- 每次轉移都會在 `protocol_activities` 留一筆 `to_value`）。
               COALESCE(
                 (SELECT pa.created_at
                    FROM protocol_activities pa
                   WHERE pa.protocol_id = p.id AND pa.to_value = p.status::text
                   ORDER BY pa.created_at DESC
                   LIMIT 1),
                 p.submitted_at,
                 p.updated_at
               ) AS since
        FROM protocols p
        LEFT JOIN vet_review_assignments vra ON vra.protocol_id = p.id
        LEFT JOIN users vu ON vu.id = vra.vet_id
        LEFT JOIN users pi ON pi.id = p.pi_user_id
        -- 同上：軟刪除靠 status='DELETED'，狀態白名單已排除。
        WHERE p.id = ANY($1)
          AND (p.status::text = ANY($3) OR p.status::text = ANY($2))
        "#,
    )
    .bind(protocol_ids)
    .bind(&revision_statuses)
    .bind(&assignee_statuses)
    .fetch_all(pool)
    .await?;

    let mut result = HashMap::with_capacity(rows.len());
    for row in rows {
        let owner = match row.status.as_str() {
            // 委員會審查：列出委員姓名。能走到這裡代表檢視者有權限——
            // 無權者的 `UNDER_REVIEW` 在上面的狀態過濾就被排除了。
            "UNDER_REVIEW" => PendingOwner::from_candidates(
                STAGE_PROTOCOL_UNDER_REVIEW,
                PendingOwnerKind::Role,
                Some(ROLE_REVIEWER.to_string()),
                row.reviewer_names,
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
    viewer: &CurrentUser,
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
    if protocol_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut result = resolve_protocol_intake(pool, protocol_ids).await?;
    result.extend(resolve_protocol_assignees(pool, protocol_ids, viewer).await?);
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
    viewer: &CurrentUser,
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

    result.extend(resolve_amendment_assignees(pool, amendment_ids, viewer).await?);

    Ok(result)
}

/// 卡在委員會 / 申請人身上的變更申請列。
#[derive(Debug, sqlx::FromRow)]
struct AmendmentAssigneeRow {
    id: Uuid,
    status: String,
    /// 申請人顯示名（`REVISION_REQUIRED` 用）。
    applicant_name: Option<String>,
    /// 委員會審查關的指派委員姓名。查詢只在檢視者有權限時才把 `UNDER_REVIEW`
    /// 納入狀態過濾，所以無權者這裡恆為空——**姓名根本不會離開資料庫**。
    reviewer_names: Vec<String>,
    since: Option<DateTime<Utc>>,
}

/// 變更申請：委員會審查（僅 IACUC 行政方可見）與需修正（卡在申請人）。
async fn resolve_amendment_assignees(
    pool: &PgPool,
    amendment_ids: &[Uuid],
    viewer: &CurrentUser,
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
    // 與計畫端同一條規則：無權者的 `UNDER_REVIEW` 不進狀態過濾。
    let mut statuses = vec!["REVISION_REQUIRED".to_string()];
    if can_see_reviewer_identities(viewer) {
        statuses.push("UNDER_REVIEW".to_string());
    }

    let rows = sqlx::query_as::<_, AmendmentAssigneeRow>(
        r#"
        SELECT a.id,
               a.status::text AS status,
               COALESCE(su.display_name, cu.display_name) AS applicant_name,
               (SELECT COALESCE(array_agg(ru.display_name ORDER BY ru.display_name), '{}')
                  FROM amendment_review_assignments ara
                  JOIN users ru ON ru.id = ara.reviewer_id
                 WHERE ara.amendment_id = a.id AND ara.decided_at IS NULL) AS reviewer_names,
               -- CodeRabbit #31：`since` 要算「進入目前這一關的時間」，不是原始送審時間。
               -- 一份變更申請可能被退回補件、重送、再退回好幾輪，`submitted_at` 從頭到尾
               -- 不變——用它當 UNDER_REVIEW / REVISION_REQUIRED 的起算點，等於把「已經
               -- 卡在這一關多久」算成「從最初送出到現在多久」，案子繞了幾圈就多算幾圈。
               -- 改成「最後一次轉進目前 status 的時間」（同一 status 沒有查無記錄的問題，
               -- 因為 `record_status_change` 是唯一寫入路徑，每次轉移都會留一筆）。
               COALESCE(
                 (SELECT ash.created_at
                    FROM amendment_status_history ash
                   WHERE ash.amendment_id = a.id AND ash.to_status = a.status
                   ORDER BY ash.created_at DESC
                   LIMIT 1),
                 a.submitted_at,
                 a.updated_at
               ) AS since
        FROM amendments a
        LEFT JOIN users su ON su.id = a.submitted_by
        LEFT JOIN users cu ON cu.id = a.created_by
        WHERE a.id = ANY($1)
          AND a.status::text = ANY($2)
        "#,
    )
    .bind(amendment_ids)
    .bind(&statuses)
    .fetch_all(pool)
    .await?;

    let mut result = HashMap::with_capacity(rows.len());
    for row in rows {
        let owner = if row.status == "UNDER_REVIEW" {
            // 能走到這裡代表檢視者有權限（無權者上面已被狀態過濾排除）。
            PendingOwner::from_candidates(
                STAGE_AMENDMENT_UNDER_REVIEW,
                PendingOwnerKind::Role,
                Some(ROLE_REVIEWER.to_string()),
                row.reviewer_names,
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
