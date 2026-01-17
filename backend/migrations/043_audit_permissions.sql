-- Audit Trail Permissions
-- Version: 1.0
-- Created: 2026-01-17
-- Description: Add permissions for admin-only audit trail access

-- ============================================
-- Add Audit Permissions
-- ============================================

INSERT INTO permissions (id, code, name, created_at) VALUES
    -- View audit logs
    (gen_random_uuid(), 'admin.audit.view', '檢視審計日誌', NOW()),
    (gen_random_uuid(), 'admin.audit.view.activities', '檢視使用者活動記錄', NOW()),
    (gen_random_uuid(), 'admin.audit.view.logins', '檢視登入記錄', NOW()),
    (gen_random_uuid(), 'admin.audit.view.sessions', '檢視使用者連線', NOW()),
    
    -- Export audit logs
    (gen_random_uuid(), 'admin.audit.export', '匯出審計日誌', NOW()),
    
    -- Session management
    (gen_random_uuid(), 'admin.audit.force_logout', '強制登出使用者', NOW()),
    
    -- Security alerts
    (gen_random_uuid(), 'admin.audit.alerts.view', '檢視安全警報', NOW()),
    (gen_random_uuid(), 'admin.audit.alerts.resolve', '處理安全警報', NOW()),
    
    -- Dashboard
    (gen_random_uuid(), 'admin.audit.dashboard', '檢視審計儀表板', NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- Assign to SYSTEM_ADMIN role
-- ============================================

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM roles r, permissions p 
WHERE r.code = 'admin' 
  AND p.code LIKE 'admin.audit.%'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- ============================================
-- Also assign to PROGRAM_ADMIN role
-- ============================================

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM roles r, permissions p 
WHERE r.code = 'program_admin' 
  AND p.code LIKE 'admin.audit.%'
ON CONFLICT (role_id, permission_id) DO NOTHING;
