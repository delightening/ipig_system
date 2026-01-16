-- ============================================
-- 遷移 035: 為 PI 和 CLIENT 角色分配權限，並擴展 protocol_role 支援 CO_EDITOR
-- 說明: 
--   1. 擴展 protocol_role enum 支援 CO_EDITOR
--   2. 為 PI 角色分配完整的 AUP 協議權限和豬隻管理權限
--   3. 為 CLIENT 角色分配 AUP 協議查看權限和豬隻管理查看權限
-- ============================================

-- ============================================
-- 1. 擴展 protocol_role enum 支援 CO_EDITOR
-- ============================================

-- 1.1 備份現有數據（如果有需要）
-- 注意：PostgreSQL 不支援直接修改 ENUM，需要先新增值
ALTER TYPE protocol_role ADD VALUE IF NOT EXISTS 'CO_EDITOR';

-- ============================================
-- 2. 為 PI 角色分配 AUP 協議權限
-- ============================================

-- 2.1 計畫管理權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'PI'
AND p.code IN (
    -- 計畫管理權限
    'aup.protocol.view_own',       -- 查看自己的計畫
    'aup.protocol.create',         -- 建立計畫
    'aup.protocol.edit',           -- 編輯計畫
    'aup.protocol.delete',         -- 刪除計畫
    'aup.protocol.submit',         -- 提交計畫
    -- 審查流程權限
    'aup.review.view',             -- 查看審查意見
    -- 附件管理權限
    'aup.attachment.upload',       -- 上傳附件
    'aup.attachment.view',         -- 查看附件
    'aup.attachment.download',     -- 下載附件
    'aup.attachment.delete',       -- 刪除附件
    -- 版本管理權限
    'aup.version.view',            -- 查看版本歷史
    'aup.version.restore'          -- 還原版本
)
ON CONFLICT DO NOTHING;

-- ============================================
-- 3. 為 CLIENT 角色分配 AUP 協議權限（僅查看和附件）
-- ============================================

-- 3.1 計畫查看和附件管理權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'CLIENT'
AND p.code IN (
    -- 計畫管理權限（僅查看）
    'aup.protocol.view_own',       -- 查看自己的計畫
    -- 審查流程權限
    'aup.review.view',             -- 查看審查意見
    -- 附件管理權限
    'aup.attachment.upload',       -- 上傳附件
    'aup.attachment.view',         -- 查看附件
    'aup.attachment.download',     -- 下載附件
    'aup.attachment.delete',       -- 刪除附件
    -- 版本管理權限
    'aup.version.view'             -- 查看版本歷史
    -- 注意：CLIENT 不應有 submit, edit, delete, restore 權限
)
ON CONFLICT DO NOTHING;

-- ============================================
-- 4. 為 PI 角色分配豬隻管理權限
-- ============================================

-- 4.1 豬隻查看和紀錄權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'PI'
AND p.code IN (
    -- 豬隻基本操作
    'pig.pig.view_project',        -- 查看計畫內豬隻
    'pig.pig.export',              -- 匯出豬隻資料
    -- 紀錄管理
    'pig.record.view',             -- 查看紀錄
    'pig.record.observation',      -- 觀察紀錄
    'pig.record.surgery',          -- 手術紀錄
    'pig.record.weight',           -- 體重紀錄
    'pig.record.vaccine',          -- 疫苗紀錄
    'pig.record.sacrifice',        -- 犧牲紀錄
    -- 匯出功能
    'pig.export.experiment',       -- 匯出試驗紀錄
    -- 病理報告
    'pig.pathology.view'           -- 查看病理報告
)
ON CONFLICT DO NOTHING;

-- ============================================
-- 5. 為 CLIENT 角色分配豬隻管理權限（僅查看）
-- ============================================

-- 5.1 豬隻查看和匯出權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'CLIENT'
AND p.code IN (
    -- 豬隻基本操作（僅查看）
    'pig.pig.view_project',        -- 查看計畫內豬隻
    'pig.pig.export',              -- 匯出豬隻資料
    -- 紀錄管理（僅查看）
    'pig.record.view',             -- 查看紀錄
    -- 匯出功能
    'pig.export.experiment',       -- 匯出試驗紀錄
    -- 病理報告
    'pig.pathology.view'           -- 查看病理報告
    -- 注意：CLIENT 不應有 create, edit, delete 紀錄的權限
)
ON CONFLICT DO NOTHING;

-- ============================================
-- 6. 說明與注意事項
-- ============================================

-- 已完成的權限分配：
-- 
-- PI (計畫主持人):
--   ✅ AUP 協議: 建立、編輯、刪除、提交、查看、附件管理、版本管理
--   ✅ 豬隻管理: 查看計畫內豬隻、查看各種紀錄、匯出功能、病理報告
-- 
-- CLIENT (委託人):
--   ✅ AUP 協議: 查看自己的計畫、查看審查意見、附件管理、查看版本歷史
--   ✅ 豬隻管理: 查看計畫內豬隻、查看紀錄、匯出功能、病理報告
-- 
-- protocol_role enum:
--   ✅ 已新增 CO_EDITOR 值，可用於指派 EXPERIMENT_STAFF 為 co-editor
-- 
-- 注意：
--   1. PI 和 CLIENT 的權限分配已完成，但委託單位主管的特殊權限
--      （可見轄下所有人員之計畫與豬隻）需要在應用程式邏輯層實現
--   2. CO_EDITOR 機制需要在前端和後端 API 中實現指派功能
--   3. EXPERIMENT_STAFF 作為 co-editor 時的權限應透過 user_protocols 
--      表檢查，而非全域角色權限
