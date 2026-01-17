-- ============================================
-- Migration 004: HR System
-- 
-- 包含：
-- - 請假類型與狀態
-- - 出勤打卡紀錄
-- - 加班紀錄
-- - 年假額度
-- - 補休餘額
-- - 請假申請
-- - 請假審核紀錄
-- - 相關函式
-- ============================================

-- ============================================
-- 1. 請假類型枚舉
-- ============================================

CREATE TYPE leave_type AS ENUM (
    'ANNUAL',           -- 特休假
    'PERSONAL',         -- 事假
    'SICK',             -- 病假
    'COMPENSATORY',     -- 補休假
    'MARRIAGE',         -- 婚假
    'BEREAVEMENT',      -- 喪假
    'MATERNITY',        -- 產假
    'PATERNITY',        -- 陪產假
    'MENSTRUAL',        -- 生理假
    'OFFICIAL',         -- 公假
    'UNPAID'            -- 無薪假
);

-- ============================================
-- 2. 請假狀態枚舉
-- ============================================

CREATE TYPE leave_status AS ENUM (
    'DRAFT',            -- 草稿
    'PENDING_L1',       -- 待一級審核（直屬主管）
    'PENDING_L2',       -- 待二級審核（部門主管）
    'PENDING_HR',       -- 待行政審核
    'PENDING_GM',       -- 待總經理核准
    'APPROVED',         -- 已核准
    'REJECTED',         -- 已駁回
    'CANCELLED',        -- 已取消
    'REVOKED'           -- 已銷假
);

-- ============================================
-- 3. 出勤打卡紀錄表
-- ============================================

CREATE TABLE attendance_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Date and time
    work_date DATE NOT NULL,
    clock_in_time TIMESTAMPTZ,
    clock_out_time TIMESTAMPTZ,
    
    -- Calculated fields
    regular_hours NUMERIC(5,2) DEFAULT 0,
    overtime_hours NUMERIC(5,2) DEFAULT 0,
    
    -- Status
    status VARCHAR(20) DEFAULT 'normal', -- 'normal', 'late', 'early_leave', 'absent', 'leave', 'holiday'
    
    -- Source tracking
    clock_in_source VARCHAR(20), -- 'web', 'mobile', 'import', 'manual'
    clock_in_ip INET,
    clock_out_source VARCHAR(20),
    clock_out_ip INET,
    
    -- Notes
    remark TEXT,
    
    -- Approval for manual corrections
    is_corrected BOOLEAN DEFAULT false,
    corrected_by UUID REFERENCES users(id),
    corrected_at TIMESTAMPTZ,
    correction_reason TEXT,
    original_clock_in TIMESTAMPTZ,
    original_clock_out TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(user_id, work_date)
);

CREATE INDEX idx_attendance_user_date ON attendance_records(user_id, work_date DESC);
CREATE INDEX idx_attendance_date ON attendance_records(work_date DESC);
CREATE INDEX idx_attendance_status ON attendance_records(status);

-- ============================================
-- 4. 加班紀錄表
-- ============================================

CREATE TABLE overtime_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    attendance_id UUID REFERENCES attendance_records(id),
    
    -- When
    overtime_date DATE NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    
    -- Duration
    hours NUMERIC(5,2) NOT NULL,
    
    -- Type and multiplier
    overtime_type VARCHAR(20) NOT NULL, -- 'weekday', 'weekend', 'holiday'
    multiplier NUMERIC(3,2) DEFAULT 1.0, -- 1.0, 1.33, 1.66, 2.0
    
    -- Comp time generation
    comp_time_hours NUMERIC(5,2) NOT NULL,
    comp_time_expires_at DATE NOT NULL,
    comp_time_used_hours NUMERIC(5,2) DEFAULT 0,
    
    -- Approval
    status VARCHAR(20) DEFAULT 'draft', -- 'draft', 'pending', 'approved', 'rejected'
    submitted_at TIMESTAMPTZ,
    approved_by UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    rejected_by UUID REFERENCES users(id),
    rejected_at TIMESTAMPTZ,
    rejection_reason TEXT,
    
    -- Reason
    reason TEXT NOT NULL,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_overtime_user ON overtime_records(user_id, overtime_date DESC);
CREATE INDEX idx_overtime_status ON overtime_records(status);
CREATE INDEX idx_overtime_expires ON overtime_records(comp_time_expires_at) 
    WHERE status = 'approved' AND comp_time_used_hours < comp_time_hours;

-- ============================================
-- 5. 年假額度表
-- ============================================

CREATE TABLE annual_leave_entitlements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Entitlement period
    entitlement_year INTEGER NOT NULL,
    
    -- Days
    entitled_days NUMERIC(5,2) NOT NULL,
    used_days NUMERIC(5,2) DEFAULT 0,
    
    -- Expiration
    expires_at DATE NOT NULL,
    
    -- Source
    calculation_basis VARCHAR(50), -- 'seniority', 'prorated', 'manual', 'carry_forward'
    seniority_years NUMERIC(4,2),
    
    -- Status
    is_expired BOOLEAN DEFAULT false,
    expired_days NUMERIC(5,2) DEFAULT 0,
    expiry_processed_at TIMESTAMPTZ,
    
    -- Notes
    notes TEXT,
    adjustment_days NUMERIC(5,2) DEFAULT 0,
    
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(user_id, entitlement_year)
);

