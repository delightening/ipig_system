// HR 模組 Models
// 包含：Attendance, Overtime, Leave, Balances

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

// ============================================
// Attendance (出勤)
// ============================================

// 出勤紀錄。欄位含 IP / GPS 座標屬稽核必要項目（查對打卡地點是否合理），
// 非個資敏感類型，空 impl 即可。correction_reason 為管理員填入的更正理由，
// 同樣屬稽核項目。
impl crate::models::audit_diff::AuditRedact for AttendanceRecord {}

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
    /// GPS 定位座標
    pub clock_in_latitude: Option<f64>,
    pub clock_in_longitude: Option<f64>,
    pub clock_out_latitude: Option<f64>,
    pub clock_out_longitude: Option<f64>,
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

#[derive(Debug, Clone, Deserialize)]
pub struct AttendanceQuery {
    pub user_id: Option<Uuid>,
    /// 是否查看所有人的出勤（需 hr.attendance.view_all 權限）
    pub view_all: Option<bool>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ClockInRequest {
    pub source: Option<String>,
    /// GPS 緯度
    pub latitude: Option<f64>,
    /// GPS 經度
    pub longitude: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ClockOutRequest {
    pub source: Option<String>,
    /// GPS 緯度
    pub latitude: Option<f64>,
    /// GPS 經度
    pub longitude: Option<f64>,
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

// 加班紀錄。欄位皆稽核項目（時數、乘數、補休時數、到期日、審核狀態），
// 無敏感資料。rejection_reason / reason 為使用者填入的理由。
impl crate::models::audit_diff::AuditRedact for OvertimeRecord {}

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
    /// 計費單位：hour=平日按時數分段(A)；day=值班按天(B/C/D)
    pub calc_unit: String,
    /// 平日加班前 2 小時時數（×1.33）
    pub tier1_hours: Decimal,
    /// 平日加班超過 2 小時時數（×1.66）
    pub tier2_hours: Decimal,
    /// 加權係數時數 = tier1×1.33 + tier2×1.66（供薪資模組換算加班費）
    pub weighted_hours: Decimal,
    /// 值班天數（calc_unit=day 時使用，B/C/D）
    pub day_count: Decimal,
    pub status: String,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_by: Option<Uuid>,
    pub rejected_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    /// R86-2：作廢已核准加班單的管理者（ADMIN 單簽）
    pub voided_by: Option<Uuid>,
    pub voided_at: Option<DateTime<Utc>>,
    /// R86-2：作廢理由（必填，一併進稽核鏈）
    pub void_reason: Option<String>,
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
    /// R86-2：作廢理由（status='voided' 時才有值）
    pub void_reason: Option<String>,
    /// R72-2：當前使用者是否可核准此列（依狀態 + 角色於 service 計算，非 DB 欄位）
    #[sqlx(default)]
    pub can_approve: bool,
}

#[derive(Debug, Deserialize)]
pub struct OvertimeQuery {
    pub user_id: Option<Uuid>,
    pub status: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub pending_approval: Option<bool>,
    pub view_all: Option<bool>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
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

/// R86-2：作廢已核准加班單。理由必填（service 層再 trim 檢查一次）。
#[derive(Debug, Deserialize)]
pub struct VoidOvertimeRequest {
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
        }
    }

    /// 由 DB 文字（SCREAMING_SNAKE_CASE，如 "ANNUAL"）解析；未知值回 None。
    pub fn from_db_str(s: &str) -> Option<Self> {
        Some(match s {
            "ANNUAL" => Self::Annual,
            "PERSONAL" => Self::Personal,
            "SICK" => Self::Sick,
            "COMPENSATORY" => Self::Compensatory,
            "MARRIAGE" => Self::Marriage,
            "BEREAVEMENT" => Self::Bereavement,
            "MATERNITY" => Self::Maternity,
            "PATERNITY" => Self::Paternity,
            "MENSTRUAL" => Self::Menstrual,
            "OFFICIAL" => Self::Official,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "leave_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LeaveStatus {
    Draft,
    PendingProxy,
    PendingL1,
    PendingL2,
    PendingHr,
    PendingGm,
    PendingDirector,
    Approved,
    Rejected,
    Cancelled,
    Revoked,
}

impl LeaveStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            LeaveStatus::Draft => "DRAFT",
            LeaveStatus::PendingProxy => "PENDING_PROXY",
            LeaveStatus::PendingL1 => "PENDING_L1",
            LeaveStatus::PendingL2 => "PENDING_L2",
            LeaveStatus::PendingHr => "PENDING_HR",
            LeaveStatus::PendingGm => "PENDING_GM",
            LeaveStatus::PendingDirector => "PENDING_DIRECTOR",
            LeaveStatus::Approved => "APPROVED",
            LeaveStatus::Rejected => "REJECTED",
            LeaveStatus::Cancelled => "CANCELLED",
            LeaveStatus::Revoked => "REVOKED",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            LeaveStatus::Draft => "草稿",
            LeaveStatus::PendingProxy => "待代理確認",
            LeaveStatus::PendingL1 => "待單位主管審核",
            LeaveStatus::PendingL2 => "待二級審核",
            LeaveStatus::PendingHr => "待行政審核",
            LeaveStatus::PendingGm => "待總經理核准",
            LeaveStatus::PendingDirector => "待負責人簽核",
            LeaveStatus::Approved => "已核准",
            LeaveStatus::Rejected => "已駁回",
            LeaveStatus::Cancelled => "已取消",
            LeaveStatus::Revoked => "已銷假",
        }
    }
}

// 無敏感欄位（reason / cancellation_reason / revocation_reason 皆為使用者
// 主動填入的文字，屬於稽核應保留的內容）
impl crate::models::audit_diff::AuditRedact for LeaveRequest {}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LeaveRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub proxy_user_id: Option<Uuid>, // 代理人
    pub leave_type: String,          // 用 String 避免 sqlx enum 問題
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub total_days: Decimal,
    pub total_hours: Option<Decimal>,
    pub reason: String,
    pub supporting_documents: Option<serde_json::Value>,
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
    pub total_hours: Option<Decimal>,
    pub reason: String,
    pub is_urgent: bool,
    pub is_retroactive: bool,
    pub status: String,
    pub current_approver_id: Option<Uuid>,
    pub current_approver_name: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    /// R72-2：當前使用者是否可核准此列（依狀態 + 角色/部門主管於 service 計算，非 DB 欄位）
    #[sqlx(default)]
    pub can_approve: bool,
    /// 當前使用者是否為此列（PENDING_PROXY）的職務代理人、可確認/退回（於 service 計算，非 DB 欄位）
    #[sqlx(default)]
    pub can_confirm_proxy: bool,
}

