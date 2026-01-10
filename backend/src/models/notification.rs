use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
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
}

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
}

/// 通知設定
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
}

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
