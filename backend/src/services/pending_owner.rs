//! 待處理人解析：把「這筆單現在卡在誰手上」算出來給前端顯示。
//!
//! # 為什麼要有這一層
//!
//! 「待核准」徽章本身回答不了使用者真正的問題——要去催誰。這一層的職責是把各模組
//! **已經寫在授權碼裡**的判準抽出來，轉成前端可統一呈現的 [`PendingOwner`]。
//!
//! # 唯一的硬規則
//!
//! **候選人名單必須與該關卡的授權判準同源。** 兩邊分岔的後果是使用者看到
//! 「卡在王倉管」，王倉管點下去拿 403——比不顯示更糟。所以本檔每一個關卡的
//! 權限碼 / 角色碼 / SoD 排除規則都標了它抄自哪一行，改動授權時請一併改這裡。
//!
//! # 效能
//!
//! 解析器一律接受**一整頁的 id**，內部每個關卡的候選名單只查一次
//! （`OnceCell` 語意，見 `CandidateCache`），不做逐列 roundtrip。

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::constants::{ROLE_ADMIN_LEGACY, ROLE_SYSTEM_ADMIN, ROLE_WAREHOUSE_MANAGER};
use crate::error::AppError;
use crate::models::{DocStatus, DocType, PendingOwner, PendingOwnerKind};
use crate::repositories::pending_owner as repo;

/// 單據倉管核准關的 i18n stage key。
const STAGE_DOC_WM_APPROVE: &str = "doc_wm_approve";
/// 單據終審關（大額 ADJ 經倉管核准後）的 i18n stage key。
const STAGE_DOC_FINAL_APPROVE: &str = "doc_final_approve";
/// 沖銷單核准關的 i18n stage key。
const STAGE_DOC_REVERSE_APPROVE: &str = "doc_reverse_approve";

/// R97-1 SoD：這些單據類型的建立者不得自核。
///
/// 與 `services/document/workflow.rs` 的 `SELF_APPROVAL_FORBIDDEN_DOC_TYPES` 是同一份清單。
/// ⚠️ 那邊改了這裡要跟著改，否則候選名單會列出實際會被擋下的人。
const SELF_APPROVAL_FORBIDDEN_DOC_TYPES: &[DocType] = &[DocType::ADJ, DocType::GRN];

/// 解析單據待處理人所需的欄位。刻意獨立於 `Document`：只取這幾欄，
/// 列表頁不必為了 tooltip 把整張單撈回來。
#[derive(Debug, sqlx::FromRow)]
struct DocumentStageRow {
    id: Uuid,
    doc_type: DocType,
    status: DocStatus,
    created_by: Uuid,
    approved_by: Option<Uuid>,
    system_generated: bool,
    reverses_doc_id: Option<Uuid>,
    requires_manager_approval: Option<bool>,
    manager_approval_status: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    approved_at: Option<DateTime<Utc>>,
}

/// 單據可能停在的三個關卡。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DocumentStage {
    /// 一般送審 → 等倉庫管理員核准。
    Warehouse,
    /// 大額 ADJ 已由倉管核准 → 等負責人終審。
    Final,
    /// 沖銷單 → 等管理員核准反向鏡射。
    Reversal,
}

