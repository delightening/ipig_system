-- ============================================
-- 動物生命週期擴充
-- 新增猝死記錄表
-- ============================================

-- 猝死記錄表
CREATE TABLE animal_sudden_deaths (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    animal_id UUID NOT NULL REFERENCES animals(id) UNIQUE,
    discovered_at TIMESTAMPTZ NOT NULL,
    discovered_by UUID NOT NULL REFERENCES users(id),
    probable_cause TEXT,
    iacuc_no VARCHAR(20),
    location VARCHAR(100),
    remark TEXT,
    requires_pathology BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sudden_deaths_animal ON animal_sudden_deaths(animal_id);
CREATE INDEX idx_sudden_deaths_discovered_by ON animal_sudden_deaths(discovered_by);
