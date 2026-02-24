-- ============================================
-- Migration 012: Fix Enum Casts
-- 
-- 解決 "operator does not exist: version_record_type = text" 錯誤
-- 建立 ENUM 與 TEXT 之間的隱式轉型 (Implicit Cast)
-- ============================================

-- 1. version_record_type 到 text 的轉型
CREATE OR REPLACE FUNCTION version_record_type_to_text(version_record_type) RETURNS text AS $$
    SELECT $1::text;
$$ LANGUAGE SQL IMMUTABLE;

DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_cast c JOIN pg_type s ON c.castsource = s.oid JOIN pg_type t ON c.casttarget = t.oid WHERE s.typname = 'version_record_type' AND t.typname = 'text') THEN
        CREATE CAST (version_record_type AS text) WITH FUNCTION version_record_type_to_text(version_record_type) AS IMPLICIT;
    END IF;
END $$;

-- 2. text 到 version_record_type 的轉型
CREATE OR REPLACE FUNCTION text_to_version_record_type(text) RETURNS version_record_type AS $$
    SELECT $1::version_record_type;
$$ LANGUAGE SQL IMMUTABLE;

DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_cast c JOIN pg_type s ON c.castsource = s.oid JOIN pg_type t ON c.casttarget = t.oid WHERE s.typname = 'text' AND t.typname = 'version_record_type') THEN
        CREATE CAST (text AS version_record_type) WITH FUNCTION text_to_version_record_type(text) AS IMPLICIT;
    END IF;
END $$;

-- 3. 其他可能遇到類似問題的 ENUM 也一併建立轉型 (例如 animal_record_type)

-- animal_record_type 到 text
CREATE OR REPLACE FUNCTION animal_record_type_to_text(animal_record_type) RETURNS text AS $$
    SELECT $1::text;
$$ LANGUAGE SQL IMMUTABLE;

DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_cast c JOIN pg_type s ON c.castsource = s.oid JOIN pg_type t ON c.casttarget = t.oid WHERE s.typname = 'animal_record_type' AND t.typname = 'text') THEN
        CREATE CAST (animal_record_type AS text) WITH FUNCTION animal_record_type_to_text(animal_record_type) AS IMPLICIT;
    END IF;
END $$;

-- record_type 到 text
CREATE OR REPLACE FUNCTION record_type_to_text(record_type) RETURNS text AS $$
    SELECT $1::text;
$$ LANGUAGE SQL IMMUTABLE;

DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_cast c JOIN pg_type s ON c.castsource = s.oid JOIN pg_type t ON c.casttarget = t.oid WHERE s.typname = 'record_type' AND t.typname = 'text') THEN
        CREATE CAST (record_type AS text) WITH FUNCTION record_type_to_text(record_type) AS IMPLICIT;
    END IF;
END $$;
