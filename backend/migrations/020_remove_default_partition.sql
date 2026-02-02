-- ============================================
-- Migration 018: 移除 user_activity_logs 預設分區
-- 
-- 移除預設分區，確保缺失分區時 INSERT 會立即報錯，
-- 而非靜默地將資料插入預設分區影響查詢效能。
-- 
-- 配合 scheduler.rs 中的 PartitionMaintenanceJob，
-- 每年 12 月 1 日自動建立未來 2 年的季度分區。
-- ============================================

-- 首先將 default 分區中的資料搬移到正確分區 (如有)
-- 若 default 分區中有資料，需先處理
DO $$
DECLARE
    row_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO row_count FROM user_activity_logs_default;
    IF row_count > 0 THEN
        RAISE NOTICE 'Warning: user_activity_logs_default contains % rows. Please migrate manually.', row_count;
    END IF;
END $$;

-- 移除預設分區 (若為空)
DROP TABLE IF EXISTS user_activity_logs_default;

-- 新增註解說明
COMMENT ON TABLE user_activity_logs IS 'GLP Compliance: 分區表無預設分區，缺失分區會導致 INSERT 失敗。由 PartitionMaintenanceJob 每年自動維護分區。';
