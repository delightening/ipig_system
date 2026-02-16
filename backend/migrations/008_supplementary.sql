-- ============================================
-- Migration 008: 補充模組
-- 
-- 包含：
-- - 通知路由規則表 + Seed
-- - 電子簽章表
-- - 記錄附註表
-- 
-- 編碼: UTF-8 (無 BOM)
-- ============================================

-- ============================================
-- 1. 通知路由規則表
-- ============================================

CREATE TABLE notification_routing (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(80) NOT NULL,                          -- 事件識別碼
    role_code VARCHAR(50) NOT NULL REFERENCES roles(code),    -- 對應 roles.code
    channel VARCHAR(20) NOT NULL DEFAULT 'in_app',            -- 'in_app' | 'email' | 'both'
    is_active BOOLEAN NOT NULL DEFAULT true,
    description TEXT,                                          -- 管理者可讀說明
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (event_type, role_code),
    CONSTRAINT chk_channel CHECK (channel IN ('in_app', 'email', 'both'))
);

CREATE INDEX idx_notification_routing_event ON notification_routing(event_type, is_active);

-- ============================================
-- 2. 通知路由 Seed 資料
-- ============================================

INSERT INTO notification_routing (event_type, role_code, channel, description) VALUES
    -- AUP 計畫生命週期
    ('protocol_submitted',               'IACUC_STAFF',       'in_app', '計畫提交 → 通知執行秘書'),
    ('protocol_vet_review',              'VET',               'in_app', '進入獸醫審查 → 通知獸醫師'),
    ('protocol_under_review',            'IACUC_STAFF',       'in_app', '進入委員審查 → 通知執行秘書'),
    ('protocol_resubmitted',             'IACUC_STAFF',       'in_app', '重新提交 → 通知執行秘書'),
    ('protocol_approved',                'IACUC_CHAIR',       'both',   '計畫核准 → 通知 IACUC 主席'),
    ('protocol_rejected',                'IACUC_CHAIR',       'both',   '計畫駁回 → 通知 IACUC 主席'),
    -- 審查意見
    ('review_comment_created',           'IACUC_STAFF',       'in_app', '新審查意見 → 通知執行秘書'),
    -- HR 請假/加班
    ('leave_submitted',                  'ADMIN_STAFF',       'in_app', '請假申請 → 通知行政人員'),
    ('leave_submitted',                  'admin',             'in_app', '請假申請 → 通知系統管理員'),
    ('overtime_submitted',               'ADMIN_STAFF',       'in_app', '加班申請 → 通知行政人員'),
    ('overtime_submitted',               'admin',             'in_app', '加班申請 → 通知系統管理員'),
    -- ERP 採購單
    ('document_submitted',               'WAREHOUSE_MANAGER', 'in_app', '採購單提交 → 通知倉庫管理員'),
    -- 庫存預警
    ('low_stock_alert',                  'admin',             'in_app', '低庫存預警 → 通知系統管理員'),
    ('low_stock_alert',                  'WAREHOUSE_MANAGER', 'in_app', '低庫存預警 → 通知倉庫管理員'),
    ('expiry_alert',                     'admin',             'in_app', '效期預警 → 通知系統管理員'),
    ('expiry_alert',                     'WAREHOUSE_MANAGER', 'in_app', '效期預警 → 通知倉庫管理員'),
    -- 緊急給藥
    ('emergency_medication',             'VET',               'in_app', '緊急給藥 → 通知獸醫師'),
    -- 修正案
    ('amendment_submitted',              'IACUC_STAFF',       'in_app', '修正案提交 → 通知執行秘書'),
    ('amendment_decision_recorded',      'IACUC_STAFF',       'in_app', '修正案審查決定 → 通知執行秘書'),
    ('amendment_approved',               'IACUC_CHAIR',       'both',   '修正案核准 → 通知 IACUC 主席'),
    ('amendment_rejected',               'IACUC_CHAIR',       'both',   '修正案駁回 → 通知 IACUC 主席')
ON CONFLICT (event_type, role_code) DO NOTHING;

-- ============================================
-- 3. 電子簽章表
-- ============================================

CREATE TABLE IF NOT EXISTS electronic_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL,          -- 簽章對象類型（如 sacrifice, surgery）
    entity_id VARCHAR(100) NOT NULL,           -- 簽章對象 ID
    signer_id UUID NOT NULL REFERENCES users(id),
    signature_type VARCHAR(20) NOT NULL,       -- APPROVE / CONFIRM / WITNESS
    content_hash VARCHAR(64) NOT NULL,         -- SHA-256 內容雜湊
    signature_data VARCHAR(128) NOT NULL,      -- 簽章資料（雜湊值）
    ip_address VARCHAR(45),                    -- 簽章者 IP
    user_agent TEXT,                           -- 瀏覽器 User-Agent
    signed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_valid BOOLEAN NOT NULL DEFAULT true,
    invalidated_reason TEXT,
    invalidated_at TIMESTAMPTZ,
    invalidated_by UUID REFERENCES users(id),
    -- 手寫簽名欄位
    handwriting_svg TEXT,                      -- SVG 向量簽名影像
    stroke_data JSONB,                         -- 原始筆跡點資料（防偽用）
    signature_method VARCHAR(20) DEFAULT 'password'  -- password / handwriting
);

CREATE INDEX IF NOT EXISTS idx_esig_entity
    ON electronic_signatures (entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_esig_signer
    ON electronic_signatures (signer_id);
CREATE INDEX IF NOT EXISTS idx_esig_signed_at
    ON electronic_signatures (signed_at DESC);

COMMENT ON TABLE electronic_signatures IS '電子簽章記錄（GLP 合規）';
COMMENT ON COLUMN electronic_signatures.handwriting_svg IS '手寫簽名 SVG 向量圖';
COMMENT ON COLUMN electronic_signatures.stroke_data IS '手寫簽名原始筆跡點 JSON（含座標、壓力、時間戳記）';
COMMENT ON COLUMN electronic_signatures.signature_method IS '簽章方法：password（密碼驗證）或 handwriting（手寫簽名）';

-- ============================================
-- 4. 記錄附註表
-- ============================================

CREATE TABLE IF NOT EXISTS record_annotations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    record_type VARCHAR(50) NOT NULL,          -- 記錄類型（observation, surgery, sacrifice）
    record_id INTEGER NOT NULL,                -- 記錄 ID
    annotation_type VARCHAR(20) NOT NULL,      -- NOTE / CORRECTION / ADDENDUM
    content TEXT NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    signature_id UUID REFERENCES electronic_signatures(id)
);

CREATE INDEX IF NOT EXISTS idx_annot_record
    ON record_annotations (record_type, record_id);
CREATE INDEX IF NOT EXISTS idx_annot_created_by
    ON record_annotations (created_by);

COMMENT ON TABLE record_annotations IS '記錄附註（含更正需簽章機制）';

-- ============================================
-- 完成
-- ============================================
