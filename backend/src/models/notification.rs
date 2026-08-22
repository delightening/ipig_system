use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// 通知類型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "notification_type", rename_all = "snake_case")]
pub enum NotificationType {
    LowStock,
    ExpiryWarning,
    DocumentApproval,
    ProtocolStatus,
    ProtocolSubmitted,
    ReviewAssignment,
    ReviewComment,
    LeaveApproval,
    OvertimeApproval,
    VetRecommendation,
    SystemAlert,
    MonthlyReport,
}

impl NotificationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationType::LowStock => "low_stock",
            NotificationType::ExpiryWarning => "expiry_warning",
            NotificationType::DocumentApproval => "document_approval",
            NotificationType::ProtocolStatus => "protocol_status",
            NotificationType::ProtocolSubmitted => "protocol_submitted",
            NotificationType::ReviewAssignment => "review_assignment",
            NotificationType::ReviewComment => "review_comment",
            NotificationType::LeaveApproval => "leave_approval",
            NotificationType::OvertimeApproval => "overtime_approval",
            NotificationType::VetRecommendation => "vet_recommendation",
            NotificationType::SystemAlert => "system_alert",
            NotificationType::MonthlyReport => "monthly_report",
        }
    }
}

/// 通知
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub r#type: String,
    pub title: String,
    pub content: Option<String>,
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    /// 0=一般；1=緊急置頂（待辦，完成前排在清單最上方）
    pub priority: i16,
    /// `'info'` | `'action'`，見 [`KIND_INFO`] / [`KIND_ACTION`]
    pub kind: String,
}

/// 通知優先級：一般（隨時間新到舊排序）。
pub const PRIORITY_NORMAL: i16 = 0;
/// 通知優先級：緊急置頂（待辦，完成前恆排在清單最上方；完成後由 hook 降級回 [`PRIORITY_NORMAL`]）。
pub const PRIORITY_PINNED: i16 = 1;

/// 通知性質：一般告知，只需看過（鈴鐺入口）。
pub const KIND_INFO: &str = "info";
/// 通知性質：**待辦** —— 指派給本人、需其做出動作、且系統能判定完成（驚嘆號入口）。
pub const KIND_ACTION: &str = "action";

/// 置頂待辦的收件人身分：這則通知**因為收件人扮演哪個角色**而建立，與「收件人是誰」
/// （會隨組織兼職而重疊）分開記錄。目前僅請假置頂待辦使用——「代理確認」與「核准」
/// 可能落到同一人身上（例如代理人剛好是申請人的單位主管），若只憑收件人判斷用途，
/// reconcile 對帳會分不出這則 pin 是哪一種、可能誤清真正待處理的核准 pin（見 R92-1）。
pub const RECIPIENT_ROLE_PROXY: &str = "proxy";
/// 置頂待辦的收件人身分：核准人。見 [`RECIPIENT_ROLE_PROXY`]。
pub const RECIPIENT_ROLE_APPROVER: &str = "approver";

/// `kind` 與 `priority` 的分工（兩者都要，缺一不可）：
///
/// - `kind = 'action'`：這則通知的**性質**是待辦。歷史事實，一旦是就永遠是。
/// - `priority > 0`：這則待辦**還沒完成**。完成後降回 0。
///
/// 「待處理」清單 ＝ `kind='action' AND priority>0`；完成後該列離開待處理清單，
/// 但仍以一般通知留在鈴鐺歷史裡，使用者可回顧「我當初處理過哪些事」。
///
/// 只用 `priority` 的話做不到最後這件事 —— 降級後就與一般通知無從區分。
pub const fn _kind_priority_doc() {}

/// 通知列表項目（含額外資訊）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NotificationItem {
    pub id: Uuid,
    pub r#type: String,
    pub title: String,
    pub content: Option<String>,
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    /// 0=一般；1=緊急置頂（待辦，完成前排在清單最上方）
    pub priority: i16,
    /// `'info'` | `'action'`，見 [`KIND_INFO`] / [`KIND_ACTION`]
    pub kind: String,
}

/// 通知設定
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct NotificationSettings {
    pub user_id: Uuid,
    pub email_low_stock: bool,
    pub email_expiry_warning: bool,
    pub email_document_approval: bool,
    pub email_protocol_status: bool,
    pub email_monthly_report: bool,
    pub expiry_warning_days: i32,
    pub low_stock_notify_immediately: bool,
    pub updated_at: DateTime<Utc>,
}

/// 更新通知設定請求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateNotificationSettingsRequest {
    pub email_low_stock: Option<bool>,
    pub email_expiry_warning: Option<bool>,
    pub email_document_approval: Option<bool>,
    pub email_protocol_status: Option<bool>,
    pub email_monthly_report: Option<bool>,
    #[validate(range(min = 1, max = 90, message = "Expiry warning days must be 1-90"))]
    pub expiry_warning_days: Option<i32>,
    pub low_stock_notify_immediately: Option<bool>,
}

/// 建立通知請求（內部使用）
#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    pub content: Option<String>,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<Uuid>,
}

