use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// 計畫狀態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(type_name = "protocol_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProtocolStatus {
    Draft,
    Submitted,
    PreReview,
    PreReviewRevisionRequired,
    VetReview,
    VetRevisionRequired,
    UnderReview,
    RevisionRequired,
    Resubmitted,
    Approved,
    ApprovedWithConditions,
    Deferred,
    Rejected,
    Suspended,
    Closed,
    Deleted,
}

impl ProtocolStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProtocolStatus::Draft => "DRAFT",
            ProtocolStatus::Submitted => "SUBMITTED",
            ProtocolStatus::PreReview => "PRE_REVIEW",
            ProtocolStatus::PreReviewRevisionRequired => "PRE_REVIEW_REVISION_REQUIRED",
            ProtocolStatus::VetReview => "VET_REVIEW",
            ProtocolStatus::VetRevisionRequired => "VET_REVISION_REQUIRED",
            ProtocolStatus::UnderReview => "UNDER_REVIEW",
            ProtocolStatus::RevisionRequired => "REVISION_REQUIRED",
            ProtocolStatus::Resubmitted => "RESUBMITTED",
            ProtocolStatus::Approved => "APPROVED",
            ProtocolStatus::ApprovedWithConditions => "APPROVED_WITH_CONDITIONS",
            ProtocolStatus::Deferred => "DEFERRED",
            ProtocolStatus::Rejected => "REJECTED",
            ProtocolStatus::Suspended => "SUSPENDED",
            ProtocolStatus::Closed => "CLOSED",
            ProtocolStatus::Deleted => "DELETED",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ProtocolStatus::Draft => "草稿",
            ProtocolStatus::Submitted => "已提交",
            ProtocolStatus::PreReview => "行政預審",
            ProtocolStatus::PreReviewRevisionRequired => "行政預審補件",
            ProtocolStatus::VetReview => "獸醫審查",
            ProtocolStatus::VetRevisionRequired => "獸醫要求修訂",
            ProtocolStatus::UnderReview => "審查中",
            ProtocolStatus::RevisionRequired => "需修訂",
            ProtocolStatus::Resubmitted => "已重送",
            ProtocolStatus::Approved => "已核准",
            ProtocolStatus::ApprovedWithConditions => "附條件核准",
            ProtocolStatus::Deferred => "延後審議",
            ProtocolStatus::Rejected => "已否決",
            ProtocolStatus::Suspended => "已暫停",
            ProtocolStatus::Closed => "已結案",
            ProtocolStatus::Deleted => "已刪除",
        }
    }

    /// 終態：不得再經泛型 `change_status` 離開（CSO-r2 #2，GLP/IACUC 核准完整性）。
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ProtocolStatus::Rejected | ProtocolStatus::Closed | ProtocolStatus::Deleted
        )
    }

    /// 泛型 `change_status` 端點允許的狀態轉移（CSO-r2 #2）。
    ///
    /// 原本對 Rejected / Suspended / Closed / Deferred / RevisionRequired / Draft /
    /// Submitted / Resubmitted 完全沒有 from-state 驗證，導致「已核准」計畫可被單一秘書
    /// 角色退回 review pipeline、否決、暫停、結案甚至刪除，繞過委員會核准完整性。
    ///
    /// 此處對「已核准 / 終態」叢集加上 egress 白名單（僅保留合法的結案 / 暫停 / 復原 /
    /// 達成附條件等生命週期轉移）；其餘審查工作流（非終態、非已核准）維持既有
    /// per-target entry guard 行為，但額外禁止「直接跳至暫停 / 結案」（此二為已核准後
    /// 生命週期狀態，不應從草稿 / 審查中等狀態直接到達）。
    pub fn can_change_status_to(&self, to: ProtocolStatus) -> bool {
        use ProtocolStatus::*;
        // 終態不可離開
        if self.is_terminal() {
            return false;
        }
        match self {
            // 已核准叢集：僅允許結案 / 暫停（暫停可再復原或結案）。
            Approved => matches!(to, Closed | Suspended),
            ApprovedWithConditions => matches!(to, Approved | Closed | Suspended),
            Suspended => matches!(to, Approved | ApprovedWithConditions | Closed),
            // 其餘審查工作流：維持既有 entry-guard 行為，但禁止直接跳至暫停 / 結案。
            _ => !matches!(to, Suspended | Closed),
        }
    }
}

