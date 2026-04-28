-- R30-17: data_retention_policies + 排程驅動的資料保留執行
--
-- Why:
--   GLP §11.10(c) + OECD §10：每筆業務紀錄都需有明確保留年限與最終處置策
--   略；本系統一律走 soft-delete（services/animal/* `*_with_reason`），
--   實際 hard delete 由本表驅動的 cron job (services/retention_enforcer.rs)
--   每日 03:00 UTC 執行：找出 deleted_at 已超過 retention_years 的 row 才
--   真實刪除。
--
-- 設計重點：
--   1. retention_years = NULL → 永久保留（never strategy）
--   2. delete_strategy:
--        'hard_delete'    — 一般表，DELETE WHERE deleted_at < cutoff
--        'partition_drop' — 分區表（user_activity_logs），不能 row DELETE
--                           因 R30-F migration 041 加了 BEFORE DELETE trigger；
--                           走 ALTER TABLE ... DETACH PARTITION + DROP TABLE
--                           不觸發 ROW trigger，符合 R30-F 設計
--        'never'          — 永久保留，policy 行僅作為文件
--   3. 缺 deleted_at 欄位的表：retention_enforcer 會 information_schema 偵測
--      若無 deleted_at 則 skip（log 'no deleted_at column'），不額外建欄。
--      避免大規模 schema migration（60+ 表）的副作用。
--
-- ⚠️ STAGING-ONLY：先空跑（dry run）觀察 enforcement 報表 ≥7 天，確認
--   沒誤刪後再啟用實際 delete（後續 PR 加 enable flag）。本 PR 只建表 +
--   service + scheduler 註冊。

CREATE TABLE data_retention_policies (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name      TEXT         NOT NULL UNIQUE,
    retention_years INTEGER,     -- NULL = 永久保留
    delete_strategy TEXT         NOT NULL CHECK (delete_strategy IN ('hard_delete', 'partition_drop', 'never')),
    description     TEXT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE data_retention_policies IS
    'GLP §11.10(c) + OECD §10 record retention policy。為每個業務表定義保留年限與刪除策略；'
    '排程任務 services/retention_enforcer.rs 每日 03:00 UTC 跑，找出 deleted_at + retention_years '
    '已過的 row 實際刪除。retention_years NULL = 永久；delete_strategy = never 表示僅文件用途。';
COMMENT ON COLUMN data_retention_policies.table_name      IS '對應業務表名稱（與 information_schema.tables 對應）';
COMMENT ON COLUMN data_retention_policies.retention_years IS 'NULL = 永久不刪；數值 = 滿年限後 hard delete';
COMMENT ON COLUMN data_retention_policies.delete_strategy IS 'hard_delete / partition_drop / never';

-- ===== Seed：依 R30-E 用戶決議「全部 20 年」+ 永久 5 表 + audit partition =====
-- 僅 seed 已存在於 schema 的表。其他在 R30 規格列出但本 codebase 尚未建立
-- 的（pain_assessments / equipment_validations / hr_attendance / 等）暫不
-- seed；待對應業務表建立後另開 PR 補。

-- ── 永久（never delete） ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('electronic_signatures',            NULL, 'never', '§11.70 簽章不可刪'),
    ('protocols',                        NULL, 'never', '試驗計畫源頭'),
    ('amendments',                       NULL, 'never', '計畫變更歷史'),
    ('study_final_reports',              NULL, 'never', 'OECD §10 final report'),
    ('animals',                          NULL, 'never', '動物個體生命週期完整可追溯');

-- ── 動物業務 20 年 ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('animal_observations',              20, 'hard_delete', 'OECD §8.3 raw data'),
    ('animal_surgeries',                 20, 'hard_delete', 'OECD §8.2 procedures'),
    ('animal_blood_tests',               20, 'hard_delete', 'OECD §8.3 specimen analysis'),
    ('animal_weights',                   20, 'hard_delete', 'OECD §8.3 measurement'),
    ('animal_vaccinations',              20, 'hard_delete', 'OECD §8.2 procedures'),
    ('animal_sacrifices',                20, 'hard_delete', 'OECD §8.4 termination'),
    ('care_medication_records',          20, 'hard_delete', 'OECD §8.2 用藥紀錄'),
    ('vet_patrol_reports',               20, 'hard_delete', 'OECD §8.3 health monitoring'),
    ('vet_patrol_entries',               20, 'hard_delete', 'OECD §8.3 health monitoring'),
    ('animal_vet_advices',               20, 'hard_delete', '醫療判斷'),
    ('animal_vet_advice_records',        20, 'hard_delete', '醫療判斷紀錄'),
    ('euthanasia_orders',                20, 'hard_delete', '不可逆動物處置'),
    ('euthanasia_appeals',               20, 'hard_delete', '不可逆動物處置');

-- ── 人員 §1.4 ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('training_records',                 20, 'hard_delete', '§11.10(i) personnel training'),
    ('competency_assessments',           20, 'hard_delete', '§1.4 competency'),
    ('role_training_requirements',       20, 'hard_delete', '§1.4 training matrix');

-- ── QA §1.3 ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('qa_inspections',                   20, 'hard_delete', '§1.3 QAU inspections'),
    ('qa_non_conformances',              20, 'hard_delete', '§1.3 QAU findings'),
    ('qa_audit_schedules',               20, 'hard_delete', '§1.3 QAU plans'),
    ('qa_sop_documents',                 20, 'hard_delete', '§1.3 QAU SOPs');

-- ── 設備 §4 ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('equipment',                        20, 'hard_delete', '§4 apparatus'),
    ('equipment_calibrations',           20, 'hard_delete', '§4.2 calibration'),
    ('equipment_maintenance_records',    20, 'hard_delete', '§4.3 maintenance'),
    ('equipment_idle_requests',          20, 'hard_delete', '§4 maintenance');

-- ── 試劑 §6 ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('reference_standards',              20, 'hard_delete', '§6.3 reference items'),
    ('formulation_records',              20, 'hard_delete', '§6.1 formulation');

-- ── 供應商 §6.2 ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('partners',                         20, 'hard_delete', '§6.2 supplier qualification');

-- ── SOP / 文件控制 §7 ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('controlled_documents',             20, 'hard_delete', '§7 SOPs'),
    ('document_revisions',               20, 'hard_delete', '§7.2 SOP version'),
    ('document_acknowledgments',         20, 'hard_delete', '§7 SOP acknowledgment'),
    ('change_requests',                  20, 'hard_delete', '§7.3 change control');

-- ── 設施 §3 ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('buildings',                        20, 'hard_delete', '§3.1 housing'),
    ('zones',                            20, 'hard_delete', '§3.1 housing'),
    ('pens',                             20, 'hard_delete', '§3.1 housing'),
    ('animal_sources',                   20, 'hard_delete', '§5.2 animal source');

-- ── 環境 §3.1 ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('environment_monitoring_points',    20, 'hard_delete', '§3.1 environment'),
    ('environment_readings',             20, 'hard_delete', '§3.1 environment data');

-- ── 管理 §1.5 ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('management_reviews',               20, 'hard_delete', '§1.5 management oversight'),
    ('risk_register',                    20, 'hard_delete', '§1.5 risk management');

-- ── Audit / Security §11.10(d)(e) ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('user_activity_logs',               20, 'partition_drop', '§11.10(e) audit trail；走 partition drop 不走 row delete (R30-F trigger)'),
    ('user_activity_aggregates',         20, 'hard_delete', 'audit aggregate'),
    ('security_alerts',                  20, 'hard_delete', '§11.10(d) security incidents'),
    ('security_alert_config',            20, 'hard_delete', '§11.10(d) security'),
    ('security_notification_channels',   20, 'hard_delete', '§11.10(d) security'),
    ('login_events',                     20, 'hard_delete', '§11.10(d) authentication audit'),
    ('ip_blocklist',                     20, 'hard_delete', '§11.10(d) attack records'),
    ('user_sessions',                    20, 'hard_delete', '§11.10(d) session audit');

-- ── 邀請 / API key ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('invitations',                      20, 'hard_delete', '§1.4 onboarding'),
    ('user_mcp_keys',                    20, 'hard_delete', 'API key audit trail');

-- ── 庫存 / 試劑流動 §3.2/§6.2 ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('stock_ledger',                     20, 'hard_delete', '§3.2 / §6.2 試劑流向'),
    ('storage_locations',                20, 'hard_delete', '§3.2 storage');

-- ── 試驗相關文件 ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('documents',                        20, 'hard_delete', 'GLP-related documents'),
    ('attachments',                      20, 'hard_delete', 'GLP-related attachments');

-- ── HR §1.4（schema 中名稱：attendance_records / leave_requests / overtime_records） ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('attendance_records',               20, 'hard_delete', '§1.4 personnel work record'),
    ('leave_requests',                   20, 'hard_delete', '§1.4 personnel leave'),
    ('overtime_records',                 20, 'hard_delete', '§1.4 personnel overtime');

-- ── 商業營運 ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('journal_entries',                  20, 'hard_delete', '商業帳務'),
    ('journal_entry_lines',              20, 'hard_delete', '商業帳務'),
    ('ap_payments',                      20, 'hard_delete', '商業帳務'),
    ('ar_receipts',                      20, 'hard_delete', '商業帳務');

-- ── 運維 ──
INSERT INTO data_retention_policies (table_name, retention_years, delete_strategy, description) VALUES
    ('notifications',                    20, 'hard_delete', '通知紀錄'),
    ('ai_query_logs',                    20, 'hard_delete', 'AI 查詢稽核');

-- token 表（jwt_blacklist / refresh_tokens / password_reset_tokens）不在
-- policy 表內：自動清由各自 TTL 機制處理。
