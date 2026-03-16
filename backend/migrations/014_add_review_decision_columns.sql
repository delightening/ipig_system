-- 為 review_assignments 補上審查決定相關欄位
-- 對齊 amendment_review_assignments 已有的 decision / decided_at 設計
ALTER TABLE review_assignments ADD COLUMN IF NOT EXISTS decision VARCHAR(20);
ALTER TABLE review_assignments ADD COLUMN IF NOT EXISTS decided_at TIMESTAMPTZ;
