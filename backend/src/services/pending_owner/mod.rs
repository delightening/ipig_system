//! 待處理人解析：把「這筆現在卡在誰手上」算出來給前端顯示。
//!
//! # 為什麼要有這一層
//!
//! 「待核准」徽章本身回答不了使用者真正的問題——要去催誰。這一層的職責是把各模組
//! **已經寫在授權碼裡**的判準抽出來，轉成前端可統一呈現的 [`PendingOwner`]。
//!
//! # 唯一的硬規則
//!
//! **候選人名單必須與該關卡的授權判準同源。** 兩邊分岔的後果是使用者看到
//! 「卡在王倉管」，王倉管點下去拿 403——比不顯示更糟。所以每個關卡的權限碼 /
//! 角色碼 / SoD 排除規則都在註解裡標了它抄自哪一行，改動授權時看得到要一併改這裡。
//!
//! # 結構
//!
//! - [`document`]：ERP 單據三關（多關卡、判定較複雜，自成一支）
//! - [`hr`] / [`equipment`] / [`aup`]：其餘關卡，都是「單一關卡 + 單一候選來源 +
//!   至多一位 SoD 排除對象」的形狀，共用 [`resolve_single_stage`]
//!
//! # 刻意不涵蓋
//!
//! - **動物欄位更正**：`animal.field_correction.review` 2026-08-26 實查**沒有授予
//!   任何角色**，候選人恆為「靠短路取得權限的管理員」；而該頁路由本身就限
//!   `role="admin"`（`App.tsx:526`）。看的人與能處理的人是同一群，零資訊量。
//! - **PI 開通信**：同理，`handlers/protocol/pi_provision.rs:80` 限管理員檢視。
//!
//! # 效能
//!
//! 解析器一律接受**一整頁的 id**，每個關卡的候選名單只查一次，不做逐列 roundtrip。

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{PendingOwner, PendingOwnerKind};
use crate::repositories::pending_owner as repo;

pub mod aup;
pub mod document;
pub mod equipment;
pub mod hr;

pub use aup::{resolve_for_amendments, resolve_for_protocols};
pub use document::resolve_for_documents;
pub use equipment::{resolve_for_disposals, resolve_for_idle_requests, resolve_for_maintenance};
pub use hr::resolve_for_overtime;

/// 一個關卡的合法處理人是怎麼查出來的。
///
/// 每個變體都對應到 handler / service 守衛的一種寫法。**選錯變體＝候選名單與守衛分岔**，
/// 所以新增關卡時請照抄守衛那幾行，不要靠印象。
#[derive(Debug, Clone, Copy)]
pub(crate) enum CandidateSource {
    /// `require_permission!(x)` 或 `has_permission(x)` 單一條件。
    Permission(&'static str),
    /// `has_permission(a) || has_permission(b)`——任一即可。
    AnyPermission(&'static [&'static str]),
    /// 權限**且**角色，兩個都要（例：單據倉管核准）。
    PermissionAndAnyRole(&'static str, &'static [&'static str]),
    /// 純角色比對，沒有對應的權限碼（例：加班兩關看 `is_admin()` / `ADMIN_STAFF`）。
    AnyRole(&'static [&'static str]),
}

impl CandidateSource {
    pub(crate) async fn resolve(self, pool: &PgPool) -> Result<Vec<repo::UserRef>, AppError> {
        let owned = |codes: &'static [&'static str]| -> Vec<String> {
            codes.iter().map(|c| (*c).to_string()).collect()
        };
        match self {
            Self::Permission(code) => repo::list_users_with_permission(pool, code).await,
            Self::AnyPermission(codes) => repo::list_users_with_any_permission(pool, codes).await,
            Self::PermissionAndAnyRole(code, roles) => {
                repo::list_users_with_permission_and_any_role(pool, code, &owned(roles)).await
            }
            Self::AnyRole(roles) => repo::list_users_with_any_role(pool, &owned(roles)).await,
        }
    }
}