/// 通知查詢參數
#[derive(Debug, Deserialize)]
pub struct NotificationQuery {
    pub is_read: Option<bool>,
    pub notification_type: Option<String>,
    /// 由哪個入口發問：[`ENTRY_BELL`] | [`ENTRY_TODO`]；未帶＝不篩選（維持舊前端相容）。
    pub entry: Option<String>,
}

/// 鈴鐺入口：一切**不是未完成待辦**的通知。
///
/// 注意不等於 `kind='info'` —— 已完成的待辦（`kind='action'` 但 `priority=0`）
/// 也屬於鈴鐺，那正是「我當初處理過哪些事」的歷史。
pub const ENTRY_BELL: &str = "bell";
/// 驚嘆號入口：未完成待辦（`kind='action' AND priority>0`）。
pub const ENTRY_TODO: &str = "todo";

/// 未讀通知數量
#[derive(Debug, Serialize)]
pub struct UnreadNotificationCount {
    pub count: i64,
}

/// 標記已讀請求
#[derive(Debug, Deserialize)]
pub struct MarkNotificationsReadRequest {
    pub notification_ids: Vec<Uuid>,
}

// LowStockAlert is defined in models/stock.rs to avoid duplication

/// 效期預警項目
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExpiryAlert {
    pub product_id: Uuid,
    pub sku: String,
    pub product_name: String,
    pub spec: Option<String>,
    pub category_code: Option<String>,
    pub warehouse_id: Uuid,
    pub warehouse_code: String,
    pub warehouse_name: String,
    pub batch_no: Option<String>,
    pub expiry_date: chrono::NaiveDate,
    pub on_hand_qty: rust_decimal::Decimal,
    pub base_uom: String,
    pub days_until_expiry: i32,
    pub expiry_status: String,
    /// 同品項同倉庫所有批號的合計庫存量
    pub total_qty: rust_decimal::Decimal,
}

// ============================================
// 定期報表相關
// ============================================

/// 排程類型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "schedule_type", rename_all = "lowercase")]
pub enum ScheduleType {
    Daily,
    Weekly,
    Monthly,
}

/// 報表類型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "report_type", rename_all = "snake_case")]
pub enum ReportType {
    StockOnHand,
    StockLedger,
    PurchaseSummary,
    CostSummary,
    ExpiryReport,
    LowStockReport,
}

/// 定期報表設定
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScheduledReport {
    pub id: Uuid,
    pub report_type: String,
    pub schedule_type: String,
    pub day_of_week: Option<i32>,
    pub day_of_month: Option<i32>,
    pub hour_of_day: i32,
    pub parameters: Option<serde_json::Value>,
    pub recipients: Vec<Uuid>,
    pub is_active: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 報表歷史記錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportHistory {
    pub id: Uuid,
    pub scheduled_report_id: Option<Uuid>,
    pub report_type: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: Option<i32>,
    pub parameters: Option<serde_json::Value>,
    pub generated_at: DateTime<Utc>,
    pub generated_by: Option<Uuid>,
}

/// 建立定期報表請求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateScheduledReportRequest {
    pub report_type: String,
    pub schedule_type: String,
    #[validate(range(min = 0, max = 6, message = "Day of week must be 0-6"))]
    pub day_of_week: Option<i32>,
    #[validate(range(min = 1, max = 31, message = "Day of month must be 1-31"))]
    pub day_of_month: Option<i32>,
    #[validate(range(min = 0, max = 23, message = "Hour must be 0-23"))]
    pub hour_of_day: i32,
    pub parameters: Option<serde_json::Value>,
    #[validate(length(min = 1, message = "At least one recipient required"))]
    pub recipients: Vec<Uuid>,
}

/// 更新定期報表請求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateScheduledReportRequest {
    pub day_of_week: Option<i32>,
    pub day_of_month: Option<i32>,
    pub hour_of_day: Option<i32>,
    pub parameters: Option<serde_json::Value>,
    pub recipients: Option<Vec<Uuid>>,
    pub is_active: Option<bool>,
}

// ============================================
// 通知路由規則
// ============================================

/// 通知路由規則（事件→收件人來源→通道的對應）
///
/// 收件人來源由 `target_kind` 決定：
/// - `role`：`target_value` 為 `roles.code`，收件人＝持有該角色者（`role_code` 同步保留供舊路徑）。
/// - `resolver`：`target_value` 為關係型解析器 key（見 `services/notification/resolvers.rs`），
///   收件人由觸發事件的實體關係算出（例：這張假單的申請人、這個計畫的 PI）。`role_code` 為 NULL。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NotificationRouting {
    pub id: Uuid,
    pub event_type: String,
    /// 角色型列為角色代碼；resolver 型列為 NULL（過渡期保留供 get_recipients_by_event 等舊路徑）
    pub role_code: Option<String>,
    pub channel: String,
    pub is_active: bool,
    pub description: Option<String>,
    /// 批次通知頻率：immediate | daily | weekly | monthly
    pub frequency: String,
    /// 批次通知執行小時（0-23）
    pub hour_of_day: i16,
    /// weekly 時有效：0=週日, 1=週一 ... 6=週六
    pub day_of_week: Option<i16>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// 收件人來源種類：`role` | `resolver`
    pub target_kind: String,
    /// `role` 時為 roles.code；`resolver` 時為 resolver key
    pub target_value: String,
}

