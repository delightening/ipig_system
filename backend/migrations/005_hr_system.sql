-- ============================================
-- Migration 005: HR System
-- 
-- 包含：
-- - 出勤打卡紀錄
-- - 加班紀錄
-- - 年假額度
-- - 補休餘額
-- - 請假申請
-- - 請假審核
-- - 行事曆同步
-- - 輔助函式
-- 
-- 編碼: UTF-8 (無 BOM)
-- ============================================

-- ============================================
-- 1. 出勤打卡紀錄表
-- ============================================

CREATE TABLE attendance_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- 日期與時間
    work_date DATE NOT NULL,
    clock_in_time TIMESTAMPTZ,
    clock_out_time TIMESTAMPTZ,
    
    -- 計算欄位
    regular_hours NUMERIC(5,2) DEFAULT 0,
    overtime_hours NUMERIC(5,2) DEFAULT 0,
    
    -- 狀態
    status VARCHAR(20) DEFAULT 'normal',
    
    -- 來源追蹤
    clock_in_source VARCHAR(20),
    clock_in_ip INET,
    clock_out_source VARCHAR(20),
    clock_out_ip INET,
    
    -- GPS 定位（打卡座標）
    clock_in_latitude  DOUBLE PRECISION,
    clock_in_longitude DOUBLE PRECISION,
    clock_out_latitude  DOUBLE PRECISION,
    clock_out_longitude DOUBLE PRECISION,
    
    -- 備註
    remark TEXT,
    
    -- 手動更正
    is_corrected BOOLEAN DEFAULT false,
    corrected_by UUID REFERENCES users(id),
    corrected_at TIMESTAMPTZ,
    correction_reason TEXT,
    original_clock_in TIMESTAMPTZ,
    original_clock_out TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(user_id, work_date),
    CONSTRAINT chk_status CHECK (status IN ('normal', 'late', 'early_leave', 'absent', 'leave', 'holiday'))
);

CREATE INDEX idx_attendance_user_date ON attendance_records(user_id, work_date DESC);
CREATE INDEX idx_attendance_date ON attendance_records(work_date DESC);
CREATE INDEX idx_attendance_status ON attendance_records(status);

-- ============================================
-- 2. 加班紀錄表
-- ============================================

CREATE TABLE overtime_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    attendance_id UUID REFERENCES attendance_records(id),
    
    -- 時間
    overtime_date DATE NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    
    -- 時數
    hours NUMERIC(5,2) NOT NULL,
    
    -- 類型與倍率
    overtime_type VARCHAR(20) NOT NULL,
    multiplier NUMERIC(3,2) DEFAULT 1.0,
    
    -- 補休計算
    comp_time_hours NUMERIC(5,2) NOT NULL,
    comp_time_expires_at DATE NOT NULL,
    comp_time_used_hours NUMERIC(5,2) DEFAULT 0,
    
    -- 審核
    status VARCHAR(20) DEFAULT 'draft',
    submitted_at TIMESTAMPTZ,
    approved_by UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    rejected_by UUID REFERENCES users(id),
    rejected_at TIMESTAMPTZ,
    rejection_reason TEXT,
    
    -- 原因
    reason TEXT NOT NULL,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT chk_overtime_type CHECK (overtime_type IN ('A', 'B', 'C', 'D')),
    CONSTRAINT chk_overtime_status CHECK (status IN ('draft', 'pending', 'pending_admin_staff', 'pending_admin', 'approved', 'rejected'))
);

CREATE INDEX idx_overtime_user ON overtime_records(user_id, overtime_date DESC);
CREATE INDEX idx_overtime_status ON overtime_records(status);
CREATE INDEX idx_overtime_expires ON overtime_records(comp_time_expires_at) 
    WHERE status = 'approved' AND comp_time_used_hours < comp_time_hours;

-- ============================================
-- 2b. 加班審核紀錄表
-- ============================================

CREATE TABLE overtime_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    overtime_record_id UUID NOT NULL REFERENCES overtime_records(id) ON DELETE CASCADE,
    
    -- 審核者
    approver_id UUID NOT NULL REFERENCES users(id),
    approval_level VARCHAR(20) NOT NULL,
    
    -- 動作
    action VARCHAR(20) NOT NULL,
    comments TEXT,
    
    -- 時間
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT chk_overtime_approval_action CHECK (action IN ('APPROVE', 'REJECT'))
);