#[derive(Debug, Deserialize)]
pub struct LeaveQuery {
    pub user_id: Option<Uuid>,
    pub status: Option<String>,
    pub leave_type: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub pending_approval: Option<bool>,
    pub view_all: Option<bool>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLeaveRequest {
    pub leave_type: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub total_days: f64,
    pub total_hours: Option<f64>,
    pub reason: Option<String>,                    // 特休假不用填理由
    pub supporting_documents: Option<Vec<String>>, // 附件圖片 URLs
    pub is_urgent: Option<bool>,
    pub is_retroactive: Option<bool>,
    pub proxy_user_id: Option<Uuid>, // 代理人
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
    pub proxy_user_id: Option<Uuid>, // 代理人
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

/// 代理人退回請假申請（退回草稿，供申請人重新指定代理人）。
#[derive(Debug, Deserialize)]
pub struct ProxyRejectRequest {
    pub reason: Option<String>,
}

// ============================================
// Leave Approvals (審核記錄)
// ============================================

// 審核事件記錄，欄位全為系統/使用者填入的審核資訊，無敏感欄位
impl crate::models::audit_diff::AuditRedact for LeaveApproval {}

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

// 年度特休餘額快照，無敏感欄位（天數 / 到期日 / 工作年資等皆稽核項目）
impl crate::models::audit_diff::AuditRedact for AnnualLeaveEntitlement {}

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

// 補休餘額快照，無敏感欄位
impl crate::models::audit_diff::AuditRedact for CompTimeBalance {}

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
    pub is_expired: bool, // 是否已過期（待補償）
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
    pub hire_date: Option<NaiveDate>, // 到職日，用於計算到期日（到職週年日 + 2年）
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
    pub remaining_days: f64, // 待補償天數
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
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
}

#[derive(Debug, Serialize)]
pub struct DashboardCalendarData {
    pub today: NaiveDate,
    pub today_leaves: Vec<TodayLeaveInfo>,
    pub today_events: Vec<crate::models::calendar::CalendarEvent>,
    pub upcoming_leaves: Vec<TodayLeaveInfo>,
}

// ============================================
// 請假行事曆（原生，直接讀 leave_requests）
// ============================================

/// 行事曆上的一筆假單。
///
/// 與 Google 同步的 `CalendarEvent` 是兩回事：這裡直接讀 `leave_requests`，
/// 因此看得到**審核中**的假單，且可依觀看者身分決定可見範圍——
/// 這兩件事都是 Google 路線做不到的（同步只處理已核准、日曆授權無法逐筆過濾）。
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LeaveCalendarEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub department_name: Option<String>,
    pub leave_type: String,
    pub leave_type_display: String,
    pub status: String,
    /// 尚在審核中時，目前卡在哪一關（已核准為 `None`）。
    pub pending_stage_display: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    /// FullCalendar 的擺放方式：無起訖時間就只能畫成整天色塊。
    ///
    /// ⚠️ 這**不**等於「請了一整天」——見 `is_partial_day`。實測 prod 全部 19 張
    /// 假單的 `start_time` / `end_time` 皆為 NULL（請假表單只收 `total_hours`，
    /// 不收時段），其中 3 張其實是半天假。
    pub is_all_day: bool,
    /// 時數不足整日（`total_days` 非整數）。
    ///
    /// 因為沒有起訖時間可畫時段，UI 只能維持整天色塊，但必須把時數標出來，
    /// 否則 4.5 小時的假看起來與整天假一模一樣，會誤導排班。
    pub is_partial_day: bool,
    pub total_days: Decimal,
    pub total_hours: Option<Decimal>,
    pub proxy_user_id: Option<Uuid>,
    pub proxy_user_name: Option<String>,
    /// 代理人是否已確認。`PENDING_PROXY` 階段代理人還沒按確認、甚至可能退回，
    /// 前端必須據此標註「未確認」，不可把代理關係顯示成既定事實。
    pub proxy_confirmed: bool,
    /// 這是不是我自己請的假。
    pub is_mine: bool,
    /// 我是不是這張假單的代理人（用於「與我相關」篩選與跨部門穿透）。
    pub is_my_proxy_duty: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LeaveCalendarResponse {
    pub entries: Vec<LeaveCalendarEntry>,
    /// 本次查詢實際套用的可見範圍，前端用來決定是否顯示「全場／與我相關」切換。
    pub scope: LeaveCalendarScope,
}

#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LeaveCalendarScope {
    /// 全場可見：管理員、具 `hr.leave.view_all`、或本身是任一部門的主管。
    All,
    /// 部門可見：自己所屬部門 + 上層 + 下層部門。
    Department,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leave_type_display_name() {
        assert_eq!(LeaveType::Annual.display_name(), "特休假");
        assert_eq!(LeaveType::Personal.display_name(), "事假");
        assert_eq!(LeaveType::Sick.display_name(), "病假");
        assert_eq!(LeaveType::Compensatory.display_name(), "補休假");
        assert_eq!(LeaveType::Marriage.display_name(), "婚假");
        assert_eq!(LeaveType::Bereavement.display_name(), "喪假");
        assert_eq!(LeaveType::Maternity.display_name(), "產假");
        assert_eq!(LeaveType::Paternity.display_name(), "陪產假");
        assert_eq!(LeaveType::Menstrual.display_name(), "生理假");
        assert_eq!(LeaveType::Official.display_name(), "公假");
    }

