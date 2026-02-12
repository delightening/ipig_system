-- 新增報廢主管簽核相關欄位到 documents 表
-- 當調整單(ADJ)報廢金額超過門檻時，需要主管簽核

ALTER TABLE documents ADD COLUMN IF NOT EXISTS requires_manager_approval BOOLEAN DEFAULT FALSE;
ALTER TABLE documents ADD COLUMN IF NOT EXISTS scrap_total_amount DECIMAL;
ALTER TABLE documents ADD COLUMN IF NOT EXISTS manager_approval_status VARCHAR(20);  -- pending, approved, rejected
ALTER TABLE documents ADD COLUMN IF NOT EXISTS manager_approved_by UUID REFERENCES users(id);
ALTER TABLE documents ADD COLUMN IF NOT EXISTS manager_approved_at TIMESTAMPTZ;
ALTER TABLE documents ADD COLUMN IF NOT EXISTS manager_reject_reason TEXT;
