-- 設備閒置審批流程（GLP/ISO 合規 — NC-02 矯正）

CREATE TABLE equipment_idle_requests (
    id                      UUID                PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id            UUID        NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    request_type            TEXT        NOT NULL CHECK (request_type IN ('idle', 'restore')),
    reason                  TEXT        NOT NULL,
    status                  disposal_status NOT NULL DEFAULT 'pending',
    applied_by              UUID        NOT NULL REFERENCES users(id),
    applied_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    applicant_signature_id  UUID        REFERENCES electronic_signatures(id),
    approved_by             UUID        REFERENCES users(id),
    approved_at             TIMESTAMPTZ,
    approver_signature_id   UUID        REFERENCES electronic_signatures(id),
    rejection_reason        TEXT,
    notes                   TEXT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_equipment_idle_requests_equipment ON equipment_idle_requests(equipment_id);
CREATE INDEX idx_equipment_idle_requests_status    ON equipment_idle_requests(status);

-- 閒置審批權限
INSERT INTO permissions (id, code, name, module, description) VALUES
    (gen_random_uuid(), 'equipment.idle.approve', '核准設備閒置', 'equipment', '可核准設備閒置/恢復申請')
ON CONFLICT (code) DO NOTHING;

-- 授權給管理員
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'admin' AND p.code = 'equipment.idle.approve'
ON CONFLICT DO NOTHING;

-- 通知路由
INSERT INTO notification_routing (event_type, role_code, channel, description) VALUES
    ('equipment_idle_request', 'EQUIPMENT_MAINTENANCE', 'in_app', '設備閒置申請通知'),
    ('equipment_idle_request', 'admin', 'both', '設備閒置申請通知（機構負責人）')
ON CONFLICT DO NOTHING;
