-- ============================================
-- Migration 014: Protocol Activities
-- 
-- 建立 Protocol 專屬活動歷程表
-- 取代原有的 protocol_status_history 表
-- 
-- 編碼: UTF-8 (無 BOM)
-- ============================================

-- ============================================
-- 1. 建立活動類型 ENUM
-- ============================================

CREATE TYPE protocol_activity_type AS ENUM (
    -- 生命週期
    'CREATED',                   -- PI 創建計畫
    'UPDATED',                   -- 編輯修改
    'SUBMITTED',                 -- 送審
    'RESUBMITTED',               -- 重送
    'APPROVED',                  -- 通過
    'APPROVED_WITH_CONDITIONS',  -- 附條件通過
    'CLOSED',                    -- 結案
    'REJECTED',                  -- 否決
    'SUSPENDED',                 -- 暫停
    'DELETED',                   -- 刪除
    
    -- 審查流程
    'STATUS_CHANGED',            -- 狀態變更（其他一般狀態轉換）
    'REVIEWER_ASSIGNED',         -- 指派審查委員
    'VET_ASSIGNED',              -- 指派獸醫師
    'COEDITOR_ASSIGNED',         -- 指派共同編輯者
    'COEDITOR_REMOVED',          -- 移除共同編輯者
    
    -- 審查意見
    'COMMENT_ADDED',             -- 新增審查意見
    'COMMENT_REPLIED',           -- 回覆審查意見
    'COMMENT_RESOLVED',          -- 解決審查意見
    
    -- 附件
    'ATTACHMENT_UPLOADED',       -- 上傳附件
    'ATTACHMENT_DELETED',        -- 刪除附件
    
    -- 版本
    'VERSION_CREATED',           -- 建立版本快照
    'VERSION_RECOVERED',         -- 回復至版本
    
    -- 修正案
    'AMENDMENT_CREATED',         -- 建立修正案
    'AMENDMENT_SUBMITTED',       -- 送審修正案
    
    -- 動物管理
    'PIG_ASSIGNED',              -- 動物分配至本計畫
    'PIG_UNASSIGNED'             -- 動物移出本計畫
);

-- ============================================
-- 2. 建立 Protocol 活動歷程表
-- ============================================

CREATE TABLE protocol_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    activity_type protocol_activity_type NOT NULL,
    
    -- 行為者
    actor_id UUID NOT NULL REFERENCES users(id),
    actor_name VARCHAR(100),
    actor_email VARCHAR(255),
    
    -- 變更內容
    from_value TEXT,                    -- 變更前的值（如狀態）
    to_value TEXT,                      -- 變更後的值
    target_entity_type VARCHAR(50),     -- 目標實體類型 (reviewer, attachment, comment...)
    target_entity_id UUID,              -- 目標實體 ID
    target_entity_name VARCHAR(255),    -- 目標實體名稱
    
    -- 備註
    remark TEXT,
    
    -- 額外資料（JSON 格式儲存複雜變更）
    extra_data JSONB,
    
    -- 時間戳
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================
-- 3. 建立索引
-- ============================================

-- 主要查詢：依 protocol_id 查詢活動歷程
CREATE INDEX idx_protocol_activities_protocol_id ON protocol_activities(protocol_id, created_at DESC);

-- 依行為者查詢
CREATE INDEX idx_protocol_activities_actor_id ON protocol_activities(actor_id);

-- 依活動類型查詢
CREATE INDEX idx_protocol_activities_type ON protocol_activities(activity_type, created_at DESC);

-- ============================================
-- 4. 遷移現有資料
-- ============================================

-- 將 protocol_status_history 資料遷移至 protocol_activities
INSERT INTO protocol_activities (
    protocol_id, 
    activity_type, 
    actor_id, 
    actor_name, 
    actor_email,
    from_value, 
    to_value, 
    remark, 
    created_at
)
SELECT 
    psh.protocol_id,
    'STATUS_CHANGED'::protocol_activity_type,
    psh.changed_by,
    COALESCE(u.display_name, u.email),
    u.email,
    psh.from_status::text,
    psh.to_status::text,
    psh.remark,
    psh.created_at
FROM protocol_status_history psh
JOIN users u ON u.id = psh.changed_by;

-- ============================================
-- 5. 刪除舊表
-- ============================================

DROP INDEX IF EXISTS idx_protocol_status_history_protocol_id;
DROP TABLE IF EXISTS protocol_status_history;

-- ============================================
-- 6. 新增說明
-- ============================================

COMMENT ON TABLE protocol_activities IS 'Protocol 專屬活動歷程表，記錄所有對計畫的操作行為';
COMMENT ON COLUMN protocol_activities.activity_type IS '活動類型（見 protocol_activity_type ENUM）';
COMMENT ON COLUMN protocol_activities.actor_id IS '執行操作的使用者 ID';
COMMENT ON COLUMN protocol_activities.from_value IS '變更前的值（如狀態變更）';
COMMENT ON COLUMN protocol_activities.to_value IS '變更後的值';
COMMENT ON COLUMN protocol_activities.target_entity_type IS '目標實體類型（如 reviewer, attachment）';
COMMENT ON COLUMN protocol_activities.extra_data IS '額外資料（JSON 格式）';

-- ============================================
-- 完成
-- ============================================
