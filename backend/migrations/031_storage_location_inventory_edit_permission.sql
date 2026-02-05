-- =============================================================================
-- Migration 031: Storage Location Inventory Edit Permission
-- 新增儲位庫存直接編輯權限（僅管理員）
-- =============================================================================

-- 新增權限
INSERT INTO permissions (id, code, name, module, description, created_at)
VALUES (
    gen_random_uuid(),
    'erp.storage_location.inventory.edit',
    '儲位庫存直接編輯',
    'erp',
    '允許直接修改儲位中的庫存數量（不透過單據）',
    NOW()
) ON CONFLICT (code) DO NOTHING;

-- 將此權限授予 SYSTEM_ADMIN 角色
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'SYSTEM_ADMIN'
  AND p.code = 'erp.storage_location.inventory.edit'
ON CONFLICT (role_id, permission_id) DO NOTHING;
