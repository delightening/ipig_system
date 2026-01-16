-- ============================================
-- 遷移 039: 為試驗工作人員(EXPERIMENT_STAFF)分配 co-editor 相關權限
-- 說明: 
--   1. 擴展 review_comments 表支援回覆功能
--   2. 新增回覆審查意見權限 (aup.review.reply)
--   3. 為 EXPERIMENT_STAFF 分配協議相關權限（當作為 co-editor 時使用）
--   4. 為 PI 分配回覆審查意見權限
-- ============================================

-- ============================================
-- 1. 擴展 review_comments 表支援回覆功能
-- ============================================

-- 1.1 新增 parent_comment_id 欄位（用於回覆）
ALTER TABLE review_comments 
    ADD COLUMN IF NOT EXISTS parent_comment_id UUID REFERENCES review_comments(id) ON DELETE CASCADE,
    ADD COLUMN IF NOT EXISTS replied_by UUID REFERENCES users(id);

-- 1.2 新增索引以提升查詢效能
CREATE INDEX IF NOT EXISTS idx_review_comments_parent_id ON review_comments(parent_comment_id);
CREATE INDEX IF NOT EXISTS idx_review_comments_replied_by ON review_comments(replied_by);

-- 1.3 添加註解
COMMENT ON COLUMN review_comments.parent_comment_id IS '父審查意見 ID（NULL 表示原始意見，非 NULL 表示回覆）';
COMMENT ON COLUMN review_comments.replied_by IS '回覆者 ID（僅回覆時使用）';

-- ============================================
-- 2. 新增回覆審查意見權限
-- ============================================

INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'aup.review.reply', '回覆審查意見', 'aup', '可回覆審查委員的意見（PI 或 co-editor）', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

-- ============================================
-- 3. 為 PI 角色分配回覆審查意見權限
-- ============================================

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'PI'
AND p.code = 'aup.review.reply'
ON CONFLICT DO NOTHING;

-- ============================================
-- 4. 為 EXPERIMENT_STAFF 分配協議相關權限
--    注意：這些權限主要用於當 EXPERIMENT_STAFF 被指派為 co-editor 時
--    實際的權限檢查會結合 user_protocols 表中的 role_in_protocol = 'CO_EDITOR'
-- ============================================

-- 4.1 計畫基本權限（用於 co-editor 時）
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'EXPERIMENT_STAFF'
AND p.code IN (
    -- 計畫查看權限（查看指派給自己的計畫）
    'aup.protocol.view_own',       -- 查看自己的計畫（作為 co-editor）
    
    -- 計畫管理權限（作為 co-editor 時）
    'aup.protocol.create',         -- 建立計畫
    'aup.protocol.edit',           -- 編輯計畫（作為 co-editor）
    'aup.protocol.delete',         -- 刪除計畫（作為 co-editor）
    'aup.protocol.submit',         -- 提交計畫（作為 co-editor）
    
    -- 審查流程權限
    'aup.review.view',             -- 查看審查意見
    'aup.review.reply',            -- 回覆審查意見（PI 或 co-editor）
    
    -- 附件管理權限
    'aup.attachment.upload',       -- 上傳附件
    'aup.attachment.view',         -- 查看附件
    'aup.attachment.download',     -- 下載附件
    'aup.attachment.delete',       -- 刪除附件
    
    -- 版本管理權限
    'aup.version.view'             -- 查看版本歷史
)
ON CONFLICT DO NOTHING;

-- ============================================
-- 5. 驗證 EXPERIMENT_STAFF 豬隻權限完整性
--    確保試驗工作人員擁有所有必需的豬隻管理權限
-- ============================================

