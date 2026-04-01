-- ============================================================
-- Migration 002: 認證與使用者（最終狀態）
-- 來源: 002_users_auth.sql, 003_notifications_roles_seed.sql,
--       008_supplementary.sql (+totp columns, +version),
--       009_glp_extensions.sql (+QAU/EQUIPMENT_MAINTENANCE roles),
--       018_equipment_maintenance.sql (+equipment perms/roles),
--       023_intern_role_and_user_expiry.sql (+INTERN, +expires_at),
--       024_invitations.sql, 029_qa_plan.sql (+QAU qa perms)
-- ============================================================

-- ── users ────────────────────────────────────────────────────
CREATE TABLE users (
    id                      UUID         PRIMARY KEY,
    email                   VARCHAR(255) NOT NULL UNIQUE,
    password_hash           VARCHAR(255) NOT NULL,
    display_name            VARCHAR(100) NOT NULL,
    phone                   VARCHAR(20),
    phone_ext               VARCHAR(20),
    organization            VARCHAR(200),
    is_internal             BOOLEAN      NOT NULL DEFAULT true,
    is_active               BOOLEAN      NOT NULL DEFAULT true,
    must_change_password    BOOLEAN      NOT NULL DEFAULT true,
    last_login_at           TIMESTAMPTZ,
    login_attempts          INTEGER      NOT NULL DEFAULT 0,
    locked_until            TIMESTAMPTZ,
    theme_preference        VARCHAR(20)  NOT NULL DEFAULT 'light',
    language_preference     VARCHAR(10)  NOT NULL DEFAULT 'zh-TW',
    entry_date              DATE,
    position                VARCHAR(100),
    aup_roles               VARCHAR(255)[] DEFAULT '{}',
    years_experience        INTEGER      NOT NULL DEFAULT 0,
    trainings               JSONB        NOT NULL DEFAULT '[]',
    deleted_at              TIMESTAMPTZ,
    created_at              TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    -- 008: TOTP 2FA
    totp_enabled            BOOLEAN      NOT NULL DEFAULT FALSE,
    totp_secret_encrypted   TEXT,
    totp_backup_codes       TEXT[],
    -- 008: 樂觀鎖
    version                 INTEGER      NOT NULL DEFAULT 1,
    -- 023: 帳號到期（實習生等臨時帳號）
    expires_at              TIMESTAMPTZ  DEFAULT NULL,
    CONSTRAINT chk_theme_preference    CHECK (theme_preference    IN ('light', 'dark', 'system')),
    CONSTRAINT chk_language_preference CHECK (language_preference IN ('zh-TW', 'en'))
);
CREATE INDEX idx_users_email      ON users(email);
CREATE INDEX idx_users_is_active  ON users(is_active);
CREATE INDEX idx_users_expires_at ON users(expires_at) WHERE expires_at IS NOT NULL;
COMMENT ON COLUMN users.expires_at IS '帳號到期日，NULL 表示永不過期。用於實習生等臨時帳號';

