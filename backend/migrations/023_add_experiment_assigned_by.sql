-- ============================================
-- Migration 023: 新增實驗分配者欄位
--
-- 記錄動物「被誰」轉換至實驗中狀態
-- 搭配既有的 experiment_date 欄位（記錄「什麼時候」）
-- ============================================

ALTER TABLE pigs ADD COLUMN IF NOT EXISTS experiment_assigned_by UUID REFERENCES users(id);

-- 為既有已在實驗中的豬隻，以 created_by 作為預設分配者
UPDATE pigs
SET experiment_assigned_by = created_by
WHERE status = 'in_experiment' AND experiment_assigned_by IS NULL;
