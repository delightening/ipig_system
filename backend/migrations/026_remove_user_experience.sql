-- 移除 users 表的 experience 欄位（單位簡歷）

ALTER TABLE users DROP COLUMN IF EXISTS experience;
