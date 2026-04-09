-- 獸醫巡場報告
CREATE TABLE IF NOT EXISTS vet_patrol_reports (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    patrol_date   DATE        NOT NULL DEFAULT CURRENT_DATE,
    week_start    DATE,
    week_end      DATE,
    status        VARCHAR(20) NOT NULL DEFAULT 'draft',
    created_by    UUID        REFERENCES users(id),
    updated_by    UUID        REFERENCES users(id),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at    TIMESTAMPTZ
);

-- 巡場報告條目（每筆一個類別+一隻動物的觀察/建議/追蹤）
CREATE TABLE IF NOT EXISTS vet_patrol_entries (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    report_id       UUID        NOT NULL REFERENCES vet_patrol_reports(id) ON DELETE CASCADE,
    category        VARCHAR(50) NOT NULL,
    animal_id       UUID        REFERENCES animals(id),
    observation     TEXT        NOT NULL DEFAULT '',
    suggestion      TEXT        NOT NULL DEFAULT '',
    follow_up       TEXT        NOT NULL DEFAULT '',
    sort_order      INT         NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_vet_patrol_entries_report ON vet_patrol_entries(report_id);
CREATE INDEX IF NOT EXISTS idx_vet_patrol_entries_animal ON vet_patrol_entries(animal_id);
