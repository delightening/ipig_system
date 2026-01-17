-- HR/Attendance Permissions
-- Version: 1.0
-- Created: 2026-01-17
-- Description: Permissions for attendance, overtime, and leave management

-- ============================================
-- Attendance Permissions
-- ============================================

INSERT INTO permissions (id, code, name, created_at) VALUES
    -- Personal attendance
    (gen_random_uuid(), 'hr.attendance.view.own', '檢視個人出勤記錄', NOW()),
    (gen_random_uuid(), 'hr.attendance.clock', '打卡（上下班）', NOW()),
    
    -- Department attendance (for managers)
    (gen_random_uuid(), 'hr.attendance.view.department', '檢視部門出勤記錄', NOW()),
    
    -- All attendance (for HR/admin)
    (gen_random_uuid(), 'hr.attendance.view.all', '檢視所有出勤記錄', NOW()),
    (gen_random_uuid(), 'hr.attendance.correct', '更正出勤記錄', NOW()),
    (gen_random_uuid(), 'hr.attendance.import', '匯入出勤資料', NOW()),
    (gen_random_uuid(), 'hr.attendance.export', '匯出出勤報表', NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- Overtime Permissions
-- ============================================

INSERT INTO permissions (id, code, name, created_at) VALUES
    -- Personal overtime
    (gen_random_uuid(), 'hr.overtime.view.own', '檢視個人加班記錄', NOW()),
    (gen_random_uuid(), 'hr.overtime.create', '申請加班', NOW()),
    (gen_random_uuid(), 'hr.overtime.edit.own', '編輯個人加班申請', NOW()),
    (gen_random_uuid(), 'hr.overtime.delete.own', '刪除個人加班申請', NOW()),
    
    -- Department overtime
    (gen_random_uuid(), 'hr.overtime.view.department', '檢視部門加班記錄', NOW()),
    (gen_random_uuid(), 'hr.overtime.approve', '審核加班申請', NOW()),
    
    -- All overtime
    (gen_random_uuid(), 'hr.overtime.view.all', '檢視所有加班記錄', NOW()),
    (gen_random_uuid(), 'hr.overtime.manage', '管理加班記錄', NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- Leave Permissions
-- ============================================

INSERT INTO permissions (id, code, name, created_at) VALUES
    -- Personal leave
    (gen_random_uuid(), 'hr.leave.view.own', '檢視個人請假記錄', NOW()),
    (gen_random_uuid(), 'hr.leave.create', '申請請假', NOW()),
    (gen_random_uuid(), 'hr.leave.edit.own', '編輯個人請假申請', NOW()),
    (gen_random_uuid(), 'hr.leave.delete.own', '刪除個人請假申請', NOW()),
    (gen_random_uuid(), 'hr.leave.cancel.own', '取消個人請假', NOW()),
    
    -- Balance viewing
    (gen_random_uuid(), 'hr.balance.view.own', '檢視個人假期餘額', NOW()),
    (gen_random_uuid(), 'hr.balance.view.department', '檢視部門假期餘額', NOW()),
    (gen_random_uuid(), 'hr.balance.view.all', '檢視所有假期餘額', NOW()),
    
    -- Department leave
    (gen_random_uuid(), 'hr.leave.view.department', '檢視部門請假記錄', NOW()),
    
    -- Approvals
    (gen_random_uuid(), 'hr.leave.approve.l1', '一級審核（直屬主管）', NOW()),
    (gen_random_uuid(), 'hr.leave.approve.l2', '二級審核（部門主管）', NOW()),
    (gen_random_uuid(), 'hr.leave.approve.hr', '行政審核', NOW()),
    (gen_random_uuid(), 'hr.leave.approve.gm', '總經理審核', NOW()),
    
    -- All leave
    (gen_random_uuid(), 'hr.leave.view.all', '檢視所有請假記錄', NOW()),
    (gen_random_uuid(), 'hr.leave.manage', '管理請假記錄', NOW()),
    
    -- Balance management
    (gen_random_uuid(), 'hr.balance.manage', '管理假期額度', NOW()),
    (gen_random_uuid(), 'hr.balance.adjust', '調整假期額度', NOW()),
    
    -- Reports
    (gen_random_uuid(), 'hr.leave.export', '匯出請假報表', NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- Assign permissions to roles
-- ============================================

-- All internal staff get basic personal permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM roles r, permissions p 
WHERE r.code IN ('warehouse', 'experiment_staff', 'vet', 'iacuc_staff')
  AND p.code IN (
    'hr.attendance.view.own', 'hr.attendance.clock',
    'hr.overtime.view.own', 'hr.overtime.create', 'hr.overtime.edit.own', 'hr.overtime.delete.own',
    'hr.leave.view.own', 'hr.leave.create', 'hr.leave.edit.own', 'hr.leave.delete.own', 'hr.leave.cancel.own',
    'hr.balance.view.own'
  )
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Admin gets all HR permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM roles r, permissions p 
WHERE r.code = 'admin' AND p.code LIKE 'hr.%'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- IACUC staff (執行秘書) gets extra HR permissions for administrative tasks
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM roles r, permissions p 
WHERE r.code = 'iacuc_staff' 
  AND p.code IN (
    'hr.attendance.view.all', 'hr.attendance.correct', 'hr.attendance.export',
    'hr.overtime.view.all', 'hr.overtime.approve',
    'hr.leave.view.all', 'hr.leave.approve.hr', 'hr.leave.export',
    'hr.balance.view.all', 'hr.balance.manage'
  )
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- ============================================
-- Create HR_ADMIN role for dedicated HR staff
-- ============================================

INSERT INTO roles (id, code, name, created_at, updated_at)
VALUES (gen_random_uuid(), 'hr_admin', '人資管理員', NOW(), NOW())
ON CONFLICT (code) DO NOTHING;

-- Assign all HR permissions to HR_ADMIN
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM roles r, permissions p 
WHERE r.code = 'hr_admin' AND p.code LIKE 'hr.%'
ON CONFLICT (role_id, permission_id) DO NOTHING;
