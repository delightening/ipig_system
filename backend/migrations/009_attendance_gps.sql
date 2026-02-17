-- ============================================
-- Migration 009: 出勤打卡 GPS 定位
--
-- 新增經緯度欄位，用於記錄打卡時的 GPS 座標
-- ============================================

ALTER TABLE attendance_records
  ADD COLUMN IF NOT EXISTS clock_in_latitude  DOUBLE PRECISION,
  ADD COLUMN IF NOT EXISTS clock_in_longitude DOUBLE PRECISION,
  ADD COLUMN IF NOT EXISTS clock_out_latitude  DOUBLE PRECISION,
  ADD COLUMN IF NOT EXISTS clock_out_longitude DOUBLE PRECISION;
