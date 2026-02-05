-- ============================================
-- Migration 015: Multi-Level Approval Workflow
-- 
-- 請假審核：部門主管 → 行政 → 負責人
-- 加班審核：行政 → 負責人
-- ============================================

-- 注意：leave_status 已經支援 PENDING_L1, PENDING_HR, PENDING_GM
-- 不需要修改 leave_status enum

-- ============================================
-- 1. 更新 overtime_records 狀態欄位說明
-- ============================================

-- 加班狀態：
-- draft: 草稿
-- pending_admin_staff: 待行政審核
-- pending_admin: 待負責人審核  
-- approved: 已核准
-- rejected: 已駁回

-- 由於 overtime status 是 VARCHAR，不需要修改 enum
-- 只需更新現有的 'pending' 資料為 'pending_admin_staff'
UPDATE overtime_records 
SET status = 'pending_admin_staff' 
WHERE status = 'pending';

-- ============================================
-- 2. 新增加班審核紀錄表（可選，追蹤每層審核）
-- ============================================

CREATE TABLE IF NOT EXISTS overtime_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    overtime_record_id UUID NOT NULL REFERENCES overtime_records(id) ON DELETE CASCADE,
    
    -- Approver
    approver_id UUID NOT NULL REFERENCES users(id),
    approval_level VARCHAR(20) NOT NULL, -- 'admin_staff', 'admin'
    
    -- Action
    action VARCHAR(20) NOT NULL, -- 'APPROVE', 'REJECT'
    comments TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_overtime_approval_record ON overtime_approvals(overtime_record_id, created_at);
CREATE INDEX IF NOT EXISTS idx_overtime_approval_approver ON overtime_approvals(approver_id, created_at DESC);

-- ============================================
-- 完成
-- ============================================
