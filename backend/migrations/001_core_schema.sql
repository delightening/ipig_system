-- ============================================
-- Migration 001: Core Schema (合併原 001 + 002)
-- 
-- 包含：Types、users、roles、permissions、notifications、
--       attachments、audit_logs、user_preferences、
--       權限定義、角色權限分配
-- ============================================

-- ============================================
-- 1. 自訂類型 (Custom Types)
-- ============================================

CREATE TYPE partner_type AS ENUM ('supplier', 'customer');
CREATE TYPE supplier_category AS ENUM ('drug', 'consumable', 'feed', 'equipment', 'other');
CREATE TYPE customer_category AS ENUM ('internal', 'external', 'research', 'other');
CREATE TYPE doc_type AS ENUM ('PO', 'GRN', 'PR', 'SO', 'DO', 'SR', 'TR', 'STK', 'ADJ', 'RTN');
CREATE TYPE doc_status AS ENUM ('draft', 'submitted', 'approved', 'cancelled');
CREATE TYPE stock_direction AS ENUM ('in', 'out', 'transfer_in', 'transfer_out', 'adjust_in', 'adjust_out');

CREATE TYPE protocol_role AS ENUM ('PI', 'CLIENT', 'CO_EDITOR');
CREATE TYPE protocol_status AS ENUM (
    'DRAFT', 'SUBMITTED', 'PRE_REVIEW', 'PRE_REVIEW_REVISION_REQUIRED',
    'VET_REVIEW', 'VET_REVISION_REQUIRED', 'UNDER_REVIEW', 'REVISION_REQUIRED',
    'RESUBMITTED', 'APPROVED', 'APPROVED_WITH_CONDITIONS', 'DEFERRED', 'REJECTED',
    'SUSPENDED', 'CLOSED', 'DELETED'
);

CREATE TYPE animal_status AS ENUM ('unassigned', 'in_experiment', 'completed', 'euthanized', 'sudden_death', 'transferred');
CREATE TYPE animal_breed AS ENUM ('miniature', 'white', 'LYD', 'other');
CREATE TYPE animal_gender AS ENUM ('male', 'female');
CREATE TYPE record_type AS ENUM ('abnormal', 'experiment', 'observation');
CREATE TYPE animal_record_type AS ENUM ('observation', 'surgery', 'sacrifice', 'pathology', 'blood_test');
CREATE TYPE animal_file_type AS ENUM ('photo', 'attachment', 'report');
CREATE TYPE vet_record_type AS ENUM ('observation', 'surgery');
CREATE TYPE care_record_mode AS ENUM ('legacy', 'pain_assessment');
CREATE TYPE version_record_type AS ENUM ('observation', 'surgery', 'weight', 'vaccination', 'sacrifice', 'pathology', 'blood_test');

CREATE TYPE protocol_activity_type AS ENUM (
    'CREATED', 'UPDATED', 'SUBMITTED', 'RESUBMITTED', 'APPROVED', 'APPROVED_WITH_CONDITIONS',
    'CLOSED', 'REJECTED', 'SUSPENDED', 'DELETED', 'STATUS_CHANGED', 'REVIEWER_ASSIGNED',
    'VET_ASSIGNED', 'COEDITOR_ASSIGNED', 'COEDITOR_REMOVED', 'COMMENT_ADDED', 'COMMENT_REPLIED',
    'COMMENT_RESOLVED', 'ATTACHMENT_UPLOADED', 'ATTACHMENT_DELETED', 'VERSION_CREATED',
    'VERSION_RECOVERED', 'AMENDMENT_CREATED', 'AMENDMENT_SUBMITTED', 'ANIMAL_ASSIGNED', 'ANIMAL_UNASSIGNED'
);

CREATE TYPE notification_type AS ENUM (
    'low_stock', 'expiry_warning', 'document_approval', 'protocol_status', 'protocol_submitted',
    'review_assignment', 'review_comment', 'leave_approval', 'overtime_approval', 'vet_recommendation',
    'system_alert', 'monthly_report'
);
CREATE TYPE schedule_type AS ENUM ('daily', 'weekly', 'monthly');
CREATE TYPE report_type AS ENUM ('stock_on_hand', 'stock_ledger', 'purchase_summary', 'cost_summary', 'expiry_report', 'low_stock_report');

