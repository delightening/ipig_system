-- ============================================
-- Migration 019: Fix Enum Cast Recursion
--
-- 解決 version_record_type_to_text 與 text_to_version_record_type 遞迴呼叫
-- 導致 "stack depth limit exceeded"。改用 pg_enum 查詢取得 label，避免 ::text 觸發自身 cast。
-- ============================================

-- 1. version_record_type 到 text：用 array_position + pg_enum 取得 label，避免 $1::text 遞迴
CREATE OR REPLACE FUNCTION version_record_type_to_text(version_record_type) RETURNS text AS $$
    SELECT (SELECT enumlabel FROM pg_enum WHERE enumtypid = 'version_record_type'::regtype ORDER BY enumsortorder OFFSET (array_position(enum_range(NULL::version_record_type), $1) - 1) LIMIT 1);
$$ LANGUAGE SQL STABLE;

-- 2. text 到 version_record_type：用 enum_range 比對取得 enum，避免 $1::version_record_type 遞迴
-- 注意：r.v::text 會觸發 version_record_type_to_text，但該函數已修正為非遞迴
CREATE OR REPLACE FUNCTION text_to_version_record_type(text) RETURNS version_record_type AS $$
    SELECT r.v
    FROM unnest(enum_range(NULL::version_record_type)) AS r(v)
    WHERE version_record_type_to_text(r.v) = $1
    LIMIT 1;
$$ LANGUAGE SQL STABLE;
