-- 新增獸醫師審查表欄位
ALTER TABLE vet_review_assignments ADD COLUMN review_form JSONB;

-- 為現有紀錄初始化空物件（選填，視需求而定）
UPDATE vet_review_assignments SET review_form = '{}' WHERE review_form IS NULL;

-- 新增註解
COMMENT ON COLUMN vet_review_assignments.review_form IS '獸醫師 12 項查檢表資料 (JSON 格式)';
