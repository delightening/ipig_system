-- 027: 權限清理 — 移除孤兒 species.* 角色綁定
--
-- 背景：species.read / species.create / species.update 在 migration 002 中被分配給
-- EXPERIMENT_STAFF 和 INTERN，但 species handler 實際上使用 facility.manage 作為
-- 寫入權限。這些 species.* 對應的 require_permission! 從未存在，屬於孤兒分配。
--
-- 決策（2026-04-16）：物種的新增/編輯/刪除由 admin 管理（facility.manage）；
-- 試驗工作人員透過 animal.animal.edit 在動物上設定 species_id tag，不需要 species.*。

DELETE FROM role_permissions
WHERE role_id IN (
    SELECT id FROM roles WHERE code IN ('EXPERIMENT_STAFF', 'INTERN')
)
AND permission_id IN (
    SELECT id FROM permissions WHERE code IN ('species.read', 'species.create', 'species.update')
);
