use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{
        Amendment, AmendmentListItem, AmendmentQuery, AmendmentReviewAssignmentResponse,
        AmendmentStatus, AmendmentStatusHistory, AmendmentVersion,
    },
    Result,
};

use super::AmendmentService;

/// staff 需要動手的變更申請狀態（待分類 SUBMITTED/RESUBMITTED + 待審查 CLASSIFIED/UNDER_REVIEW）。
/// 抽為單一常數供 `get_pending_count` 與 `get_pending_count_for_user` 共用其共同部分，
/// 防兩者在這四個狀態上定義分歧（CodeRabbit #772）。
const PENDING_AMENDMENT_STATUSES_STAFF: [&str; 4] =
    ["SUBMITTED", "RESUBMITTED", "CLASSIFIED", "UNDER_REVIEW"];

/// 申請人角度多出來的待處理狀態：`REVISION_REQUIRED`。
///
/// ⚠️ **不能把它併進 `PENDING_AMENDMENT_STATUSES_STAFF` 本身**（CodeRabbit #31 指出）：
/// `REVISION_REQUIRED` 時球在申請人手上，staff 不需要做任何事，不該算進 staff 的
/// 全域 triage badge；但對申請人來說，那正是「我需要交修正版」的待辦，
/// 沒算進去的話申請人自己的 badge 會漏算自己被退回補件的案子。
///
/// `get_pending_count_for_user` 用 `PENDING_AMENDMENT_STATUSES_STAFF` 加這一個
/// 組成自己的清單（見下方），不是重新打一份四個狀態——共同的四個永遠只有一份
/// 定義，不會重蹈 #772（兩份各自維護、在共同狀態上分歧）的覆轍。
const PENDING_AMENDMENT_STATUS_REVISION_REQUIRED: &str = "REVISION_REQUIRED";

impl AmendmentService {
    /// 取得單一變更申請（含關聯資訊）
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Amendment> {
        Self::get_by_id_raw(pool, id).await
    }

    /// 列出變更申請
    ///
    /// ⚠️ 本查詢原為 `sqlx::query_as!`（編譯期檢查）。`AmendmentListItem` 加入
    /// `pending_owner` 後不得不改為執行期形式——那個巨集會依 SELECT 欄位逐一構造 struct，
    /// 不接受任何不在查詢裡的欄位，而 `pending_owner` 是程式算出來的、不可能出現在 SELECT。
    /// 改動後與下方 `list_for_user`（本來就是執行期形式）一致。
    pub async fn list(
        pool: &PgPool,
        query: &AmendmentQuery,
        viewer: &crate::middleware::CurrentUser,
    ) -> Result<Vec<AmendmentListItem>> {
        let mut amendments = sqlx::query_as::<_, AmendmentListItem>(
            r#"
            SELECT
                a.id, a.protocol_id, a.amendment_no, a.revision_number,
                a.amendment_type, a.status,
                a.title, a.description, a.change_items,
                a.submitted_at, a.classified_at,
                a.created_at, a.updated_at,
                a.effective_from, a.is_historical,
                p.iacuc_no as protocol_iacuc_no,
                p.title as protocol_title,
                u.display_name as submitted_by_name,
                c.display_name as classified_by_name
            FROM amendments a
            JOIN protocols p ON a.protocol_id = p.id
            LEFT JOIN users u ON a.submitted_by = u.id
            LEFT JOIN users c ON a.classified_by = c.id
            WHERE
                ($1::uuid IS NULL OR a.protocol_id = $1)
                AND ($2::text IS NULL OR a.status::text = $2)
                AND ($3::text IS NULL OR a.amendment_type::text = $3)
            ORDER BY a.created_at DESC
            "#,
        )
        .bind(query.protocol_id)
        .bind(query.status.map(|s| s.as_str().to_string()))
        .bind(query.amendment_type.map(|t| t.as_str().to_string()))
        .fetch_all(pool)
        .await?;

        Self::attach_pending_owners(pool, &mut amendments, viewer).await?;
        Ok(amendments)
    }