CREATE TYPE leave_type AS ENUM ('ANNUAL', 'PERSONAL', 'SICK', 'COMPENSATORY', 'MARRIAGE', 'BEREAVEMENT', 'MATERNITY', 'PATERNITY', 'MENSTRUAL', 'OFFICIAL');
CREATE TYPE leave_status AS ENUM ('DRAFT', 'PENDING_L1', 'PENDING_L2', 'PENDING_HR', 'PENDING_GM', 'APPROVED', 'REJECTED', 'CANCELLED', 'REVOKED');

CREATE TYPE amendment_type AS ENUM ('MAJOR', 'MINOR', 'PENDING');
CREATE TYPE amendment_status AS ENUM ('DRAFT', 'SUBMITTED', 'CLASSIFIED', 'UNDER_REVIEW', 'REVISION_REQUIRED', 'RESUBMITTED', 'APPROVED', 'REJECTED', 'ADMIN_APPROVED');

CREATE TYPE euthanasia_order_status AS ENUM ('pending_pi', 'appealed', 'chair_arbitration', 'approved', 'rejected', 'executed', 'cancelled');
CREATE TYPE import_type AS ENUM ('animal_basic', 'animal_weight');
CREATE TYPE import_status AS ENUM ('pending', 'processing', 'completed', 'failed');
CREATE TYPE export_type AS ENUM ('medical_summary', 'observation_records', 'surgery_records', 'experiment_records');
CREATE TYPE export_format AS ENUM ('pdf', 'excel');

CREATE TYPE animal_transfer_status AS ENUM ('pending', 'vet_evaluated', 'plan_assigned', 'pi_approved', 'completed', 'rejected');

-- ============================================
-- 2. 用戶與權限相關表
-- ============================================

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
    entry_date DATE,
    position VARCHAR(100),
    aup_roles VARCHAR(255)[] DEFAULT '{}',
    years_experience INTEGER NOT NULL DEFAULT 0,
    trainings JSONB NOT NULL DEFAULT '[]',
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_theme_preference CHECK (theme_preference IN ('light', 'dark', 'system')),
    CONSTRAINT chk_language_preference CHECK (language_preference IN ('zh-TW', 'en'))
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_is_active ON users(is_active);

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

CREATE TABLE permissions (
    id UUID PRIMARY KEY,
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    module VARCHAR(50),
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ DEFAULT NOW(),
    assigned_by UUID REFERENCES users(id),
    PRIMARY KEY (user_id, role_id)
);

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
-- 7. 插入基礎角色
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
-- 8. 插入權限
-- ============================================

INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'admin.user.view', '查看使用者', 'admin', '可查看使用者列表', NOW()),
    (gen_random_uuid(), 'admin.user.view_all', '查看所有使用者', 'admin', '可查看所有使用者資料', NOW()),
    (gen_random_uuid(), 'admin.user.create', '建立使用者', 'admin', '可建立新使用者帳號', NOW()),
    (gen_random_uuid(), 'admin.user.edit', '編輯使用者', 'admin', '可編輯使用者資料', NOW()),
    (gen_random_uuid(), 'admin.user.delete', '停用使用者', 'admin', '可停用使用者帳號', NOW()),
    (gen_random_uuid(), 'admin.user.reset_password', '重設密碼', 'admin', '可重設他人密碼', NOW()),
    (gen_random_uuid(), 'admin.role.view', '查看角色', 'admin', '可查看角色列表', NOW()),
    (gen_random_uuid(), 'admin.role.manage', '管理角色', 'admin', '可管理角色定義', NOW()),
    (gen_random_uuid(), 'admin.permission.manage', '管理權限', 'admin', '可管理權限定義', NOW()),
    (gen_random_uuid(), 'admin.audit.view', '查看稽核紀錄', 'admin', '可查看系統稽核紀錄', NOW()),
    (gen_random_uuid(), 'aup.protocol.view_all', '查看所有計畫', 'aup', '可查看系統中所有計畫', NOW()),
    (gen_random_uuid(), 'aup.protocol.view_own', '查看自己的計畫', 'aup', '可查看自己相關的計畫', NOW()),
    (gen_random_uuid(), 'aup.protocol.create', '建立計畫', 'aup', '可建立新計畫', NOW()),
    (gen_random_uuid(), 'aup.protocol.edit', '編輯計畫', 'aup', '可編輯計畫草稿', NOW()),
    (gen_random_uuid(), 'aup.protocol.submit', '提交計畫', 'aup', '可提交計畫送審', NOW()),
    (gen_random_uuid(), 'aup.protocol.review', '審查計畫', 'aup', '可審查計畫並提供意見', NOW()),
    (gen_random_uuid(), 'aup.protocol.approve', '核准/否決', 'aup', '可核准或否決計畫', NOW()),
    (gen_random_uuid(), 'aup.protocol.change_status', '變更狀態', 'aup', '可變更計畫狀態', NOW()),
    (gen_random_uuid(), 'aup.protocol.delete', '刪除計畫', 'aup', '可刪除計畫', NOW()),
    (gen_random_uuid(), 'aup.review.view', '查看審查', 'aup', '可查看審查意見', NOW()),
    (gen_random_uuid(), 'aup.review.assign', '指派審查人員', 'aup', '可指派審查人員', NOW()),
    (gen_random_uuid(), 'aup.review.comment', '新增審查意見', 'aup', '可新增審查意見', NOW()),
    (gen_random_uuid(), 'aup.attachment.view', '查看附件', 'aup', '可查看計畫附件', NOW()),
    (gen_random_uuid(), 'aup.attachment.download', '下載附件', 'aup', '可下載計畫附件', NOW()),
    (gen_random_uuid(), 'aup.version.view', '查看版本', 'aup', '可查看計畫版本歷史', NOW()),
    (gen_random_uuid(), 'animal.animal.view_all', '查看所有動物', 'animal', '可查看所有動物資料', NOW()),
    (gen_random_uuid(), 'animal.animal.view_project', '查看計畫內動物', 'animal', '可查看計畫內的動物', NOW()),
    (gen_random_uuid(), 'animal.animal.create', '新增動物', 'animal', '可新增動物', NOW()),
    (gen_random_uuid(), 'animal.animal.edit', '編輯動物資料', 'animal', '可編輯動物基本資料', NOW()),
    (gen_random_uuid(), 'animal.animal.assign', '分配動物至計畫', 'animal', '可將動物分配至計畫', NOW()),
    (gen_random_uuid(), 'animal.animal.import', '匯入動物資料', 'animal', '可批次匯入動物資料', NOW()),
    (gen_random_uuid(), 'animal.animal.delete', '刪除動物', 'animal', '可刪除動物資料', NOW()),
    (gen_random_uuid(), 'animal.record.view', '查看紀錄', 'animal', '可查看動物相關紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.create', '新增紀錄', 'animal', '可新增動物相關紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.edit', '編輯紀錄', 'animal', '可編輯動物相關紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.delete', '刪除紀錄', 'animal', '可刪除動物相關紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.observation', '新增觀察紀錄', 'animal', '可新增觀察紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.surgery', '新增手術紀錄', 'animal', '可新增手術紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.weight', '新增體重紀錄', 'animal', '可新增體重紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.vaccine', '新增疫苗紀錄', 'animal', '可新增疫苗紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.sacrifice', '新增犧牲紀錄', 'animal', '可新增犧牲紀錄', NOW()),
    (gen_random_uuid(), 'animal.vet.recommend', '新增獸醫師建議', 'animal', '可新增獸醫師建議', NOW()),
    (gen_random_uuid(), 'animal.vet.read', '標記獸醫師已讀', 'animal', '可標記紀錄已讀', NOW()),
    (gen_random_uuid(), 'animal.export.medical', '匯出病歷', 'animal', '可匯出動物病歷', NOW()),
    (gen_random_uuid(), 'animal.export.observation', '匯出觀察紀錄', 'animal', '可匯出觀察紀錄', NOW()),
    (gen_random_uuid(), 'animal.export.surgery', '匯出手術紀錄', 'animal', '可匯出手術紀錄', NOW()),
    (gen_random_uuid(), 'animal.export.experiment', '匯出實驗紀錄', 'animal', '可匯出實驗紀錄', NOW()),
    (gen_random_uuid(), 'animal.emergency.stop', '緊急停止', 'animal', '可緊急停止實驗', NOW()),
    (gen_random_uuid(), 'animal.euthanasia.recommend', '提出安樂死建議', 'animal', '可建議動物安樂死', NOW()),
    (gen_random_uuid(), 'animal.euthanasia.approve', '批准安樂死', 'animal', '可批准安樂死建議', NOW()),
    (gen_random_uuid(), 'animal.pathology.view', '查看病理報告', 'animal', '可查看動物病理報告', NOW()),
    (gen_random_uuid(), 'animal.pathology.upload', '上傳病理報告', 'animal', '可上傳動物病理報告', NOW()),
    (gen_random_uuid(), 'animal.info.assign', '分配動物資訊', 'animal', '可分配動物至計畫', NOW()),
    (gen_random_uuid(), 'animal.info.edit', '編輯動物資訊', 'animal', '可編輯動物資訊', NOW()),
    (gen_random_uuid(), 'animal.record.emergency', '緊急給藥', 'animal', '可執行緊急給藥', NOW()),
    (gen_random_uuid(), 'animal.record.copy', '複製紀錄', 'animal', '可複製動物紀錄', NOW()),
    (gen_random_uuid(), 'animal.vet.upload_attachment', '上傳獸醫附件', 'animal', '可上傳獸醫建議附件', NOW()),
    (gen_random_uuid(), 'erp.warehouse.view', '查看倉庫', 'erp', '可查看倉庫資料', NOW()),
    (gen_random_uuid(), 'erp.warehouse.create', '建立倉庫', 'erp', '可建立倉庫', NOW()),
    (gen_random_uuid(), 'erp.warehouse.edit', '編輯倉庫', 'erp', '可編輯倉庫', NOW()),
    (gen_random_uuid(), 'erp.product.view', '查看產品', 'erp', '可查看產品資料', NOW()),
    (gen_random_uuid(), 'erp.product.create', '建立產品', 'erp', '可建立產品', NOW()),
    (gen_random_uuid(), 'erp.product.edit', '編輯產品', 'erp', '可編輯產品', NOW()),
    (gen_random_uuid(), 'erp.partner.view', '查看夥伴', 'erp', '可查看夥伴資料', NOW()),
    (gen_random_uuid(), 'erp.partner.create', '建立夥伴', 'erp', '可建立夥伴', NOW()),
    (gen_random_uuid(), 'erp.partner.edit', '編輯夥伴', 'erp', '可編輯夥伴', NOW()),
    (gen_random_uuid(), 'erp.document.view', '查看單據', 'erp', '可查看單據', NOW()),
    (gen_random_uuid(), 'erp.document.create', '建立單據', 'erp', '可建立單據', NOW()),
    (gen_random_uuid(), 'erp.document.edit', '編輯單據', 'erp', '可編輯單據', NOW()),
    (gen_random_uuid(), 'erp.document.submit', '送審單據', 'erp', '可送審單據', NOW()),
    (gen_random_uuid(), 'erp.document.approve', '核准單據', 'erp', '可核准單據', NOW()),
    (gen_random_uuid(), 'erp.inventory.view', '查看庫存', 'erp', '可查看庫存現況', NOW()),
    (gen_random_uuid(), 'erp.purchase.create', '建立採購單', 'erp', '可建立採購單', NOW()),
    (gen_random_uuid(), 'erp.purchase.approve', '核准採購單', 'erp', '可核准採購單', NOW()),
    (gen_random_uuid(), 'erp.grn.create', '建立進貨單', 'erp', '可建立進貨單', NOW()),
    (gen_random_uuid(), 'erp.pr.create', '建立採購退貨', 'erp', '可建立採購退貨', NOW()),
    (gen_random_uuid(), 'erp.stock.in', '入庫操作', 'erp', '可執行入庫操作', NOW()),
    (gen_random_uuid(), 'erp.stock.out', '出庫操作', 'erp', '可執行出庫操作', NOW()),
    (gen_random_uuid(), 'erp.stock.view', '查看庫存', 'erp', '可查看庫存', NOW()),
    (gen_random_uuid(), 'erp.stock.adjust', '庫存調整', 'erp', '可執行庫存調整', NOW()),
    (gen_random_uuid(), 'erp.stock.transfer', '調撥', 'erp', '可執行庫存調撥', NOW()),
    (gen_random_uuid(), 'erp.stocktake.create', '盤點', 'erp', '可執行庫存盤點', NOW()),
    (gen_random_uuid(), 'erp.report.view', '查看報表', 'erp', '可查看 ERP 報表', NOW()),
    (gen_random_uuid(), 'erp.report.export', '匯出報表', 'erp', '可匯出 ERP 報表', NOW()),
    (gen_random_uuid(), 'erp.report.download', '下載報表', 'erp', '可下載報表', NOW()),
    (gen_random_uuid(), 'erp.storage.view', '查看儲位', 'erp', '可查看儲位', NOW()),
    (gen_random_uuid(), 'erp.storage.edit', '編輯儲位', 'erp', '可編輯儲位', NOW()),
    (gen_random_uuid(), 'erp.storage.inventory.view', '查看儲位庫存', 'erp', '可查看儲位庫存', NOW()),
    (gen_random_uuid(), 'erp.storage.inventory.edit', '編輯儲位庫存', 'erp', '可編輯儲位庫存', NOW()),
    (gen_random_uuid(), 'hr.attendance.view', '查看出勤紀錄', 'hr', '可查看出勤紀錄', NOW()),
    (gen_random_uuid(), 'hr.attendance.view_all', '查看所有出勤', 'hr', '可查看所有人的出勤紀錄', NOW()),
    (gen_random_uuid(), 'hr.attendance.clock', '打卡', 'hr', '可進行上下班打卡', NOW()),
    (gen_random_uuid(), 'hr.attendance.correct', '更正打卡', 'hr', '可更正打卡紀錄', NOW()),
    (gen_random_uuid(), 'hr.overtime.view', '查看加班紀錄', 'hr', '可查看加班紀錄', NOW()),
    (gen_random_uuid(), 'hr.overtime.create', '申請加班', 'hr', '可申請加班', NOW()),
    (gen_random_uuid(), 'hr.overtime.approve', '審核加班', 'hr', '可審核加班申請', NOW()),
    (gen_random_uuid(), 'hr.leave.view', '查看請假', 'hr', '可查看請假紀錄', NOW()),
    (gen_random_uuid(), 'hr.leave.view_all', '查看所有請假', 'hr', '可查看所有人的請假紀錄', NOW()),
    (gen_random_uuid(), 'hr.leave.create', '申請請假', 'hr', '可申請請假', NOW()),
    (gen_random_uuid(), 'hr.leave.approve', '審核請假', 'hr', '可審核請假申請', NOW()),
    (gen_random_uuid(), 'hr.leave.manage', '管理假別', 'hr', '可管理假別設定', NOW()),
    (gen_random_uuid(), 'hr.balance.view', '查看餘額', 'hr', '可查看假期餘額', NOW()),
    (gen_random_uuid(), 'hr.balance.view_all', '查看所有餘額', 'hr', '可查看所有人的假期餘額', NOW()),
    (gen_random_uuid(), 'hr.balance.manage', '管理餘額', 'hr', '可管理假期餘額', NOW()),
    (gen_random_uuid(), 'audit.logs.view', '查看稽核日誌', 'audit', '可查看稽核日誌', NOW()),
    (gen_random_uuid(), 'audit.logs.export', '匯出稽核日誌', 'audit', '可匯出稽核日誌', NOW()),
    (gen_random_uuid(), 'audit.timeline.view', '查看活動時間軸', 'audit', '可查看使用者活動時間軸', NOW()),
    (gen_random_uuid(), 'audit.alerts.view', '查看安全警報', 'audit', '可查看安全警報', NOW()),
    (gen_random_uuid(), 'audit.alerts.manage', '管理安全警報', 'audit', '可處理安全警報', NOW()),
    (gen_random_uuid(), 'amendment.create', '建立修正案', 'aup', '可建立計畫修正案', NOW()),
    (gen_random_uuid(), 'amendment.submit', '提交修正案', 'aup', '可提交修正案送審', NOW()),
    (gen_random_uuid(), 'amendment.read', '查看修正案', 'aup', '可查看修正案內容', NOW()),
    (gen_random_uuid(), 'amendment.review', '審查修正案', 'aup', '可審查修正案', NOW()),
    (gen_random_uuid(), 'notification.view', '查看通知', 'notification', '可查看自己的通知', NOW()),
    (gen_random_uuid(), 'notification.manage', '管理通知設定', 'notification', '可管理通知設定', NOW()),
    (gen_random_uuid(), 'notification.send', '發送通知', 'notification', '可發送系統通知', NOW()),
    (gen_random_uuid(), 'report.schedule', '排程報表', 'report', '可設定定期報表', NOW()),
    (gen_random_uuid(), 'report.download', '下載報表', 'report', '可下載報表檔案', NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 9. 角色權限分配
-- ============================================

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p WHERE r.code = 'admin'
ON CONFLICT (role_id, permission_id) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'EXPERIMENT_STAFF' AND p.code IN (
    'animal.animal.view_all','animal.animal.view_project','animal.animal.create','animal.animal.edit',
    'animal.animal.assign','animal.animal.import','animal.animal.delete','animal.record.view',
    'animal.record.create','animal.record.edit','animal.record.delete','animal.record.observation',
    'animal.record.surgery','animal.record.weight','animal.record.vaccine','animal.record.sacrifice',
    'animal.export.medical','animal.export.observation','animal.export.surgery','animal.export.experiment',
    'animal.pathology.view','animal.pathology.upload','animal.info.assign','animal.info.edit',
    'animal.record.emergency','animal.record.copy','animal.vet.upload_attachment',
    'aup.protocol.view_own','aup.attachment.view','aup.attachment.download',
    'hr.attendance.view','hr.attendance.clock','hr.leave.view','hr.leave.create',
    'hr.overtime.view','hr.overtime.create','hr.balance.view','erp.inventory.view','erp.product.view','notification.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'VET' AND p.code IN (
    'aup.protocol.view_all','aup.protocol.review','aup.review.view','aup.review.comment',
    'amendment.read','amendment.review','animal.animal.view_all','animal.animal.view_project',
    'animal.record.view','animal.export.medical','animal.export.observation','animal.export.surgery',
    'animal.export.experiment','animal.vet.recommend','animal.vet.read','animal.emergency.stop',
    'animal.euthanasia.recommend','animal.euthanasia.approve','animal.pathology.view','animal.pathology.upload',
    'animal.info.assign','animal.info.edit','animal.record.emergency','animal.record.copy','animal.vet.upload_attachment','notification.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'PI' AND p.code IN (
    'aup.protocol.view_own','aup.protocol.create','aup.protocol.edit','aup.protocol.submit',
    'aup.attachment.view','aup.attachment.download','aup.version.view','amendment.create','amendment.submit','amendment.read',
    'animal.animal.view_project','animal.record.view','notification.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'IACUC_STAFF' AND p.code IN (
    'aup.protocol.view_all','aup.protocol.change_status','aup.review.assign','aup.attachment.view',
    'aup.attachment.download','aup.version.view','amendment.read','admin.user.view',
    'animal.animal.view_all','animal.record.view','notification.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'CLIENT' AND p.code IN (
    'aup.protocol.view_own','aup.attachment.view','animal.animal.view_project','animal.record.view','notification.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'REVIEWER' AND p.code IN (
    'aup.protocol.view_all','aup.protocol.review','aup.review.view','aup.review.comment',
    'aup.attachment.view','aup.attachment.download','aup.version.view','amendment.read','amendment.review','notification.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'IACUC_CHAIR' AND p.code IN (
    'aup.protocol.view_all','aup.protocol.review','aup.protocol.approve','aup.protocol.change_status',
    'aup.review.view','aup.review.assign','aup.review.comment','aup.attachment.view','aup.attachment.download',
    'aup.version.view','amendment.read','amendment.review','notification.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'WAREHOUSE_MANAGER' AND p.code IN (
    'erp.warehouse.view','erp.warehouse.create','erp.warehouse.edit','erp.product.view','erp.product.create','erp.product.edit',
    'erp.partner.view','erp.partner.create','erp.partner.edit','erp.document.view','erp.document.create','erp.document.edit',
    'erp.document.submit','erp.document.approve','erp.inventory.view','erp.purchase.create','erp.purchase.approve',
    'erp.grn.create','erp.pr.create','erp.stock.in','erp.stock.out','erp.stock.view','erp.stock.adjust','erp.stock.transfer',
    'erp.stocktake.create','erp.report.view','erp.report.export','erp.report.download','erp.storage.view','erp.storage.edit',
    'erp.storage.inventory.view','erp.storage.inventory.edit','notification.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'ADMIN_STAFF' AND p.code IN (
    'admin.user.view','admin.user.view_all','admin.user.create','admin.user.edit','admin.role.view',
    'hr.attendance.view','hr.attendance.view_all','hr.attendance.clock','hr.attendance.correct',
    'hr.leave.view','hr.leave.view_all','hr.leave.create','hr.leave.approve','hr.leave.manage',
    'hr.overtime.view','hr.overtime.create','hr.overtime.approve','hr.balance.view','hr.balance.view_all','hr.balance.manage','notification.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- ============================================
-- 10. 使用者偏好設定表
-- ============================================

CREATE TABLE user_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    preference_key VARCHAR(100) NOT NULL,
    preference_value JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, preference_key)
);
CREATE INDEX idx_user_preferences_user_id ON user_preferences(user_id);
