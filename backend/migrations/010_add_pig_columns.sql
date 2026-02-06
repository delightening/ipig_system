-- ============================================
-- Migration 010: Add missing columns to pigs table
-- 
-- 新增動物相關欄位
-- 
-- 編碼: UTF-8 (無 BOM)
-- ============================================

-- ============================================
-- 1. 新增 animal_no 欄位（由使用者命名）
-- ============================================

ALTER TABLE pigs ADD COLUMN IF NOT EXISTS animal_no VARCHAR(50);

-- ============================================
-- 2. 新增 deletion_reason 欄位（GLP 合規）
-- ============================================

ALTER TABLE pigs ADD COLUMN IF NOT EXISTS deletion_reason TEXT;

-- ============================================
-- 3. 新增 animal_id 欄位（動物 ID）
-- ============================================

ALTER TABLE pigs ADD COLUMN IF NOT EXISTS animal_id UUID;

-- ============================================
-- 4. 新增 breed_other 欄位（其他品種說明）
-- ============================================

ALTER TABLE pigs ADD COLUMN IF NOT EXISTS breed_other VARCHAR(100);

-- ============================================
-- 完成
-- ============================================
