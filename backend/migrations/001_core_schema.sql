-- ============================================
-- Migration 001a: Core Schema
-- 
-- 包含：
-- - 所有自訂類型 (Custom Types / ENUMs)
-- - 用戶與權限相關表
-- - 基礎資料表 (Master Data)
-- - 觸發器與函式
-- 
-- 編碼: UTF-8 (無 BOM)
-- 預設密碼: admin123 (由後端 ensure_admin_user 函式在啟動時自動建立，此處 hash 僅供參考)
-- ============================================

-- ============================================
-- 1. 自訂類型 (Custom Types)
-- ============================================

-- ERP 相關類型
CREATE TYPE partner_type AS ENUM ('supplier', 'customer');
CREATE TYPE supplier_category AS ENUM ('drug', 'consumable', 'feed', 'equipment', 'other');
CREATE TYPE doc_type AS ENUM ('PO', 'GRN', 'PR', 'SO', 'DO', 'SR', 'TR', 'STK', 'ADJ', 'RTN');
CREATE TYPE doc_status AS ENUM ('draft', 'submitted', 'approved', 'cancelled');
CREATE TYPE stock_direction AS ENUM ('in', 'out', 'transfer_in', 'transfer_out', 'adjust_in', 'adjust_out');

-- AUP 相關類型
CREATE TYPE protocol_role AS ENUM ('PI', 'CLIENT', 'CO_EDITOR');
CREATE TYPE protocol_status AS ENUM (
    'DRAFT',
    'SUBMITTED',
    'PRE_REVIEW',
    'PRE_REVIEW_REVISION_REQUIRED',
    'VET_REVIEW',
    'VET_REVISION_REQUIRED',
    'UNDER_REVIEW',
    'REVISION_REQUIRED',
    'RESUBMITTED',
    'APPROVED',
    'APPROVED_WITH_CONDITIONS',
    'DEFERRED',
    'REJECTED',
    'SUSPENDED',
    'CLOSED',
    'DELETED'
);

-- 動物管理相關類型
CREATE TYPE pig_status AS ENUM ('unassigned', 'in_experiment', 'completed');
CREATE TYPE pig_breed AS ENUM ('miniature', 'white', 'LYD', 'other');
CREATE TYPE pig_gender AS ENUM ('male', 'female');
CREATE TYPE record_type AS ENUM ('abnormal', 'experiment', 'observation');
CREATE TYPE pig_record_type AS ENUM ('observation', 'surgery', 'sacrifice', 'pathology', 'blood_test');
CREATE TYPE pig_file_type AS ENUM ('photo', 'attachment', 'report');
CREATE TYPE vet_record_type AS ENUM ('observation', 'surgery');
CREATE TYPE care_record_mode AS ENUM ('legacy', 'pain_assessment');
CREATE TYPE version_record_type AS ENUM ('observation', 'surgery', 'weight', 'vaccination', 'sacrifice', 'pathology', 'blood_test');

-- Protocol 活動類型
CREATE TYPE protocol_activity_type AS ENUM (
    -- 生命週期
    'CREATED',
    'UPDATED',
    'SUBMITTED',
    'RESUBMITTED',
    'APPROVED',
    'APPROVED_WITH_CONDITIONS',
    'CLOSED',
    'REJECTED',
    'SUSPENDED',
    'DELETED',
    -- 審查流程
    'STATUS_CHANGED',
    'REVIEWER_ASSIGNED',
    'VET_ASSIGNED',
    'COEDITOR_ASSIGNED',
    'COEDITOR_REMOVED',
    -- 審查意見
    'COMMENT_ADDED',
    'COMMENT_REPLIED',
    'COMMENT_RESOLVED',
    -- 附件
    'ATTACHMENT_UPLOADED',
    'ATTACHMENT_DELETED',
    -- 版本
    'VERSION_CREATED',
    'VERSION_RECOVERED',
    -- 修正案
    'AMENDMENT_CREATED',
    'AMENDMENT_SUBMITTED',
    -- 動物管理
    'PIG_ASSIGNED',
    'PIG_UNASSIGNED'
);

