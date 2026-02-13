-- ============================================
-- Migration 008: AUP 審查增強 + Protocol 活動記錄
-- 
-- 合併自原 011+012+013+014+015+017：
-- - 系統設定表
-- - 獸醫審查指派表（含審查表欄位）
-- - Protocol 活動歷程表
-- - 審查往返歷史記錄表
-- 
-- 注意：Enum 擴展與欄位修改已回併至 001/004
-- 編碼: UTF-8 (無 BOM)
-- ============================================

-- ============================================
-- 1. 系統設定表
-- ============================================

CREATE TABLE IF NOT EXISTS system_settings (
    key VARCHAR(100) PRIMARY KEY,
    value JSONB NOT NULL,
    description TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID REFERENCES users(id)
);

-- 預設獸醫師設定
INSERT INTO system_settings (key, value, description) VALUES 
('default_vet_reviewer', '{"user_id": null}', '預設獸醫審查員，VET_REVIEW 階段會自動指派此獸醫師')
ON CONFLICT (key) DO NOTHING;

-- ============================================
-- 2. 獸醫審查指派表
-- ============================================

CREATE TABLE IF NOT EXISTS vet_review_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    vet_id UUID NOT NULL REFERENCES users(id),
    assigned_by UUID REFERENCES users(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    decision VARCHAR(20),
    decision_remark TEXT,
    -- 獸醫師審查表（原 017）
    review_form JSONB,
    UNIQUE (protocol_id)
);

CREATE INDEX IF NOT EXISTS idx_vet_review_assignments_protocol ON vet_review_assignments(protocol_id);
CREATE INDEX IF NOT EXISTS idx_vet_review_assignments_vet ON vet_review_assignments(vet_id);

COMMENT ON COLUMN vet_review_assignments.review_form IS '獸醫師 12 項查檢表資料 (JSON 格式)';

-- ============================================
-- 3. Protocol 活動歷程表
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
    from_value TEXT,
    to_value TEXT,
    target_entity_type VARCHAR(50),
    target_entity_id UUID,
    target_entity_name VARCHAR(255),
    
    -- 備註
    remark TEXT,
    
    -- 額外資料
    extra_data JSONB,
    
    -- 時間戳
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_protocol_activities_protocol_id ON protocol_activities(protocol_id, created_at DESC);
CREATE INDEX idx_protocol_activities_actor_id ON protocol_activities(actor_id);
CREATE INDEX idx_protocol_activities_type ON protocol_activities(activity_type, created_at DESC);

COMMENT ON TABLE protocol_activities IS 'Protocol 專屬活動歷程表，記錄所有對計畫的操作行為';
COMMENT ON COLUMN protocol_activities.activity_type IS '活動類型（見 protocol_activity_type ENUM）';
COMMENT ON COLUMN protocol_activities.actor_id IS '執行操作的使用者 ID';
COMMENT ON COLUMN protocol_activities.from_value IS '變更前的值（如狀態變更）';
COMMENT ON COLUMN protocol_activities.to_value IS '變更後的值';
COMMENT ON COLUMN protocol_activities.target_entity_type IS '目標實體類型（如 reviewer, attachment）';
COMMENT ON COLUMN protocol_activities.extra_data IS '額外資料（JSON 格式）';

-- ============================================
-- 4. 審查往返歷史記錄表
-- ============================================

CREATE TABLE IF NOT EXISTS review_round_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    review_stage VARCHAR(30) NOT NULL,
    round_number INTEGER NOT NULL DEFAULT 1,
    action VARCHAR(30) NOT NULL,
    actor_id UUID NOT NULL REFERENCES users(id),
    remark TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_review_round_history_protocol ON review_round_history(protocol_id);
CREATE INDEX IF NOT EXISTS idx_review_round_history_stage ON review_round_history(review_stage);

-- ============================================
-- 5. 新增權限定義（原 020）
-- ============================================

INSERT INTO permissions (id, code, name, module, description, created_at)
VALUES
    (gen_random_uuid(), 'animal.pathology.view', '查看病理報告', 'animal', '可查看動物病理報告', NOW()),
    (gen_random_uuid(), 'animal.pathology.upload', '上傳病理報告', 'animal', '可上傳動物病理報告', NOW()),
    (gen_random_uuid(), 'animal.info.assign', '分配動物資訊', 'animal', '可分配動物至計畫', NOW()),
    (gen_random_uuid(), 'animal.info.edit', '編輯動物資訊', 'animal', '可編輯動物資訊', NOW()),
    (gen_random_uuid(), 'animal.record.emergency', '緊急給藥', 'animal', '可執行緊急給藥', NOW()),
    (gen_random_uuid(), 'animal.record.copy', '複製紀錄', 'animal', '可複製動物紀錄', NOW()),
    (gen_random_uuid(), 'animal.vet.upload_attachment', '上傳獸醫附件', 'animal', '可上傳獸醫建議附件', NOW())
ON CONFLICT (code) DO NOTHING;

-- 分配權限給 EXPERIMENT_STAFF 角色
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'EXPERIMENT_STAFF'
AND p.code IN (
    'animal.pathology.view',
    'animal.pathology.upload',
    'animal.info.assign',
    'animal.info.edit',
    'animal.record.emergency',
    'animal.record.copy',
    'animal.vet.upload_attachment'
)
ON CONFLICT DO NOTHING;

-- 分配權限給 VET 角色
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'VET'
AND p.code IN (
    'animal.pathology.view',
    'animal.pathology.upload',
    'animal.info.assign',
    'animal.info.edit',
    'animal.record.emergency',
    'animal.record.copy',
    'animal.vet.upload_attachment'
)
ON CONFLICT DO NOTHING;

-- ============================================
-- 完成
-- ============================================
