use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use validator::Validate;

use crate::models::audit_diff::AuditRedact;

/// 變更申請類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(type_name = "amendment_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AmendmentType {
    Major,   // 重大變更
    Minor,   // 小變更
    Pending, // 待分類
}

impl AmendmentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AmendmentType::Major => "MAJOR",
            AmendmentType::Minor => "MINOR",
            AmendmentType::Pending => "PENDING",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AmendmentType::Major => "重大變更",
            AmendmentType::Minor => "小變更",
            AmendmentType::Pending => "待分類",
        }
    }
}

/// 變更申請狀態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(type_name = "amendment_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AmendmentStatus {
    Draft,
    Submitted,
    Classified,
    UnderReview,
    RevisionRequired,
    Resubmitted,
    Approved,
    Rejected,
    AdminApproved,
    /// R30-25 GLP §58：APPROVED 後正式生效的終態。`effective_from` 時間戳同步寫入。
    /// Schema-only 階段：尚無 service 寫入路徑（R30-25b/c 後續處理）。
    Effective,
}

impl AmendmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AmendmentStatus::Draft => "DRAFT",
            AmendmentStatus::Submitted => "SUBMITTED",
            AmendmentStatus::Classified => "CLASSIFIED",
            AmendmentStatus::UnderReview => "UNDER_REVIEW",
            AmendmentStatus::RevisionRequired => "REVISION_REQUIRED",
            AmendmentStatus::Resubmitted => "RESUBMITTED",
            AmendmentStatus::Approved => "APPROVED",
            AmendmentStatus::Rejected => "REJECTED",
            AmendmentStatus::AdminApproved => "ADMIN_APPROVED",
            AmendmentStatus::Effective => "EFFECTIVE",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AmendmentStatus::Draft => "草稿",
            AmendmentStatus::Submitted => "已提交",
            AmendmentStatus::Classified => "已分類",
            AmendmentStatus::UnderReview => "審查中",
            AmendmentStatus::RevisionRequired => "需修訂",
            AmendmentStatus::Resubmitted => "已重送",
            AmendmentStatus::Approved => "已核准",
            AmendmentStatus::Rejected => "已否決",
            AmendmentStatus::AdminApproved => "行政核准",
            AmendmentStatus::Effective => "已生效",
        }
    }

    /// 終態：不得再透過泛型 change_status 變更（GLP change-control，CSO #3）。
    /// EFFECTIVE 另由專屬生效流程處理（change_status 本就拒絕）。
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            AmendmentStatus::Approved
                | AmendmentStatus::Rejected
                | AmendmentStatus::AdminApproved
                | AmendmentStatus::Effective
        )
    }

    /// 泛型 change_status 允許的狀態轉移（CSO #3：原本零驗證 → 終態可被退回重開）。
    /// 僅允許審查工作流的前進 / 修訂往返；一律禁止由終態退回。
    /// EFFECTIVE 為目標時由 change_status 另行拒絕（需專屬生效流程）。
    pub fn can_transition_to(&self, to: AmendmentStatus) -> bool {
        use AmendmentStatus::*;
        // 終態不可再經泛型端點變更
        if self.is_terminal() {
            return false;
        }
        // 自我轉移無意義
        if *self == to {
            return false;
        }
        matches!(
            (self, to),
            (Draft, Submitted)
                | (Submitted, Classified)
                | (Submitted, Rejected)
                | (Classified, UnderReview)
                | (Classified, Rejected)
                | (UnderReview, RevisionRequired)
                | (UnderReview, Approved)
                | (UnderReview, AdminApproved)
                | (UnderReview, Rejected)
                | (RevisionRequired, Resubmitted)
                | (RevisionRequired, Rejected)
                | (Resubmitted, UnderReview)
                | (Resubmitted, Rejected)
        )
    }
}

/// 變更申請主表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Amendment {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub amendment_no: String,
    pub revision_number: i32,
    pub amendment_type: AmendmentType,
    pub status: AmendmentStatus,
    pub title: String,
    pub description: Option<String>,
    pub change_items: Option<Vec<String>>,
    pub changes_content: Option<serde_json::Value>,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub classified_by: Option<Uuid>,
    pub classified_at: Option<DateTime<Utc>>,
    pub classification_remark: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// C2 (GLP §11.50/§11.70)：核准決定的電子簽章 FK。NOT NULL 後 update 被拒。
    pub approved_signature_id: Option<Uuid>,
    /// C2 (GLP §11.50/§11.70)：否決決定的電子簽章 FK。
    pub rejected_signature_id: Option<Uuid>,
    /// R30-25 GLP §58：amendment 正式生效時點。NULL = 尚未生效（含 APPROVED 但未啟用）。
    pub effective_from: Option<DateTime<Utc>>,
    pub version: i32,
    /// 補登歷史變更標記（P6）：true = 紙本核准回溯，跳過 live 審查、無 live 電子簽章。
    pub is_historical: bool,
}

