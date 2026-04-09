-- Migration 020: 獸醫師建議結構化表單
-- 每隻動物一筆，sections JSONB 包含四個類別的觀察/建議/追蹤改善

CREATE TABLE IF NOT EXISTS animal_vet_advices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    animal_id UUID NOT NULL REFERENCES animals(id),
    sections JSONB NOT NULL DEFAULT '{}',
    created_by UUID REFERENCES users(id),
    updated_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_animal_vet_advices_animal
    ON animal_vet_advices(animal_id);
