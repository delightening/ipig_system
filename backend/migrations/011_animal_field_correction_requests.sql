-- ============================================
-- Migration 011: 動物欄位修正申請（需 admin 批准）
-- ============================================
-- 耳號、出生日期、性別、品種等欄位建立後不可直接修改，
-- 若 staff 輸入錯誤，可提交修正申請，經 admin 批准後套用。

CREATE TABLE animal_field_correction_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    animal_id UUID NOT NULL REFERENCES animals(id) ON DELETE CASCADE,
    field_name VARCHAR(50) NOT NULL,
    old_value TEXT,
    new_value TEXT NOT NULL,
    reason TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    requested_by UUID NOT NULL REFERENCES users(id),
    reviewed_by UUID REFERENCES users(id),
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_field_name CHECK (field_name IN ('ear_tag', 'birth_date', 'gender', 'breed')),
    CONSTRAINT chk_status CHECK (status IN ('pending', 'approved', 'rejected'))
);

CREATE INDEX idx_afcr_animal_id ON animal_field_correction_requests(animal_id);
CREATE INDEX idx_afcr_status ON animal_field_correction_requests(status);
CREATE INDEX idx_afcr_requested_by ON animal_field_correction_requests(requested_by);
CREATE INDEX idx_afcr_created_at ON animal_field_correction_requests(created_at DESC);

COMMENT ON TABLE animal_field_correction_requests IS '動物不可變欄位修正申請，需 admin 批准後套用';
COMMENT ON COLUMN animal_field_correction_requests.field_name IS '欄位名稱：ear_tag, birth_date, gender, breed';
COMMENT ON COLUMN animal_field_correction_requests.status IS 'pending=待審核, approved=已批准, rejected=已拒絕';
