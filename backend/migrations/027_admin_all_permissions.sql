-- ============================================
-- Migration 027: Ensure Admin Has All Permissions
-- 
-- 確保 admin 角色擁有資料庫中所有現有權限
-- 這個遷移會在每次新增權限後確保 admin 仍擁有完整權限
-- ============================================

-- 為 admin 角色指派所有現有權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'admin'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 同時也為 SYSTEM_ADMIN 角色（如果存在）指派所有權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'SYSTEM_ADMIN'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 顯示結果
DO $$
DECLARE
    admin_permission_count INTEGER;
    total_permission_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO total_permission_count FROM permissions;
    
    SELECT COUNT(*) INTO admin_permission_count 
    FROM role_permissions rp
    JOIN roles r ON rp.role_id = r.id
    WHERE r.code = 'admin';
    
    RAISE NOTICE 'Admin role now has % out of % total permissions', admin_permission_count, total_permission_count;
END $$;
