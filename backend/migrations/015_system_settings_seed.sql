-- Migration 015: Seed system_settings with initial values
-- These defaults mirror the .env configuration; admin can override via UI.

INSERT INTO system_settings (key, value, description) VALUES
('company_name',          '"iPig System"',           '公司 / 系統名稱'),
('default_warehouse_id',  '""',                      '預設倉庫 UUID'),
('cost_method',           '"weighted_average"',      '成本計算方式 (weighted_average / moving_average)'),
('smtp_host',             '""',                      'SMTP 伺服器主機位址'),
('smtp_port',             '"587"',                   'SMTP 伺服器連接埠'),
('smtp_username',         '""',                      'SMTP 帳號'),
('smtp_password',         '""',                      'SMTP 密碼'),
('smtp_from_email',       '"noreply@erp.local"',     '寄件人 Email'),
('smtp_from_name',        '"iPig System"',           '寄件人顯示名稱'),
('session_timeout_minutes', '"360"',                 'Session 逾時（分鐘）')
ON CONFLICT (key) DO NOTHING;
