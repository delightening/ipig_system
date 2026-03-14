-- 銷貨單（SO/DO）直接關聯計畫（取代手動建立客戶）
-- 進行中（已核准、未結案）之計畫即為銷貨客戶，無需在 partners 表另建客戶記錄

ALTER TABLE documents
    ADD COLUMN protocol_id UUID REFERENCES protocols(id) ON DELETE SET NULL;

CREATE INDEX idx_documents_protocol_id ON documents(protocol_id);
