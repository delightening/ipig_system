-- ============================================
-- Migration 011: 通知路由規則表
--
-- 將原本硬編碼在 Rust 程式碼中的
-- 「事件 → 收件角色 → 通知通道」對應關係
-- 抽取到資料庫中，使管理員可透過 API/UI 調整
-- ============================================

CREATE TABLE notification_routing (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(80) NOT NULL,                          -- 事件識別碼
    role_code VARCHAR(50) NOT NULL REFERENCES roles(code),    -- 對應 roles.code
    channel VARCHAR(20) NOT NULL DEFAULT 'in_app',            -- 'in_app' | 'email' | 'both'
    is_active BOOLEAN NOT NULL DEFAULT true,
    description TEXT,                                          -- 管理者可讀說明
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (event_type, role_code),
    CONSTRAINT chk_channel CHECK (channel IN ('in_app', 'email', 'both'))
);

CREATE INDEX idx_notification_routing_event ON notification_routing(event_type, is_active);

-- ============================================
-- Seed: 依據現有硬編碼邏輯插入預設路由規則
-- ============================================

INSERT INTO notification_routing (event_type, role_code, channel, description) VALUES
    -- AUP 計畫生命週期
    ('protocol_submitted',               'IACUC_STAFF',       'in_app', '計畫提交 → 通知執行秘書'),
    ('protocol_vet_review',              'VET',               'in_app', '進入獸醫審查 → 通知獸醫師'),
    ('protocol_under_review',            'IACUC_STAFF',       'in_app', '進入委員審查 → 通知執行秘書'),
    ('protocol_resubmitted',             'IACUC_STAFF',       'in_app', '重新提交 → 通知執行秘書'),
    ('protocol_approved',                'IACUC_CHAIR',       'both',   '計畫核准 → 通知 IACUC 主席'),
    ('protocol_rejected',                'IACUC_CHAIR',       'both',   '計畫駁回 → 通知 IACUC 主席'),
    -- 審查意見
    ('review_comment_created',           'IACUC_STAFF',       'in_app', '新審查意見 → 通知執行秘書'),
    -- HR 請假/加班
    ('leave_submitted',                  'ADMIN_STAFF',       'in_app', '請假申請 → 通知行政人員'),
    ('leave_submitted',                  'admin',             'in_app', '請假申請 → 通知系統管理員'),
    ('overtime_submitted',               'ADMIN_STAFF',       'in_app', '加班申請 → 通知行政人員'),
    ('overtime_submitted',               'admin',             'in_app', '加班申請 → 通知系統管理員'),
    -- ERP 採購單
    ('document_submitted',               'WAREHOUSE_MANAGER', 'in_app', '採購單提交 → 通知倉庫管理員'),
    -- 庫存預警
    ('low_stock_alert',                  'admin',             'in_app', '低庫存預警 → 通知系統管理員'),
    ('low_stock_alert',                  'WAREHOUSE_MANAGER', 'in_app', '低庫存預警 → 通知倉庫管理員'),
    ('expiry_alert',                     'admin',             'in_app', '效期預警 → 通知系統管理員'),
    ('expiry_alert',                     'WAREHOUSE_MANAGER', 'in_app', '效期預警 → 通知倉庫管理員'),
    -- 緊急給藥
    ('emergency_medication',             'VET',               'in_app', '緊急給藥 → 通知獸醫師'),
    -- 修正案
    ('amendment_submitted',              'IACUC_STAFF',       'in_app', '修正案提交 → 通知執行秘書'),
    ('amendment_decision_recorded',      'IACUC_STAFF',       'in_app', '修正案審查決定 → 通知執行秘書'),
    ('amendment_approved',               'IACUC_CHAIR',       'both',   '修正案核准 → 通知 IACUC 主席'),
    ('amendment_rejected',               'IACUC_CHAIR',       'both',   '修正案駁回 → 通知 IACUC 主席')
ON CONFLICT (event_type, role_code) DO NOTHING;
