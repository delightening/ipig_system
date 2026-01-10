-- 006_attachments.sql
-- 附件管理系統

-- 建立附件表
CREATE TABLE IF NOT EXISTS attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL,       -- 實體類型: protocol, pig, pathology, vet_recommendation
    entity_id VARCHAR(255) NOT NULL,        -- 實體 ID
    file_name VARCHAR(500) NOT NULL,        -- 原始檔名
    file_path VARCHAR(1000) NOT NULL,       -- 儲存路徑
    file_size BIGINT NOT NULL DEFAULT 0,    -- 檔案大小 (bytes)
    mime_type VARCHAR(255) NOT NULL,        -- MIME 類型
    uploaded_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 建立索引
CREATE INDEX IF NOT EXISTS idx_attachments_entity ON attachments(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_attachments_uploaded_by ON attachments(uploaded_by);
CREATE INDEX IF NOT EXISTS idx_attachments_created_at ON attachments(created_at DESC);

-- 添加註解
COMMENT ON TABLE attachments IS '附件管理表';
COMMENT ON COLUMN attachments.entity_type IS '實體類型 (protocol, pig, pathology, vet_recommendation)';
COMMENT ON COLUMN attachments.entity_id IS '關聯實體 ID';
COMMENT ON COLUMN attachments.file_name IS '原始檔名';
COMMENT ON COLUMN attachments.file_path IS '相對儲存路徑';
COMMENT ON COLUMN attachments.file_size IS '檔案大小 (bytes)';
COMMENT ON COLUMN attachments.mime_type IS 'MIME 類型';
COMMENT ON COLUMN attachments.uploaded_by IS '上傳者';
