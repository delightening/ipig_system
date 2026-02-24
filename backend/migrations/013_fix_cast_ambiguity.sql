-- ============================================
-- Migration 013: Fix Cast Ambiguity
-- 
-- 解決 "operator is not unique: record_type = record_type" 錯誤
-- 將原本在 012 建立的 IMPLICIT CAST 改為 ASSIGNMENT CAST
-- 賦值轉型 (ASSIGNMENT) 允許在 INSERT/UPDATE 時自動轉型，
-- 但在比較操作 (如 WHERE) 中需要明確轉型，從而消除歧義。
-- ============================================

-- 1. 刪除舊的隱式轉型 (如果存在)
DROP CAST IF EXISTS (version_record_type AS text);
DROP CAST IF EXISTS (text AS version_record_type);
DROP CAST IF EXISTS (animal_record_type AS text);
DROP CAST IF EXISTS (record_type AS text);

-- 2. 重新建立為賦值轉型 (ASSIGNMENT)

-- version_record_type
CREATE CAST (version_record_type AS text) WITH FUNCTION version_record_type_to_text(version_record_type) AS ASSIGNMENT;
CREATE CAST (text AS version_record_type) WITH FUNCTION text_to_version_record_type(text) AS ASSIGNMENT;

-- animal_record_type
CREATE CAST (animal_record_type AS text) WITH FUNCTION animal_record_type_to_text(animal_record_type) AS ASSIGNMENT;

-- record_type
CREATE CAST (record_type AS text) WITH FUNCTION record_type_to_text(record_type) AS ASSIGNMENT;
