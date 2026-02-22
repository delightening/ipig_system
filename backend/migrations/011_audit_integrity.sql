-- SEC-34: 稽核日誌防篡改 — 新增 HMAC 完整性雜湊鏈欄位
-- 每筆日誌記錄包含 integrity_hash（本筆雜湊）和 previous_hash（前筆雜湊）
-- 構成鏈式結構，任何篡改都會導致鏈斷裂

ALTER TABLE user_activity_logs
    ADD COLUMN IF NOT EXISTS integrity_hash VARCHAR(128),
    ADD COLUMN IF NOT EXISTS previous_hash VARCHAR(128);

-- 建立索引以加速鏈完整性驗證
CREATE INDEX IF NOT EXISTS idx_activity_logs_integrity
    ON user_activity_logs (created_at, integrity_hash);

COMMENT ON COLUMN user_activity_logs.integrity_hash IS 'HMAC-SHA256 雜湊值，用於驗證日誌完整性';
COMMENT ON COLUMN user_activity_logs.previous_hash IS '前一筆日誌的 integrity_hash，構成雜湊鏈';
