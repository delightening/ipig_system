-- 028: 動物模組權限修正
--
-- 背景（2026-04-16 權限審查）：
--
-- 1. 孤兒權限清理：
--    - animal.animal.assign：早期設計，已被 animal.info.assign 取代
--      （batch_assign_animals handler 實際使用 animal.info.assign）
--    - animal.info.edit：早期設計，已被 animal.animal.edit 取代
--    兩者不對應任何現有 handler 的 require_permission!，屬於孤兒分配。
--
-- 2. animal.animal.delete 改為 admin 專屬：
--    - 原 migration 002 將此權限分配給 EXPERIMENT_STAFF / INTERN
--    - 決策：刪除動物為高風險操作，僅限系統管理員
--    - admin 透過 ensure_required_permissions() 補回（ON CONFLICT DO NOTHING）
--
-- 3. Handler 命名修正（程式碼同步）：
--    - delete_animal_source: animal.animal.delete → animal.source.manage
--    - create_animal_source: animal.animal.create → animal.source.manage
--    - update_animal_source: animal.animal.edit  → animal.source.manage
--    - delete_animal:        animal.animal.edit  → animal.animal.delete

-- 移除孤兒權限的角色綁定（從所有角色移除）
DELETE FROM role_permissions
WHERE permission_id IN (
    SELECT id FROM permissions WHERE code IN ('animal.animal.assign', 'animal.info.edit')
);

-- 移除 animal.animal.delete 從 EXPERIMENT_STAFF / INTERN
-- （改為 admin 專屬，透過 startup 補回 admin 的分配）
DELETE FROM role_permissions
WHERE role_id IN (
    SELECT id FROM roles WHERE code IN ('EXPERIMENT_STAFF', 'INTERN')
)
AND permission_id IN (
    SELECT id FROM permissions WHERE code = 'animal.animal.delete'
);
