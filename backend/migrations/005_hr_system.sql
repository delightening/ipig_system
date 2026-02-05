-- ============================================
-- Migration 004a: HR System
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
    
    CONSTRAINT chk_overtime_type CHECK (overtime_type IN ('weekday', 'weekend', 'holiday')),
    CONSTRAINT chk_overtime_status CHECK (status IN ('draft', 'pending', 'approved', 'rejected'))
);

CREATE INDEX idx_overtime_user ON overtime_records(user_id, overtime_date DESC);
CREATE INDEX idx_overtime_status ON overtime_records(status);
CREATE INDEX idx_overtime_expires ON overtime_records(comp_time_expires_at) 
    WHERE status = 'approved' AND comp_time_used_hours < comp_time_hours;

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
-- 8. 行事曆同步設定表
-- ============================================

CREATE TABLE calendar_sync_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE UNIQUE,
    google_calendar_enabled BOOLEAN DEFAULT false,
    google_calendar_id VARCHAR(255),
    google_refresh_token TEXT,
    sync_leave_requests BOOLEAN DEFAULT true,
    sync_overtime BOOLEAN DEFAULT true,
    last_sync_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

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
