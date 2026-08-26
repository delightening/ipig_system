//! HR 加班的待處理人解析。
//!
//! 加班是**純角色型**：`services/hr/overtime.rs:247-249` 的 `can_approve` 直接比對
//! `is_admin()` 與 `ADMIN_STAFF` 角色，沒有對應的權限碼。
//!
//! ⚠️ 只處理 `pending_admin_staff` / `pending_admin` 兩關。legacy 的 `pending`
//! （`chk_overtime_status` 仍允許）在 `can_approve` 的 match 裡落到 `_ => false`
//! ——**沒有任何人能審它**，給它一份候選名單等於憑空造出不存在的授權。

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::constants::{ROLE_ADMIN_LEGACY, ROLE_ADMIN_STAFF, ROLE_SYSTEM_ADMIN};
use crate::error::AppError;
use crate::models::PendingOwner;

use super::{resolve_single_stage, CandidateSource, PendingRow};

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
        SELECT id, status, user_id, submitted_at
        FROM overtime_records
        WHERE id = ANY($1)
          AND status IN ('pending_admin_staff', 'pending_admin')
        "#,
    )
    .bind(overtime_ids)
    .fetch_all(pool)
    .await?;

    // SoD：申請人不得審自己的加班（`overtime.rs:241-245`，服務端強制執行）。
    let to_pending_row = |r: &OvertimeStageRow| PendingRow {
        id: r.id,
        excluded: Some(r.user_id),
        since: r.submitted_at,
    };

    let first_stage: Vec<PendingRow> = rows
        .iter()
        .filter(|r| r.status == "pending_admin_staff")
        .map(to_pending_row)
        .collect();
    let final_stage: Vec<PendingRow> = rows
        .iter()
        .filter(|r| r.status == "pending_admin")
        .map(to_pending_row)
        .collect();

    let mut result = resolve_single_stage(
        pool,
        STAGE_ADMIN_STAFF,
        Some(ROLE_ADMIN_STAFF),
        CandidateSource::AnyRole(ADMIN_STAFF_ROLES),
        &first_stage,
    )
    .await?;

    result.extend(
        resolve_single_stage(
            pool,
            STAGE_ADMIN,
            Some(ROLE_ADMIN_LEGACY),
            CandidateSource::AnyRole(ADMIN_ROLES),
            &final_stage,
        )
        .await?,
    );

    Ok(result)
}