    /// 批次補上「卡在誰」。只送待分類 / 已分類待送審那幾筆進解析器。
    async fn attach_pending_owners(
        pool: &PgPool,
        amendments: &mut [AmendmentListItem],
        viewer: &crate::middleware::CurrentUser,
    ) -> Result<()> {
        let pending_ids: Vec<Uuid> = amendments
            .iter()
            .filter(|a| {
                matches!(
                    a.status,
                    AmendmentStatus::Submitted
                        | AmendmentStatus::Resubmitted
                        | AmendmentStatus::Classified
                        | AmendmentStatus::UnderReview
                        | AmendmentStatus::RevisionRequired
                )
            })
            .map(|a| a.id)
            .collect();
        if pending_ids.is_empty() {
            return Ok(());
        }

        let mut owners =
            crate::services::pending_owner::resolve_for_amendments(pool, &pending_ids, viewer)
                .await?;
        for row in amendments.iter_mut() {
            row.pending_owner = owners.remove(&row.id);
        }
        Ok(())
    }

    /// 列出使用者可見的變更申請（SQL 層過濾，避免取全部再客端 filter）
    ///
    /// 🔴 **既有 bug 修正（2026-08-26）**：本查詢原本沿用了 `sqlx::query_as!` 的
    /// 型別標註語法 `as "amendment_type: AmendmentType"`，但它是**執行期** `query_as::<_, T>`
    /// ——那串東西在執行期不是型別標註，是一個**欄位別名**。實測（測試庫）
    /// `SELECT a.status as "status: AmendmentStatus"` 產生的欄位名就叫
    /// `status: AmendmentStatus`，於是 `FromRow` 找不到 `status` / `amendment_type`。
    ///
    /// 影響：**沒有 `aup.protocol.view_all` 的使用者**（PI 本人、試驗工作人員）
    /// 打 `GET /amendments` 會落到本函式（`handlers/amendment.rs:122`）並在解列時失敗。
    /// 有該權限者走 `list()`，不受影響——所以這個洞只對計畫方可見，內部人員測不出來。
    pub async fn list_for_user(
        pool: &PgPool,
        query: &AmendmentQuery,
        user_id: Uuid,
        viewer: &crate::middleware::CurrentUser,
    ) -> Result<Vec<AmendmentListItem>> {
        let mut amendments = sqlx::query_as::<_, AmendmentListItem>(
            r#"
            SELECT
                a.id, a.protocol_id, a.amendment_no, a.revision_number,
                a.amendment_type, a.status,
                a.title, a.description, a.change_items,
                a.submitted_at, a.classified_at,
                a.created_at, a.updated_at,
                a.effective_from, a.is_historical,
                p.iacuc_no as protocol_iacuc_no,
                p.title as protocol_title,
                u.display_name as submitted_by_name,
                c.display_name as classified_by_name
            FROM amendments a
            JOIN protocols p ON a.protocol_id = p.id
            LEFT JOIN users u ON a.submitted_by = u.id
            LEFT JOIN users c ON a.classified_by = c.id
            WHERE
                a.protocol_id IN (SELECT protocol_id FROM user_protocols WHERE user_id = $1)
                AND ($2::uuid IS NULL OR a.protocol_id = $2)
                AND ($3::text IS NULL OR a.status::text = $3)
                AND ($4::text IS NULL OR a.amendment_type::text = $4)
            ORDER BY a.created_at DESC
            "#,
        )
        .bind(user_id)
        .bind(query.protocol_id)
        .bind(query.status.map(|s| s.as_str().to_string()))
        .bind(query.amendment_type.map(|t| t.as_str().to_string()))
        .fetch_all(pool)
        .await?;

        Self::attach_pending_owners(pool, &mut amendments, viewer).await?;
        Ok(amendments)
    }

