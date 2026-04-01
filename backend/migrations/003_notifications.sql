-- ============================================================
-- Migration 003: 通知、稽核、電子簽章、系統設定
-- 來源: 003_notifications_roles_seed.sql, 008_supplementary.sql,
--       027_notification_routing_schedule.sql
-- ============================================================

-- ── notifications ────────────────────────────────────────────
CREATE TABLE notifications (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID            NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type                notification_type NOT NULL,
    title               VARCHAR(200)    NOT NULL,
    content             TEXT,
    is_read             BOOLEAN         NOT NULL DEFAULT false,
    read_at             TIMESTAMPTZ,
    related_entity_type VARCHAR(50),
    related_entity_id   UUID,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_notifications_user_id         ON notifications(user_id);
CREATE INDEX idx_notifications_is_read         ON notifications(user_id, is_read);
CREATE INDEX idx_notifications_type            ON notifications(type);
CREATE INDEX idx_notifications_created_at      ON notifications(created_at);
CREATE INDEX idx_notifications_user_read_created ON notifications(user_id, is_read, created_at DESC);

-- ── notification_settings ────────────────────────────────────
CREATE TABLE notification_settings (
    user_id                     UUID    PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    email_low_stock             BOOLEAN NOT NULL DEFAULT true,
    email_expiry_warning        BOOLEAN NOT NULL DEFAULT true,
    email_document_approval     BOOLEAN NOT NULL DEFAULT true,
    email_protocol_status       BOOLEAN NOT NULL DEFAULT true,
    email_monthly_report        BOOLEAN NOT NULL DEFAULT true,
    expiry_warning_days         INTEGER NOT NULL DEFAULT 30,
    low_stock_notify_immediately BOOLEAN NOT NULL DEFAULT true,
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- trigger: 新使用者自動建立 notification_settings
CREATE OR REPLACE FUNCTION create_default_notification_settings()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO notification_settings (user_id)
    VALUES (NEW.id)
    ON CONFLICT (user_id) DO NOTHING;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_create_notification_settings
    AFTER INSERT ON users
    FOR EACH ROW
    EXECUTE FUNCTION create_default_notification_settings();

-- ── notification_routing (008 + 027) ─────────────────────────
-- 最終欄位：含 027 新增的 frequency / hour_of_day / day_of_week
CREATE TABLE notification_routing (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type  VARCHAR(80) NOT NULL,
    role_code   VARCHAR(50) NOT NULL REFERENCES roles(code),
    channel     VARCHAR(20) NOT NULL DEFAULT 'in_app',
    is_active   BOOLEAN     NOT NULL DEFAULT true,
    description TEXT,
    -- 027: 排程頻率欄位
    frequency   VARCHAR(20) NOT NULL DEFAULT 'immediate'
        CONSTRAINT chk_nr_frequency CHECK (frequency IN ('immediate','daily','weekly','monthly')),
    hour_of_day SMALLINT    NOT NULL DEFAULT 8
        CONSTRAINT chk_nr_hour CHECK (hour_of_day BETWEEN 0 AND 23),
    day_of_week SMALLINT    DEFAULT NULL
        CONSTRAINT chk_nr_dow CHECK (day_of_week BETWEEN 0 AND 6),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (event_type, role_code),
    CONSTRAINT chk_channel CHECK (channel IN ('in_app', 'email', 'both'))
);
CREATE INDEX idx_notification_routing_event ON notification_routing(event_type, is_active);

COMMENT ON COLUMN notification_routing.frequency   IS 'immediate=事件即時觸發, daily/weekly/monthly=批次排程';
COMMENT ON COLUMN notification_routing.hour_of_day IS '批次通知的執行小時（0-23），immediate 時忽略';
COMMENT ON COLUMN notification_routing.day_of_week IS 'weekly 時有效：0=週日, 1=週一 ... 6=週六';

-- Seed: 通知路由（008 seed + 018 seed，批次型事件直接設 daily）
INSERT INTO notification_routing (event_type, role_code, channel, description, frequency) VALUES
    -- AUP 流程（即時）
    ('protocol_submitted',           'IACUC_STAFF',       'in_app', '計畫提交',             'immediate'),
    ('protocol_vet_review',          'VET',               'in_app', '進入獸醫審查',         'immediate'),
    ('protocol_under_review',        'IACUC_STAFF',       'in_app', '進入委員審查',         'immediate'),
    ('protocol_resubmitted',         'IACUC_STAFF',       'in_app', '重新提交',             'immediate'),
    ('protocol_approved',            'IACUC_CHAIR',       'both',   '計畫核准',             'immediate'),
    ('protocol_rejected',            'IACUC_CHAIR',       'both',   '計畫駁回',             'immediate'),
    ('review_comment_created',       'IACUC_STAFF',       'in_app', '新審查意見',           'immediate'),
    ('all_reviews_completed',        'IACUC_STAFF',       'in_app', '所有審查完成',         'immediate'),
    ('all_comments_resolved',        'IACUC_CHAIR',       'in_app', '所有意見已解決',       'immediate'),
    -- Amendment
    ('amendment_submitted',          'IACUC_STAFF',       'in_app', '修正案提交',           'immediate'),
    ('amendment_decision_recorded',  'IACUC_STAFF',       'in_app', '修正案審查決定',       'immediate'),
    ('amendment_approved',           'IACUC_CHAIR',       'both',   '修正案核准',           'immediate'),
    ('amendment_rejected',           'IACUC_CHAIR',       'both',   '修正案駁回',           'immediate'),
    -- HR
    ('leave_submitted',              'ADMIN_STAFF',       'in_app', '請假申請',             'immediate'),
    ('leave_submitted',              'admin',             'in_app', '請假申請',             'immediate'),
    ('leave_cancelled',              'ADMIN_STAFF',       'in_app', '請假取消',             'immediate'),
    ('leave_cancelled',              'admin',             'in_app', '請假取消',             'immediate'),
    ('overtime_submitted',           'ADMIN_STAFF',       'in_app', '加班申請',             'immediate'),
    ('overtime_submitted',           'admin',             'in_app', '加班申請',             'immediate'),
    -- ERP
    ('document_submitted',           'WAREHOUSE_MANAGER', 'in_app', '採購單提交',           'immediate'),
    ('po_pending_receipt',           'WAREHOUSE_MANAGER', 'in_app', '採購單未入庫提醒',     'daily'),
    -- 庫存/效期（批次型，daily）
    ('low_stock_alert',              'admin',             'in_app', '低庫存預警',           'daily'),
    ('low_stock_alert',              'WAREHOUSE_MANAGER', 'in_app', '低庫存預警',           'daily'),
    ('low_stock_alert',              'PURCHASING',        'in_app', '低庫存預警',           'daily'),
    ('expiry_alert',                 'admin',             'in_app', '效期預警',             'daily'),
    ('expiry_alert',                 'WAREHOUSE_MANAGER', 'in_app', '效期預警',             'daily'),
    -- 動物
    ('emergency_medication',         'VET',               'in_app', '緊急給藥',             'immediate'),
    ('animal_abnormal_record',       'VET',               'both',   '動物異常紀錄',         'immediate'),
    ('animal_sudden_death',          'VET',               'both',   '動物猝死',             'immediate'),
    -- 設備（018，批次型 daily）
    ('equipment_overdue',            'EQUIPMENT_MAINTENANCE', 'both',   '設備校正/確效逾期提醒',     'daily'),
    ('equipment_unrepairable',       'EQUIPMENT_MAINTENANCE', 'both',   '設備無法維修通知',           'immediate'),
    ('equipment_unrepairable',       'admin',                 'both',   '設備無法維修通知（機構負責人）', 'immediate'),
    ('equipment_disposal',           'EQUIPMENT_MAINTENANCE', 'in_app', '設備報廢申請通知',           'immediate'),
    ('equipment_disposal',           'admin',                 'both',   '設備報廢申請通知（機構負責人）','immediate')
ON CONFLICT (event_type, role_code) DO NOTHING;

-- ── attachments ──────────────────────────────────────────────
CREATE TABLE attachments (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    category    VARCHAR(50) NOT NULL,
    entity_id   UUID,
    entity_type VARCHAR(50),
    file_name   VARCHAR(255) NOT NULL,
    file_path   VARCHAR(500) NOT NULL,
    file_size   INTEGER      NOT NULL,
    mime_type   VARCHAR(100) NOT NULL,
    uploaded_by UUID         NOT NULL REFERENCES users(id),
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_attachments_category ON attachments(category);
CREATE INDEX idx_attachments_entity   ON attachments(entity_type, entity_id);

-- ── audit_logs ───────────────────────────────────────────────
CREATE TABLE audit_logs (
    id            UUID        PRIMARY KEY,
    actor_user_id UUID        NOT NULL REFERENCES users(id),
    action        VARCHAR(50) NOT NULL,
    entity_type   VARCHAR(50) NOT NULL,
    entity_id     UUID        NOT NULL,
    before_data   JSONB,
    after_data    JSONB,
    ip_address    VARCHAR(45),
    user_agent    TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_audit_logs_entity     ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_logs_actor      ON audit_logs(actor_user_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);

-- ── user_preferences ─────────────────────────────────────────
CREATE TABLE user_preferences (
    id               UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    preference_key   VARCHAR(100) NOT NULL,
    preference_value JSONB        NOT NULL DEFAULT '{}',
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, preference_key)
);
CREATE INDEX idx_user_preferences_user_id ON user_preferences(user_id);

-- ── electronic_signatures (008) ──────────────────────────────
CREATE TABLE electronic_signatures (
    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type         VARCHAR(50) NOT NULL,
    entity_id           VARCHAR(100) NOT NULL,
    signer_id           UUID        NOT NULL REFERENCES users(id),
    signature_type      VARCHAR(20) NOT NULL,
    content_hash        VARCHAR(64) NOT NULL,
    signature_data      VARCHAR(128) NOT NULL,
    ip_address          VARCHAR(45),
    user_agent          TEXT,
    signed_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_valid            BOOLEAN     NOT NULL DEFAULT true,
    invalidated_reason  TEXT,
    invalidated_at      TIMESTAMPTZ,
    invalidated_by      UUID        REFERENCES users(id),
    handwriting_svg     TEXT,
    stroke_data         JSONB,
    signature_method    VARCHAR(20) DEFAULT 'password'
);
CREATE INDEX idx_esig_entity    ON electronic_signatures(entity_type, entity_id);
CREATE INDEX idx_esig_signer_id ON electronic_signatures(signer_id);

-- ── record_annotations (008) ─────────────────────────────────
CREATE TABLE record_annotations (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    record_type     VARCHAR(50) NOT NULL,
    record_id       INTEGER     NOT NULL,
    annotation_type VARCHAR(20) NOT NULL,
    content         TEXT        NOT NULL,
    created_by      UUID        NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    signature_id    UUID        REFERENCES electronic_signatures(id)
);
CREATE INDEX idx_annot_record ON record_annotations(record_type, record_id);

-- ── jwt_blacklist (008) ──────────────────────────────────────
CREATE TABLE jwt_blacklist (
    jti        VARCHAR(64) PRIMARY KEY,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_jwt_blacklist_expires ON jwt_blacklist(expires_at);

-- ── system_settings (005 + 008) ──────────────────────────────
CREATE TABLE system_settings (
    key         VARCHAR(100) PRIMARY KEY,
    value       JSONB        NOT NULL,
    description TEXT,
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_by  UUID         REFERENCES users(id)
);

INSERT INTO system_settings (key, value, description) VALUES
    ('default_vet_reviewer',         '{"user_id": null}',       '預設獸醫審查員，VET_REVIEW 階段會自動指派此獸醫師'),
    ('company_name',                 '"iPig System"',           '公司/系統名稱'),
    ('default_warehouse_id',         '""',                      '預設倉庫 UUID'),
    ('cost_method',                  '"weighted_average"',      '成本計算方式'),
    ('smtp_host',                    '""',                      'SMTP 主機'),
    ('smtp_port',                    '"587"',                   'SMTP 埠'),
    ('smtp_username',                '""',                      'SMTP 帳號'),
    ('smtp_password',                '""',                      'SMTP 密碼'),
    ('smtp_from_email',              '"noreply@erp.local"',     '寄件人 Email'),
    ('smtp_from_name',               '"iPig System"',           '寄件人顯示名稱'),
    ('session_timeout_minutes',      '"360"',                   'Session 逾時（分鐘）')
ON CONFLICT (key) DO NOTHING;