-- ── roles ────────────────────────────────────────────────────
CREATE TABLE roles (
    id          UUID        PRIMARY KEY,
    code        VARCHAR(50) NOT NULL UNIQUE,
    name        VARCHAR(100) NOT NULL,
    description TEXT,
    is_internal BOOLEAN     NOT NULL DEFAULT true,
    is_system   BOOLEAN     NOT NULL DEFAULT false,
    is_active   BOOLEAN     NOT NULL DEFAULT true,
    is_deleted  BOOLEAN     NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── permissions ──────────────────────────────────────────────
CREATE TABLE permissions (
    id          UUID         PRIMARY KEY,
    code        VARCHAR(100) NOT NULL UNIQUE,
    name        VARCHAR(200) NOT NULL,
    module      VARCHAR(50),
    description TEXT,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- ── role_permissions ─────────────────────────────────────────
CREATE TABLE role_permissions (
    role_id       UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- ── user_roles ───────────────────────────────────────────────
CREATE TABLE user_roles (
    user_id     UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id     UUID        NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ DEFAULT NOW(),
    assigned_by UUID        REFERENCES users(id),
    PRIMARY KEY (user_id, role_id)
);

-- ── refresh_tokens ───────────────────────────────────────────
CREATE TABLE refresh_tokens (
    id         UUID         PRIMARY KEY,
    user_id    UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ  NOT NULL,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);
CREATE INDEX idx_refresh_tokens_user_id    ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);

-- ── password_reset_tokens ────────────────────────────────────
CREATE TABLE password_reset_tokens (
    id         UUID         PRIMARY KEY,
    user_id    UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ  NOT NULL,
    used_at    TIMESTAMPTZ,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_password_reset_tokens_user_id    ON password_reset_tokens(user_id);
CREATE INDEX idx_password_reset_tokens_token_hash ON password_reset_tokens(token_hash);

-- ── invitations (024) ────────────────────────────────────────
CREATE TABLE invitations (
    id               UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    email            VARCHAR(255) NOT NULL,
    organization     VARCHAR(255),
    invitation_token VARCHAR(255) UNIQUE NOT NULL,
    invited_by       UUID         NOT NULL REFERENCES users(id),
    status           VARCHAR(20)  NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'accepted', 'expired', 'revoked')),
    expires_at       TIMESTAMPTZ  NOT NULL,
    accepted_at      TIMESTAMPTZ,
    created_user_id  UUID         REFERENCES users(id),
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX idx_invitations_email_pending ON invitations(email) WHERE status = 'pending';
CREATE INDEX idx_invitations_token      ON invitations(invitation_token);
CREATE INDEX idx_invitations_expires_at ON invitations(expires_at) WHERE status = 'pending';

-- ============================================================
-- Seed: Roles
-- 來源: 003, 009, 023
-- ============================================================
INSERT INTO roles (id, code, name, description, is_internal, is_system, created_at, updated_at) VALUES
    (gen_random_uuid(), 'admin',                '系統管理員',   '全系統最高權限，使用者管理、系統維運',                                      true,  true, NOW(), NOW()),
    (gen_random_uuid(), 'ADMIN_STAFF',          '行政人員',     '處理行政事務、協助管理',                                                    true,  true, NOW(), NOW()),
    (gen_random_uuid(), 'WAREHOUSE_MANAGER',    '倉庫管理員',   '專責 ERP 進銷存系統',                                                      true,  true, NOW(), NOW()),
    (gen_random_uuid(), 'PURCHASING',           '採購人員',     '負責採購作業',                                                              true,  true, NOW(), NOW()),
    (gen_random_uuid(), 'PI',                   '計畫主持人',   '提交計畫、管理自己的計畫與動物',                                            false, true, NOW(), NOW()),
    (gen_random_uuid(), 'VET',                  '獸醫師',       '審查計畫、動物健康管理、提供建議',                                          true,  true, NOW(), NOW()),
    (gen_random_uuid(), 'REVIEWER',             '審查委員',     'IACUC 計畫審查',                                                            true,  true, NOW(), NOW()),
    (gen_random_uuid(), 'IACUC_CHAIR',          'IACUC 主席',   '主導審查決策',                                                              true,  true, NOW(), NOW()),
    (gen_random_uuid(), 'IACUC_STAFF',          '執行秘書',     '行政流程管理、管理所有計劃進度',                                            true,  true, NOW(), NOW()),
    (gen_random_uuid(), 'EXPERIMENT_STAFF',     '試驗工作人員', '執行實驗操作、記錄數據',                                                    true,  true, NOW(), NOW()),
    (gen_random_uuid(), 'CLIENT',               '委託人',       '查看委託計畫與動物紀錄',                                                    false, true, NOW(), NOW()),
    (gen_random_uuid(), 'QAU',                  '品質保證單位', 'GLP 合規：獨立於研究執行，可檢視研究狀態、審查進度、稽核摘要、動物紀錄',    true,  true, NOW(), NOW()),
    (gen_random_uuid(), 'EQUIPMENT_MAINTENANCE','設備維護人員', '設備與校準紀錄管理',                                                        true,  true, NOW(), NOW()),
    (gen_random_uuid(), 'INTERN',               '實習生',       '實習人員，權限同試驗工作人員，帳號有到期日',                                true,  true, NOW(), NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================================
-- Seed: Permissions
-- 來源: 003, 009, 018, 029
-- ============================================================
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    -- admin
    (gen_random_uuid(), 'admin.user.view',            '查看使用者',       'admin', '可查看使用者列表',           NOW()),
    (gen_random_uuid(), 'admin.user.view_all',        '查看所有使用者',   'admin', '可查看所有使用者資料',       NOW()),
    (gen_random_uuid(), 'admin.user.create',          '建立使用者',       'admin', '可建立新使用者帳號',         NOW()),
    (gen_random_uuid(), 'admin.user.edit',            '編輯使用者',       'admin', '可編輯使用者資料',           NOW()),
    (gen_random_uuid(), 'admin.user.delete',          '停用使用者',       'admin', '可停用使用者帳號',           NOW()),
    (gen_random_uuid(), 'admin.user.reset_password',  '重設密碼',         'admin', '可重設他人密碼',             NOW()),
    (gen_random_uuid(), 'admin.role.view',            '查看角色',         'admin', '可查看角色列表',             NOW()),
    (gen_random_uuid(), 'admin.role.manage',          '管理角色',         'admin', '可管理角色定義',             NOW()),
    (gen_random_uuid(), 'admin.permission.manage',    '管理權限',         'admin', '可管理權限定義',             NOW()),
    (gen_random_uuid(), 'admin.audit.view',           '查看稽核紀錄',     'admin', '可查看系統稽核紀錄',         NOW()),
    -- aup
    (gen_random_uuid(), 'aup.protocol.view_all',      '查看所有計畫',     'aup', '可查看系統中所有計畫',         NOW()),
    (gen_random_uuid(), 'aup.protocol.view_own',      '查看自己的計畫',   'aup', '可查看自己相關的計畫',         NOW()),
    (gen_random_uuid(), 'aup.protocol.create',        '建立計畫',         'aup', '可建立新計畫',                 NOW()),
    (gen_random_uuid(), 'aup.protocol.edit',          '編輯計畫',         'aup', '可編輯計畫草稿',               NOW()),
    (gen_random_uuid(), 'aup.protocol.submit',        '提交計畫',         'aup', '可提交計畫送審',               NOW()),
    (gen_random_uuid(), 'aup.protocol.review',        '審查計畫',         'aup', '可審查計畫並提供意見',         NOW()),
    (gen_random_uuid(), 'aup.protocol.approve',       '核准/否決',        'aup', '可核准或否決計畫',             NOW()),
    (gen_random_uuid(), 'aup.protocol.change_status', '變更狀態',         'aup', '可變更計畫狀態',               NOW()),
    (gen_random_uuid(), 'aup.protocol.delete',        '刪除計畫',         'aup', '可刪除計畫',                   NOW()),
    (gen_random_uuid(), 'aup.review.view',            '查看審查',         'aup', '可查看審查意見',               NOW()),
    (gen_random_uuid(), 'aup.review.assign',          '指派審查人員',     'aup', '可指派審查人員',               NOW()),
    (gen_random_uuid(), 'aup.review.comment',         '新增審查意見',     'aup', '可新增審查意見',               NOW()),
    (gen_random_uuid(), 'aup.attachment.view',        '查看附件',         'aup', '可查看計畫附件',               NOW()),
    (gen_random_uuid(), 'aup.attachment.download',    '下載附件',         'aup', '可下載計畫附件',               NOW()),
    (gen_random_uuid(), 'aup.version.view',           '查看版本',         'aup', '可查看計畫版本歷史',           NOW()),
    -- animal
    (gen_random_uuid(), 'animal.animal.view_all',             '查看所有動物',       'animal', '可查看所有動物資料',                             NOW()),
    (gen_random_uuid(), 'animal.animal.view_project',         '查看計畫內動物',     'animal', '可查看計畫內的動物',                             NOW()),
    (gen_random_uuid(), 'animal.animal.create',               '新增動物',           'animal', '可新增動物',                                   NOW()),
    (gen_random_uuid(), 'animal.animal.edit',                 '編輯動物資料',       'animal', '可編輯動物基本資料',                             NOW()),
    (gen_random_uuid(), 'animal.animal.assign',               '分配動物至計畫',     'animal', '可將動物分配至計畫',                             NOW()),
    (gen_random_uuid(), 'animal.animal.import',               '匯入動物資料',       'animal', '可批次匯入動物資料',                             NOW()),
    (gen_random_uuid(), 'animal.animal.delete',               '刪除動物',           'animal', '可刪除動物資料',                               NOW()),
    (gen_random_uuid(), 'animal.record.view',                 '查看紀錄',           'animal', '可查看動物相關紀錄',                             NOW()),
    (gen_random_uuid(), 'animal.record.create',               '新增紀錄',           'animal', '可新增動物相關紀錄',                             NOW()),
    (gen_random_uuid(), 'animal.record.edit',                 '編輯紀錄',           'animal', '可編輯動物相關紀錄',                             NOW()),
    (gen_random_uuid(), 'animal.record.delete',               '刪除紀錄',           'animal', '可刪除動物相關紀錄',                             NOW()),
    (gen_random_uuid(), 'animal.blood_test_template.manage',  '血檢項目管理',       'animal', '可檢視與編輯血檢項目模板、組合、常用組合',         NOW()),
    (gen_random_uuid(), 'animal.record.observation',          '新增觀察紀錄',       'animal', '可新增觀察紀錄',                               NOW()),
    (gen_random_uuid(), 'animal.record.surgery',              '新增手術紀錄',       'animal', '可新增手術紀錄',                               NOW()),
    (gen_random_uuid(), 'animal.record.weight',               '新增體重紀錄',       'animal', '可新增體重紀錄',                               NOW()),
    (gen_random_uuid(), 'animal.record.vaccine',              '新增疫苗紀錄',       'animal', '可新增疫苗紀錄',                               NOW()),
    (gen_random_uuid(), 'animal.record.sacrifice',            '新增犧牲紀錄',       'animal', '可新增犧牲紀錄',                               NOW()),
    (gen_random_uuid(), 'animal.vet.recommend',               '新增獸醫師建議',     'animal', '可新增獸醫師建議',                               NOW()),
    (gen_random_uuid(), 'animal.vet.read',                    '標記獸醫師已讀',     'animal', '可標記紀錄已讀',                               NOW()),
    (gen_random_uuid(), 'animal.export.medical',              '匯出病歷',           'animal', '可匯出動物病歷',                               NOW()),
    (gen_random_uuid(), 'animal.export.observation',          '匯出觀察紀錄',       'animal', '可匯出觀察紀錄',                               NOW()),
    (gen_random_uuid(), 'animal.export.surgery',              '匯出手術紀錄',       'animal', '可匯出手術紀錄',                               NOW()),
    (gen_random_uuid(), 'animal.export.experiment',           '匯出實驗紀錄',       'animal', '可匯出實驗紀錄',                               NOW()),
    (gen_random_uuid(), 'animal.emergency.stop',              '緊急停止',           'animal', '可緊急停止實驗',                               NOW()),
    (gen_random_uuid(), 'animal.euthanasia.recommend',        '提出安樂死建議',     'animal', '可建議動物安樂死',                               NOW()),
    (gen_random_uuid(), 'animal.euthanasia.approve',          '批准安樂死',         'animal', '可批准安樂死建議',                               NOW()),
    (gen_random_uuid(), 'animal.pathology.view',              '查看病理報告',       'animal', '可查看動物病理報告',                             NOW()),
    (gen_random_uuid(), 'animal.pathology.upload',            '上傳病理報告',       'animal', '可上傳動物病理報告',                             NOW()),
    (gen_random_uuid(), 'animal.info.assign',                 '分配動物資訊',       'animal', '可分配動物至計畫',                               NOW()),
    (gen_random_uuid(), 'animal.info.edit',                   '編輯動物資訊',       'animal', '可編輯動物資訊',                               NOW()),
    (gen_random_uuid(), 'animal.record.emergency',            '緊急給藥',           'animal', '可執行緊急給藥',                               NOW()),
    (gen_random_uuid(), 'animal.record.copy',                 '複製紀錄',           'animal', '可複製動物紀錄',                               NOW()),
    (gen_random_uuid(), 'animal.vet.upload_attachment',       '上傳獸醫附件',       'animal', '可上傳獸醫建議附件',                             NOW()),
    -- erp
    (gen_random_uuid(), 'erp.warehouse.view',           '查看倉庫',     'erp', '可查看倉庫資料',         NOW()),
    (gen_random_uuid(), 'erp.warehouse.create',         '建立倉庫',     'erp', '可建立倉庫',             NOW()),
    (gen_random_uuid(), 'erp.warehouse.edit',           '編輯倉庫',     'erp', '可編輯倉庫',             NOW()),
    (gen_random_uuid(), 'erp.warehouse.delete',         '刪除倉庫',     'erp', '可刪除倉庫',             NOW()),
    (gen_random_uuid(), 'erp.product.view',             '查看產品',     'erp', '可查看產品資料',         NOW()),
    (gen_random_uuid(), 'erp.product.create',           '建立產品',     'erp', '可建立產品',             NOW()),
    (gen_random_uuid(), 'erp.product.edit',             '編輯產品',     'erp', '可編輯產品',             NOW()),
    (gen_random_uuid(), 'erp.partner.view',             '查看夥伴',     'erp', '可查看夥伴資料',         NOW()),
    (gen_random_uuid(), 'erp.partner.create',           '建立夥伴',     'erp', '可建立夥伴',             NOW()),
    (gen_random_uuid(), 'erp.partner.edit',             '編輯夥伴',     'erp', '可編輯夥伴',             NOW()),
    (gen_random_uuid(), 'erp.document.view',            '查看單據',     'erp', '可查看單據',             NOW()),
    (gen_random_uuid(), 'erp.document.create',          '建立單據',     'erp', '可建立單據',             NOW()),
    (gen_random_uuid(), 'erp.document.edit',            '編輯單據',     'erp', '可編輯單據',             NOW()),
    (gen_random_uuid(), 'erp.document.submit',          '送審單據',     'erp', '可送審單據',             NOW()),
    (gen_random_uuid(), 'erp.document.approve',         '核准單據',     'erp', '可核准單據',             NOW()),
    (gen_random_uuid(), 'erp.purchase.create',          '建立採購單',   'erp', '可建立採購單',           NOW()),
    (gen_random_uuid(), 'erp.purchase.approve',         '核准採購單',   'erp', '可核准採購單',           NOW()),
    (gen_random_uuid(), 'erp.grn.create',               '建立進貨單',   'erp', '可建立進貨單',           NOW()),
    (gen_random_uuid(), 'erp.pr.create',                '建立採購退貨', 'erp', '可建立採購退貨',         NOW()),
    (gen_random_uuid(), 'erp.stock.in',                 '入庫操作',     'erp', '可執行入庫操作',         NOW()),
    (gen_random_uuid(), 'erp.stock.out',                '出庫操作',     'erp', '可執行出庫操作',         NOW()),
    (gen_random_uuid(), 'erp.stock.view',               '查看庫存',     'erp', '可查看庫存',             NOW()),
    (gen_random_uuid(), 'erp.stock.adjust',             '庫存調整',     'erp', '可執行庫存調整',         NOW()),
    (gen_random_uuid(), 'erp.stock.transfer',           '調撥',         'erp', '可執行庫存調撥',         NOW()),
    (gen_random_uuid(), 'erp.stocktake.create',         '盤點',         'erp', '可執行庫存盤點',         NOW()),
    (gen_random_uuid(), 'erp.report.view',              '查看報表',     'erp', '可查看 ERP 報表',        NOW()),
    (gen_random_uuid(), 'erp.report.export',            '匯出報表',     'erp', '可匯出 ERP 報表',        NOW()),
    (gen_random_uuid(), 'erp.report.download',          '下載報表',     'erp', '可下載報表',             NOW()),
    (gen_random_uuid(), 'erp.storage.view',             '查看儲位',     'erp', '可查看儲位',             NOW()),
    (gen_random_uuid(), 'erp.storage.edit',             '編輯儲位',     'erp', '可編輯儲位',             NOW()),
    (gen_random_uuid(), 'erp.storage.inventory.view',   '查看儲位庫存', 'erp', '可查看儲位庫存',         NOW()),
    (gen_random_uuid(), 'erp.storage.inventory.edit',   '編輯儲位庫存', 'erp', '可編輯儲位庫存',         NOW()),
    -- hr
    (gen_random_uuid(), 'hr.attendance.view',       '查看出勤紀錄', 'hr', '可查看出勤紀錄',             NOW()),
    (gen_random_uuid(), 'hr.attendance.view_all',   '查看所有出勤', 'hr', '可查看所有人的出勤紀錄',     NOW()),
    (gen_random_uuid(), 'hr.attendance.clock',      '打卡',         'hr', '可進行上下班打卡',           NOW()),
    (gen_random_uuid(), 'hr.attendance.correct',    '更正打卡',     'hr', '可更正打卡紀錄',             NOW()),
    (gen_random_uuid(), 'hr.overtime.view',         '查看加班紀錄', 'hr', '可查看加班紀錄',             NOW()),
    (gen_random_uuid(), 'hr.overtime.create',       '申請加班',     'hr', '可申請加班',                 NOW()),
    (gen_random_uuid(), 'hr.overtime.approve',      '審核加班',     'hr', '可審核加班申請',             NOW()),
    (gen_random_uuid(), 'hr.leave.view',            '查看請假',     'hr', '可查看請假紀錄',             NOW()),
    (gen_random_uuid(), 'hr.leave.view_all',        '查看所有請假', 'hr', '可查看所有人的請假紀錄',     NOW()),
    (gen_random_uuid(), 'hr.leave.create',          '申請請假',     'hr', '可申請請假',                 NOW()),
    (gen_random_uuid(), 'hr.leave.approve',         '審核請假',     'hr', '可審核請假申請',             NOW()),
    (gen_random_uuid(), 'hr.leave.manage',          '管理假別',     'hr', '可管理假別設定',             NOW()),
    (gen_random_uuid(), 'hr.balance.view',          '查看餘額',     'hr', '可查看假期餘額',             NOW()),
    (gen_random_uuid(), 'hr.balance.view_all',      '查看所有餘額', 'hr', '可查看所有人的假期餘額',     NOW()),
    (gen_random_uuid(), 'hr.balance.manage',        '管理餘額',     'hr', '可管理假期餘額',             NOW()),
    -- audit
    (gen_random_uuid(), 'audit.logs.view',       '查看稽核日誌',     'audit', '可查看稽核日誌',             NOW()),
    (gen_random_uuid(), 'audit.logs.export',     '匯出稽核日誌',     'audit', '可匯出稽核日誌',             NOW()),
    (gen_random_uuid(), 'audit.timeline.view',   '查看活動時間軸',   'audit', '可查看使用者活動時間軸',     NOW()),
    (gen_random_uuid(), 'audit.alerts.view',     '查看安全警報',     'audit', '可查看安全警報',             NOW()),
    (gen_random_uuid(), 'audit.alerts.manage',   '管理安全警報',     'audit', '可處理安全警報',             NOW()),
    -- amendment
    (gen_random_uuid(), 'amendment.create', '建立修正案', 'aup', '可建立計畫修正案', NOW()),
    (gen_random_uuid(), 'amendment.submit', '提交修正案', 'aup', '可提交修正案送審', NOW()),
    (gen_random_uuid(), 'amendment.read',   '查看修正案', 'aup', '可查看修正案內容', NOW()),
    (gen_random_uuid(), 'amendment.review', '審查修正案', 'aup', '可審查修正案',     NOW()),
    -- notification / report
    (gen_random_uuid(), 'notification.view',    '查看通知',     'notification', '可查看自己的通知',  NOW()),
    (gen_random_uuid(), 'notification.manage',  '管理通知設定', 'notification', '可管理通知設定',    NOW()),
    (gen_random_uuid(), 'notification.send',    '發送通知',     'notification', '可發送系統通知',    NOW()),
    (gen_random_uuid(), 'report.schedule',      '排程報表',     'report',       '可設定定期報表',    NOW()),
    (gen_random_uuid(), 'report.download',      '下載報表',     'report',       '可下載報表檔案',    NOW()),
    -- training (009)
    (gen_random_uuid(), 'training.view',       '查看訓練紀錄',         'training', '可查看人員訓練紀錄',                 NOW()),
    (gen_random_uuid(), 'training.manage',     '管理訓練紀錄',         'training', '可新增、編輯、刪除訓練紀錄',         NOW()),
    (gen_random_uuid(), 'training.manage_own', '管理自己的訓練紀錄',   'training', '可新增、編輯、刪除自己的訓練紀錄',   NOW()),
    -- equipment (009/018)
    (gen_random_uuid(), 'equipment.view',               '查看設備',       'equipment', '可查看設備與校準紀錄',           NOW()),
    (gen_random_uuid(), 'equipment.manage',             '管理設備',       'equipment', '可新增、編輯、刪除設備與校準紀錄', NOW()),
    (gen_random_uuid(), 'equipment.disposal.approve',   '核准設備報廢',   'equipment', '可核准設備報廢申請',             NOW()),
    (gen_random_uuid(), 'equipment.maintenance.manage', '管理維修保養',   'equipment', '可新增、編輯維修保養紀錄',       NOW()),
    (gen_random_uuid(), 'equipment.plan.manage',        '管理年度計畫',   'equipment', '可產生與編輯年度維護校正計畫表', NOW()),
    -- qau (009/029)
    (gen_random_uuid(), 'qau.dashboard.view',       '查看 QAU 儀表板',    'qau', 'GLP 品質保證：可查看研究狀態、審查進度、稽核摘要', NOW()),
    (gen_random_uuid(), 'qau.protocol.view',        'QAU 檢視計畫',       'qau', '唯讀檢視所有計畫書',                               NOW()),
    (gen_random_uuid(), 'qau.audit.view',           'QAU 檢視稽核',       'qau', '唯讀檢視稽核日誌',                                 NOW()),
    (gen_random_uuid(), 'qau.animal.view',          'QAU 檢視動物',       'qau', '唯讀檢視動物紀錄',                                 NOW()),
    (gen_random_uuid(), 'qau.inspection.view',      'QAU 檢視稽查報告',   'qau', '查看稽查報告列表與詳情',                           NOW()),
    (gen_random_uuid(), 'qau.inspection.manage',    'QAU 管理稽查報告',   'qau', '建立、編輯、關閉稽查報告',                         NOW()),
    (gen_random_uuid(), 'qau.nc.view',              'QAU 檢視不符合事項', 'qau', '查看 NC 與 CAPA 列表',                             NOW()),
    (gen_random_uuid(), 'qau.nc.manage',            'QAU 管理不符合事項', 'qau', '建立、指派、結案不符合事項',                       NOW()),
    (gen_random_uuid(), 'qau.sop.view',             'QAU 檢視 SOP',       'qau', '查看 SOP 文件列表',                               NOW()),
    (gen_random_uuid(), 'qau.sop.manage',           'QAU 管理 SOP',       'qau', '建立、版本控制 SOP 文件',                         NOW()),
    (gen_random_uuid(), 'qau.schedule.view',        'QAU 檢視稽查排程',   'qau', '查看年度稽查計畫',                                 NOW()),
    (gen_random_uuid(), 'qau.schedule.manage',      'QAU 管理稽查排程',   'qau', '建立、維護年度稽查計畫',                           NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================================
-- Seed: Role Permissions
-- ============================================================

-- admin: 全部
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p WHERE r.code = 'admin'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- EXPERIMENT_STAFF
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'EXPERIMENT_STAFF' AND p.code IN (
    'animal.animal.view_all','animal.animal.view_project','animal.animal.create','animal.animal.edit',
    'animal.animal.assign','animal.animal.import','animal.animal.delete','animal.record.view',
    'animal.record.create','animal.record.edit','animal.record.delete','animal.blood_test_template.manage',
    'animal.record.observation','animal.record.surgery','animal.record.weight','animal.record.vaccine',
    'animal.record.sacrifice','animal.export.medical','animal.export.observation','animal.export.surgery',
    'animal.export.experiment','animal.pathology.view','animal.pathology.upload','animal.info.assign',
    'animal.info.edit','animal.record.emergency','animal.record.copy','animal.vet.upload_attachment',
    'aup.protocol.view_own','aup.attachment.view','aup.attachment.download',
    'hr.attendance.view','hr.attendance.clock','hr.leave.view','hr.leave.create',
    'hr.overtime.view','hr.overtime.create','hr.balance.view','erp.stock.view','erp.product.view',
    'notification.view','training.view','training.manage_own'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- VET
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'VET' AND p.code IN (
    'aup.protocol.view_all','aup.protocol.review','aup.review.view','aup.review.comment',
    'amendment.read','amendment.review','animal.animal.view_all','animal.animal.view_project',
    'animal.record.view','animal.export.medical','animal.export.observation','animal.export.surgery',
    'animal.export.experiment','animal.vet.recommend','animal.vet.read','animal.emergency.stop',
    'animal.euthanasia.recommend','animal.euthanasia.approve','animal.pathology.view','animal.pathology.upload',
    'animal.info.assign','animal.info.edit','animal.record.emergency','animal.record.copy',
    'animal.vet.upload_attachment','notification.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- PI
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'PI' AND p.code IN (
    'aup.protocol.view_own','aup.protocol.create','aup.protocol.edit','aup.protocol.submit',
    'aup.attachment.view','aup.attachment.download','aup.version.view',
    'amendment.create','amendment.submit','amendment.read',
    'animal.animal.view_project','animal.record.view','notification.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- IACUC_STAFF
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'IACUC_STAFF' AND p.code IN (
    'aup.protocol.view_all','aup.protocol.change_status','aup.review.assign',
    'aup.attachment.view','aup.attachment.download','aup.version.view',
    'amendment.read','admin.user.view',
    'animal.animal.view_all','animal.record.view','notification.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- CLIENT
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'CLIENT' AND p.code IN (
    'aup.protocol.view_own','aup.attachment.view','animal.animal.view_project',
    'animal.record.view','notification.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- REVIEWER
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'REVIEWER' AND p.code IN (
    'aup.protocol.view_all','aup.protocol.view_own','aup.protocol.review',
    'aup.review.view','aup.review.comment','aup.attachment.view','aup.attachment.download',
    'aup.version.view','amendment.read','amendment.review',
    'animal.animal.view_all','animal.record.view','notification.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- IACUC_CHAIR
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'IACUC_CHAIR' AND p.code IN (
    'aup.protocol.view_all','aup.protocol.review','aup.protocol.approve','aup.protocol.change_status',
    'aup.review.view','aup.review.assign','aup.review.comment',
    'aup.attachment.view','aup.attachment.download','aup.version.view',
    'amendment.read','amendment.review','notification.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- WAREHOUSE_MANAGER
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'WAREHOUSE_MANAGER' AND p.code IN (
    'erp.warehouse.view','erp.warehouse.create','erp.warehouse.edit','erp.warehouse.delete',
    'erp.product.view','erp.product.create','erp.product.edit',
    'erp.partner.view','erp.partner.create','erp.partner.edit',
    'erp.document.view','erp.document.create','erp.document.edit','erp.document.submit','erp.document.approve',
    'erp.purchase.create','erp.purchase.approve','erp.grn.create','erp.pr.create',
    'erp.stock.in','erp.stock.out','erp.stock.view','erp.stock.adjust','erp.stock.transfer',
    'erp.stocktake.create','erp.report.view','erp.report.export','erp.report.download',
    'erp.storage.view','erp.storage.edit','erp.storage.inventory.view','erp.storage.inventory.edit',
    'notification.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- ADMIN_STAFF
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'ADMIN_STAFF' AND p.code IN (
    'admin.user.view','admin.user.view_all','admin.user.create','admin.user.edit','admin.role.view',
    'hr.attendance.view','hr.attendance.view_all','hr.attendance.clock','hr.attendance.correct',
    'hr.leave.view','hr.leave.view_all','hr.leave.create','hr.leave.approve','hr.leave.manage',
    'hr.overtime.view','hr.overtime.create','hr.overtime.approve',
    'hr.balance.view','hr.balance.view_all','hr.balance.manage','notification.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- QAU (009 + 029)
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r CROSS JOIN permissions p
WHERE r.code = 'QAU' AND p.code IN (
    'qau.dashboard.view','qau.protocol.view','qau.audit.view','qau.animal.view',
    'aup.protocol.view_all','aup.review.view','aup.attachment.view','aup.attachment.download',
    'aup.version.view','audit.logs.view','animal.animal.view_all','animal.record.view',
    'qau.inspection.view','qau.inspection.manage',
    'qau.nc.view','qau.nc.manage',
    'qau.sop.view','qau.sop.manage',
    'qau.schedule.view','qau.schedule.manage'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- EQUIPMENT_MAINTENANCE (009 + 018)
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'EQUIPMENT_MAINTENANCE' AND p.code IN (
    'equipment.view','equipment.manage',
    'training.view','training.manage_own',
    'equipment.maintenance.manage','equipment.plan.manage'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;
