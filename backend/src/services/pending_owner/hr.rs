//! HR 加班的待處理人解析。
//!
//! 加班是**純角色型**：`services/hr/overtime.rs:247-249` 的 `can_approve` 直接比對
//! `is_admin()` 與 `ADMIN_STAFF` 角色，沒有對應的權限碼。
//!
//! ⚠️ 只處理 `pending_admin_staff` / `pending_admin` 兩關。legacy 的 `pending`
//! （`chk_overtime_status` 仍允許）在 `can_approve` 的 match 裡落到 `_ => false`
//! ——**沒有任何人能審它**，給它一份候選名單等於憑空造出不存在的授權。
//!
//! ⚠️ 終審關另有一條「批過前關者不得再批」的職務分離，且**只在還有其他人可簽時
//! 才收緊**（`overtime.rs` 的 `approve_overtime`）。那是逐筆判定，角色清單表達不了，
//! 所以終審關的候選名單走 `HrService::final_stage_eligible_approvers`——
//! 守衛與名單同一個查詢。我原先只排除申請人，會把「已批過前關、按下去必被擋」的人
//! 列進去（CodeRabbit 於 #30 指出）。

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::constants::{ROLE_ADMIN_LEGACY, ROLE_ADMIN_STAFF, ROLE_SYSTEM_ADMIN};
use crate::error::AppError;
use crate::models::PendingOwner;

use super::{
    resolve_single_stage, resolve_single_stage_with_eligible, CandidateSource, PendingRow,
};

/// 第一關：行政人員或管理員（`overtime.rs:248`）。
const STAGE_ADMIN_STAFF: &str = "ot_admin_staff";
/// 第二關：僅管理員（`overtime.rs:249`）。
const STAGE_ADMIN: &str = "ot_admin";

/// `is_admin()` 認的兩個角色代碼（`middleware/auth.rs:70-74`）。
const ADMIN_ROLES: &[&str] = &[ROLE_SYSTEM_ADMIN, ROLE_ADMIN_LEGACY];
/// 第一關另外放行行政人員。
const ADMIN_STAFF_ROLES: &[&str] = &[ROLE_SYSTEM_ADMIN, ROLE_ADMIN_LEGACY, ROLE_ADMIN_STAFF];

#[derive(Debug, sqlx::FromRow)]
struct OvertimeStageRow {
    id: Uuid,
    status: String,
    user_id: Uuid,
    submitted_at: Option<DateTime<Utc>>,
    /// 第一關核准的時間戳（`overtime.rs` 的 UPDATE 同時寫 status 與 `approved_at`）。
    /// 終審關的「已等待 N 天」要從這裡起算，不是從送出起算。
    approved_at: Option<DateTime<Utc>>,
}

/// 一次算出多筆加班申請的待處理人。非待審中的 id 不會出現在結果裡。
pub async fn resolve_for_overtime(
    pool: &PgPool,
    overtime_ids: &[Uuid],
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
    if overtime_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as::<_, OvertimeStageRow>(
        r#"
        SELECT id, status, user_id, submitted_at, approved_at
        FROM overtime_records
        WHERE id = ANY($1)
          AND status IN ('pending_admin_staff', 'pending_admin')
        "#,
    )
    .bind(overtime_ids)
    .fetch_all(pool)
    .await?;

    // SoD：申請人不得審自己的加班（`overtime.rs:241-245`，服務端強制執行）。
    let first_stage: Vec<PendingRow> = rows
        .iter()
        .filter(|r| r.status == "pending_admin_staff")
        .map(|r| PendingRow {
            id: r.id,
            excluded: Some(r.user_id),
            since: r.submitted_at,
        })
        .collect();

    // 終審關的起點是**第一關核准的那一刻**，不是送出的那一刻。
    // 用 submitted_at 會把第一關的等待時間也算進終審關的「已等待 N 天」
    // （CodeRabbit 於 #30 指出）。`approved_at` 由第一關核准的 UPDATE 與 status
    // 同一句寫入，所以 status 是 pending_admin 時它必定已填。
    let final_stage: Vec<PendingRow> = rows
        .iter()
        .filter(|r| r.status == "pending_admin")
        .map(|r| PendingRow {
            id: r.id,
            excluded: Some(r.user_id),
            since: r.approved_at.or(r.submitted_at),
        })
        .collect();

    let mut result = resolve_single_stage(
        pool,
        STAGE_ADMIN_STAFF,
        Some(ROLE_ADMIN_STAFF),
        CandidateSource::AnyRole(ADMIN_STAFF_ROLES),
        &first_stage,
    )
    .await?;

    // 終審關的 SoD 比「申請人不得自審」更嚴：批過前關的人不得再批終審，
    // **但只在還有其他人可簽時才收緊**（否則單一審批人組織會卡死）。
    // 這條逐筆才算得出來，候選來源表達不了，所以走權威來源
    // `HrService::final_stage_eligible_approvers`——`approve_overtime` 的守衛
    // 建在同一個查詢上，兩邊不會分岔。
    let final_ids: Vec<Uuid> = final_stage.iter().map(|r| r.id).collect();
    let eligible = if final_ids.is_empty() {
        HashMap::new()
    } else {
        crate::services::HrService::final_stage_eligible_approvers(pool, &final_ids).await?
    };

    result.extend(
        resolve_single_stage_with_eligible(
            pool,
            STAGE_ADMIN,
            Some(ROLE_ADMIN_LEGACY),
            CandidateSource::AnyRole(ADMIN_ROLES),
            &final_stage,
            &eligible,
        )
        .await?,
    );

    Ok(result)
}