/// 計畫書主表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Protocol {
    pub id: Uuid,
    pub protocol_no: String,
    pub iacuc_no: Option<String>,
    pub title: String,
    pub status: ProtocolStatus,
    pub pi_user_id: Uuid,
    pub working_content: Option<serde_json::Value>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// R30-B optimistic lock：每次 UPDATE 自動 +1，前端送 PUT 時帶當前 version
    /// 防 lost update。NULL 跳過版本檢查（向後相容舊客戶端）。
    #[serde(default)]
    pub version: i32,
    /// 申請時間（歷史匯入用，記錄原始送件日期）
    pub submitted_at: Option<NaiveDate>,
    /// 通過時間（歷史匯入用，記錄原始核准日期）
    pub approved_at: Option<NaiveDate>,
    /// 申請編號（例 APIG-103001），與 iacuc_no（核准編號 PIG-115001）區分（匯入補登）
    pub application_no: Option<String>,
    /// 匯入補登中：APPROVED 但允許編輯 working_content，完成補登後清除（import P1）
    #[serde(default)]
    pub import_pending: bool,
    /// 原計劃書版本號文字（補登，例 v2.1）
    pub original_version_label: Option<String>,
    /// 計劃負責人 / Study Director（本公司員工，自 EXPERIMENT_STAFF 挑選）
    pub study_director_user_id: Option<Uuid>,
    /// 匯入計劃建立時間（import_approved 寫入）；非 NULL = 補登匯入計劃（永久標記，
    /// 與暫態 import_pending 區分）。補登歷史變更僅允許於此類計劃（P6）。
    pub imported_at: Option<DateTime<Utc>>,
    /// 計畫書表單版本鍵（C/D/E/F…）；驅動版本名冊 manifest 渲染；null=最新版，
    /// 變更升級最新版時更新。與 original_version_label（原版標籤自由文字）區分。
    pub source_form_version: Option<String>,
    /// GLP 計畫。**判定的權威來源**（migration 006 / 裁定 14）。
    ///
    /// ⚠️ 不要改用 `working_content -> basic -> is_glp` 做規則判定——那是設計上
    /// 就可編輯的工作中內容，拿它當判定來源等於沒有規則：分兩個 request
    /// （先關掉 is_glp，再改 SD）就能繞過「GLP 案不可換 SD」。
    ///
    /// `working_content` 裡那份仍然存在，作為表單內容的一部分由使用者編輯；
    /// 未鎖定時會同步到本欄位，鎖定後兩者可能不同，**一律以本欄位為準**。
    #[serde(default)]
    pub is_glp: bool,
    /// 結案雙簽：PI 那一簽（migration 008 / 設計 A）。
    ///
    /// ⚠️ **非空不等於「PI 已有效簽署」。** 這是外鍵，只保證那一列簽章存在，
    /// 不保證它是**這份計畫的**、**結案用的**、**還有效的**。判斷「可以結案了嗎」
    /// 一律走 `services::protocol::closure::dual_signature_ready`（7 條條件），
    /// 不要在別處自己寫 `is_some()` 判斷——那正是這個設計要防的繞過方式。
    pub close_pi_signature_id: Option<Uuid>,
    /// 結案雙簽：SD 那一簽。注意事項同 `close_pi_signature_id`。
    pub close_sd_signature_id: Option<Uuid>,
}

/// Protocol 無敏感欄位需脫敏（GLP 稽核需要完整內容；working_content 雖為 jsonb
/// 可能含 PII 但 protocol 本身就是稽核標的，不 redact）。
impl crate::models::audit_diff::AuditRedact for Protocol {}

/// 計畫版本快照
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ProtocolVersion {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub version_no: i32,
    pub content_snapshot: serde_json::Value,
    pub submitted_at: DateTime<Utc>,
    pub submitted_by: Uuid,
}

/// 計畫活動類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(
    type_name = "protocol_activity_type",
    rename_all = "SCREAMING_SNAKE_CASE"
)]
pub enum ProtocolActivityType {
    // 生命週期
    Created,
    Updated,
    Submitted,
    Resubmitted,
    Approved,
    ApprovedWithConditions,
    Closed,
    Rejected,
    Suspended,
    Deleted,
    // 審查流程
    StatusChanged,
    ReviewerAssigned,
    VetAssigned,
    /// 指派／變更計劃負責人（SD）。裁定 21：SD 變更必須有專屬事件，
    /// 不能混在通用的 PROTOCOL_UPDATE 裡——稽核報表上看不出「這次改的是 SD」。
    SdAssigned,
    // CO_EDITOR 角色已拆除（R76-2），但保留下列活動類型供讀取歷史稽核紀錄（不再新寫）。
    CoeditorAssigned,
    CoeditorRemoved,
    // 審查意見
    CommentAdded,
    CommentReplied,
    CommentResolved,
    // 附件
    AttachmentUploaded,
    AttachmentDeleted,
    // 版本
    VersionCreated,
    VersionRecovered,
    // 修正案
    AmendmentCreated,
    AmendmentSubmitted,
    // 動物管理
    AnimalAssigned,
    AnimalUnassigned,
    // MCP 稽核
    McpRead,
}

