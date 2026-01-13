-- ============================================
-- 遷移 019: 給 admin 角色分配所有權限
-- 確保 admin 角色擁有系統中所有現有和未來的權限
-- ============================================

-- 為 admin 角色分配所有現有權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'admin'
ON CONFLICT (role_id, permission_id) DO NOTHING;
