-- Squash 起點 §1：cluster 層級物件（ROLE）。
--
-- 為什麼需要獨立一支：ROLE 是 **cluster 層級**，不屬於任何單一資料庫，因此
-- `pg_dump`（對單一 DB 做的）不會輸出 `CREATE ROLE`——但它**會**輸出引用該 role 的
-- GRANT（002_schema.sql 裡有 19 處 `TO grafana_readonly`）。
-- 所以順序必須是 001 先建 role、002 才能 GRANT 給它，否則整份 002 會在第一個
-- GRANT 就失敗（role 不存在）。
--
-- 內容合併自舊 migration 032/074/075，取其**最終狀態**而非重放中間步驟：
--   032：建立 grafana_readonly + CONNECT/USAGE 授權
--   074：把硬編的 placeholder 密碼換成隨機值（一次性操作，實際密碼一律於部署時
--        由 Docker Secret 以 ALTER ROLE 設定，故此處只保留 placeholder）
--   075：撤銷 032 當時加的「未來新表自動 SELECT」default privilege
--        → 因此本檔**不寫** `ALTER DEFAULT PRIVILEGES ... GRANT`。寫了等於重現一個
--          後來被明確撤銷的授權；逐表授權已完整包含在 002_schema.sql 的 dump 內容裡。
--
-- extension（pg_stat_statements / pg_trgm）不在此檔：它們**屬於資料庫層級**，
-- pg_dump 有輸出，已含在 002_schema.sql（且皆帶 IF NOT EXISTS）。
--
-- 冪等：可安全重跑。

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'grafana_readonly') THEN
        CREATE ROLE grafana_readonly LOGIN PASSWORD 'CHANGE_ME_AT_DEPLOY' NOSUPERUSER NOINHERIT NOCREATEDB NOCREATEROLE;
    END IF;
END
$$;

-- 用 current_database() 動態取名，避免硬編 DB 名在不同環境失敗（沿用 032 的做法）。
DO $$
BEGIN
    EXECUTE 'GRANT CONNECT ON DATABASE ' || quote_ident(current_database()) || ' TO grafana_readonly';
END
$$;

GRANT USAGE ON SCHEMA public TO grafana_readonly;