/// R30-25b：Amendment 內無敏感欄位（密碼/token 等），audit log 全欄保留無 redact。
impl AuditRedact for Amendment {}

/// 變更申請版本
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AmendmentVersion {
    pub id: Uuid,
    pub amendment_id: Uuid,
    pub version_no: i32,
    pub content_snapshot: serde_json::Value,
    pub submitted_at: DateTime<Utc>,
    pub submitted_by: Uuid,
}

/// 變更申請審查指派
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AmendmentReviewAssignment {
    pub id: Uuid,
    pub amendment_id: Uuid,
    /// 系統內委員 FK；院外委員（補登歷史變更）為 NULL，改用 `reviewer_name`（migration 088）
    pub reviewer_id: Option<Uuid>,
    pub assigned_by: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub decision: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub comment: Option<String>,
}

/// 變更申請狀態歷程
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AmendmentStatusHistory {
    pub id: Uuid,
    pub amendment_id: Uuid,
    pub from_status: Option<AmendmentStatus>,
    pub to_status: AmendmentStatus,
    pub changed_by: Uuid,
    pub remark: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================
// Request/Response DTOs
// ============================================

/// 建立變更申請請求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateAmendmentRequest {
    pub protocol_id: Uuid,
    #[validate(length(min = 1, max = 200, message = "Title must be 1-200 characters"))]
    pub title: String,
    pub description: Option<String>,
    pub change_items: Option<Vec<String>>,
    pub changes_content: Option<serde_json::Value>,
}

/// 更新變更申請請求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAmendmentRequest {
    #[validate(length(min = 1, max = 200, message = "Title must be 1-200 characters"))]
    pub title: Option<String>,
    pub description: Option<String>,
    pub change_items: Option<Vec<String>>,
    pub changes_content: Option<serde_json::Value>,
    pub version: Option<i32>,
}

/// 分類變更申請請求
#[derive(Debug, Deserialize)]
pub struct ClassifyAmendmentRequest {
    pub amendment_type: AmendmentType,
    pub remark: Option<String>,
}

/// 變更狀態請求
#[derive(Debug, Deserialize)]
pub struct ChangeAmendmentStatusRequest {
    pub to_status: AmendmentStatus,
    pub remark: Option<String>,
}

/// R30-25b：標記 amendment 為 EFFECTIVE。
/// `effective_from` 留空時取 NOW()；指定時可回填過去日期（例如核准後一段時間補登）。
#[derive(Debug, Deserialize, Validate)]
pub struct MarkAmendmentEffectiveRequest {
    pub effective_from: Option<DateTime<Utc>>,
    #[validate(length(max = 1000, message = "Remark must be at most 1000 characters"))]
    pub remark: Option<String>,
}

/// 補登歷史變更申請請求（P6）：建立 is_historical 草稿，跳過 live 審查。
/// `amendment_type` 須為 MAJOR / MINOR（歷史紙本早已分類，不可 PENDING）。
#[derive(Debug, Deserialize, Validate)]
pub struct CreateHistoricalAmendmentRequest {
    pub protocol_id: Uuid,
    #[validate(length(min = 1, max = 200, message = "Title must be 1-200 characters"))]
    pub title: String,
    pub description: Option<String>,
    pub change_items: Option<Vec<String>>,
    pub changes_content: Option<serde_json::Value>,
    /// 歷史分類：MAJOR（委員審）/ MINOR（執秘行政核准）
    pub amendment_type: AmendmentType,
    /// 原始送件日期（回填，可空）
    pub submitted_at: Option<DateTime<Utc>>,
    /// 原始分類日期（回填，可空）
    pub classified_at: Option<DateTime<Utc>>,
    #[validate(length(
        max = 1000,
        message = "Classification remark must be at most 1000 characters"
    ))]
    pub classification_remark: Option<String>,
}

/// 完成補登歷史變更請求（P6）：DRAFT → EFFECTIVE。
/// `effective_from` 留空取 NOW()；指定時回填原始生效日期。
#[derive(Debug, Deserialize, Validate)]
pub struct FinalizeHistoricalAmendmentRequest {
    pub effective_from: Option<DateTime<Utc>>,
    #[validate(length(max = 1000, message = "Remark must be at most 1000 characters"))]
    pub remark: Option<String>,
}

/// 補登歷史變更的單一委員審查（P6-3）。系統內委員填 `reviewer_id`；院外填 `reviewer_name`。
#[derive(Debug, Deserialize)]
pub struct HistoricalAmendmentReviewer {
    pub reviewer_id: Option<Uuid>,
    pub reviewer_name: Option<String>,
    /// APPROVE / REJECT / REVISION（可空：僅記錄指派、未留決定）
    pub decision: Option<String>,
    pub comment: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
}

