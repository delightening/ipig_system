-- ============================================
-- 遷移 010: 為 IACUC_STAFF 角色分配 AUP 相關權限
-- 確保執行秘書可以查看和管理所有計畫
-- ============================================

-- 為 IACUC_STAFF 角色分配 AUP 計畫管理權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'IACUC_STAFF'
AND p.code IN (
    -- 計畫管理權限
    'aup.protocol.view_all',      -- 查看所有計畫
    'aup.protocol.view_own',       -- 查看自己的計畫
    'aup.protocol.create',         -- 建立計畫
    'aup.protocol.edit',           -- 編輯計畫
    'aup.protocol.submit',         -- 提交計畫
    'aup.protocol.change_status',  -- 變更狀態
    -- 審查流程權限
    'aup.review.view',             -- 查看審查意見
    'aup.review.assign',           -- 指派審查人員
    'aup.review.comment',          -- 新增審查意見
    'aup.review.edit',             -- 編輯審查意見
    'aup.review.delete',           -- 刪除審查意見
    'aup.protocol.review',         -- 審查計畫
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

-- 為 CHAIR 角色分配查看所有計畫的權限（如果還沒有）
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'CHAIR'
AND p.code = 'aup.protocol.view_all'
ON CONFLICT DO NOTHING;
