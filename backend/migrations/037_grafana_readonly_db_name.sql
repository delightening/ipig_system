-- Fix: 032 硬編 `ipig_db` 會在非 ipig_db 資料庫（例如 CI、測試）初始化時失敗。
-- 因 sqlx 不允許修改已 apply 的 migration，故在此新增一筆等效補正。
-- 對已經授權 grafana_readonly 的 DB 來說，這一步是 idempotent no-op。

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'grafana_readonly') THEN
        EXECUTE 'GRANT CONNECT ON DATABASE ' || quote_ident(current_database()) || ' TO grafana_readonly';
    END IF;
END
$$;