/// 單關卡解析的每列輸入。
#[derive(Debug, Clone, Copy)]
pub(crate) struct PendingRow {
    pub id: Uuid,
    /// 職務分離要排除的人（申請人 / 登錄者本人）；該關卡無 SoD 時為 None。
    pub excluded: Option<Uuid>,
    /// 進入本關的時間，前端用來算已等待幾天。
    pub since: Option<DateTime<Utc>>,
}

/// 「單一關卡 + 單一候選來源 + 至多一位 SoD 排除對象」的共用解析。
///
/// 候選名單只查一次，逐列只做排除與截斷。
///
/// 候選名單被 SoD 排空時（例：唯一有權者就是申請人）**仍回一筆 `candidates` 為空的結果**。
/// 這是有意義的資訊：代表這筆目前沒有任何人能處理。靜默略過會讓卡死的申請看起來
/// 跟正常待審的一樣。
pub(crate) async fn resolve_single_stage(
    pool: &PgPool,
    stage: &'static str,
    role_code: Option<&'static str>,
    source: CandidateSource,
    rows: &[PendingRow],
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
    resolve_single_stage_with_eligible(pool, stage, role_code, source, rows, &HashMap::new()).await
}

/// 同 [`resolve_single_stage`]，但額外接受**該關卡權威判準算出來的合法處理人**。
///
/// 用於守衛比「候選來源」更精確的關卡——典型是逐筆才知道的職務分離
/// （例：加班終審不得由批過前關的人再批）。候選來源只能表達「哪些人有這個角色 /
/// 權限」，表達不了「這一筆，這個人已經簽過了」。
///
/// `eligible` 語意：
/// - 該 id 對應**非空**清單 → 只列出清單內的人
/// - 該 id 對應**空**清單 → 權威判準說「沒有其他人可簽」，退回候選來源全體。
///   這對應「卡關代批」：SoD 只在真的還有別人時才收緊，否則單一審批人組織會卡死
///   （`services/hr/overtime.rs` 終審關、`services/hr/leave.rs` 負責人關都是這個規則）。
///   **退回而不是留空**——留空會讓「能簽但要動用代批」看起來跟「沒人能簽」一樣。
/// - 該 id **不在 map 裡** → 不套用逐筆判準，行為同 [`resolve_single_stage`]
///
/// ⚠️ 後兩者的結果相同，但**理由不同,不可合併**：空清單是「判準跑過了,答案是沒人」，
/// 缺席是「這一關沒有逐筆判準」。權威來源必須用空清單表達前者
/// （`final_stage_eligible_approvers` 的 `LEFT JOIN LATERAL` 就是為此），
/// 否則「無人可簽」與「查無此單」在這裡會走到同一條路,而它們該做的事不一樣。
pub(crate) async fn resolve_single_stage_with_eligible(
    pool: &PgPool,
    stage: &'static str,
    role_code: Option<&'static str>,
    source: CandidateSource,
    rows: &[PendingRow],
    eligible: &HashMap<Uuid, Vec<Uuid>>,
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
    if rows.is_empty() {
        return Ok(HashMap::new());
    }

    let candidates = source.resolve(pool).await?;
    let mut result = HashMap::with_capacity(rows.len());

    for row in rows {
        let after_sod: Vec<&repo::UserRef> = candidates
            .iter()
            .filter(|(id, _)| Some(*id) != row.excluded)
            .collect();

        let names: Vec<String> = match eligible.get(&row.id) {
            Some(ids) if !ids.is_empty() => after_sod
                .iter()
                .filter(|(id, _)| ids.contains(id))
                .map(|(_, name)| name.clone())
                .collect(),
            _ => after_sod.iter().map(|(_, name)| name.clone()).collect(),
        };

        result.insert(
            row.id,
            PendingOwner::from_candidates(
                stage,
                PendingOwnerKind::Role,
                role_code.map(str::to_string),
                names,
                row.since,
            ),
        );
    }

    Ok(result)
}
