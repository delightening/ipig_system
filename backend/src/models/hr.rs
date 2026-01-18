// HR 模組 Models
// 包含：Attendance, Overtime, Leave, Balances

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================
// Attendance (出勤)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AttendanceRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub work_date: NaiveDate,
    pub clock_in_time: Option<DateTime<Utc>>,
    pub clock_out_time: Option<DateTime<Utc>>,
    pub regular_hours: Option<Decimal>,
    pub overtime_hours: Option<Decimal>,
    pub status: String,
    pub clock_in_source: Option<String>,
    pub clock_in_ip: Option<String>,
    pub clock_out_source: Option<String>,
    pub clock_out_ip: Option<String>,
    pub remark: Option<String>,
    pub is_corrected: bool,
    pub corrected_by: Option<Uuid>,
    pub corrected_at: Option<DateTime<Utc>>,
    pub correction_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AttendanceWithUser {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub user_name: String,
    pub work_date: NaiveDate,
    pub clock_in_time: Option<DateTime<Utc>>,
    pub clock_out_time: Option<DateTime<Utc>>,
    pub regular_hours: Option<Decimal>,
    pub overtime_hours: Option<Decimal>,
    pub status: String,
    pub remark: Option<String>,
    pub is_corrected: bool,
}

#[derive(Debug, Deserialize)]
pub struct AttendanceQuery {
    pub user_id: Option<Uuid>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ClockInRequest {
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClockOutRequest {
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AttendanceCorrectionRequest {
    pub clock_in_time: Option<DateTime<Utc>>,
    pub clock_out_time: Option<DateTime<Utc>>,
    pub reason: String,
}

// ============================================
// Overtime (加班)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OvertimeRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub attendance_id: Option<Uuid>,
    pub overtime_date: NaiveDate,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub hours: Decimal,
    pub overtime_type: String,
    pub multiplier: Decimal,
    pub comp_time_hours: Decimal,
    pub comp_time_expires_at: NaiveDate,
    pub comp_time_used_hours: Decimal,
    pub status: String,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_by: Option<Uuid>,
    pub rejected_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct OvertimeWithUser {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub user_name: String,
    pub overtime_date: NaiveDate,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub hours: Decimal,
    pub overtime_type: String,
    pub multiplier: Decimal,
    pub comp_time_hours: Decimal,
    pub comp_time_expires_at: NaiveDate,
    pub status: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct OvertimeQuery {
    pub user_id: Option<Uuid>,
    pub status: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub pending_approval: Option<bool>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOvertimeRequest {
    pub overtime_date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub overtime_type: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOvertimeRequest {
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub overtime_type: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RejectOvertimeRequest {
    pub reason: String,
}

// ============================================
// Leave (請假)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "leave_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LeaveType {
    Annual,
    Personal,
    Sick,
    Compensatory,
    Marriage,
    Bereavement,
    Maternity,
    Paternity,
    Menstrual,
    Official,
    Unpaid,
}

impl LeaveType {
    pub fn display_name(&self) -> &'static str {
        match self {
            LeaveType::Annual => "特休假",
            LeaveType::Personal => "事假",
            LeaveType::Sick => "病假",
            LeaveType::Compensatory => "補休假",
            LeaveType::Marriage => "婚假",
            LeaveType::Bereavement => "喪假",
            LeaveType::Maternity => "產假",
            LeaveType::Paternity => "陪產假",
            LeaveType::Menstrual => "生理假",
            LeaveType::Official => "公假",
            LeaveType::Unpaid => "無薪假",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "leave_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LeaveStatus {
    Draft,
    PendingL1,
    PendingL2,
    PendingHr,
    PendingGm,
    Approved,
    Rejected,
    Cancelled,
    Revoked,
}

impl LeaveStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            LeaveStatus::Draft => "草稿",
            LeaveStatus::PendingL1 => "待一級審核",
            LeaveStatus::PendingL2 => "待二級審核",
            LeaveStatus::PendingHr => "待行政審核",
            LeaveStatus::PendingGm => "待總經理核准",
            LeaveStatus::Approved => "已核准",
            LeaveStatus::Rejected => "已駁回",
            LeaveStatus::Cancelled => "已取消",
            LeaveStatus::Revoked => "已銷假",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LeaveRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub proxy_user_id: Option<Uuid>,  // 代理人
    pub leave_type: String, // 用 String 避免 sqlx enum 問題
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub total_days: Decimal,
    pub total_hours: Option<Decimal>,
    pub reason: String,
    pub supporting_documents: Option<serde_json::Value>,
    pub comp_time_source_ids: Option<Vec<Uuid>>,
    pub annual_leave_source_id: Option<Uuid>,
    pub is_urgent: bool,
    pub is_retroactive: bool,
    pub status: String,
    pub current_approver_id: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub revocation_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct LeaveRequestWithUser {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub user_name: String,
    pub proxy_user_id: Option<Uuid>,
    pub proxy_user_name: Option<String>,
    pub leave_type: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub total_days: Decimal,
    pub reason: String,
    pub is_urgent: bool,
    pub is_retroactive: bool,
    pub status: String,
    pub current_approver_id: Option<Uuid>,
    pub current_approver_name: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LeaveQuery {
    pub user_id: Option<Uuid>,
    pub status: Option<String>,
    pub leave_type: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub pending_approval: Option<bool>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeaveRequest {
    pub leave_type: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub total_days: f64,
    pub total_hours: Option<f64>,
    pub reason: Option<String>,  // 特休假不用填理由
    pub supporting_documents: Option<Vec<String>>,  // 附件圖片 URLs
    pub is_urgent: Option<bool>,
    pub is_retroactive: Option<bool>,
    pub proxy_user_id: Option<Uuid>,  // 代理人
}

#[derive(Debug, Deserialize)]
pub struct UpdateLeaveRequest {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub total_days: Option<f64>,
    pub total_hours: Option<f64>,
    pub reason: Option<String>,
    pub proxy_user_id: Option<Uuid>,  // 代理人
}

#[derive(Debug, Deserialize)]
pub struct ApproveLeaveRequest {
    pub comments: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RejectLeaveRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct CancelLeaveRequest {
    pub reason: Option<String>,
}

// ============================================
// Leave Approvals (審核記錄)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LeaveApproval {
    pub id: Uuid,
    pub leave_request_id: Uuid,
    pub approver_id: Uuid,
    pub approval_level: String,
    pub action: String,
    pub comments: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================
// Balances (餘額)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnnualLeaveEntitlement {
    pub id: Uuid,
    pub user_id: Uuid,
    pub entitlement_year: i32,
    pub entitled_days: Decimal,
    pub used_days: Decimal,
    pub expires_at: NaiveDate,
    pub calculation_basis: Option<String>,
    pub seniority_years: Option<Decimal>,
    pub is_expired: bool,
    pub expired_days: Decimal,
    pub expiry_processed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CompTimeBalance {
    pub id: Uuid,
    pub user_id: Uuid,
    pub overtime_record_id: Uuid,
    pub original_hours: Decimal,
    pub used_hours: Decimal,
    pub earned_date: NaiveDate,
    pub expires_at: NaiveDate,
    pub is_expired: bool,
    pub expired_hours: Decimal,
    pub converted_to_pay: bool,
    pub expiry_processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AnnualLeaveBalanceView {
    pub entitlement_year: i32,
    pub entitled_days: f64,
    pub used_days: f64,
    pub remaining_days: f64,
    pub expires_at: NaiveDate,
    pub days_until_expiry: i32,
    pub is_expired: bool,  // 是否已過期（待補償）
}

#[derive(Debug, Serialize)]
pub struct CompTimeBalanceView {
    pub id: Uuid,
    pub earned_date: NaiveDate,
    pub original_hours: f64,
    pub used_hours: f64,
    pub remaining_hours: f64,
    pub expires_at: NaiveDate,
    pub days_until_expiry: i32,
}

#[derive(Debug, Serialize)]
pub struct BalanceSummary {
    pub user_id: Uuid,
    pub user_name: String,
    pub annual_leave_total: f64,
    pub annual_leave_used: f64,
    pub annual_leave_remaining: f64,
    pub comp_time_total: f64,
    pub comp_time_used: f64,
    pub comp_time_remaining: f64,
    pub expiring_soon_days: f64,
    pub expiring_soon_hours: f64,
}

#[derive(Debug, Deserialize)]
pub struct BalanceQuery {
    pub user_id: Option<Uuid>,
    pub year: Option<i32>,
    pub include_expired: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAnnualLeaveRequest {
    pub user_id: Uuid,
    pub entitlement_year: i32,
    pub entitled_days: f64,
    pub hire_date: Option<NaiveDate>,  // 到職日，用於計算到期日（到職週年日 + 2年）
    pub calculation_basis: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdjustBalanceRequest {
    pub adjustment_days: f64,
    pub reason: String,
}

/// 過期特休假報表（待補償）
#[derive(Debug, Serialize)]
pub struct ExpiredLeaveReport {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub entitlement_year: i32,
    pub entitled_days: f64,
    pub used_days: f64,
    pub remaining_days: f64,  // 待補償天數
    pub expires_at: NaiveDate,
}

// ============================================
// Dashboard Calendar (儀表板日曆)
// ============================================

#[derive(Debug, Clone, Serialize)]
pub struct TodayLeaveInfo {
    pub user_id: Uuid,
    pub user_name: String,
    pub leave_type: String,
    pub leave_type_display: String,
    pub is_all_day: bool,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Debug, Serialize)]
pub struct DashboardCalendarData {
    pub today: NaiveDate,
    pub today_leaves: Vec<TodayLeaveInfo>,
    pub today_events: Vec<crate::models::calendar::CalendarEvent>,
    pub upcoming_leaves: Vec<TodayLeaveInfo>,
}