/// 建立通知路由規則請求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateNotificationRoutingRequest {
    #[validate(length(min = 1, max = 80, message = "event_type 不可為空且最多 80 字元"))]
    pub event_type: String,
    #[validate(length(min = 1, max = 50, message = "role_code 不可為空且最多 50 字元"))]
    pub role_code: String,
    pub channel: Option<String>,
    pub description: Option<String>,
    pub frequency: Option<String>,
    #[validate(range(min = 0, max = 23, message = "hour_of_day 必須介於 0-23"))]
    pub hour_of_day: Option<i16>,
    #[validate(range(min = 0, max = 6, message = "day_of_week 必須介於 0-6"))]
    pub day_of_week: Option<i16>,
}

/// 更新通知路由規則請求
#[derive(Debug, Deserialize)]
pub struct UpdateNotificationRoutingRequest {
    pub channel: Option<String>,
    pub is_active: Option<bool>,
    pub description: Option<String>,
    pub frequency: Option<String>,
    pub hour_of_day: Option<i16>,
    pub day_of_week: Option<i16>,
}

// ============================================
// 效期通知範圍設定
// ============================================

/// 系統層級效期通知範圍設定（全系統單一列）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ExpiryNotificationConfig {
    pub id: Uuid,
    /// 提前幾天開始預警（預設 60）
    pub warn_days: i16,
    /// 過期超過幾天後停止通知（預設 90）
    pub cutoff_days: i16,
    /// 過期超過此天數後轉月度彙整通知；None=停用
    pub monthly_threshold_days: Option<i16>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

/// 更新效期通知範圍設定請求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateExpiryNotificationConfigRequest {
    #[validate(range(min = 1, max = 365, message = "warn_days 必須介於 1-365"))]
    pub warn_days: Option<i16>,
    #[validate(range(min = 1, max = 730, message = "cutoff_days 必須介於 1-730"))]
    pub cutoff_days: Option<i16>,
    /// None = 停用月度模式
    pub monthly_threshold_days: Option<i16>,
}

/// 事件類型分類（含主要分組、子分類名稱與事件清單）
/// group: AUP | Animal | ERP | HR，供前端分頁/分區顯示
#[derive(Debug, Clone, Serialize)]
pub struct EventTypeCategory {
    /// 主要分組：AUP | Animal | ERP | HR
    pub group: String,
    /// 子分類顯示名稱
    pub category: String,
    pub event_types: Vec<EventTypeInfo>,
}

/// 事件類型資訊
#[derive(Debug, Clone, Serialize)]
pub struct EventTypeInfo {
    pub code: String,
    pub name: String,
}

/// 角色資訊（供通知路由下拉選單使用）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RoleInfo {
    pub code: String,
    pub name: String,
}

/// 某事件的「具體收件人」預覽（供 admin 路由頁巢狀收合顯示「實際是誰」）。
#[derive(Debug, Clone, Serialize)]
pub struct RoutingRecipientsPreview {
    pub event_type: String,
    pub rules: Vec<RoutingRuleRecipients>,
}

/// 單一路由規則解析後的收件人資訊。
#[derive(Debug, Clone, Serialize)]
pub struct RoutingRuleRecipients {
    pub rule_id: Uuid,
    pub target_kind: String,
    pub target_value: String,
    pub channel: String,
    pub is_active: bool,
    /// 人類可讀標籤（角色中文名 or resolver 標籤）。
    pub label: String,
    /// resolver 型：對象由事件動態決定的說明；角色型為 None。
    pub description: Option<String>,
    /// 是否「固定收件人、不可調整」（resolver 型為 true）。
    pub fixed: bool,
    /// 角色型：實際持有該角色的使用者；resolver 型為空（依事件動態決定，無法靜態列舉）。
    pub members: Vec<RecipientMember>,
}

/// 收件人成員（角色型規則展開用）。
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct RecipientMember {
    pub id: Uuid,
    pub display_name: String,
    pub email: String,
}

/// 「固定通知」目錄項：收件人由系統流程決定、不經 notification_routing、不可調整。
/// 供 admin 路由頁顯示為唯讀 notes，讓管理者看見「有什麼狀況會通知誰」。
#[derive(Debug, Clone, Serialize)]
pub struct FixedNotification {
    /// 中文事件名稱。
    pub event_label: String,
    /// 觸發情境。
    pub trigger: String,
    /// 收件人（描述，由流程動態決定）。
    pub recipient_label: String,
    /// 通知管道：'in_app' | 'email' | 'both'。
    pub channel: String,
    /// 分組（對齊路由頁分組，如 Animal）。
    pub group: String,
    /// 為何固定 / 補充說明。
    pub note: String,
}
