-- 015: 電子簽章與記錄附註系統
-- 建立 electronic_signatures 和 record_annotations 表，含手寫簽名欄位

-- ============================================
-- 電子簽章表
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

-- 索引
CREATE INDEX IF NOT EXISTS idx_esig_entity
    ON electronic_signatures (entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_esig_signer
    ON electronic_signatures (signer_id);
CREATE INDEX IF NOT EXISTS idx_esig_signed_at
    ON electronic_signatures (signed_at DESC);

-- 欄位註解
COMMENT ON TABLE electronic_signatures IS '電子簽章記錄（GLP 合規）';
COMMENT ON COLUMN electronic_signatures.handwriting_svg IS '手寫簽名 SVG 向量圖';
COMMENT ON COLUMN electronic_signatures.stroke_data IS '手寫簽名原始筆跡點 JSON（含座標、壓力、時間戳記）';
COMMENT ON COLUMN electronic_signatures.signature_method IS '簽章方法：password（密碼驗證）或 handwriting（手寫簽名）';

-- ============================================
-- 記錄附註表
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

-- 索引
CREATE INDEX IF NOT EXISTS idx_annot_record
    ON record_annotations (record_type, record_id);
CREATE INDEX IF NOT EXISTS idx_annot_created_by
    ON record_annotations (created_by);

COMMENT ON TABLE record_annotations IS '記錄附註（含更正需簽章機制）';