impl DocumentStage {
    fn key(self) -> &'static str {
        match self {
            Self::Warehouse => STAGE_DOC_WM_APPROVE,
            Self::Final => STAGE_DOC_FINAL_APPROVE,
            Self::Reversal => STAGE_DOC_REVERSE_APPROVE,
        }
    }

    /// 這一關對外顯示的角色代碼。終審關不綁單一角色（純看
    /// `erp.document.final_approve`），故為 None。
    fn role_code(self) -> Option<&'static str> {
        match self {
            Self::Warehouse => Some(ROLE_WAREHOUSE_MANAGER),
            Self::Final => None,
            Self::Reversal => Some(ROLE_SYSTEM_ADMIN),
        }
    }

    /// 這一關的合法處理人查詢。**權限碼 / 角色碼必須與 handler 的守衛完全一致**，
    /// 否則候選名單會列出點下去拿 403 的人。
    async fn candidates(self, pool: &PgPool) -> Result<Vec<repo::UserRef>, AppError> {
        match self {
            // `handlers/document.rs:210` 要 `erp.document.approve`，
            // :213 再要 `WAREHOUSE_MANAGER` 角色——兩個條件都要滿足。
            Self::Warehouse => {
                repo::list_users_with_permission_and_any_role(
                    pool,
                    "erp.document.approve",
                    &[ROLE_WAREHOUSE_MANAGER.to_string()],
                )
                .await
            }
            // `handlers/document.rs:267` 只要 `erp.document.final_approve`
            // （刻意不再要求倉管階段的 approve 權，否則負責人會被擋在閘外）。
            Self::Final => {
                repo::list_users_with_permission(pool, "erp.document.final_approve").await
            }
            // `handlers/document.rs:343` 要 `erp.document.reverse_approve`，
            // `services/document/reversal.rs:196` 再要 `is_admin()`。
            Self::Reversal => {
                repo::list_users_with_permission_and_any_role(
                    pool,
                    "erp.document.reverse_approve",
                    &[ROLE_SYSTEM_ADMIN.to_string(), ROLE_ADMIN_LEGACY.to_string()],
                )
                .await
            }
        }
    }
}

/// 判定單據停在哪一關。非 `submitted` 一律 None（草稿、已核准、已作廢都不在等人）。
///
/// ⚠️ **沖銷單必須先判**：`create_reversal` 建出的沖銷單同樣是
/// `requires_manager_approval = true` + `manager_approval_status = 'wm_approved'`
/// （`services/document/reversal.rs:93`），先判終審會把它歸錯關，
/// 顯示的候選人也會錯（終審看 `final_approve`，沖銷看 `reverse_approve` + admin）。
fn stage_of(row: &DocumentStageRow) -> Option<DocumentStage> {
    if row.status != DocStatus::Submitted {
        return None;
    }
    if row.reverses_doc_id.is_some() {
        return Some(DocumentStage::Reversal);
    }
    if row.requires_manager_approval == Some(true)
        && row.manager_approval_status.as_deref() == Some("wm_approved")
    {
        return Some(DocumentStage::Final);
    }
    Some(DocumentStage::Warehouse)
}

/// 進入本關的時間。
///
/// - 倉管核准關：`updated_at`（送審時最後一次寫入）。
/// - 終審關：`approved_at`（倉管核准的那一刻，即本關開始等待的時間）。
/// - 沖銷核准關：`created_at`（沖銷單一建立就在等核准）。
fn stage_since(row: &DocumentStageRow, stage: DocumentStage) -> Option<DateTime<Utc>> {
    match stage {
        DocumentStage::Warehouse => Some(row.updated_at),
        DocumentStage::Final => row.approved_at,
        DocumentStage::Reversal => Some(row.created_at),
    }
}

/// 本關要排除哪些人（職務分離）。回傳的是**該筆單獨有**的排除名單。
fn sod_excluded(row: &DocumentStageRow, stage: DocumentStage) -> HashSet<Uuid> {
    let mut excluded = HashSet::new();

    // 建立者不得自核（僅 ADJ / GRN，且系統自動產生的單豁免）。
    // 抄自 `services/document/workflow.rs::assert_document_sod`。
    let creator_barred =
        SELF_APPROVAL_FORBIDDEN_DOC_TYPES.contains(&row.doc_type) && !row.system_generated;

    match stage {
        DocumentStage::Warehouse => {
            if creator_barred {
                excluded.insert(row.created_by);
            }
        }
        DocumentStage::Final => {
            if creator_barred {
                excluded.insert(row.created_by);
            }
            // 第二道 SoD：倉管核准人不得再擔任終審人
            // （`services/document/workflow.rs:617`，兩階段核准必須是兩個人）。
            if let Some(wm_approver) = row.approved_by {
                excluded.insert(wm_approver);
            }
        }
        DocumentStage::Reversal => {
            // 沖銷單的發起人不得自核，且不分單據類型
            // （`services/document/reversal.rs:220`）。
            excluded.insert(row.created_by);
        }
    }

    excluded
}

/// 各關卡候選名單的單次查詢快取（一次解析內共用，不跨請求）。
#[derive(Default)]
struct CandidateCache {
    by_stage: HashMap<DocumentStage, Vec<repo::UserRef>>,
}

