-- ============================================
-- 動物轉讓流程
-- 轉讓狀態 enum + 轉讓記錄表 + 獸醫評估表
-- ============================================

-- 轉讓狀態 enum
CREATE TYPE animal_transfer_status AS ENUM (
    'pending',         -- 發起
    'vet_evaluated',   -- 獸醫評估完成
    'plan_assigned',   -- 新計劃已指定
    'pi_approved',     -- 新 PI 同意
    'completed',       -- 轉讓完成
    'rejected'         -- 拒絕
);

-- 轉讓記錄表
CREATE TABLE animal_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    animal_id UUID NOT NULL REFERENCES animals(id),
    from_iacuc_no VARCHAR(20) NOT NULL,
    to_iacuc_no VARCHAR(20),
    status animal_transfer_status NOT NULL DEFAULT 'pending',
    initiated_by UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,
    remark TEXT,
    rejected_by UUID REFERENCES users(id),
    rejected_reason TEXT,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 轉讓獸醫評估表
CREATE TABLE transfer_vet_evaluations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transfer_id UUID NOT NULL REFERENCES animal_transfers(id) UNIQUE,
    vet_id UUID NOT NULL REFERENCES users(id),
    health_status TEXT NOT NULL,
    is_fit_for_transfer BOOLEAN NOT NULL,
    conditions TEXT,
    evaluated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_transfers_animal ON animal_transfers(animal_id);
CREATE INDEX idx_transfers_from_iacuc ON animal_transfers(from_iacuc_no);
CREATE INDEX idx_transfers_to_iacuc ON animal_transfers(to_iacuc_no);
CREATE INDEX idx_transfers_status ON animal_transfers(status);
CREATE INDEX idx_transfer_vet_eval ON transfer_vet_evaluations(transfer_id);