-- 檢查並補充缺失的豬隻權限（如果有的話）
-- 注意：大部分權限應已在 migration 013 中分配

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'EXPERIMENT_STAFF'
AND p.code IN (
    -- 使用新的權限代碼格式（確保向後兼容）
    'pig.pig.view_all',            -- 查看所有豬隻
    'pig.pig.view_project',        -- 查看計畫內豬隻
    'pig.pig.create',              -- 新增豬隻
    'pig.pig.edit',                -- 編輯豬隻資料
    'pig.pig.delete',              -- 刪除豬隻
    'pig.pig.import',              -- 匯入豬隻資料
    'pig.pig.export',              -- 匯出豬隻資料
    
    -- 紀錄管理
    'pig.record.view',             -- 查看紀錄
    'pig.record.create',           -- 新增紀錄
    'pig.record.edit',             -- 編輯紀錄
    'pig.record.delete',           -- 刪除紀錄
    'pig.record.copy',             -- 複製紀錄
    'pig.record.observation',      -- 觀察紀錄
    'pig.record.surgery',          -- 手術紀錄
    'pig.record.weight',           -- 體重紀錄
    'pig.record.vaccine',          -- 疫苗紀錄
    'pig.record.sacrifice',        -- 犧牲紀錄
    
    -- 匯出功能
    'pig.export.surgery',          -- 匯出手術紀錄
    'pig.export.all',              -- 匯出所有資料
    'pig.export.medical',          -- 匯出病歷
    'pig.export.experiment',       -- 匯出試驗紀錄
    'pig.export.observation',      -- 匯出觀察紀錄
    
    -- 病理報告
    'pig.pathology.upload',        -- 上傳病理報告
    'pig.pathology.view',          -- 查看病理報告
    'pig.pathology.delete'         -- 刪除病理報告
)
ON CONFLICT DO NOTHING;

-- 同時確保舊的 animal.* 權限代碼也存在（向後兼容）
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'EXPERIMENT_STAFF'
AND p.code IN (
    -- 舊的權限代碼（向後兼容）
    'animal.pig.view_all',
    'animal.pig.view_project',
    'animal.pig.create',
    'animal.pig.edit',
    'animal.pig.delete',
    'animal.pig.import',
    'animal.pig.export',
    'animal.record.view',
    'animal.record.create',
    'animal.record.edit',
    'animal.record.delete',
    'animal.record.copy',
    'animal.record.observation',
    'animal.record.surgery',
    'animal.record.weight',
    'animal.record.vaccine',
    'animal.record.sacrifice',
    'animal.export.surgery',
    'animal.export.all',
    'animal.export.medical',
    'animal.export.experiment',
    'animal.export.observation',
    'animal.pathology.upload',
    'animal.pathology.view',
    'animal.pathology.delete'
)
ON CONFLICT DO NOTHING;

-- ============================================
-- 6. 說明與注意事項
-- ============================================

-- 已完成的權限分配：
-- 
-- EXPERIMENT_STAFF (試驗工作人員) 作為 co-editor 時：
--   ✅ AUP 協議: 建立、編輯、刪除、提交、查看、附件管理、版本歷史
--   ✅ 審查流程: 查看審查意見、回覆審查意見
--   ✅ 豬隻管理: 完整權限（查看、新增、編輯、刪除、匯入、匯出）
--   ✅ 紀錄管理: 完整權限（查看、新增、編輯、刪除、複製、各類紀錄）
--   ✅ 匯出功能: 手術紀錄、所有資料、病歷、試驗紀錄、觀察紀錄
--   ✅ 病理報告: 上傳、查看、刪除
-- 
-- 回覆審查意見功能：
--   ✅ 已新增 parent_comment_id 和 replied_by 欄位到 review_comments 表
--   ✅ 已新增 aup.review.reply 權限
--   ✅ PI 和 EXPERIMENT_STAFF（作為 co-editor）均可回覆審查意見
-- 
-- 注意：
--   1. EXPERIMENT_STAFF 的協議權限主要用於當他們被指派為 co-editor 時
--   2. 實際的協議權限檢查會結合 user_protocols 表（role_in_protocol = 'CO_EDITOR'）
--   3. 權限檢查邏輯需要在 handlers 中確保：
--      - co-editor 可以查看、編輯、刪除、提交他們被指派的協議
--      - co-editor 可以回覆審查意見
--   4. Chair、審查人員、執行秘書、委託單位代表人、PI 或 co-editor 均可查看回覆內容
--   5. 回覆內容會記錄 replied_by，以便識別是誰回覆的