impl CandidateCache {
    async fn get(
        &mut self,
        pool: &PgPool,
        stage: DocumentStage,
    ) -> Result<&[repo::UserRef], AppError> {
        if let Entry::Vacant(slot) = self.by_stage.entry(stage) {
            slot.insert(stage.candidates(pool).await?);
        }
        Ok(self.by_stage.get(&stage).map(Vec::as_slice).unwrap_or(&[]))
    }
}

/// 一次算出多筆單據的待處理人。
///
/// 只回「真的還在等人」的那幾筆——非 `submitted` 的 id 不會出現在結果裡，
/// 呼叫端用 `HashMap::remove` / `get` 取即可。
///
/// 候選名單全被 SoD 排空時（例：唯一的倉管就是建單者），仍回一筆 `candidates` 為空、
/// `overflow = 0` 的結果。這是**有意義的資訊**：代表這張單目前沒有任何人能核准，
/// 靜默略過會讓卡死的單看起來跟正常待審的單一樣。
pub async fn resolve_for_documents(
    pool: &PgPool,
    document_ids: &[Uuid],
) -> Result<HashMap<Uuid, PendingOwner>, AppError> {
    if document_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as::<_, DocumentStageRow>(
        r#"
        SELECT id, doc_type, status, created_by, approved_by, system_generated,
               reverses_doc_id, requires_manager_approval, manager_approval_status,
               created_at, updated_at, approved_at
        FROM documents
        WHERE id = ANY($1) AND status = 'submitted'
        "#,
    )
    .bind(document_ids)
    .fetch_all(pool)
    .await?;

    let mut cache = CandidateCache::default();
    let mut result = HashMap::with_capacity(rows.len());

    for row in &rows {
        let Some(stage) = stage_of(row) else { continue };
        let excluded = sod_excluded(row, stage);
        let names: Vec<String> = cache
            .get(pool, stage)
            .await?
            .iter()
            .filter(|(id, _)| !excluded.contains(id))
            .map(|(_, name)| name.clone())
            .collect();

        result.insert(
            row.id,
            PendingOwner::from_candidates(
                stage.key(),
                PendingOwnerKind::Role,
                stage.role_code().map(str::to_string),
                names,
                stage_since(row, stage),
            ),
        );
    }

    Ok(result)
}

