-- 獸醫師建議紀錄（per animal，多筆列表）
CREATE TABLE IF NOT EXISTS animal_vet_advice_records (
    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    animal_id           UUID        NOT NULL REFERENCES animals(id),
    advice_date         DATE        NOT NULL DEFAULT CURRENT_DATE,
    observation         TEXT        NOT NULL DEFAULT '',
    suggested_treatment TEXT        NOT NULL DEFAULT '',
    created_by          UUID        REFERENCES users(id),
    updated_by          UUID        REFERENCES users(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_vet_advice_records_animal
    ON animal_vet_advice_records(animal_id);
