-- ============================================
-- Migration 018: Remove Array Foreign Key
-- 
-- 移除 comp_time_source_ids UUID[] 陣列欄位
-- 改用 leave_balance_usage 關聯表確保參照完整性
-- ============================================

ALTER TABLE leave_requests DROP COLUMN IF EXISTS comp_time_source_ids;
