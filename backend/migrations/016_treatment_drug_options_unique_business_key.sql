-- 業務鍵唯一約束：同一「名稱 + 分類」僅允許一筆啟用中的藥物選項。
-- 使用部分唯一索引（僅 is_active = true），已軟刪除的重複列可保留不影響。
CREATE UNIQUE INDEX idx_treatment_drug_options_business_key
ON treatment_drug_options (lower(trim(name)), COALESCE(category, ''))
WHERE is_active = true;