    /// 列出計畫的所有變更申請
    pub async fn list_by_protocol(
        pool: &PgPool,
        protocol_id: Uuid,
        viewer: &crate::middleware::CurrentUser,
    ) -> Result<Vec<AmendmentListItem>> {
        Self::list(
            pool,
            &AmendmentQuery {
                protocol_id: Some(protocol_id),
                status: None,
                amendment_type: None,
            },
            viewer,
        )
        .await
    }

    /// 取得版本列表
    pub async fn get_versions(pool: &PgPool, amendment_id: Uuid) -> Result<Vec<AmendmentVersion>> {
        let versions = sqlx::query_as!(
            AmendmentVersion,
            r#"
            SELECT id, amendment_id, version_no, content_snapshot, submitted_at, submitted_by
            FROM amendment_versions
            WHERE amendment_id = $1
            ORDER BY version_no DESC
            "#,
            amendment_id
        )
        .fetch_all(pool)
        .await?;

        Ok(versions)
    }

    /// 取得狀態歷程
    pub async fn get_status_history(
        pool: &PgPool,
        amendment_id: Uuid,
    ) -> Result<Vec<AmendmentStatusHistory>> {
        let history = sqlx::query_as!(
            AmendmentStatusHistory,
            r#"
            SELECT 
                id, amendment_id,
                from_status as "from_status: AmendmentStatus",
                to_status as "to_status: AmendmentStatus",
                changed_by, remark, created_at
            FROM amendment_status_history
            WHERE amendment_id = $1
            ORDER BY created_at DESC
            "#,
            amendment_id
        )
        .fetch_all(pool)
        .await?;

        Ok(history)
    }

    /// 取得審查委員指派列表
    pub async fn get_review_assignments(
        pool: &PgPool,
        amendment_id: Uuid,
    ) -> Result<Vec<AmendmentReviewAssignmentResponse>> {
        let assignments = sqlx::query_as!(
            AmendmentReviewAssignmentResponse,
            r#"
            SELECT
                ara.id, ara.amendment_id, ara.reviewer_id, ara.assigned_by, ara.assigned_at,
                ara.decision, ara.decided_at, ara.comment,
                COALESCE(u.display_name, ara.reviewer_name, '') as "reviewer_name!",
                COALESCE(u.email, '') as "reviewer_email!"
            FROM amendment_review_assignments ara
            LEFT JOIN users u ON ara.reviewer_id = u.id
            WHERE ara.amendment_id = $1
            ORDER BY ara.assigned_at
            "#,
            amendment_id
        )
        .fetch_all(pool)
        .await?;

        Ok(assignments)
    }

    /// 取得待處理變更申請數量 (包含待分類 SUBMITTED/RESUBMITTED 和待審查 CLASSIFIED/UNDER_REVIEW)
    ///
    /// 全域版：供 staff（`aup.protocol.view_all`）的審查 triage badge。非 staff 走
    /// `get_pending_count_for_user`（R75-9：原 handler 對所有人回全域數，洩漏全院審查工作量）。
    pub async fn get_pending_count(pool: &PgPool) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM amendments
            WHERE status::text = ANY($1)
            "#,
        )
        .bind(&PENDING_AMENDMENT_STATUSES_STAFF[..])
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }

    /// R75-9：非 staff 的待處理數量——僅計使用者可見計畫（`user_protocols`）的 pending
    /// amendments，與 `list_for_user` 的可見範圍一致，避免全域工作量洩漏給 PI/CLIENT。
    ///
    /// ⚠️ 比 staff 版多算 `REVISION_REQUIRED`（CodeRabbit #31）：見上方常數的說明。
    pub async fn get_pending_count_for_user(pool: &PgPool, user_id: Uuid) -> Result<i64> {
        let statuses: Vec<&str> = PENDING_AMENDMENT_STATUSES_STAFF
            .iter()
            .copied()
            .chain(std::iter::once(PENDING_AMENDMENT_STATUS_REVISION_REQUIRED))
            .collect();
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM amendments
            WHERE status::text = ANY($1)
              AND protocol_id IN (SELECT protocol_id FROM user_protocols WHERE user_id = $2)
            "#,
        )
        .bind(&statuses[..])
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }
}
