-- ============================================
-- Migration 008: Fixes (合併原 012, 013, 014, 015, 016, 019)
-- ============================================

-- Enum cast 函式 (019 修正遞迴版本)
CREATE OR REPLACE FUNCTION version_record_type_to_text(version_record_type) RETURNS text AS $$
    SELECT (SELECT enumlabel FROM pg_enum WHERE enumtypid = 'version_record_type'::regtype ORDER BY enumsortorder OFFSET (array_position(enum_range(NULL::version_record_type), $1) - 1) LIMIT 1);
$$ LANGUAGE SQL STABLE;

CREATE OR REPLACE FUNCTION text_to_version_record_type(text) RETURNS version_record_type AS $$
    SELECT r.v FROM unnest(enum_range(NULL::version_record_type)) AS r(v) WHERE version_record_type_to_text(r.v) = $1 LIMIT 1;
$$ LANGUAGE SQL STABLE;

CREATE OR REPLACE FUNCTION animal_record_type_to_text(animal_record_type) RETURNS text AS $$
    SELECT $1::text;
$$ LANGUAGE SQL IMMUTABLE;

CREATE OR REPLACE FUNCTION record_type_to_text(record_type) RETURNS text AS $$
    SELECT $1::text;
$$ LANGUAGE SQL IMMUTABLE;

-- ASSIGNMENT cast (013) - 先刪除可能存在的隱式/舊轉型
DROP CAST IF EXISTS (version_record_type AS text);
DROP CAST IF EXISTS (text AS version_record_type);
DROP CAST IF EXISTS (animal_record_type AS text);
DROP CAST IF EXISTS (record_type AS text);

CREATE CAST (version_record_type AS text) WITH FUNCTION version_record_type_to_text(version_record_type) AS ASSIGNMENT;
CREATE CAST (text AS version_record_type) WITH FUNCTION text_to_version_record_type(text) AS ASSIGNMENT;
CREATE CAST (animal_record_type AS text) WITH FUNCTION animal_record_type_to_text(animal_record_type) AS ASSIGNMENT;
CREATE CAST (record_type AS text) WITH FUNCTION record_type_to_text(record_type) AS ASSIGNMENT;

-- Optimistic locking (014)
ALTER TABLE animals ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE protocols ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE animal_observations ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE animal_surgeries ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;

-- System settings seed (015)
INSERT INTO system_settings (key, value, description) VALUES
('company_name', '"iPig System"', '公司/系統名稱'),
('default_warehouse_id', '""', '預設倉庫 UUID'),
('cost_method', '"weighted_average"', '成本計算方式'),
('smtp_host', '""', 'SMTP 主機'),('smtp_port', '"587"', 'SMTP 埠'),('smtp_username', '""', 'SMTP 帳號'),('smtp_password', '""', 'SMTP 密碼'),
('smtp_from_email', '"noreply@erp.local"', '寄件人 Email'),('smtp_from_name', '"iPig System"', '寄件人顯示名稱'),
('session_timeout_minutes', '"360"', 'Session 逾時（分鐘）')
ON CONFLICT (key) DO NOTHING;

-- TOTP 2FA (016)
ALTER TABLE users ADD COLUMN IF NOT EXISTS totp_enabled BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE users ADD COLUMN IF NOT EXISTS totp_secret_encrypted TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS totp_backup_codes TEXT[];
