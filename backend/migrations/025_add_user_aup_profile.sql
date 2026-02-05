-- 員工 AUP 相關資料欄位
-- 用於儲存 AUP 第 8 節人員資料的預設值

ALTER TABLE users 
  ADD COLUMN IF NOT EXISTS entry_date DATE,
  ADD COLUMN IF NOT EXISTS position VARCHAR(100),
  ADD COLUMN IF NOT EXISTS aup_roles TEXT[] DEFAULT '{}',
  ADD COLUMN IF NOT EXISTS years_experience INTEGER DEFAULT 0,
  ADD COLUMN IF NOT EXISTS trainings JSONB DEFAULT '[]';

COMMENT ON COLUMN users.entry_date IS '入職日期 (Entry Date)';
COMMENT ON COLUMN users.position IS '職稱 (Position)';
COMMENT ON COLUMN users.aup_roles IS '工作內容 a~i (Roles in Project)';
COMMENT ON COLUMN users.years_experience IS '參與動物試驗年數 (Years of Experience in Animal Experiments)';
COMMENT ON COLUMN users.trainings IS '訓練/資格/證書編號 (Trainings/Certifications)';

-- trainings JSONB 格式範例:
-- [
--   {"code": "A", "certificate_no": "12345", "received_date": "2024-01-15"},
--   {"code": "B", "certificate_no": "67890", "received_date": "2024-03-20"}
-- ]