/// 供**通知層**取得「這張單這一關的合法處理人」（已扣除 SoD），回傳
/// `(關卡 key, 收件人 user_id)`。非 `submitted`（不在等任何人）→ `None`。
///
/// 與 [`resolve_for_documents`] 共用同一份 `stage_of` / `candidates` / `sod_excluded`，
/// **這是刻意的**：待處理清單的收件人與列表頁顯示的「卡在誰」若各算各的，使用者會看到
/// 「卡在王倉管」卻是李倉管收到待辦。本函式存在的唯一理由就是不讓那件事發生。
///
/// 回傳空的收件人清單也是有意義的結果（該關卡候選人被 SoD 排光＝沒有人能處理），
/// 呼叫端據此把既有待辦解除，而不是留著一則沒有人能完成的事項。
pub async fn stage_recipients_for_document(
    pool: &PgPool,
    document_id: Uuid,
) -> Result<Option<(&'static str, Vec<Uuid>)>, AppError> {
    let row = sqlx::query_as::<_, DocumentStageRow>(
        r#"
        SELECT id, doc_type, status, created_by, approved_by, system_generated,
               reverses_doc_id, requires_manager_approval, manager_approval_status,
               created_at, updated_at, approved_at
        FROM documents
        WHERE id = $1
        "#,
    )
    .bind(document_id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else { return Ok(None) };
    let Some(stage) = stage_of(&row) else {
        return Ok(None);
    };

    let excluded = sod_excluded(&row, stage);
    let recipients = stage
        .candidates(pool)
        .await?
        .into_iter()
        .map(|(id, _)| id)
        .filter(|id| !excluded.contains(id))
        .collect();

    Ok(Some((stage.key(), recipients)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(status: DocStatus, doc_type: DocType) -> DocumentStageRow {
        DocumentStageRow {
            id: Uuid::nil(),
            doc_type,
            status,
            created_by: Uuid::from_u128(1),
            approved_by: None,
            system_generated: false,
            reverses_doc_id: None,
            requires_manager_approval: None,
            manager_approval_status: None,
            created_at: DateTime::from_timestamp(1_700_000_000, 0).expect("固定時間"),
            updated_at: DateTime::from_timestamp(1_700_000_100, 0).expect("固定時間"),
            approved_at: None,
        }
    }

    #[test]
    fn only_submitted_documents_are_waiting_on_someone() {
        for status in [DocStatus::Draft, DocStatus::Approved, DocStatus::Cancelled] {
            assert!(
                stage_of(&row(status, DocType::PO)).is_none(),
                "{status:?} 不在等任何人"
            );
        }
        assert_eq!(
            stage_of(&row(DocStatus::Submitted, DocType::PO)),
            Some(DocumentStage::Warehouse)
        );
    }

    #[test]
    fn reversal_is_checked_before_final_approval() {
        // 沖銷單與大額 ADJ 終審的欄位組合完全相同，只差 reverses_doc_id。
        let mut r = row(DocStatus::Submitted, DocType::ADJ);
        r.requires_manager_approval = Some(true);
        r.manager_approval_status = Some("wm_approved".into());
        assert_eq!(stage_of(&r), Some(DocumentStage::Final));

        r.reverses_doc_id = Some(Uuid::from_u128(9));
        assert_eq!(
            stage_of(&r),
            Some(DocumentStage::Reversal),
            "沖銷單必須先判，否則會拿終審的候選人名單去顯示"
        );
    }

    #[test]
    fn pending_manager_approval_still_waits_on_the_warehouse() {
        // manager_approval_status = 'pending' 表示倉管**還沒**核准。
        let mut r = row(DocStatus::Submitted, DocType::ADJ);
        r.requires_manager_approval = Some(true);
        r.manager_approval_status = Some("pending".into());
        assert_eq!(stage_of(&r), Some(DocumentStage::Warehouse));
    }

    #[test]
    fn creator_is_excluded_only_for_adj_and_grn() {
        let adj = row(DocStatus::Submitted, DocType::ADJ);
        assert!(sod_excluded(&adj, DocumentStage::Warehouse).contains(&adj.created_by));

        let po = row(DocStatus::Submitted, DocType::PO);
        assert!(
            sod_excluded(&po, DocumentStage::Warehouse).is_empty(),
            "PO 刻意不納入自核禁令（workflow.rs 的清單註解）"
        );
    }

    #[test]
    fn system_generated_documents_have_no_real_author_to_exclude() {
        let mut r = row(DocStatus::Submitted, DocType::ADJ);
        r.system_generated = true;
        assert!(
            sod_excluded(&r, DocumentStage::Warehouse).is_empty(),
            "盤點差異單的 created_by 只是系統借用的欄位"
        );
    }

    #[test]
    fn final_approval_also_excludes_the_warehouse_approver() {
        let mut r = row(DocStatus::Submitted, DocType::ADJ);
        r.approved_by = Some(Uuid::from_u128(2));
        let excluded = sod_excluded(&r, DocumentStage::Final);
        assert!(excluded.contains(&r.created_by), "建單者");
        assert!(excluded.contains(&Uuid::from_u128(2)), "倉管核准人");
    }

    #[test]
    fn reversal_excludes_initiator_regardless_of_doc_type() {
        // 沖銷單的 SoD 不看 doc_type——PO 的沖銷單一樣禁止發起人自核。
        let r = row(DocStatus::Submitted, DocType::PO);
        assert!(sod_excluded(&r, DocumentStage::Reversal).contains(&r.created_by));
    }

    #[test]
    fn stage_since_uses_the_moment_this_stage_began() {
        let mut r = row(DocStatus::Submitted, DocType::ADJ);
        r.approved_at = DateTime::from_timestamp(1_700_000_200, 0);
        assert_eq!(
            stage_since(&r, DocumentStage::Final),
            r.approved_at,
            "終審關從倉管核准那一刻開始等"
        );
        assert_eq!(
            stage_since(&r, DocumentStage::Warehouse),
            Some(r.updated_at)
        );
        assert_eq!(stage_since(&r, DocumentStage::Reversal), Some(r.created_at));
    }
}
