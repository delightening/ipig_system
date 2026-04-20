-- R24-4: Grafana read-only user for security dashboard SQL panels
-- 僅授予稽核相關表的 SELECT 權限，禁止 DML / DDL。
-- 密碼由 docker-compose secrets 傳入（見 docker-compose.prod.yml grafana_pg_password）。

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'grafana_readonly') THEN
        -- 由 app migration 建立時無法讀到 secret；預設隨機密碼，部署時再用 ALTER USER 設定
        CREATE ROLE grafana_readonly LOGIN PASSWORD 'CHANGE_ME_AT_DEPLOY' NOSUPERUSER NOINHERIT NOCREATEDB NOCREATEROLE;
    END IF;
END
$$;

-- schema 預設 public；限制 CONNECT 與 USAGE（用 current_database() 避免硬編 DB 名）
DO $$ BEGIN
    EXECUTE 'GRANT CONNECT ON DATABASE ' || current_database() || ' TO grafana_readonly';
END $$;
GRANT USAGE ON SCHEMA public TO grafana_readonly;

-- 稽核相關表 SELECT
GRANT SELECT ON TABLE security_alerts         TO grafana_readonly;
GRANT SELECT ON TABLE user_activity_logs      TO grafana_readonly;
GRANT SELECT ON TABLE login_events            TO grafana_readonly;
GRANT SELECT ON TABLE user_sessions           TO grafana_readonly;
GRANT SELECT ON TABLE ip_blocklist            TO grafana_readonly;

-- 未來新增稽核表時自動授權（預設 privileges）
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT SELECT ON TABLES TO grafana_readonly;