impl ProtocolActivityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProtocolActivityType::Created => "CREATED",
            ProtocolActivityType::Updated => "UPDATED",
            ProtocolActivityType::Submitted => "SUBMITTED",
            ProtocolActivityType::Resubmitted => "RESUBMITTED",
            ProtocolActivityType::Approved => "APPROVED",
            ProtocolActivityType::ApprovedWithConditions => "APPROVED_WITH_CONDITIONS",
            ProtocolActivityType::Closed => "CLOSED",
            ProtocolActivityType::Rejected => "REJECTED",
            ProtocolActivityType::Suspended => "SUSPENDED",
            ProtocolActivityType::Deleted => "DELETED",
            ProtocolActivityType::StatusChanged => "STATUS_CHANGED",
            ProtocolActivityType::ReviewerAssigned => "REVIEWER_ASSIGNED",
            ProtocolActivityType::VetAssigned => "VET_ASSIGNED",
            ProtocolActivityType::SdAssigned => "SD_ASSIGNED",
            ProtocolActivityType::CoeditorAssigned => "COEDITOR_ASSIGNED",
            ProtocolActivityType::CoeditorRemoved => "COEDITOR_REMOVED",
            ProtocolActivityType::CommentAdded => "COMMENT_ADDED",
            ProtocolActivityType::CommentReplied => "COMMENT_REPLIED",
            ProtocolActivityType::CommentResolved => "COMMENT_RESOLVED",
            ProtocolActivityType::AttachmentUploaded => "ATTACHMENT_UPLOADED",
            ProtocolActivityType::AttachmentDeleted => "ATTACHMENT_DELETED",
            ProtocolActivityType::VersionCreated => "VERSION_CREATED",
            ProtocolActivityType::VersionRecovered => "VERSION_RECOVERED",
            ProtocolActivityType::AmendmentCreated => "AMENDMENT_CREATED",
            ProtocolActivityType::AmendmentSubmitted => "AMENDMENT_SUBMITTED",
            ProtocolActivityType::AnimalAssigned => "ANIMAL_ASSIGNED",
            ProtocolActivityType::AnimalUnassigned => "ANIMAL_UNASSIGNED",
            ProtocolActivityType::McpRead => "MCP_READ",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ProtocolActivityType::Created => "創建草稿",
            ProtocolActivityType::Updated => "編輯計畫",
            ProtocolActivityType::Submitted => "送審",
            ProtocolActivityType::Resubmitted => "重新送審",
            ProtocolActivityType::Approved => "通過",
            ProtocolActivityType::ApprovedWithConditions => "附條件通過",
            ProtocolActivityType::Closed => "結案",
            ProtocolActivityType::Rejected => "否決",
            ProtocolActivityType::Suspended => "暫停",
            ProtocolActivityType::Deleted => "刪除",
            ProtocolActivityType::StatusChanged => "狀態變更",
            ProtocolActivityType::ReviewerAssigned => "指派審查委員",
            ProtocolActivityType::VetAssigned => "指派獸醫師",
            ProtocolActivityType::SdAssigned => "指派計劃負責人",
            ProtocolActivityType::CoeditorAssigned => "指派共同編輯者",
            ProtocolActivityType::CoeditorRemoved => "移除共同編輯者",
            ProtocolActivityType::CommentAdded => "新增審查意見",
            ProtocolActivityType::CommentReplied => "回覆審查意見",
            ProtocolActivityType::CommentResolved => "解決審查意見",
            ProtocolActivityType::AttachmentUploaded => "上傳附件",
            ProtocolActivityType::AttachmentDeleted => "刪除附件",
            ProtocolActivityType::VersionCreated => "建立版本快照",
            ProtocolActivityType::VersionRecovered => "回復至版本",
            ProtocolActivityType::AmendmentCreated => "建立修正案",
            ProtocolActivityType::AmendmentSubmitted => "送審修正案",
            ProtocolActivityType::AnimalAssigned => "分配動物",
            ProtocolActivityType::AnimalUnassigned => "移除動物",
            ProtocolActivityType::McpRead => "MCP 閱覽",
        }
    }
}

