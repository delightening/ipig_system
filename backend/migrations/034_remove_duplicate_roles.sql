-- ============================================
-- 遷移 034: 刪除重複的角色
-- 說明: 刪除 'warehouse' (倉管人員) 和 'sales' (業務人員) 角色
--       'warehouse' 與 'WAREHOUSE_MANAGER' 重複，已改用 'WAREHOUSE_MANAGER'
--       'sales' 角色不再使用
-- ============================================

-- ============================================
-- 1. 處理使用舊角色的用戶
-- ============================================

-- 1.1 將使用 'warehouse' 角色的用戶遷移到 'WAREHOUSE_MANAGER'
INSERT INTO user_roles (user_id, role_id, assigned_at, assigned_by)
SELECT ur.user_id, r_new.id, NOW(), NULL
FROM user_roles ur
INNER JOIN roles r_old ON ur.role_id = r_old.id AND r_old.code = 'warehouse'
INNER JOIN roles r_new ON r_new.code = 'WAREHOUSE_MANAGER'
WHERE NOT EXISTS (
    -- 避免重複分配
    SELECT 1 FROM user_roles ur2
    INNER JOIN roles r2 ON ur2.role_id = r2.id AND r2.code = 'WAREHOUSE_MANAGER'
    WHERE ur2.user_id = ur.user_id
)
ON CONFLICT DO NOTHING;

-- ============================================
-- 2. 刪除角色權限關聯
-- ============================================

-- 2.1 刪除 'warehouse' 角色的權限關聯
DELETE FROM role_permissions
WHERE role_id IN (SELECT id FROM roles WHERE code = 'warehouse');

-- 2.2 刪除 'sales' 角色的權限關聯
DELETE FROM role_permissions
WHERE role_id IN (SELECT id FROM roles WHERE code = 'sales');

-- ============================================
-- 3. 刪除用戶角色關聯
-- ============================================

-- 3.1 刪除 'warehouse' 角色的用戶關聯（已遷移到 WAREHOUSE_MANAGER）
DELETE FROM user_roles
WHERE role_id IN (SELECT id FROM roles WHERE code = 'warehouse');

-- 3.2 刪除 'sales' 角色的用戶關聯
DELETE FROM user_roles
WHERE role_id IN (SELECT id FROM roles WHERE code = 'sales');

-- ============================================
-- 4. 刪除角色本身
-- ============================================

-- 4.1 刪除 'warehouse' 角色
DELETE FROM roles WHERE code = 'warehouse';

-- 4.2 刪除 'sales' 角色
DELETE FROM roles WHERE code = 'sales';

-- ============================================
-- 5. 記錄變更（可選）
-- ============================================

-- 角色已刪除：
-- - 'warehouse' (倉管人員) -> 已合併到 'WAREHOUSE_MANAGER' (倉庫管理員)
-- - 'sales' (業務人員) -> 已移除（不再使用）
