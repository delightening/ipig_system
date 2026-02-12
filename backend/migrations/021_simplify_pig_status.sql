-- 021: 簡化 pig_status enum
-- 將 6 種狀態精簡為 3 種: unassigned, in_experiment, completed
-- 移除: assigned, transferred, deceased

-- 步驟 1: 將既有資料轉換為新狀態
UPDATE pigs SET status = 'in_experiment' WHERE status::text = 'assigned';
UPDATE pigs SET status = 'completed' WHERE status::text = 'deceased';
UPDATE pigs SET status = 'completed' WHERE status::text = 'transferred';

-- 步驟 2: 移除 DEFAULT（因為 DEFAULT 依賴 pig_status enum）
ALTER TABLE pigs ALTER COLUMN status DROP DEFAULT;

-- 步驟 3: 將欄位轉為 TEXT（解除對 pig_status enum 的所有依賴）
ALTER TABLE pigs ALTER COLUMN status TYPE TEXT USING status::text;

-- 步驟 4: 刪除舊 enum
DROP TYPE pig_status;

-- 步驟 5: 建立新 enum（只有 3 個值）
CREATE TYPE pig_status AS ENUM ('unassigned', 'in_experiment', 'completed');

-- 步驟 6: 將欄位轉回新 enum
ALTER TABLE pigs ALTER COLUMN status TYPE pig_status USING status::pig_status;

-- 步驟 7: 重新設定預設值
ALTER TABLE pigs ALTER COLUMN status SET DEFAULT 'unassigned';