/// 計畫活動歷程
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ProtocolActivity {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub activity_type: ProtocolActivityType,
    pub actor_id: Uuid,
    pub actor_name: Option<String>,
    pub actor_email: Option<String>,
    pub from_value: Option<String>,
    pub to_value: Option<String>,
    pub target_entity_type: Option<String>,
    pub target_entity_id: Option<Uuid>,
    pub target_entity_name: Option<String>,
    pub remark: Option<String>,
    pub extra_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// 計畫活動歷程回應（用於 API）
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ProtocolActivityResponse {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub activity_type: ProtocolActivityType,
    pub activity_type_display: String,
    pub actor_id: Uuid,
    pub actor_name: String,
    pub actor_email: String,
    pub from_value: Option<String>,
    pub to_value: Option<String>,
    pub target_entity_type: Option<String>,
    pub target_entity_id: Option<Uuid>,
    pub target_entity_name: Option<String>,
    pub remark: Option<String>,
    pub extra_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl From<ProtocolActivity> for ProtocolActivityResponse {
    fn from(activity: ProtocolActivity) -> Self {
        Self {
            id: activity.id,
            protocol_id: activity.protocol_id,
            activity_type: activity.activity_type,
            activity_type_display: activity.activity_type.display_name().to_string(),
            actor_id: activity.actor_id,
            actor_name: activity.actor_name.unwrap_or_default(),
            actor_email: activity.actor_email.unwrap_or_default(),
            from_value: activity.from_value,
            to_value: activity.to_value,
            target_entity_type: activity.target_entity_type,
            target_entity_id: activity.target_entity_id,
            target_entity_name: activity.target_entity_name,
            remark: activity.remark,
            extra_data: activity.extra_data,
            created_at: activity.created_at,
        }
    }
}

/// 審查人員指派
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ReviewAssignment {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub reviewer_id: Uuid,
    pub assigned_by: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    /// 是否為正式審查委員（可撰寫意見，限 2-3 位）
    #[sqlx(default)]
    pub is_primary_reviewer: bool,
    /// 審查階段
    #[sqlx(default)]
    pub review_stage: Option<String>,
}

/// 審查意見
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ReviewComment {
    pub id: Uuid,
    #[sqlx(default)]
    pub protocol_version_id: Option<Uuid>,
    /// 系統內審查者 FK；院外審查者（補登）為 NULL，改用 reviewer_name
    pub reviewer_id: Option<Uuid>,
    /// 院外審查者姓名（補登用，reviewer_id 為 NULL 時填）
    #[sqlx(default)]
    pub reviewer_name: Option<String>,
    pub content: String,
    /// 意見類型：`COMMENT`（一般意見，申請人需回覆）／`NO_OBJECTION`（無意見，不需回覆）。
    /// 兩者都算「已發表意見」，核准閘門（`status.rs`）一視同仁。
    #[sqlx(default)]
    pub comment_type: String,
    pub is_resolved: bool,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub parent_comment_id: Option<Uuid>,
    pub replied_by: Option<Uuid>,
    /// 草稿回覆內容（僅 PI/Coeditor 可見）
    pub draft_content: Option<String>,
    /// 草稿撰寫者
    pub drafted_by: Option<Uuid>,
    /// 草稿最後更新時間
    pub draft_updated_at: Option<DateTime<Utc>>,
    /// 直接關聯的計畫 ID（用於預審階段）
    #[sqlx(rename = "protocol_id")]
    pub protocol_id: Option<Uuid>,
    /// 審查階段（PRE_REVIEW, VET_REVIEW, UNDER_REVIEW）
    #[sqlx(default, rename = "review_stage")]
    pub review_stage: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 計畫附件
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ProtocolAttachment {
    pub id: Uuid,
    pub protocol_version_id: Option<Uuid>,
    pub protocol_id: Option<Uuid>,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i32,
    pub mime_type: String,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// 計畫中的角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "protocol_role", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProtocolRole {
    Pi,
    Client,
}

/// 使用者計畫關聯
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserProtocol {
    pub user_id: Uuid,
    pub protocol_id: Uuid,
    pub role_in_protocol: ProtocolRole,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<Uuid>,
}

/// 獸醫審查指派
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct VetReviewAssignment {
    pub id: Uuid,
    pub protocol_id: Uuid,
    /// 系統內獸醫師 FK；院外獸醫師（補登）為 NULL，改用 vet_name
    pub vet_id: Option<Uuid>,
    /// 院外獸醫師姓名（補登用，vet_id 為 NULL 時填）
    #[sqlx(default)]
    pub vet_name: Option<String>,
    pub assigned_by: Option<Uuid>,
    pub assigned_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub decision: Option<String>,
    pub decision_remark: Option<String>,
    pub review_form: Option<serde_json::Value>,
}

/// 獸醫審查查檢項
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VetReviewItem {
    pub item_name: String,
    pub compliance: String, // V, X, -
    pub comment: Option<String>,
    pub pi_reply: Option<String>,
}

/// 獸醫審查表
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VetReviewForm {
    pub items: Vec<VetReviewItem>,
    pub vet_signature: Option<String>,
    pub signed_at: Option<DateTime<Utc>>,
}

/// 系統設定
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct SystemSetting {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

// ============================================
// Request/Response DTOs
// ============================================

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateProtocolRequest {
    #[validate(length(min = 1, max = 500, message = "Title must be 1-500 characters"))]
    pub title: String,
    pub pi_user_id: Option<Uuid>,
    pub working_content: Option<serde_json::Value>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    /// 計劃負責人（SD，本公司內部 EXPERIMENT_STAFF）。選填：客戶/PI 建立時留空，
    /// 由執行秘書事後於編輯頁指派。僅 IACUC_STAFF / admin 可指派他人；其餘僅限本人。
    pub study_director_user_id: Option<Uuid>,
}

/// 匯入「已核准計劃」請求（場內既有、已通過審查的計劃，直接進系統成 APPROVED）。
/// iacuc_no 為既有真編號（必填）；跳過 IACUC 審查 state machine。
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ImportApprovedProtocolRequest {
    #[validate(length(min = 1, max = 500, message = "Title must be 1-500 characters"))]
    pub title: String,
    /// 計畫主持人（PI）系統使用者；外部 PI（無系統帳號）為 None，
    /// 此時 pi_user_id 記為匯入者本人，PI 真實身分存於 working_content.basic.pi / sponsor。
    pub pi_user_id: Option<Uuid>,
    /// 既有 IACUC 核准編號（必填，作為會計 customer code），例 PIG-115001
    #[validate(length(min = 1, max = 50, message = "IACUC no. is required"))]
    pub iacuc_no: String,
    /// 申請編號（選填），例 APIG-103001
    #[validate(length(max = 50, message = "Application no. must be at most 50 characters"))]
    pub application_no: Option<String>,
    /// 計劃負責人 / Study Director（必填，本公司員工 EXPERIMENT_STAFF）
    pub study_director_user_id: Uuid,
    pub working_content: Option<serde_json::Value>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    /// 原始申請（送件）日期（歷史時間軸里程碑）
    pub submitted_at: Option<NaiveDate>,
    /// 執行秘書行政預審日期（PRE_REVIEW）
    pub pre_review_at: Option<NaiveDate>,
    /// 獸醫師審查日期（VET_REVIEW）
    pub vet_review_at: Option<NaiveDate>,
    /// 委員第一次審查（初審）日期（UNDER_REVIEW 第一輪）
    pub committee_first_review_at: Option<NaiveDate>,
    /// 補件 / 修訂退回日期（REVISION_REQUIRED）
    pub revision_required_at: Option<NaiveDate>,
    /// 委員第二次審查（複審）日期（UNDER_REVIEW 第二輪）
    pub committee_second_review_at: Option<NaiveDate>,
    /// 原始核准（通過）日期（APPROVED）
    pub approved_at: Option<NaiveDate>,
    /// 匯入備註（寫入 audit / activity remark）
    pub remark: Option<String>,
    /// PR-E 須知簽署承接：當時生效的須知版次標籤（對應 `application_notices.version_label`）。
    /// 提供時依方案 A 記一筆歷史 acknowledgement（無電子簽章）。
    #[validate(length(max = 100, message = "須知版次標籤至多 100 字"))]
    pub notice_version_label: Option<String>,
    /// 紙本須知簽名掃描 attachment id（方案 A）。
    pub notice_attachment_id: Option<Uuid>,
    /// 須知簽署日期（歷史；未提供時記為匯入當下）。
    pub notice_acknowledged_at: Option<NaiveDate>,
    /// 計畫書表單版本鍵（C/D/E/F…），驅動版本名冊渲染；補登時由「先選版本」決定。
    #[validate(length(max = 20, message = "版本鍵至多 20 字"))]
    pub source_form_version: Option<String>,
}

/// import P1：完成補登請求（清 import_pending + 建 v1 版本快照）。
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct FinalizeImportRequest {
    /// 原計劃書版本號（選填，例 v2.1）
    #[validate(length(max = 50, message = "Version label must be at most 50 characters"))]
    pub original_version_label: Option<String>,
}

/// import P2：一條補登審查意見（系統內審查者用 id、院外用姓名擇一）。
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ImportReviewComment {
    /// 系統內審查者 FK（與 reviewer_name 擇一）
    pub reviewer_id: Option<Uuid>,
    /// 院外審查者姓名（與 reviewer_id 擇一）
    pub reviewer_name: Option<String>,
    /// 審查意見內容
    pub content: String,
    /// 申請人 / 客戶對此意見的回覆（選填，存為子意見）
    pub reply: Option<String>,
    /// 對應計畫書項次（如 4.1.2，選填），釐清意見指涉內容。
    #[serde(default)]
    pub section_no: Option<String>,
}

/// import P2：一位委員的意見（一審 + 二審）。
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ImportCommitteeReviewer {
    pub reviewer_id: Option<Uuid>,
    pub reviewer_name: Option<String>,
    /// 第一次審查意見（UNDER_REVIEW）
    #[serde(default)]
    pub first_round: Vec<ImportReviewComment>,
    /// 第二次審查意見（FINAL_REVIEW）
    #[serde(default)]
    pub second_round: Vec<ImportReviewComment>,
}

/// import P2：獸醫師評比與意見。
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ImportVetReview {
    pub vet_id: Option<Uuid>,
    pub vet_name: Option<String>,
    pub decision: Option<String>,
    pub decision_remark: Option<String>,
    #[serde(default)]
    pub items: Vec<VetReviewItem>,
    pub signed_at: Option<DateTime<Utc>>,
}

/// import P2：補登 5 份審查文件之一次性記錄請求。
/// 倫理委員會主席核准同意函以附件上傳（既有 upload_protocol_attachment），不在此 payload。
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ImportReviewsRequest {
    /// 執秘意見（行政預審 PRE_REVIEW）
    #[serde(default)]
    pub secretary_comments: Vec<ImportReviewComment>,
    /// 各委員意見
    #[serde(default)]
    pub committee_reviewers: Vec<ImportCommitteeReviewer>,
    /// 獸醫師評比與意見
    pub vet_review: Option<ImportVetReview>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProtocolRequest {
    #[validate(length(min = 1, max = 500, message = "Title must be 1-500 characters"))]
    pub title: Option<String>,
    pub working_content: Option<serde_json::Value>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    /// 計劃負責人（SD）變更：僅 IACUC_STAFF / admin 可指派他人；其餘僅限本人。
    /// GLP 計劃一旦已指派 SD 即鎖定，不可變更（GLP Study Director 為法規正式角色）。
    /// None → 不變更現有 SD（向後相容）。
    pub study_director_user_id: Option<Uuid>,
    /// R30-B optimistic lock：前端從 query 結果取當前 version 回送；
    /// None → 跳過版本檢查（向後相容）。命中 0 row → 409 Conflict。
    pub version: Option<i32>,
    /// 版本名冊：重選版本 / 升級最新版時更新表單版本鍵（C/D/E/F…）。
    /// None → 不變更（COALESCE 保留）。
    #[validate(length(max = 20, message = "版本鍵至多 20 字"))]
    pub source_form_version: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangeStatusRequest {
    pub to_status: ProtocolStatus,
    pub remark: Option<String>,
    /// 審查委員 ID 列表（當目標狀態為 UNDER_REVIEW 時必填 2-3 位）
    pub reviewer_ids: Option<Vec<Uuid>>,
    /// 獸醫師 ID（當目標狀態為 VET_REVIEW 時可選，未設定則使用預設獸醫）
    pub vet_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignReviewerRequest {
    pub protocol_id: Uuid,
    pub reviewer_id: Uuid,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateCommentRequest {
    pub protocol_version_id: Uuid,
    #[validate(length(min = 1, max = 10_000, message = "Content must be 1-10000 characters"))]
    pub content: String,
    /// 審查階段（若未提供，自動根據 protocol status 決定）
    pub review_stage: Option<String>,
    /// 意見類型：`COMMENT`（預設，一般意見）／`NO_OBJECTION`（無意見，申請人不需回覆）。
    ///
    /// ⚠️ 未提供時一律視為 `COMMENT`——舊客戶端不帶這個欄位，行為必須與改動前完全相同。
    pub comment_type: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ReplyCommentRequest {
    pub parent_comment_id: Uuid,
    #[validate(length(min = 1, max = 10_000, message = "Content must be 1-10000 characters"))]
    pub content: String,
}

/// 儲存草稿回覆請求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SaveDraftRequest {
    pub comment_id: Uuid,
    #[validate(length(
        min = 1,
        max = 10_000,
        message = "Draft content must be 1-10000 characters"
    ))]
    pub draft_content: String,
}

/// 送出回覆請求（將草稿正式送出）
#[derive(Debug, Deserialize, ToSchema)]
pub struct SubmitReplyRequest {
    pub comment_id: Uuid,
}

/// 儲存獸醫審查表請求
#[derive(Debug, Deserialize, ToSchema)]
pub struct SaveVetReviewFormRequest {
    pub protocol_id: Uuid,
    pub review_form: serde_json::Value,
}

#[derive(Debug, Default, Deserialize)]
pub struct ProtocolQuery {
    pub status: Option<ProtocolStatus>,
    pub pi_user_id: Option<Uuid>,
    pub keyword: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    /// 「可指派計畫」過濾（動物 IACUC No. 下拉用）：僅回傳已核准（APPROVED /
    /// APPROVED_WITH_CONDITIONS）且具 iacuc_no 的計畫。預設 false（不過濾）。
    /// 套用於 view_all 與「我的計劃」兩條授權路徑，不改變各自可見範圍。
    #[serde(default)]
    pub assignable: bool,
    /// SO 銷貨計畫下拉（2026-07-22 裁定）：僅列「自己是該計畫 SD」的計畫，
    /// 與 `authorize_sales_document` 對齊（admin 可開任何計畫，不受此過濾）。
    /// 預設 false（不過濾）。
    #[serde(default)]
    pub sd_only: bool,
}

/// 計畫書回應（含關聯資訊）
#[derive(Debug, Serialize, ToSchema)]
pub struct ProtocolResponse {
    pub protocol: Protocol,
    pub pi_name: Option<String>,
    pub pi_email: Option<String>,
    pub pi_organization: Option<String>,
    /// 計劃負責人（Study Director，公司內部）顯示名
    pub sd_name: Option<String>,
    /// 建立者 / 匯入者顯示名（外部 PI 匯入時 = 匯入者，與 PI 客人區分）
    pub created_by_name: Option<String>,
    pub status_display: String,
    pub vet_review: Option<VetReviewAssignment>,
    /// 當前 viewer 是否可編輯此計畫（admin / PI / SD / 補登管理者）。
    /// 由 `get_protocol` handler 以 `access::can_edit_protocol` 計算後覆寫，供前端按鈕 gating
    /// 對齊後端授權契約；service 預設 false。
    #[serde(default)]
    pub can_edit: bool,
    /// 當前 viewer 是否可建立 / 更新 / 提交此計畫的變更申請（admin / PI，不含 SD）。
    /// 由 `get_protocol` handler 以 `access::can_write_amendment` 計算後覆寫，與
    /// `Scoped<AmendmentWrite>` 同一權威判斷；service 預設 false。
    #[serde(default)]
    pub can_write_amendment: bool,
}

/// 計畫列表項目
#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct ProtocolListItem {
    pub id: Uuid,
    pub protocol_no: String,
    pub iacuc_no: Option<String>,
    pub title: String,
    pub status: ProtocolStatus,
    pub pi_user_id: Uuid,
    pub pi_name: String,
    pub pi_organization: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    #[sqlx(default)]
    pub apply_study_number: Option<String>,
    /// 匯入計劃標記（非 NULL = 補登匯入計劃）；供前端判斷可否刪除匯入計劃（admin only）
    #[sqlx(default)]
    pub imported_at: Option<DateTime<Utc>>,
    /// 當前 viewer 是否可編輯此計畫（PI / SD / admin）。供前端按鈕 gating，由列表查詢計算。
    #[sqlx(default)]
    pub can_edit: bool,
    /// 這件現在卡在誰手上（目前僅行政受理 / 預審那幾關有值；委員會審查與需修正
    /// 兩類的形狀不同，尚未涵蓋）。`skip` 而非 `default` 的理由同
    /// `DocumentListItem::pending_owner`：`PendingOwner` 不是 DB 型別、沒有 `Decode`。
    #[sqlx(skip)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_owner: Option<crate::models::PendingOwner>,
}

/// 審查意見回應（含審查者資訊）
#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct ReviewCommentResponse {
    pub id: Uuid,
    #[sqlx(default)]
    pub protocol_version_id: Option<Uuid>,
    /// 系統內審查者 FK；院外審查者（補登）為 NULL
    pub reviewer_id: Option<Uuid>,
    /// 顯示名稱：COALESCE(users.display_name, review_comments.reviewer_name)
    pub reviewer_name: String,
    /// 院外審查者無 email
    #[sqlx(default)]
    pub reviewer_email: Option<String>,
    pub content: String,
    /// 意見類型：`COMMENT`（一般意見，申請人需回覆）／`NO_OBJECTION`（無意見，不需回覆）。
    #[sqlx(default)]
    pub comment_type: String,
    /// 對應計畫書項次（如 4.1.2，補登審查文件填寫）
    #[sqlx(default)]
    pub section_no: Option<String>,
    pub is_resolved: bool,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub parent_comment_id: Option<Uuid>,
    #[sqlx(rename = "protocol_id")]
    pub protocol_id: Option<Uuid>,
    pub replied_by: Option<Uuid>,
    #[sqlx(default)]
    pub replied_by_name: Option<String>,
    #[sqlx(default)]
    pub replied_by_email: Option<String>,
    /// 草稿回覆內容（僅 PI/Coeditor 可見，審查委員不可見）
    #[sqlx(default)]
    pub draft_content: Option<String>,
    /// 草稿撰寫者
    #[sqlx(default)]
    pub drafted_by: Option<Uuid>,
    /// 草稿撰寫者姓名
    #[sqlx(default)]
    pub drafted_by_name: Option<String>,
    /// 草稿最後更新時間
    #[sqlx(default)]
    pub draft_updated_at: Option<DateTime<Utc>>,
    /// 審查階段（PRE_REVIEW, VET_REVIEW, UNDER_REVIEW）
    #[sqlx(default)]
    pub review_stage: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 審查指派回應（含審查者與指派者資訊）
#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct ReviewAssignmentResponse {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub reviewer_id: Uuid,
    pub reviewer_name: String,
    pub reviewer_email: String,
    pub assigned_by: Uuid,
    pub assigned_by_name: String,
    pub assigned_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    #[sqlx(default)]
    pub is_primary_reviewer: bool,
    #[sqlx(default)]
    pub review_stage: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_status_as_str() {
        assert_eq!(ProtocolStatus::Draft.as_str(), "DRAFT");
        assert_eq!(ProtocolStatus::Submitted.as_str(), "SUBMITTED");
        assert_eq!(ProtocolStatus::PreReview.as_str(), "PRE_REVIEW");
        assert_eq!(ProtocolStatus::VetReview.as_str(), "VET_REVIEW");
        assert_eq!(ProtocolStatus::UnderReview.as_str(), "UNDER_REVIEW");
        assert_eq!(ProtocolStatus::Approved.as_str(), "APPROVED");
        assert_eq!(
            ProtocolStatus::ApprovedWithConditions.as_str(),
            "APPROVED_WITH_CONDITIONS"
        );
        assert_eq!(ProtocolStatus::Rejected.as_str(), "REJECTED");
        assert_eq!(ProtocolStatus::Closed.as_str(), "CLOSED");
        assert_eq!(ProtocolStatus::Deleted.as_str(), "DELETED");
    }

    #[test]
    fn test_protocol_status_display_name() {
        assert_eq!(ProtocolStatus::Draft.display_name(), "草稿");
        assert_eq!(ProtocolStatus::Approved.display_name(), "已核准");
        assert_eq!(
            ProtocolStatus::ApprovedWithConditions.display_name(),
            "附條件核准"
        );
        assert_eq!(ProtocolStatus::UnderReview.display_name(), "審查中");
        assert_eq!(ProtocolStatus::Suspended.display_name(), "已暫停");
    }

    #[test]
    fn test_protocol_status_all_variants_have_as_str() {
        // 確認所有 16 個變體都有對應字串
        let variants = vec![
            ProtocolStatus::Draft,
            ProtocolStatus::Submitted,
            ProtocolStatus::PreReview,
            ProtocolStatus::PreReviewRevisionRequired,
            ProtocolStatus::VetReview,
            ProtocolStatus::VetRevisionRequired,
            ProtocolStatus::UnderReview,
            ProtocolStatus::RevisionRequired,
            ProtocolStatus::Resubmitted,
            ProtocolStatus::Approved,
            ProtocolStatus::ApprovedWithConditions,
            ProtocolStatus::Deferred,
            ProtocolStatus::Rejected,
            ProtocolStatus::Suspended,
            ProtocolStatus::Closed,
            ProtocolStatus::Deleted,
        ];
        for v in &variants {
            assert!(!v.as_str().is_empty());
            assert!(!v.display_name().is_empty());
        }
        assert_eq!(variants.len(), 16);
    }

    // CSO-r2 #2: 終態 / 已核准叢集 egress 鎖回歸測試。
    #[test]
    fn test_can_change_status_to_blocks_approved_revert() {
        use ProtocolStatus::*;
        // 已核准計畫不可被退回 review pipeline、否決、延後或刪除
        for bad in [
            Draft,
            Submitted,
            PreReview,
            VetReview,
            UnderReview,
            RevisionRequired,
            Resubmitted,
            Rejected,
            Deferred,
            Deleted,
            Approved, // 自我轉移亦不允許
        ] {
            assert!(
                !Approved.can_change_status_to(bad),
                "APPROVED 不應可變更為 {:?}",
                bad
            );
        }
        // 已核准計畫合法生命週期：結案 / 暫停
        assert!(Approved.can_change_status_to(Closed));
        assert!(Approved.can_change_status_to(Suspended));
    }

    #[test]
    fn test_can_change_status_to_terminal_states_locked() {
        use ProtocolStatus::*;
        // 終態不可離開（任何目標皆拒絕）
        for from in [Rejected, Closed, Deleted] {
            assert!(from.is_terminal());
            for to in [Draft, Submitted, UnderReview, Approved, Suspended] {
                assert!(
                    !from.can_change_status_to(to),
                    "終態 {:?} 不應可變更為 {:?}",
                    from,
                    to
                );
            }
        }
    }

    #[test]
    fn test_can_change_status_to_review_pipeline_unrestricted() {
        use ProtocolStatus::*;
        // 非終態 / 非已核准的審查工作流維持既有行為（由 change_status_tx 既有 entry guard 控制）
        assert!(Draft.can_change_status_to(Submitted));
        assert!(Submitted.can_change_status_to(UnderReview));
        assert!(UnderReview.can_change_status_to(Approved));
        assert!(RevisionRequired.can_change_status_to(Resubmitted));
        // 附條件核准可達成轉正 / 結案 / 暫停，但不可退回審查
        assert!(ApprovedWithConditions.can_change_status_to(Approved));
        assert!(ApprovedWithConditions.can_change_status_to(Closed));
        assert!(ApprovedWithConditions.can_change_status_to(Suspended));
        assert!(!ApprovedWithConditions.can_change_status_to(UnderReview));
        // 暫停可復原（轉回 Approved / ApprovedWithConditions）或結案
        assert!(Suspended.can_change_status_to(Approved));
        assert!(Suspended.can_change_status_to(ApprovedWithConditions));
        assert!(Suspended.can_change_status_to(Closed));
        assert!(!Suspended.can_change_status_to(Draft));
    }

    // gemini review：非終態 / 非已核准狀態不得直接跳至暫停 / 結案（CSO-r2 #2 收緊）。
    #[test]
    fn test_can_change_status_to_blocks_jump_to_suspend_or_close() {
        use ProtocolStatus::*;
        for from in [
            Draft,
            Submitted,
            PreReview,
            VetReview,
            UnderReview,
            RevisionRequired,
            Resubmitted,
            Deferred,
        ] {
            assert!(
                !from.can_change_status_to(Suspended),
                "{from:?} 不應可直接變更為 SUSPENDED"
            );
            assert!(
                !from.can_change_status_to(Closed),
                "{from:?} 不應可直接變更為 CLOSED"
            );
        }
    }

    #[test]
    fn test_activity_type_as_str() {
        assert_eq!(ProtocolActivityType::Created.as_str(), "CREATED");
        assert_eq!(ProtocolActivityType::Submitted.as_str(), "SUBMITTED");
        assert_eq!(
            ProtocolActivityType::ReviewerAssigned.as_str(),
            "REVIEWER_ASSIGNED"
        );
        assert_eq!(
            ProtocolActivityType::AnimalAssigned.as_str(),
            "ANIMAL_ASSIGNED"
        );
        assert_eq!(
            ProtocolActivityType::AmendmentCreated.as_str(),
            "AMENDMENT_CREATED"
        );
    }

    #[test]
    fn test_activity_type_display_name() {
        assert_eq!(ProtocolActivityType::Created.display_name(), "創建草稿");
        assert_eq!(ProtocolActivityType::Submitted.display_name(), "送審");
        assert_eq!(
            ProtocolActivityType::ReviewerAssigned.display_name(),
            "指派審查委員"
        );
        assert_eq!(
            ProtocolActivityType::AnimalAssigned.display_name(),
            "分配動物"
        );
    }

    #[test]
    fn test_activity_type_all_variants() {
        // 確認所有 28 個變體都有對應字串
        let variants = vec![
            ProtocolActivityType::Created,
            ProtocolActivityType::Updated,
            ProtocolActivityType::Submitted,
            ProtocolActivityType::Resubmitted,
            ProtocolActivityType::Approved,
            ProtocolActivityType::ApprovedWithConditions,
            ProtocolActivityType::Closed,
            ProtocolActivityType::Rejected,
            ProtocolActivityType::Suspended,
            ProtocolActivityType::Deleted,
            ProtocolActivityType::StatusChanged,
            ProtocolActivityType::ReviewerAssigned,
            ProtocolActivityType::VetAssigned,
            ProtocolActivityType::SdAssigned,
            ProtocolActivityType::CoeditorAssigned,
            ProtocolActivityType::CoeditorRemoved,
            ProtocolActivityType::CommentAdded,
            ProtocolActivityType::CommentReplied,
            ProtocolActivityType::CommentResolved,
            ProtocolActivityType::AttachmentUploaded,
            ProtocolActivityType::AttachmentDeleted,
            ProtocolActivityType::VersionCreated,
            ProtocolActivityType::VersionRecovered,
            ProtocolActivityType::AmendmentCreated,
            ProtocolActivityType::AmendmentSubmitted,
            ProtocolActivityType::AnimalAssigned,
            ProtocolActivityType::AnimalUnassigned,
            ProtocolActivityType::McpRead,
        ];
        for v in &variants {
            assert!(!v.as_str().is_empty());
            assert!(!v.display_name().is_empty());
        }
        // 28：新增 SdAssigned（裁定 21）。加 variant 時這個數字要一起改——
        // `as_str()` / `display_name()` 沒有 `_ =>`，編譯器會逼你處理那兩處，
        // 但**不會**逼你把新 variant 加進上面的清單，這行是那個缺口的提醒。
        assert_eq!(variants.len(), 28);
    }

    #[test]
    fn test_protocol_activity_response_from() {
        use chrono::Utc;
        let activity = ProtocolActivity {
            id: Uuid::new_v4(),
            protocol_id: Uuid::new_v4(),
            activity_type: ProtocolActivityType::Created,
            actor_id: Uuid::new_v4(),
            actor_name: Some("測試用戶".to_string()),
            actor_email: Some("test@example.com".to_string()),
            from_value: None,
            to_value: Some("DRAFT".to_string()),
            target_entity_type: None,
            target_entity_id: None,
            target_entity_name: None,
            remark: Some("建立計畫".to_string()),
            extra_data: None,
            created_at: Utc::now(),
        };
        let resp = ProtocolActivityResponse::from(activity);
        assert_eq!(resp.activity_type_display, "創建草稿");
        assert_eq!(resp.actor_name, "測試用戶");
    }

    #[test]
    fn test_protocol_status_serde_roundtrip() {
        let status = ProtocolStatus::ApprovedWithConditions;
        let json = serde_json::to_string(&status).expect("序列化 ProtocolStatus 失敗");
        assert_eq!(json, "\"APPROVED_WITH_CONDITIONS\"");
        let parsed: ProtocolStatus =
            serde_json::from_str(&json).expect("反序列化 ProtocolStatus 失敗");
        assert_eq!(parsed, status);
    }
}