CREATE INDEX idx_annual_leave_user ON annual_leave_entitlements(user_id, entitlement_year DESC);
CREATE INDEX idx_annual_leave_expires ON annual_leave_entitlements(expires_at) 
    WHERE NOT is_expired AND (entitled_days - used_days) > 0;

-- ============================================
-- 6. 補休餘額表
-- ============================================

CREATE TABLE comp_time_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    overtime_record_id UUID NOT NULL REFERENCES overtime_records(id) ON DELETE CASCADE,
    
    -- Hours
    original_hours NUMERIC(5,2) NOT NULL,
    used_hours NUMERIC(5,2) DEFAULT 0,
    
    -- Expiration
    earned_date DATE NOT NULL,
    expires_at DATE NOT NULL,
    
    -- Status
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
-- 7. 請假申請表
-- ============================================

CREATE TABLE leave_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    proxy_user_id UUID REFERENCES users(id),
    
    -- Leave details
    leave_type leave_type NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    start_time TIME,
    end_time TIME,
    
    -- Duration
    total_days NUMERIC(5,2) NOT NULL,
    total_hours NUMERIC(5,2),
    
    -- Reason and documents
    reason TEXT,
    supporting_documents JSONB DEFAULT '[]',
    
    -- For comp time usage
    comp_time_source_ids UUID[],
    
    -- For annual leave usage
    annual_leave_source_id UUID REFERENCES annual_leave_entitlements(id),
    
    -- Flags
    is_urgent BOOLEAN DEFAULT false,
    is_retroactive BOOLEAN DEFAULT false,
    
    -- Status
    status leave_status DEFAULT 'DRAFT',
    
    -- Approval chain
    current_approver_id UUID REFERENCES users(id),
    
    -- Timestamps
    submitted_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ,
    rejected_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    
    -- Notes
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
-- 8. 請假審核紀錄表
-- ============================================

CREATE TABLE leave_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    leave_request_id UUID NOT NULL REFERENCES leave_requests(id) ON DELETE CASCADE,
    
    -- Approver
    approver_id UUID NOT NULL REFERENCES users(id),
    approval_level VARCHAR(20) NOT NULL,
    
    -- Action
    action VARCHAR(20) NOT NULL, -- 'APPROVE', 'REJECT', 'REQUEST_REVISION', 'ESCALATE'
    comments TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_approval_request ON leave_approvals(leave_request_id, created_at);
CREATE INDEX idx_approval_approver ON leave_approvals(approver_id, created_at DESC);

-- ============================================
-- 9. 請假餘額使用紀錄表
-- ============================================

CREATE TABLE leave_balance_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    leave_request_id UUID NOT NULL REFERENCES leave_requests(id),
    
    -- Source
    source_type VARCHAR(20) NOT NULL, -- 'annual', 'comp_time'
    annual_leave_entitlement_id UUID REFERENCES annual_leave_entitlements(id),
    comp_time_balance_id UUID REFERENCES comp_time_balances(id),
    
    -- Amount used
    days_used NUMERIC(5,2),
    hours_used NUMERIC(5,2),
    
    -- Action
    action VARCHAR(20) NOT NULL, -- 'deduct', 'restore'
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_usage_request ON leave_balance_usage(leave_request_id);
CREATE INDEX idx_usage_annual ON leave_balance_usage(annual_leave_entitlement_id);
CREATE INDEX idx_usage_comp ON leave_balance_usage(comp_time_balance_id);

-- ============================================
-- 10. Helper Functions
-- ============================================

-- Get remaining annual leave for user
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

-- Get remaining comp time for user (FIFO order)
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

-- Calculate total comp time remaining
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
-- 11. HR 相關權限
-- ============================================

INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'hr.attendance.view', '查看出勤紀錄', 'hr', '可查看出勤紀錄', NOW()),
    (gen_random_uuid(), 'hr.attendance.view_all', '查看所有出勤', 'hr', '可查看所有人的出勤紀錄', NOW()),
    (gen_random_uuid(), 'hr.attendance.clock', '打卡', 'hr', '可進行上下班打卡', NOW()),
    (gen_random_uuid(), 'hr.attendance.correct', '更正打卡', 'hr', '可更正打卡紀錄', NOW()),
    (gen_random_uuid(), 'hr.overtime.view', '查看加班紀錄', 'hr', '可查看加班紀錄', NOW()),
    (gen_random_uuid(), 'hr.overtime.create', '申請加班', 'hr', '可申請加班', NOW()),
    (gen_random_uuid(), 'hr.overtime.approve', '審核加班', 'hr', '可審核加班申請', NOW()),
    (gen_random_uuid(), 'hr.leave.view', '查看請假', 'hr', '可查看請假紀錄', NOW()),
    (gen_random_uuid(), 'hr.leave.view_all', '查看所有請假', 'hr', '可查看所有人的請假紀錄', NOW()),
    (gen_random_uuid(), 'hr.leave.create', '申請請假', 'hr', '可申請請假', NOW()),
    (gen_random_uuid(), 'hr.leave.approve', '審核請假', 'hr', '可審核請假申請', NOW()),
    (gen_random_uuid(), 'hr.leave.manage', '管理假別', 'hr', '可管理假別設定', NOW()),
    (gen_random_uuid(), 'hr.balance.view', '查看餘額', 'hr', '可查看假期餘額', NOW()),
    (gen_random_uuid(), 'hr.balance.manage', '管理餘額', 'hr', '可管理假期餘額', NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 完成
-- ============================================