    #[test]
    fn test_leave_status_display_name() {
        assert_eq!(LeaveStatus::Draft.display_name(), "草稿");
        assert_eq!(LeaveStatus::PendingProxy.display_name(), "待代理確認");
        assert_eq!(LeaveStatus::PendingL1.display_name(), "待單位主管審核");
        assert_eq!(LeaveStatus::PendingL2.display_name(), "待二級審核");
        assert_eq!(LeaveStatus::PendingHr.display_name(), "待行政審核");
        assert_eq!(LeaveStatus::PendingGm.display_name(), "待總經理核准");
        assert_eq!(LeaveStatus::PendingDirector.display_name(), "待負責人簽核");
        assert_eq!(LeaveStatus::Approved.display_name(), "已核准");
        assert_eq!(LeaveStatus::Rejected.display_name(), "已駁回");
        assert_eq!(LeaveStatus::Cancelled.display_name(), "已取消");
        assert_eq!(LeaveStatus::Revoked.display_name(), "已銷假");
    }

    #[test]
    fn test_leave_type_serde_roundtrip() {
        // serde 使用預設 PascalCase（無 serde rename_all）
        let lt = LeaveType::Compensatory;
        let json = serde_json::to_string(&lt).expect("序列化 LeaveType 失敗");
        assert_eq!(json, "\"Compensatory\"");
        let parsed: LeaveType = serde_json::from_str(&json).expect("反序列化 LeaveType 失敗");
        assert_eq!(parsed, lt);
    }

    #[test]
    fn test_leave_status_serde_roundtrip() {
        let ls = LeaveStatus::PendingHr;
        let json = serde_json::to_string(&ls).expect("序列化 LeaveStatus 失敗");
        assert_eq!(json, "\"PendingHr\"");
        let parsed: LeaveStatus = serde_json::from_str(&json).expect("反序列化 LeaveStatus 失敗");
        assert_eq!(parsed, ls);
    }

    #[test]
    fn test_leave_type_all_variants() {
        let variants = vec![
            LeaveType::Annual,
            LeaveType::Personal,
            LeaveType::Sick,
            LeaveType::Compensatory,
            LeaveType::Marriage,
            LeaveType::Bereavement,
            LeaveType::Maternity,
            LeaveType::Paternity,
            LeaveType::Menstrual,
            LeaveType::Official,
        ];
        for v in &variants {
            assert!(!v.display_name().is_empty());
        }
        assert_eq!(variants.len(), 10);
    }

    #[test]
    fn test_leave_status_all_variants() {
        let variants = vec![
            LeaveStatus::Draft,
            LeaveStatus::PendingProxy,
            LeaveStatus::PendingL1,
            LeaveStatus::PendingL2,
            LeaveStatus::PendingHr,
            LeaveStatus::PendingGm,
            LeaveStatus::PendingDirector,
            LeaveStatus::Approved,
            LeaveStatus::Rejected,
            LeaveStatus::Cancelled,
            LeaveStatus::Revoked,
        ];
        for v in &variants {
            assert!(!v.display_name().is_empty());
        }
        assert_eq!(variants.len(), 11);
    }
}
