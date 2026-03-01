-- ============================================
-- Migration 007: Supplementary (合併原 008, 009, 010)
-- ============================================

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
('animal_abnormal_record','VET','both','動物異常紀錄'),('animal_sudden_death','VET','both','動物猝死'),('low_stock_alert','PURCHASING','in_app','低庫存預警')
ON CONFLICT (event_type, role_code) DO NOTHING;

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
