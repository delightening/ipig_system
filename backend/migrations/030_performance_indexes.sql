-- 效能索引：覆蓋高頻查詢路徑
-- user_protocols 複合索引（access.rs EXISTS 查詢 + amendment 子查詢）
CREATE INDEX IF NOT EXISTS idx_user_protocols_protocol_user
    ON user_protocols(protocol_id, user_id);
CREATE INDEX IF NOT EXISTS idx_user_protocols_user_protocol
    ON user_protocols(user_id, protocol_id);

-- pg_trgm 供 ILIKE '%..%' 搜尋（動物 ear_tag 等）
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX IF NOT EXISTS idx_animals_ear_tag_trgm
    ON animals USING gin (ear_tag gin_trgm_ops);