CREATE INDEX idx_overtime_approval_record ON overtime_approvals(overtime_record_id, created_at);
CREATE INDEX idx_overtime_approval_approver ON overtime_approvals(approver_id, created_at DESC);

-- ============================================
-- 3. 年假額度表
-- ============================================

CREATE TABLE annual_leave_entitlements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- 額度年度
    entitlement_year INTEGER NOT NULL,
    
    -- 天數
    entitled_days NUMERIC(5,2) NOT NULL,
    used_days NUMERIC(5,2) DEFAULT 0,
    
    -- 到期
    expires_at DATE NOT NULL,
    
    -- 來源
    calculation_basis VARCHAR(50),
    seniority_years NUMERIC(4,2),
    
    -- 狀態
    is_expired BOOLEAN DEFAULT false,
    expired_days NUMERIC(5,2) DEFAULT 0,
    expiry_processed_at TIMESTAMPTZ,
    
    -- 備註
    notes TEXT,
    adjustment_days NUMERIC(5,2) DEFAULT 0,
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(user_id, entitlement_year),
    CONSTRAINT chk_calculation_basis CHECK (calculation_basis IS NULL OR calculation_basis IN ('seniority', 'prorated', 'manual', 'carry_forward'))
);

CREATE INDEX idx_annual_leave_user ON annual_leave_entitlements(user_id, entitlement_year DESC);
CREATE INDEX idx_annual_leave_expires ON annual_leave_entitlements(expires_at) 
    WHERE NOT is_expired AND (entitled_days - used_days) > 0;

-- ============================================
-- 4. 補休餘額表
-- ============================================

CREATE TABLE comp_time_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    overtime_record_id UUID NOT NULL REFERENCES overtime_records(id) ON DELETE CASCADE,
    
    -- 時數
    original_hours NUMERIC(5,2) NOT NULL,
    used_hours NUMERIC(5,2) DEFAULT 0,
    
    -- 到期
    earned_date DATE NOT NULL,
    expires_at DATE NOT NULL,
    
    -- 狀態
    is_expired BOOLEAN DEFAULT false,
    expired_hours NUMERIC(5,2) DEFAULT 0,
    converted_to_pay BOOLEAN DEFAULT false,
    expiry_processed_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(overtime_record_id)
);

CREATE INDEX idx_comp_time_user ON comp_time_balances(user_id, earned_date DESC);
CREATE INDEX idx_comp_time_expires ON comp_time_balances(expires_at) 
    WHERE NOT is_expired AND (original_hours - used_hours) > 0;
CREATE INDEX idx_comp_time_fifo ON comp_time_balances(user_id, earned_date ASC) 
    WHERE NOT is_expired AND (original_hours - used_hours) > 0;

-- ============================================
-- 5. 請假申請表
-- ============================================

CREATE TABLE leave_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    proxy_user_id UUID REFERENCES users(id),
    
    -- 請假詳情
    leave_type leave_type NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    start_time TIME,
    end_time TIME,
    
    -- 時數
    total_days NUMERIC(5,2) NOT NULL,
    total_hours NUMERIC(5,2),
    
    -- 原因與文件
    reason TEXT,
    supporting_documents JSONB DEFAULT '[]',
    
    -- 年假來源
    annual_leave_source_id UUID REFERENCES annual_leave_entitlements(id),
    
    -- 標記
    is_urgent BOOLEAN DEFAULT false,
    is_retroactive BOOLEAN DEFAULT false,
    
    -- 狀態
    status leave_status DEFAULT 'DRAFT',
    
    -- 審核鏈
    current_approver_id UUID REFERENCES users(id),
    
    -- 時間戳
    submitted_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ,
    rejected_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    
    -- 備註
    cancellation_reason TEXT,
    revocation_reason TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_leave_user ON leave_requests(user_id, start_date DESC);
CREATE INDEX idx_leave_status ON leave_requests(status);
CREATE INDEX idx_leave_date_range ON leave_requests(start_date, end_date);
CREATE INDEX idx_leave_approver ON leave_requests(current_approver_id) 
    WHERE status IN ('PENDING_L1', 'PENDING_L2', 'PENDING_HR', 'PENDING_GM');

