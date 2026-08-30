//! 「這一關卡在誰手上」的統一回傳型別。
//!
//! 全站有 20 個以上的「待 XX」在途狀態（採購核准、加班審核、設備報廢、AUP 審查…），
//! 各自的審核授權規則散在對應的 handler / service 裡。本型別是它們共同的 API 契約：
//! 前端只認得這一個形狀，就能對任何模組畫出同一顆徽章與 hover 內容。
//!
//! ⚠️ **候選人必須由該關卡真正的授權判準推導**，不可另寫一份近似規則——
//! 否則 tooltip 說「卡在王倉管」而他點下去拿 403。既有的
//! `HrService::current_stage_approvers`（請假）就是照這個原則寫的：與
//! `can_user_approve_leave` 共用同一組查詢。
//!
//! ⚠️ **`stage` 與 `role_code` 回的是 i18n key 不是顯示文字**。後端塞中文字串會讓
//! en 介面拿到中文（CI 的 E2E 正是渲染 en）；對齊既有做法
//! （`frontend/src/pages/admin/types.ts` 的 `DISPOSAL_STATUS_LABELS` 存的也是 key）。

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

/// 一次最多列出幾個人名；其餘計入 `overflow`。
///
/// 3 是取捨後的值：倉管實際有數位、admin 可能更多，全列會讓 tooltip 高過表格列；
/// 只給角色又回答不了「卡在誰」這個問題本身。
pub const MAX_LISTED_CANDIDATES: usize = 3;

/// 這一關的「負責對象」是怎麼決定的。前端依此決定文案形狀。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PendingOwnerKind {
    /// 綁角色 / 權限：任一符合者皆可處理 → 顯示「角色 + 人員」。
    Role,
    /// 綁特定人員：只有這些人能處理（指派審查委員、職務代理人、追蹤者）→ 只顯示人員。
    Person,
    /// 球在申請人自己身上（需修正 / 補件）→ 顯示「待申請人補件：某某」。
    Applicant,
}

// ⚠️ 這裡原本有第四個變體 `Anonymous`（不列名、只給人數），給 IACUC 委員會審查用。
//
// 2026-08-27 移除：使用者裁定收斂成「IACUC 行政方看得到委員姓名、其餘所有人
// **完全沒有 tooltip**」，沒有「知道有幾位但不知道是誰」這個中間狀態，
// 所以那個變體不再有任何產生者（`services/pending_owner/aup.rs` 改為在查詢層
// 就把無權者的 `UNDER_REVIEW` 排除）。
//
// 留著它不會被編譯器抓到（`pub` enum 的未用變體不算 dead code），
// 前端也會繼續保留一條永遠走不到的分支——那正是本次一連串修正在處理的形狀：
// **看起來還活著的死路**。

/// 某筆待處理單據 / 申請「現在卡在誰」。
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PendingOwner {
    /// 關卡的 i18n key 尾段（前端查 `pendingOwner.stage.<stage>`），
    /// 例 `doc_wm_approve`。全站唯一，故一律帶模組前綴。
    pub stage: String,
    pub kind: PendingOwnerKind,
    /// 角色代碼（`kind = Role` 才有意義），前端查
    /// `pendingOwner.role.<role_code>`，例 `WAREHOUSE_MANAGER`。
    pub role_code: Option<String>,
    /// 已列出的人名，至多 [`MAX_LISTED_CANDIDATES`] 個。
    pub candidates: Vec<String>,
    /// 未列出的人數。總人數 = `candidates.len() + overflow`。
    pub overflow: i64,
    /// 進入本關的時間，前端用來算「已等待 N 天」。取不到時為 None（前端不顯示天數）。
    pub since: Option<DateTime<Utc>>,
}

impl PendingOwner {
    /// 由完整候選名單建構（自動排序、截斷並計算 `overflow`）。
    ///
    /// 呼叫端負責先把 SoD 排除者（例如單據建立者不得自核）剔掉再傳進來——
    /// 這裡看不到業務規則，不該猜。
    pub fn from_candidates(
        stage: impl Into<String>,
        kind: PendingOwnerKind,
        role_code: Option<String>,
        mut names: Vec<String>,
        since: Option<DateTime<Utc>>,
    ) -> Self {
        names.sort();
        let overflow = names.len().saturating_sub(MAX_LISTED_CANDIDATES) as i64;
        names.truncate(MAX_LISTED_CANDIDATES);
        Self {
            stage: stage.into(),
            kind,
            role_code,
            candidates: names,
            overflow,
            since,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_to_three_and_counts_overflow() {
        let owner = PendingOwner::from_candidates(
            "doc_wm_approve",
            PendingOwnerKind::Role,
            Some("WAREHOUSE_MANAGER".into()),
            vec![
                "王大明".into(),
                "李小華".into(),
                "陳美玲".into(),
                "張三".into(),
                "李四".into(),
            ],
            None,
        );
        assert_eq!(owner.candidates.len(), 3);
        assert_eq!(owner.overflow, 2, "5 人列 3 個，其餘 2 人計入 overflow");
    }

    #[test]
    fn no_overflow_when_within_limit() {
        let owner = PendingOwner::from_candidates(
            "ot_admin_staff",
            PendingOwnerKind::Role,
            None,
            vec!["王大明".into()],
            None,
        );
        assert_eq!(owner.overflow, 0);
        assert_eq!(owner.candidates, vec!["王大明".to_string()]);
    }

    #[test]
    fn sorts_before_truncating_so_output_is_stable() {
        let names = vec!["丙".to_string(), "甲".to_string(), "乙".to_string()];
        let a =
            PendingOwner::from_candidates("s", PendingOwnerKind::Role, None, names.clone(), None);
        let mut reversed = names;
        reversed.reverse();
        let b = PendingOwner::from_candidates("s", PendingOwnerKind::Role, None, reversed, None);
        assert_eq!(
            a.candidates, b.candidates,
            "同一組人不論查詢順序，列出的人名必須一致"
        );
    }
}
