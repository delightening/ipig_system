//! 站內信 access matrix — 誰能寫給誰
//!
//! 16 個 role code 內聚為 4 + 1 messaging category，動態判定 pair 是否允許。
//! 若使用者有多個 role（極少見），取最高 category：admin > vet > pi > staffs > external

use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{constants as c, AppError, Result};

/// 站內信收件人候選（依 access matrix 對 sender 可寄送的對象）
#[derive(Debug, Serialize)]
pub struct RecipientSummary {
    pub id: Uuid,
    pub display_name: String,
    pub email: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagingCategory {
    Admin,
    Pi,
    Vet,
    Staffs,
    External,
}

impl MessagingCategory {
    pub(super) fn from_role_code(code: &str) -> Self {
        match code {
            // SYSTEM_ADMIN 是新版命名，"admin"（lowercase）是 legacy；兩個都當 admin
            "SYSTEM_ADMIN" | "admin" => Self::Admin,
            // 原本 STUDY_DIRECTOR 也歸在此類，已隨全域 SD 角色移除——
            // SD 是逐計畫指派（protocols.study_director_user_id）而非角色，
            // 擔任 SD 的人本身是 EXPERIMENT_STAFF，走下面 Staffs 那條。
            c::ROLE_PI => Self::Pi,
            c::ROLE_VET => Self::Vet,
            c::ROLE_ADMIN_STAFF
            | c::ROLE_EXPERIMENT_STAFF
            | "INTERN"
            | "PURCHASING"
            | c::ROLE_WAREHOUSE_MANAGER
            | "EQUIPMENT_MAINTENANCE"
            | c::ROLE_IACUC_STAFF
            | c::ROLE_IACUC_CHAIR
            | c::ROLE_QAU
            | c::ROLE_REVIEWER => Self::Staffs,
            // CLIENT / GUEST / 未知 → external
            _ => Self::External,
        }
    }