-- ============================================
-- 6. 請假審核紀錄表
-- ============================================

CREATE TABLE leave_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    leave_request_id UUID NOT NULL REFERENCES leave_requests(id) ON DELETE CASCADE,
    
    -- 審核者
    approver_id UUID NOT NULL REFERENCES users(id),
    approval_level VARCHAR(20) NOT NULL,
    
    -- 動作
    action VARCHAR(20) NOT NULL,
    comments TEXT,
    
    -- 時間
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT chk_action CHECK (action IN ('APPROVE', 'REJECT', 'REQUEST_REVISION', 'ESCALATE'))
);

CREATE INDEX idx_approval_request ON leave_approvals(leave_request_id, created_at);
CREATE INDEX idx_approval_approver ON leave_approvals(approver_id, created_at DESC);

-- ============================================
-- 7. 請假餘額使用紀錄表
-- ============================================

CREATE TABLE leave_balance_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    leave_request_id UUID NOT NULL REFERENCES leave_requests(id),
    
    -- 來源
    source_type VARCHAR(20) NOT NULL,
    annual_leave_entitlement_id UUID REFERENCES annual_leave_entitlements(id),
    comp_time_balance_id UUID REFERENCES comp_time_balances(id),
    
    -- 使用量
    days_used NUMERIC(5,2),
    hours_used NUMERIC(5,2),
    
    -- 動作
    action VARCHAR(20) NOT NULL,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT chk_source_type CHECK (source_type IN ('annual', 'comp_time')),
    CONSTRAINT chk_balance_action CHECK (action IN ('deduct', 'restore'))
);

CREATE INDEX idx_usage_request ON leave_balance_usage(leave_request_id);
CREATE INDEX idx_usage_annual ON leave_balance_usage(annual_leave_entitlement_id);
CREATE INDEX idx_usage_comp ON leave_balance_usage(comp_time_balance_id);

-- ============================================
-- 8. Google Calendar 同步系統
-- ============================================

-- 8a. Google Calendar 設定表（系統級，僅一行）
CREATE TABLE google_calendar_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    calendar_id VARCHAR(255) NOT NULL DEFAULT '',
    calendar_name VARCHAR(255),
    calendar_description TEXT,
    auth_method VARCHAR(50) NOT NULL DEFAULT 'shared_account',
    auth_email VARCHAR(255),
    is_configured BOOLEAN NOT NULL DEFAULT false,
    sync_enabled BOOLEAN NOT NULL DEFAULT false,
    sync_schedule_morning TIME,
    sync_schedule_evening TIME,
    sync_timezone VARCHAR(50),
    sync_approved_leaves BOOLEAN NOT NULL DEFAULT true,
    sync_overtime BOOLEAN NOT NULL DEFAULT false,
    event_title_template VARCHAR(255),
    event_color_id VARCHAR(20),
    last_sync_at TIMESTAMPTZ,
    last_sync_status VARCHAR(50),
    last_sync_error TEXT,
    last_sync_events_pushed INTEGER,
    last_sync_events_pulled INTEGER,
    last_sync_conflicts INTEGER,
    last_sync_duration_ms INTEGER,
    next_sync_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 確保只有一行設定（用於 ON CONFLICT ((true))）
CREATE UNIQUE INDEX idx_google_calendar_config_singleton ON google_calendar_config ((true));

