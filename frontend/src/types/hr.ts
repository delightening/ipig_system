// 前端 API 型別定義擴展
// HR, Audit, Facility, Calendar 相關型別

// ============================================
// Audit Types
// ============================================

export interface UserActivityLog {
    id: string;
    actor_user_id: string | null;
    actor_email: string | null;
    actor_display_name: string | null;
    event_category: string;
    event_type: string;
    event_severity: string;
    entity_type: string | null;
    entity_id: string | null;
    entity_display_name: string | null;
    before_data: Record<string, unknown> | null;
    after_data: Record<string, unknown> | null;
    ip_address: string | null;
    user_agent: string | null;
    request_path: string | null;
    is_suspicious: boolean;
    suspicious_reason: string | null;
    created_at: string;
}

export interface LoginEventWithUser {
    id: string;
    user_id: string | null;
    email: string;
    user_name: string | null;
    event_type: string;
    ip_address: string | null;
    user_agent: string | null;
    device_type: string | null;
    browser: string | null;
    os: string | null;
    is_unusual_time: boolean;
    is_unusual_location: boolean;
    is_new_device: boolean;
    failure_reason: string | null;
    created_at: string;
}

export interface SessionWithUser {
    id: string;
    user_id: string;
    user_email: string;
    user_name: string;
    started_at: string;
    ended_at: string | null;
    last_activity_at: string;
    ip_address: string | null;
    user_agent: string | null;
    page_view_count: number;
    action_count: number;
    is_active: boolean;
    ended_reason: string | null;
}

export interface SecurityAlert {
    id: string;
    alert_type: string;
    severity: string;
    title: string;
    description: string | null;
    user_id: string | null;
    status: string;
    resolved_by: string | null;
    resolved_at: string | null;
    resolution_notes: string | null;
    created_at: string;
}

export interface AuditDashboardStats {
    active_users_today: number;
    active_users_week: number;
    active_users_month: number;
    total_logins_today: number;
    failed_logins_today: number;
    active_sessions: number;
    open_alerts: number;
    critical_alerts: number;
}

// ============================================
// HR Types
// ============================================

export interface AttendanceWithUser {
    id: string;
    user_id: string;
    user_email: string;
    user_name: string;
    work_date: string;
    clock_in_time: string | null;
    clock_out_time: string | null;
    regular_hours: number | null;
    overtime_hours: number | null;
    status: string;
    remark: string | null;
    is_corrected: boolean;
}

export interface OvertimeWithUser {
    id: string;
    user_id: string;
    user_email: string;
    user_name: string;
    overtime_date: string;
    start_time: string;
    end_time: string;
    hours: number;
    overtime_type: string;
    multiplier: number;
    comp_time_hours: number;
    comp_time_expires_at: string;
    status: string;
    reason: string;
}

export interface LeaveRequestWithUser {
    id: string;
    user_id: string;
    user_email: string;
    user_name: string;
    proxy_user_id: string | null;
    proxy_user_name: string | null;
    leave_type: string;
    start_date: string;
    end_date: string;
    total_days: number;
    reason: string;
    is_urgent: boolean;
    is_retroactive: boolean;
    status: string;
    current_approver_id: string | null;
    current_approver_name: string | null;
    submitted_at: string | null;
    created_at: string;
}

export interface AnnualLeaveBalanceView {
    entitlement_year: number;
    entitled_days: number;
    used_days: number;
    remaining_days: number;
    expires_at: string;
    days_until_expiry: number;
}

export interface CompTimeBalanceView {
    id: string;
    earned_date: string;
    original_hours: number;
    used_hours: number;
    remaining_hours: number;
    expires_at: string;
    days_until_expiry: number;
}

export interface BalanceSummary {
    user_id: string;
    user_name: string;
    annual_leave_total: number;
    annual_leave_used: number;
    annual_leave_remaining: number;
    comp_time_total: number;
    comp_time_used: number;
    comp_time_remaining: number;
    expiring_soon_days: number;
    expiring_soon_hours: number;
}

// Leave Type 顯示名稱映射
export const LEAVE_TYPE_NAMES: Record<string, string> = {
    ANNUAL: '特休假',
    PERSONAL: '事假',
    SICK: '病假',
    COMPENSATORY: '補休假',
    MARRIAGE: '婚假',
    BEREAVEMENT: '喪假',
    MATERNITY: '產假',
    PATERNITY: '陪產假',
    MENSTRUAL: '生理假',
    OFFICIAL: '公假',
    UNPAID: '無薪假',
};

export const LEAVE_STATUS_NAMES: Record<string, string> = {
    DRAFT: '草稿',
    PENDING_L1: '待一級審核',
    PENDING_L2: '待二級審核',
    PENDING_HR: '待行政審核',
    PENDING_GM: '待總經理核准',
    APPROVED: '已核准',
    REJECTED: '已駁回',
    CANCELLED: '已取消',
    REVOKED: '已銷假',
};

// ============================================
// Facility Types
// ============================================

export interface Species {
    id: string;
    code: string;
    name: string;
    name_en: string | null;
    icon: string | null;
    is_active: boolean;
    config: Record<string, unknown> | null;
    sort_order: number;
}

export interface Facility {
    id: string;
    code: string;
    name: string;
    address: string | null;
    phone: string | null;
    contact_person: string | null;
    is_active: boolean;
}

export interface BuildingWithFacility {
    id: string;
    facility_id: string;
    facility_code: string;
    facility_name: string;
    code: string;
    name: string;
    description: string | null;
    is_active: boolean;
    sort_order: number;
}

export interface ZoneWithBuilding {
    id: string;
    building_id: string;
    building_code: string;
    building_name: string;
    facility_id: string;
    facility_name: string;
    code: string;
    name: string | null;
    color: string | null;
    is_active: boolean;
    sort_order: number;
}

export interface PenDetails {
    id: string;
    code: string;
    name: string | null;
    capacity: number;
    current_count: number;
    status: string;
    zone_id: string;
    zone_code: string;
    zone_name: string | null;
    zone_color: string | null;
    building_id: string;
    building_code: string;
    building_name: string;
    facility_id: string;
    facility_code: string;
    facility_name: string;
}

export interface DepartmentWithManager {
    id: string;
    code: string;
    name: string;
    parent_id: string | null;
    parent_name: string | null;
    manager_id: string | null;
    manager_name: string | null;
    is_active: boolean;
    sort_order: number;
}

// ============================================
// Calendar Types
// ============================================

export interface CalendarSyncStatus {
    is_configured: boolean;
    sync_enabled: boolean;
    calendar_id: string;
    last_sync_at: string | null;
    last_sync_status: string | null;
    next_sync_at: string | null;
    pending_syncs: number;
    pending_conflicts: number;
    recent_errors: number;
}

export interface ConflictWithDetails {
    id: string;
    leave_request_id: string | null;
    user_name: string | null;
    leave_type: string | null;
    conflict_type: string;
    difference_summary: string | null;
    status: string;
    detected_at: string;
}

export interface CalendarSyncHistory {
    id: string;
    job_type: string;
    triggered_by: string | null;
    started_at: string;
    completed_at: string | null;
    duration_ms: number | null;
    status: string;
    events_created: number;
    events_updated: number;
    events_deleted: number;
    conflicts_detected: number;
    errors_count: number;
}

export interface CalendarEvent {
    id: string;
    summary: string;
    start: string;
    end: string;
    all_day: boolean;
    description?: string;
    location?: string;
    color_id?: string;
    html_link?: string;
}