    /// 多 role 取最高（對 messaging 最寬權限）
    pub(super) fn rank(self) -> u8 {
        match self {
            Self::Admin => 4,
            Self::Vet => 3,
            Self::Pi => 2,
            Self::Staffs => 1,
            Self::External => 0,
        }
    }
}

/// 查使用者所有 role → 內聚為單一 messaging category（取最高）
pub async fn user_messaging_category(pool: &PgPool, user_id: Uuid) -> Result<MessagingCategory> {
    let codes: Vec<String> = sqlx::query_scalar(
        r#"SELECT r.code FROM user_roles ur
           JOIN roles r ON r.id = ur.role_id
           WHERE ur.user_id = $1"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    if codes.is_empty() {
        return Ok(MessagingCategory::External);
    }

    Ok(codes
        .iter()
        .map(|c| MessagingCategory::from_role_code(c))
        .max_by_key(|cat| cat.rank())
        .unwrap_or(MessagingCategory::External))
}

/// Access matrix（雙向）：
/// - admin ↔ ANY（含 admin↔admin）
/// - vet ↔ vet、vet ↔ staffs
/// - pi ↔ staffs
/// - staffs ↔ staffs
/// - external 完全禁止
pub fn messaging_pair_allowed(a: MessagingCategory, b: MessagingCategory) -> bool {
    use MessagingCategory::*;
    match (a, b) {
        (External, _) | (_, External) => false,
        (Admin, _) | (_, Admin) => true,
        (Vet, Vet) | (Vet, Staffs) | (Staffs, Vet) => true,
        (Pi, Staffs) | (Staffs, Pi) => true,
        (Staffs, Staffs) => true,
        // (Vet, Pi) | (Pi, Vet) | (Pi, Pi) → false
        _ => false,
    }
}

/// 將 `(user_id, role_code)` 多列內聚為 user → 最高 rank category。
/// 抽出共用：`ensure_can_message_all` 與 `ensure_all_pairs_allowed` 皆用（DRY，避免 rank/
/// role mapping 日後在多處走樣）。
fn collapse_roles_to_categories(
    rows: Vec<(Uuid, String)>,
) -> std::collections::HashMap<Uuid, MessagingCategory> {
    let mut cats: std::collections::HashMap<Uuid, MessagingCategory> =
        std::collections::HashMap::new();
    for (uid, code) in rows {
        let cat = MessagingCategory::from_role_code(&code);
        cats.entry(uid)
            .and_modify(|existing| {
                if cat.rank() > existing.rank() {
                    *existing = cat;
                }
            })
            .or_insert(cat);
    }
    cats
}

/// 一次 SELECT 撈一組 user 的 role codes → user→category map。
async fn fetch_user_categories(
    pool: &PgPool,
    user_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, MessagingCategory>> {
    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        r#"SELECT ur.user_id, r.code
           FROM user_roles ur
           JOIN roles r ON r.id = ur.role_id
           WHERE ur.user_id = ANY($1)"#,
    )
    .bind(user_ids)
    .fetch_all(pool)
    .await?;
    Ok(collapse_roles_to_categories(rows))
}

/// 高層 helper：actor + recipients 全部驗 pair allowed；任一禁止則錯
///
/// 一次 SELECT 撈所有 user_id → role codes，避免 N+1 query（每個 recipient 一次）。
pub async fn ensure_can_message_all(
    pool: &PgPool,
    sender_id: Uuid,
    recipient_ids: &[Uuid],
) -> Result<()> {
    let sender_cat = user_messaging_category(pool, sender_id).await?;
    if sender_cat == MessagingCategory::External {
        return Err(AppError::Forbidden("您的角色無法使用站內信功能".into()));
    }
    if recipient_ids.is_empty() {
        return Ok(());
    }

    let cats = fetch_user_categories(pool, recipient_ids).await?;

    for rid in recipient_ids {
        let rcat = cats
            .get(rid)
            .copied()
            .unwrap_or(MessagingCategory::External);
        if !messaging_pair_allowed(sender_cat, rcat) {
            return Err(AppError::Forbidden(format!(
                "依角色設定，您無法寄訊息給該收件人（user_id={rid}）"
            )));
        }
    }
    Ok(())
}

/// #365 純函式：給定一組 participant 的 category，驗「所有兩兩配對」皆 allowed。
/// group thread 須通過此檢查，否則禁止配對（如 vet↔PI）可由第三方建群同桌繞過 matrix。
pub(super) fn categories_all_pairs_allowed(cats: &[MessagingCategory]) -> bool {
    for (i, a) in cats.iter().enumerate() {
        for b in &cats[i + 1..] {
            if !messaging_pair_allowed(*a, *b) {
                return false;
            }
        }
    }
    true
}

/// #365：群組 thread 須驗「所有 participant 兩兩配對」皆 allowed（含 recipient↔recipient）。
/// `ensure_can_message_all` 只驗 sender→recipient，無法涵蓋 recipient 之間 —— 否則一個
/// Staffs 可建群把禁止配對的 vet 與 PI 拉進同 thread，之後 send/get 只查 is_participant、
/// 不重跑 matrix，等同繞過「vet↔PI 禁止」這條刻意設計的隔離控制。
pub async fn ensure_all_pairs_allowed(pool: &PgPool, participant_ids: &[Uuid]) -> Result<()> {
    // 去重：重複 id（同一人送兩次）會在 all-pairs 產生假的 self-pair（如 Pi↔Pi）→ 誤拒（bot）。
    let mut unique_ids = participant_ids.to_vec();
    unique_ids.sort();
    unique_ids.dedup();
    if unique_ids.len() < 2 {
        return Ok(());
    }

    let cats = fetch_user_categories(pool, &unique_ids).await?;
    let participant_cats: Vec<MessagingCategory> = unique_ids
        .iter()
        .map(|id| cats.get(id).copied().unwrap_or(MessagingCategory::External))
        .collect();

    if !categories_all_pairs_allowed(&participant_cats) {
        return Err(AppError::Forbidden(
            "群組成員之間有不被允許的角色配對（例如獸醫與計畫主持人不可同群），無法建立此群組"
                .into(),
        ));
    }
    Ok(())
}

/// 列出 sender 依 access matrix 可寄送的所有對象。
///
/// 排除：自己、停用 / 已軟刪帳號、external（CLIENT / GUEST / 未知角色）。
/// 一次 SELECT 撈所有 active user + role codes，Rust 端內聚 category（取最高 rank）後過濾。
pub async fn list_allowed_recipients(
    pool: &PgPool,
    sender_id: Uuid,
) -> Result<Vec<RecipientSummary>> {
    let sender_cat = user_messaging_category(pool, sender_id).await?;
    if sender_cat == MessagingCategory::External {
        return Err(AppError::Forbidden("您的角色無法使用站內信功能".into()));
    }

    let rows: Vec<(Uuid, String, String, String)> = sqlx::query_as(
        r#"SELECT u.id, u.display_name, u.email, r.code
           FROM users u
           JOIN user_roles ur ON ur.user_id = u.id
           JOIN roles r ON r.id = ur.role_id
           WHERE u.is_active = true AND u.deleted_at IS NULL AND u.id <> $1"#,
    )
    .bind(sender_id)
    .fetch_all(pool)
    .await?;

    let mut out: Vec<RecipientSummary> = aggregate_recipient_categories(rows)
        .into_iter()
        .filter(|(_, a)| messaging_pair_allowed(sender_cat, a.cat))
        .map(|(id, a)| RecipientSummary {
            id,
            display_name: a.display_name,
            email: a.email,
        })
        .collect();
    out.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    Ok(out)
}

/// 候選人聚合中間態：同一 user 多 role 取最高 rank category（與 `ensure_can_message_all` 一致）
struct RecipientAcc {
    display_name: String,
    email: String,
    cat: MessagingCategory,
}

/// 將 `(user_id, display_name, email, role_code)` 多列內聚為 user → 最高 rank category。
fn aggregate_recipient_categories(
    rows: Vec<(Uuid, String, String, String)>,
) -> std::collections::HashMap<Uuid, RecipientAcc> {
    let mut map: std::collections::HashMap<Uuid, RecipientAcc> = std::collections::HashMap::new();
    for (id, display_name, email, code) in rows {
        let cat = MessagingCategory::from_role_code(&code);
        map.entry(id)
            .and_modify(|a| {
                if cat.rank() > a.cat.rank() {
                    a.cat = cat;
                }
            })
            .or_insert(RecipientAcc {
                display_name,
                email,
                cat,
            });
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_can_message_everyone() {
        for cat in [
            MessagingCategory::Admin,
            MessagingCategory::Pi,
            MessagingCategory::Vet,
            MessagingCategory::Staffs,
        ] {
            assert!(messaging_pair_allowed(MessagingCategory::Admin, cat));
            assert!(messaging_pair_allowed(cat, MessagingCategory::Admin));
        }
    }

    #[test]
    fn vet_pi_blocked() {
        assert!(!messaging_pair_allowed(
            MessagingCategory::Vet,
            MessagingCategory::Pi
        ));
        assert!(!messaging_pair_allowed(
            MessagingCategory::Pi,
            MessagingCategory::Vet
        ));
    }

    #[test]
    fn pi_pi_blocked() {
        assert!(!messaging_pair_allowed(
            MessagingCategory::Pi,
            MessagingCategory::Pi
        ));
    }

    #[test]
    fn external_blocked() {
        assert!(!messaging_pair_allowed(
            MessagingCategory::External,
            MessagingCategory::Admin
        ));
        assert!(!messaging_pair_allowed(
            MessagingCategory::Admin,
            MessagingCategory::External
        ));
    }

    #[test]
    fn staffs_can_msg_vet_pi_staffs_admin() {
        assert!(messaging_pair_allowed(
            MessagingCategory::Staffs,
            MessagingCategory::Vet
        ));
        assert!(messaging_pair_allowed(
            MessagingCategory::Staffs,
            MessagingCategory::Pi
        ));
        assert!(messaging_pair_allowed(
            MessagingCategory::Staffs,
            MessagingCategory::Staffs
        ));
        assert!(messaging_pair_allowed(
            MessagingCategory::Staffs,
            MessagingCategory::Admin
        ));
    }

    // #365 群組全配對檢查
    use MessagingCategory::{Admin, Pi, Staffs, Vet};

    #[test]
    fn group_with_vet_and_pi_via_staffs_bridge_blocked() {
        // Staffs 建群把 vet + PI 拉進來：sender→各別 allowed，但 vet↔PI 禁止 → 整組拒絕
        assert!(!categories_all_pairs_allowed(&[Staffs, Vet, Pi]));
    }

    #[test]
    fn group_two_pis_via_staffs_bridge_blocked() {
        assert!(!categories_all_pairs_allowed(&[Staffs, Pi, Pi]));
    }

    #[test]
    fn group_staffs_vet_admin_allowed() {
        assert!(categories_all_pairs_allowed(&[Staffs, Vet, Admin]));
    }

    #[test]
    fn group_all_staffs_allowed() {
        assert!(categories_all_pairs_allowed(&[Staffs, Staffs, Staffs]));
    }

    #[test]
    fn admin_in_group_does_not_whitelist_blocked_pair() {
        // admin 與所有人 allowed，但 admin 在場不會讓 vet↔PI 變 allowed
        assert!(!categories_all_pairs_allowed(&[Admin, Vet, Pi]));
    }

    #[test]
    fn single_or_empty_trivially_allowed() {
        assert!(categories_all_pairs_allowed(&[]));
        assert!(categories_all_pairs_allowed(&[Vet]));
    }
}