-- 8b. Calendar 事件同步狀態表
CREATE TABLE calendar_event_sync (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    leave_request_id UUID NOT NULL REFERENCES leave_requests(id) ON DELETE CASCADE,
    google_event_id VARCHAR(255),
    google_event_etag VARCHAR(255),
    google_event_link TEXT,
    sync_version INTEGER NOT NULL DEFAULT 0,
    local_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    google_updated_at TIMESTAMPTZ,
    last_synced_data JSONB,
    sync_status VARCHAR(50) NOT NULL DEFAULT 'pending_create',
    last_error TEXT,
    error_count INTEGER NOT NULL DEFAULT 0,
    last_error_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_calendar_event_sync_status ON calendar_event_sync(sync_status);
CREATE INDEX idx_calendar_event_sync_leave ON calendar_event_sync(leave_request_id);

-- 8c. Calendar 同步衝突表
CREATE TABLE calendar_sync_conflicts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    calendar_event_sync_id UUID REFERENCES calendar_event_sync(id) ON DELETE SET NULL,
    leave_request_id UUID REFERENCES leave_requests(id) ON DELETE SET NULL,
    conflict_type VARCHAR(50) NOT NULL,
    ipig_data JSONB NOT NULL DEFAULT '{}',
    google_data JSONB,
    difference_summary TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    resolution_notes TEXT,
    requires_new_approval BOOLEAN NOT NULL DEFAULT false,
    new_approval_request_id UUID REFERENCES leave_requests(id),
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_calendar_sync_conflicts_status ON calendar_sync_conflicts(status);

-- 8d. Calendar 同步歷史表
CREATE TABLE calendar_sync_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type VARCHAR(50) NOT NULL DEFAULT 'manual',
    triggered_by UUID REFERENCES users(id),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    duration_ms INTEGER,
    status VARCHAR(50) NOT NULL DEFAULT 'running',
    events_created INTEGER NOT NULL DEFAULT 0,
    events_updated INTEGER NOT NULL DEFAULT 0,
    events_deleted INTEGER NOT NULL DEFAULT 0,
    events_checked INTEGER NOT NULL DEFAULT 0,
    conflicts_detected INTEGER NOT NULL DEFAULT 0,
    errors_count INTEGER NOT NULL DEFAULT 0,
    error_messages JSONB,
    progress_percentage INTEGER NOT NULL DEFAULT 0,
    current_operation VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_calendar_sync_history_status ON calendar_sync_history(status);
CREATE INDEX idx_calendar_sync_history_started ON calendar_sync_history(started_at DESC);

-- ============================================
-- 9. 輔助函式
-- ============================================

-- 取得使用者年假餘額
CREATE OR REPLACE FUNCTION get_annual_leave_balance(p_user_id UUID) 
RETURNS TABLE (
    entitlement_year INTEGER,
    entitled_days NUMERIC,
    used_days NUMERIC,
    remaining_days NUMERIC,
    expires_at DATE,
    days_until_expiry INTEGER
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        ale.entitlement_year,
        ale.entitled_days,
        ale.used_days,
        (ale.entitled_days - ale.used_days) AS remaining_days,
        ale.expires_at,
        (ale.expires_at - CURRENT_DATE)::INTEGER AS days_until_expiry
    FROM annual_leave_entitlements ale
    WHERE ale.user_id = p_user_id
      AND NOT ale.is_expired
      AND (ale.entitled_days - ale.used_days) > 0
    ORDER BY ale.expires_at ASC;
END;
$$ LANGUAGE plpgsql;

-- 取得使用者補休餘額 (FIFO)
CREATE OR REPLACE FUNCTION get_comp_time_balance(p_user_id UUID) 
RETURNS TABLE (
    id UUID,
    earned_date DATE,
    original_hours NUMERIC,
    used_hours NUMERIC,
    remaining_hours NUMERIC,
    expires_at DATE,
    days_until_expiry INTEGER
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        ctb.id,
        ctb.earned_date,
        ctb.original_hours,
        ctb.used_hours,
        (ctb.original_hours - ctb.used_hours) AS remaining_hours,
        ctb.expires_at,
        (ctb.expires_at - CURRENT_DATE)::INTEGER AS days_until_expiry
    FROM comp_time_balances ctb
    WHERE ctb.user_id = p_user_id
      AND NOT ctb.is_expired
      AND (ctb.original_hours - ctb.used_hours) > 0
    ORDER BY ctb.earned_date ASC;
END;
$$ LANGUAGE plpgsql;

-- 計算補休總時數
CREATE OR REPLACE FUNCTION get_total_comp_time_hours(p_user_id UUID) 
RETURNS NUMERIC AS $$
DECLARE
    v_total NUMERIC;
BEGIN
    SELECT COALESCE(SUM(original_hours - used_hours), 0) INTO v_total
    FROM comp_time_balances
    WHERE user_id = p_user_id
      AND NOT is_expired
      AND (original_hours - used_hours) > 0;
    
    RETURN v_total;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- 完成
-- ============================================