/// 補登歷史變更審查文件請求（P6-3）：全量取代該變更的委員審查指派。
#[derive(Debug, Deserialize)]
pub struct RecordHistoricalReviewsRequest {
    pub reviewers: Vec<HistoricalAmendmentReviewer>,
}

/// 記錄審查決定請求
#[derive(Debug, Deserialize, Validate)]
pub struct RecordAmendmentDecisionRequest {
    pub decision: String, // APPROVE, REJECT, REVISION
    #[validate(length(min = 1, message = "Comment is required for decision"))]
    pub comment: Option<String>,
}

/// 變更申請查詢
#[derive(Debug, Deserialize)]
pub struct AmendmentQuery {
    pub protocol_id: Option<Uuid>,
    pub status: Option<AmendmentStatus>,
    pub amendment_type: Option<AmendmentType>,
}

/// 變更申請回應（含關聯資訊）
#[derive(Debug, Serialize)]
pub struct AmendmentResponse {
    #[serde(flatten)]
    pub amendment: Amendment,
    pub protocol_iacuc_no: Option<String>,
    pub protocol_title: Option<String>,
    pub submitted_by_name: Option<String>,
    pub classified_by_name: Option<String>,
    pub status_display: String,
    pub type_display: String,
}

/// 變更申請列表項目
#[derive(Debug, Serialize, FromRow)]
pub struct AmendmentListItem {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub amendment_no: String,
    pub revision_number: i32,
    pub amendment_type: AmendmentType,
    pub status: AmendmentStatus,
    pub title: String,
    pub description: Option<String>,
    pub change_items: Option<Vec<String>>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub classified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// R30-25 GLP §58：amendment 正式生效時點。NULL = 尚未生效。
    #[sqlx(default)]
    pub effective_from: Option<DateTime<Utc>>,
    /// P6：補登歷史變更標記（列表用 badge 區分）
    #[sqlx(default)]
    pub is_historical: bool,
    #[sqlx(default)]
    pub protocol_iacuc_no: Option<String>,
    #[sqlx(default)]
    pub protocol_title: Option<String>,
    #[sqlx(default)]
    pub submitted_by_name: Option<String>,
    #[sqlx(default)]
    pub classified_by_name: Option<String>,
}

/// 審查委員指派回應（含用戶資訊）
#[derive(Debug, Serialize, FromRow)]
pub struct AmendmentReviewAssignmentResponse {
    pub id: Uuid,
    pub amendment_id: Uuid,
    /// 系統內委員 FK；院外委員（補登）為 NULL，姓名走 `reviewer_name`
    pub reviewer_id: Option<Uuid>,
    pub assigned_by: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub decision: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub comment: Option<String>,
    #[sqlx(default)]
    pub reviewer_name: String,
    #[sqlx(default)]
    pub reviewer_email: String,
}

/// 待處理變更申請數量回應
#[derive(Debug, Serialize)]
pub struct PendingCountResponse {
    pub count: i64,
}

#[cfg(test)]
mod tests {
    use super::AmendmentStatus::*;

    #[test]
    fn terminal_states_cannot_transition() {
        // CSO #3：終態不可經泛型 change_status 退回
        for terminal in [Approved, Rejected, AdminApproved, Effective] {
            for to in [Draft, Submitted, UnderReview, Classified, Resubmitted] {
                assert!(
                    !terminal.can_transition_to(to),
                    "{:?} 不應可轉移到 {:?}",
                    terminal,
                    to
                );
            }
        }
    }

    #[test]
    fn legal_forward_transitions_allowed() {
        assert!(Draft.can_transition_to(Submitted));
        assert!(Submitted.can_transition_to(Classified));
        assert!(Classified.can_transition_to(UnderReview));
        assert!(UnderReview.can_transition_to(Approved));
        assert!(UnderReview.can_transition_to(RevisionRequired));
        assert!(RevisionRequired.can_transition_to(Resubmitted));
        assert!(Resubmitted.can_transition_to(UnderReview));
    }

    #[test]
    fn illegal_skips_and_self_transitions_rejected() {
        // 跳過 classify
        assert!(!Submitted.can_transition_to(UnderReview));
        // 跳過審查直接核准
        assert!(!Draft.can_transition_to(Approved));
        // 自我轉移
        assert!(!UnderReview.can_transition_to(UnderReview));
        // EFFECTIVE 由專屬流程處理，泛型不可達（且來源非終態時也不在白名單）
        assert!(!UnderReview.can_transition_to(Effective));
    }
}
