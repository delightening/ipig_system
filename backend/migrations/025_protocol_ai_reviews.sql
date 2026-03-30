-- R20-4: AI 預審結果表
CREATE TABLE protocol_ai_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    protocol_version_id UUID REFERENCES protocol_versions(id),
    review_type VARCHAR(30) NOT NULL
        CHECK (review_type IN ('client_pre_submit', 'staff_pre_review')),
    rule_result JSONB,
    ai_result JSONB,
    ai_model VARCHAR(50),
    ai_input_tokens INTEGER,
    ai_output_tokens INTEGER,
    total_errors INTEGER NOT NULL DEFAULT 0,
    total_warnings INTEGER NOT NULL DEFAULT 0,
    score INTEGER,
    triggered_by UUID REFERENCES users(id),
    duration_ms INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ai_reviews_protocol_latest ON protocol_ai_reviews (protocol_id, created_at DESC);
CREATE UNIQUE INDEX idx_ai_reviews_version_type ON protocol_ai_reviews (protocol_version_id, review_type) WHERE protocol_version_id IS NOT NULL;
