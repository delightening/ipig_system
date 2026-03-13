-- ============================================
-- Migration 008: 補充功能、犧牲鎖欄、轉讓類型、修正、效能
-- ============================================

-- 8.1 通知路由
CREATE TABLE notification_routing (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(80) NOT NULL,
    role_code VARCHAR(50) NOT NULL REFERENCES roles(code),
    channel VARCHAR(20) NOT NULL DEFAULT 'in_app',
    is_active BOOLEAN NOT NULL DEFAULT true,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (event_type, role_code),
    CONSTRAINT chk_channel CHECK (channel IN ('in_app', 'email', 'both'))
);
CREATE INDEX idx_notification_routing_event ON notification_routing(event_type, is_active);

INSERT INTO notification_routing (event_type, role_code, channel, description) VALUES
('protocol_submitted','IACUC_STAFF','in_app','計畫提交'),('protocol_vet_review','VET','in_app','進入獸醫審查'),('protocol_under_review','IACUC_STAFF','in_app','進入委員審查'),('protocol_resubmitted','IACUC_STAFF','in_app','重新提交'),
('protocol_approved','IACUC_CHAIR','both','計畫核准'),('protocol_rejected','IACUC_CHAIR','both','計畫駁回'),
('review_comment_created','IACUC_STAFF','in_app','新審查意見'),('leave_submitted','ADMIN_STAFF','in_app','請假申請'),('leave_submitted','admin','in_app','請假申請'),('overtime_submitted','ADMIN_STAFF','in_app','加班申請'),('overtime_submitted','admin','in_app','加班申請'),
('document_submitted','WAREHOUSE_MANAGER','in_app','採購單提交'),('low_stock_alert','admin','in_app','低庫存預警'),('low_stock_alert','WAREHOUSE_MANAGER','in_app','低庫存預警'),('expiry_alert','admin','in_app','效期預警'),('expiry_alert','WAREHOUSE_MANAGER','in_app','效期預警'),
('emergency_medication','VET','in_app','緊急給藥'),('amendment_submitted','IACUC_STAFF','in_app','修正案提交'),('amendment_decision_recorded','IACUC_STAFF','in_app','修正案審查決定'),('amendment_approved','IACUC_CHAIR','both','修正案核准'),('amendment_rejected','IACUC_CHAIR','both','修正案駁回'),
('all_reviews_completed','IACUC_STAFF','in_app','所有審查完成'),('all_comments_resolved','IACUC_CHAIR','in_app','所有意見已解決'),
('animal_abnormal_record','VET','both','動物異常紀錄'),('animal_sudden_death','VET','both','動物猝死'),('low_stock_alert','PURCHASING','in_app','低庫存預警'),
('leave_cancelled', 'ADMIN_STAFF', 'in_app', '請假取消'), ('leave_cancelled', 'admin', 'in_app', '請假取消'),
('po_pending_receipt', 'WAREHOUSE_MANAGER', 'in_app', '採購單未入庫提醒')
ON CONFLICT (event_type, role_code) DO NOTHING;

-- 8.2 電子簽章
CREATE TABLE electronic_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL,
    entity_id VARCHAR(100) NOT NULL,
    signer_id UUID NOT NULL REFERENCES users(id),
    signature_type VARCHAR(20) NOT NULL,
    content_hash VARCHAR(64) NOT NULL,
    signature_data VARCHAR(128) NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    signed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_valid BOOLEAN NOT NULL DEFAULT true,
    invalidated_reason TEXT,
    invalidated_at TIMESTAMPTZ,
    invalidated_by UUID REFERENCES users(id),
    handwriting_svg TEXT,
    stroke_data JSONB,
    signature_method VARCHAR(20) DEFAULT 'password'
);
CREATE INDEX idx_esig_entity ON electronic_signatures (entity_type, entity_id);

CREATE TABLE record_annotations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    record_type VARCHAR(50) NOT NULL,
    record_id INTEGER NOT NULL,
    annotation_type VARCHAR(20) NOT NULL,
    content TEXT NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    signature_id UUID REFERENCES electronic_signatures(id)
);
CREATE INDEX idx_annot_record ON record_annotations (record_type, record_id);