-- 通知與報表類型
CREATE TYPE notification_type AS ENUM (
    'low_stock',
    'expiry_warning',
    'document_approval',
    'protocol_status',
    'protocol_submitted',
    'review_assignment',
    'review_comment',
    'leave_approval',
    'overtime_approval',
    'vet_recommendation',
    'system_alert',
    'monthly_report'
);
CREATE TYPE schedule_type AS ENUM ('daily', 'weekly', 'monthly');
CREATE TYPE report_type AS ENUM (
    'stock_on_hand',
    'stock_ledger',
    'purchase_summary',
    'cost_summary',
    'expiry_report',
    'low_stock_report'
);

-- HR 相關類型
CREATE TYPE leave_type AS ENUM (
    'ANNUAL',
    'PERSONAL',
    'SICK',
    'COMPENSATORY',
    'MARRIAGE',
    'BEREAVEMENT',
    'MATERNITY',
    'PATERNITY',
    'MENSTRUAL',
    'OFFICIAL'
);
CREATE TYPE leave_status AS ENUM (
    'DRAFT',
    'PENDING_L1',
    'PENDING_L2',
    'PENDING_HR',
    'PENDING_GM',
    'APPROVED',
    'REJECTED',
    'CANCELLED',
    'REVOKED'
);

-- 修正案相關類型
CREATE TYPE amendment_type AS ENUM (
    'MAJOR',
    'MINOR',
    'PENDING'
);
CREATE TYPE amendment_status AS ENUM (
    'DRAFT',
    'SUBMITTED',
    'CLASSIFIED',
    'UNDER_REVIEW',
    'REVISION_REQUIRED',
    'RESUBMITTED',
    'APPROVED',
    'REJECTED',
    'ADMIN_APPROVED'
);

-- 安樂死審查類型
CREATE TYPE euthanasia_order_status AS ENUM (
    'pending_pi',
    'appealed',
    'chair_arbitration',
    'approved',
    'rejected',
    'executed',
    'cancelled'
);

-- 匯入匯出類型
CREATE TYPE import_type AS ENUM ('pig_basic', 'pig_weight');
CREATE TYPE import_status AS ENUM ('pending', 'processing', 'completed', 'failed');
CREATE TYPE export_type AS ENUM ('medical_summary', 'observation_records', 'surgery_records', 'experiment_records');
CREATE TYPE export_format AS ENUM ('pdf', 'excel');

-- ============================================
-- 2. 用戶與權限相關表
-- ============================================

-- 用戶表
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    phone VARCHAR(20),
    organization VARCHAR(200),
    is_internal BOOLEAN NOT NULL DEFAULT true,
    is_active BOOLEAN NOT NULL DEFAULT true,
    must_change_password BOOLEAN NOT NULL DEFAULT true,
    last_login_at TIMESTAMPTZ,
    login_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until TIMESTAMPTZ,
    theme_preference VARCHAR(20) NOT NULL DEFAULT 'light',
    language_preference VARCHAR(10) NOT NULL DEFAULT 'zh-TW',
    -- 員工相關欄位 (用於 AUP Section 8 經驗計算)
    entry_date DATE,                          -- 入職日期
    position VARCHAR(100),                    -- 職位
    aup_roles VARCHAR(255)[] DEFAULT '{}',    -- AUP 角色 (可多選)
    years_experience INTEGER NOT NULL DEFAULT 0,  -- 先前經驗年數
    trainings JSONB NOT NULL DEFAULT '[]',    -- 教育訓練記錄 (JSON 格式)
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_theme_preference CHECK (theme_preference IN ('light', 'dark', 'system')),
    CONSTRAINT chk_language_preference CHECK (language_preference IN ('zh-TW', 'en'))
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_is_active ON users(is_active);

