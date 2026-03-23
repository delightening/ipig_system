-- R10-L9: login_events composite indexes for common query patterns
-- Covers: email + event_type + created_at (login failure counting, rate limiting)
-- Covers: created_at + event_type (dashboard daily statistics, range queries)
--
-- 使用 DO block 處理 non-C locale 環境下 text 欄位索引可能因
-- collation 函式未標記 IMMUTABLE 而失敗的情況（PostgreSQL 16+）

DO $$ BEGIN
    CREATE INDEX IF NOT EXISTS idx_login_email_type_created
        ON login_events (email, event_type, created_at DESC);
EXCEPTION WHEN feature_not_supported THEN
    -- 若預設 collation 不支援，改用 C collation 建立索引
    CREATE INDEX IF NOT EXISTS idx_login_email_type_created
        ON login_events ((email COLLATE "C"), (event_type COLLATE "C"), created_at DESC);
END $$;

DO $$ BEGIN
    CREATE INDEX IF NOT EXISTS idx_login_created_type
        ON login_events (created_at, event_type);
EXCEPTION WHEN feature_not_supported THEN
    CREATE INDEX IF NOT EXISTS idx_login_created_type
        ON login_events (created_at, (event_type COLLATE "C"));
END $$;
