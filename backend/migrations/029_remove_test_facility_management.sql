-- 029: 移除 TEST_FACILITY_MANAGEMENT 角色
--
-- 決策（2026-04-16）：此角色定位等同 admin（機構管理階層需要完整管理權），
-- 維護兩個功能相同的角色無意義。有此需求的人員直接指派 admin 角色。
-- 確認：目前無任何 user_roles 紀錄使用此角色，可直接刪除。

-- 1. 清除此角色的所有權限分配
DELETE FROM role_permissions
WHERE role_id = (SELECT id FROM roles WHERE code = 'TEST_FACILITY_MANAGEMENT');

-- 2. 刪除角色本體（is_system = true 不影響手動刪除）
DELETE FROM roles WHERE code = 'TEST_FACILITY_MANAGEMENT';