-- 角色表
CREATE TABLE roles (
    id UUID PRIMARY KEY,
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_internal BOOLEAN NOT NULL DEFAULT true,
    is_system BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 權限表
CREATE TABLE permissions (
    id UUID PRIMARY KEY,
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    module VARCHAR(50),
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 角色權限關聯表
CREATE TABLE role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- 用戶角色關聯表
CREATE TABLE user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ DEFAULT NOW(),
    assigned_by UUID REFERENCES users(id),
    PRIMARY KEY (user_id, role_id)
);

-- Refresh Token 表
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);

CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);

-- 密碼重設 Token 表
CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);
CREATE INDEX idx_password_reset_tokens_token_hash ON password_reset_tokens(token_hash);

-- ============================================
-- 3. 通知系統表
-- ============================================

-- 通知表
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type notification_type NOT NULL,
    title VARCHAR(200) NOT NULL,
    content TEXT,
    is_read BOOLEAN NOT NULL DEFAULT false,
    read_at TIMESTAMPTZ,
    related_entity_type VARCHAR(50),
    related_entity_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_is_read ON notifications(user_id, is_read);
CREATE INDEX idx_notifications_type ON notifications(type);
CREATE INDEX idx_notifications_created_at ON notifications(created_at);

-- 通知設定表
CREATE TABLE notification_settings (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    email_low_stock BOOLEAN NOT NULL DEFAULT true,
    email_expiry_warning BOOLEAN NOT NULL DEFAULT true,
    email_document_approval BOOLEAN NOT NULL DEFAULT true,
    email_protocol_status BOOLEAN NOT NULL DEFAULT true,
    email_monthly_report BOOLEAN NOT NULL DEFAULT true,
    expiry_warning_days INTEGER NOT NULL DEFAULT 30,
    low_stock_notify_immediately BOOLEAN NOT NULL DEFAULT true,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================
-- 4. 通用附件表
-- ============================================

CREATE TABLE attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category VARCHAR(50) NOT NULL,
    entity_id UUID,
    entity_type VARCHAR(50),
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    uploaded_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_attachments_category ON attachments(category);
CREATE INDEX idx_attachments_entity ON attachments(entity_type, entity_id);

-- ============================================
-- 5. 簡易稽核日誌表
-- ============================================

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    actor_user_id UUID NOT NULL REFERENCES users(id),
    action VARCHAR(50) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    before_data JSONB,
    after_data JSONB,
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_logs_actor ON audit_logs(actor_user_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);

-- ============================================
-- 6. 觸發器
-- ============================================

-- 新使用者建立時自動建立通知設定
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

-- ============================================
-- 7. 插入基礎角色定義
-- ============================================

INSERT INTO roles (id, code, name, description, is_internal, is_system, created_at, updated_at) VALUES
    (gen_random_uuid(), 'admin', '系統管理員', '全系統最高權限，使用者管理、系統維運', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'ADMIN_STAFF', '行政人員', '處理行政事務、協助管理', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'WAREHOUSE_MANAGER', '倉庫管理員', '專責 ERP 進銷存系統', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'PURCHASING', '採購人員', '負責採購作業', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'PI', '計畫主持人', '提交計畫、管理自己的計畫與動物', false, true, NOW(), NOW()),
    (gen_random_uuid(), 'VET', '獸醫師', '審查計畫、動物健康管理、提供建議', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'REVIEWER', '審查委員', 'IACUC 計畫審查', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'IACUC_CHAIR', 'IACUC 主席', '主導審查決策', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'IACUC_STAFF', '執行秘書', '行政流程管理、管理所有計劃進度', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'EXPERIMENT_STAFF', '試驗工作人員', '執行實驗操作、記錄數據', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'CLIENT', '委託人', '查看委託計畫與動物紀錄', false, true, NOW(), NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 8. 插入種子帳號
-- ============================================

-- 注意：admin 帳號由後端 ensure_admin_user() 函式在啟動時動態建立
-- 預設帳號：admin@ipig.local
-- 預設密碼：admin123
-- 這樣可以確保密碼 hash 使用與後端驗證一致的 Argon2 參數

-- 原本的 SQL seed 已移除，由後端啟動時自動建立 admin 帳號並指派角色

-- ============================================
-- 完成
-- ============================================