-- 8.3 治療藥物選項
CREATE TABLE treatment_drug_options (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    display_name VARCHAR(200),
    default_dosage_unit VARCHAR(20),
    available_units TEXT[],
    default_dosage_value VARCHAR(50),
    erp_product_id UUID REFERENCES products(id),
    category VARCHAR(50),
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_treatment_drug_options_active ON treatment_drug_options(is_active);

INSERT INTO treatment_drug_options (name, display_name, default_dosage_unit, available_units, category, sort_order) VALUES
('Atropine','Atropine（阿托品）','mg',ARRAY['mg','ml','mg/kg'],'麻醉',10),('Stroless','Stroless','mg',ARRAY['mg','ml','mg/kg'],'麻醉',20),('Zoletil-50','Zoletil-50（乙嗪錠）','mg',ARRAY['mg','ml','mg/kg'],'麻醉',30),
('O2','O2（氧氣）','L/min',ARRAY['L/min','%'],'麻醉',40),('N2O','N2O（氧化亞氮）','L/min',ARRAY['L/min','%'],'麻醉',50),('Isoflurane','Isoflurane（異氟醚）','%',ARRAY['%','ml'],'麻醉',60),
('Meloxicam','Meloxicam（美洛昔康）','mg/kg',ARRAY['mg','ml','mg/kg'],'止痛',100),('Buprenorphine','Buprenorphine（丁丙諾啡）','mg/kg',ARRAY['mg','ml','mg/kg'],'止痛',110),('Carprofen','Carprofen（卡洛芬）','mg/kg',ARRAY['mg','ml','mg/kg'],'止痛',120),
('Enrofloxacin','Enrofloxacin（恩諾沙星）','mg/kg',ARRAY['mg','ml','mg/kg','tab'],'抗生素',200),('Amoxicillin','Amoxicillin（阿莫西林）','mg/kg',ARRAY['mg','ml','mg/kg','cap','tab'],'抗生素',210),('Cefazolin','Cefazolin（頭孢唑啉）','mg/kg',ARRAY['mg','ml','mg/kg'],'抗生素',220),
('Xylazine','Xylazine（甲苯噻嗪）','mg/kg',ARRAY['mg','ml','mg/kg'],'鎮靜',300),('Midazolam','Midazolam（咪達唑侖）','mg/kg',ARRAY['mg','ml','mg/kg'],'鎮靜',310),
('優點軟膏','優點軟膏（眼藥膏）','cm',ARRAY['cm','g','pcs'],'其他',400);

CREATE TABLE jwt_blacklist (
    jti VARCHAR(64) PRIMARY KEY,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_jwt_blacklist_expires ON jwt_blacklist(expires_at);

-- 8.4 動物犧牲簽章鎖（GLP）
ALTER TABLE animal_sacrifices
  ADD COLUMN IF NOT EXISTS is_locked BOOLEAN NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS locked_at TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS locked_by UUID REFERENCES users(id);

-- 8.5 轉讓類型：external = 轉給其他機構，internal = 仍在機構內
ALTER TABLE animal_transfers
ADD COLUMN IF NOT EXISTS transfer_type VARCHAR(20) NOT NULL DEFAULT 'internal';
COMMENT ON COLUMN animal_transfers.transfer_type IS 'external: 轉給其他機構; internal: 仍在機構內';

-- 8.6 Enum cast 函式
CREATE OR REPLACE FUNCTION version_record_type_to_text(version_record_type) RETURNS text AS $$
    SELECT (SELECT enumlabel FROM pg_enum WHERE enumtypid = 'version_record_type'::regtype ORDER BY enumsortorder OFFSET (array_position(enum_range(NULL::version_record_type), $1) - 1) LIMIT 1);
$$ LANGUAGE SQL STABLE;

CREATE OR REPLACE FUNCTION text_to_version_record_type(text) RETURNS version_record_type AS $$
    SELECT r.v FROM unnest(enum_range(NULL::version_record_type)) AS r(v) WHERE version_record_type_to_text(r.v) = $1 LIMIT 1;
$$ LANGUAGE SQL STABLE;

CREATE OR REPLACE FUNCTION animal_record_type_to_text(animal_record_type) RETURNS text AS $$
    SELECT $1::text;
$$ LANGUAGE SQL IMMUTABLE;

CREATE OR REPLACE FUNCTION record_type_to_text(record_type) RETURNS text AS $$
    SELECT $1::text;
$$ LANGUAGE SQL IMMUTABLE;

DROP CAST IF EXISTS (version_record_type AS text);
DROP CAST IF EXISTS (text AS version_record_type);
DROP CAST IF EXISTS (animal_record_type AS text);
DROP CAST IF EXISTS (record_type AS text);

CREATE CAST (version_record_type AS text) WITH FUNCTION version_record_type_to_text(version_record_type) AS ASSIGNMENT;
CREATE CAST (text AS version_record_type) WITH FUNCTION text_to_version_record_type(text) AS ASSIGNMENT;
CREATE CAST (animal_record_type AS text) WITH FUNCTION animal_record_type_to_text(animal_record_type) AS ASSIGNMENT;
CREATE CAST (record_type AS text) WITH FUNCTION record_type_to_text(record_type) AS ASSIGNMENT;

-- 8.7 Optimistic locking
ALTER TABLE animals ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE protocols ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE animal_observations ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE animal_surgeries ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;

-- 8.8 System settings seed
INSERT INTO system_settings (key, value, description) VALUES
('company_name', '"iPig System"', '公司/系統名稱'),
('default_warehouse_id', '""', '預設倉庫 UUID'),
('cost_method', '"weighted_average"', '成本計算方式'),
('smtp_host', '""', 'SMTP 主機'),('smtp_port', '"587"', 'SMTP 埠'),('smtp_username', '""', 'SMTP 帳號'),('smtp_password', '""', 'SMTP 密碼'),
('smtp_from_email', '"noreply@erp.local"', '寄件人 Email'),('smtp_from_name', '"iPig System"', '寄件人顯示名稱'),
('session_timeout_minutes', '"360"', 'Session 逾時（分鐘）')
ON CONFLICT (key) DO NOTHING;

-- 8.9 TOTP 2FA
ALTER TABLE users ADD COLUMN IF NOT EXISTS totp_enabled BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE users ADD COLUMN IF NOT EXISTS totp_secret_encrypted TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS totp_backup_codes TEXT[];

-- 8.10 效能索引
CREATE INDEX IF NOT EXISTS idx_animals_status_deleted_created ON animals(status, is_deleted, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_protocols_status_pi_created ON protocols(status, pi_user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_notifications_user_read_created ON notifications(user_id, is_read, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_entity_created ON user_activity_logs(entity_type, entity_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_attachments_entity ON attachments(entity_type, entity_id);

CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

CREATE OR REPLACE FUNCTION maintenance_vacuum_analyze() RETURNS void AS $$
BEGIN
    ANALYZE animals; ANALYZE animal_observations; ANALYZE animal_surgeries; ANALYZE animal_weights;
    ANALYZE animal_vaccinations; ANALYZE vet_recommendations; ANALYZE notifications;
    ANALYZE user_activity_logs; ANALYZE attachments; ANALYZE protocols; ANALYZE audit_logs;
    RAISE NOTICE 'maintenance_vacuum_analyze completed at %', NOW();
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE VIEW slow_queries AS
SELECT queryid, LEFT(query, 200) AS query_preview, calls, mean_exec_time AS avg_ms, total_exec_time AS total_ms, rows
FROM pg_stat_statements WHERE mean_exec_time > 100 ORDER BY mean_exec_time DESC LIMIT 50;

-- 8.11 修正操作日誌中「疑苗紀錄」錯字為「疫苗紀錄」
UPDATE user_activity_logs
SET entity_display_name = REPLACE(entity_display_name, '疑苗紀錄', '疫苗紀錄')
WHERE entity_display_name LIKE '%疑苗紀錄%';
