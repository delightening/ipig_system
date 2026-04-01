-- ============================================================
-- Migration 012: 函式、觸發器、Extension、效能設定
-- 來源: 007_audit_erp.sql (log_activity, check_brute_force),
--       008_supplementary.sql (maintenance_vacuum_analyze, slow_queries view, pg_stat_statements),
--       013_audit_integrity_trigger.sql (immutable audit log trigger),
--       028_expiry_notification_config.sql (fn_expiry_alerts)
-- 前置依賴: 所有資料表必須已建立（001-011）
-- ============================================================

-- ── Extension ────────────────────────────────────────────────
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- ── log_activity() ───────────────────────────────────────────
-- 寫入稽核日誌的便利函式
CREATE OR REPLACE FUNCTION log_activity(
    p_actor_user_id    UUID,
    p_event_category   VARCHAR(50),
    p_event_type       VARCHAR(100),
    p_entity_type      VARCHAR(50),
    p_entity_id        UUID,
    p_entity_display_name VARCHAR(255),
    p_before_data      JSONB   DEFAULT NULL,
    p_after_data       JSONB   DEFAULT NULL,
    p_ip_address       INET    DEFAULT NULL,
    p_user_agent       TEXT    DEFAULT NULL
) RETURNS UUID AS $$
DECLARE
    v_id                 UUID;
    v_actor_email        VARCHAR(255);
    v_actor_display_name VARCHAR(100);
    v_actor_roles        JSONB;
    v_changed_fields     TEXT[];
BEGIN
    SELECT email, display_name
    INTO   v_actor_email, v_actor_display_name
    FROM   users WHERE id = p_actor_user_id;

    SELECT jsonb_agg(r.code)
    INTO   v_actor_roles
    FROM   user_roles ur
    JOIN   roles r ON ur.role_id = r.id
    WHERE  ur.user_id = p_actor_user_id;

    IF p_before_data IS NOT NULL AND p_after_data IS NOT NULL THEN
        SELECT array_agg(key)
        INTO   v_changed_fields
        FROM (
            SELECT key FROM jsonb_each(p_after_data)
            EXCEPT
            SELECT key FROM jsonb_each(p_before_data)
            WHERE p_before_data->key = p_after_data->key
        ) changed_keys;
    END IF;

    INSERT INTO user_activity_logs (
        actor_user_id, actor_email, actor_display_name, actor_roles,
        event_category, event_type,
        entity_type, entity_id, entity_display_name,
        before_data, after_data, changed_fields,
        ip_address, user_agent
    ) VALUES (
        p_actor_user_id, v_actor_email, v_actor_display_name, v_actor_roles,
        p_event_category, p_event_type,
        p_entity_type, p_entity_id, p_entity_display_name,
        p_before_data, p_after_data, v_changed_fields,
        p_ip_address, p_user_agent
    ) RETURNING id INTO v_id;

    RETURN v_id;
END;
$$ LANGUAGE plpgsql;

-- ── check_brute_force() ──────────────────────────────────────
-- 15 分鐘內失敗 5 次以上回傳 TRUE
CREATE OR REPLACE FUNCTION check_brute_force(p_email VARCHAR(255)) RETURNS BOOLEAN AS $$
DECLARE
    v_failed_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO v_failed_count
    FROM   login_events
    WHERE  email      = p_email
      AND  event_type = 'login_failure'
      AND  created_at > NOW() - INTERVAL '15 minutes';
    RETURN v_failed_count >= 5;
END;
$$ LANGUAGE plpgsql;

-- ── check_user_activity_logs_immutable() + trigger ───────────
-- 僅允許更新 integrity_hash / previous_hash，其他欄位不可竄改（GLP 合規）
CREATE OR REPLACE FUNCTION check_user_activity_logs_immutable()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.id IS DISTINCT FROM NEW.id
       OR OLD.actor_user_id IS DISTINCT FROM NEW.actor_user_id
       OR OLD.actor_email IS DISTINCT FROM NEW.actor_email
       OR OLD.actor_display_name IS DISTINCT FROM NEW.actor_display_name
       OR (OLD.actor_roles IS DISTINCT FROM NEW.actor_roles)
       OR OLD.session_id IS DISTINCT FROM NEW.session_id
       OR OLD.session_started_at IS DISTINCT FROM NEW.session_started_at
       OR OLD.event_category IS DISTINCT FROM NEW.event_category
       OR OLD.event_type IS DISTINCT FROM NEW.event_type
       OR OLD.event_severity IS DISTINCT FROM NEW.event_severity
       OR OLD.entity_type IS DISTINCT FROM NEW.entity_type
       OR OLD.entity_id IS DISTINCT FROM NEW.entity_id
       OR OLD.entity_display_name IS DISTINCT FROM NEW.entity_display_name
       OR (OLD.before_data IS DISTINCT FROM NEW.before_data)
       OR (OLD.after_data IS DISTINCT FROM NEW.after_data)
       OR (OLD.changed_fields IS DISTINCT FROM NEW.changed_fields)
       OR OLD.ip_address IS DISTINCT FROM NEW.ip_address
       OR OLD.user_agent IS DISTINCT FROM NEW.user_agent
       OR OLD.request_path IS DISTINCT FROM NEW.request_path
       OR OLD.request_method IS DISTINCT FROM NEW.request_method
       OR OLD.response_status IS DISTINCT FROM NEW.response_status
       OR OLD.geo_country IS DISTINCT FROM NEW.geo_country
       OR OLD.geo_city IS DISTINCT FROM NEW.geo_city
       OR OLD.is_suspicious IS DISTINCT FROM NEW.is_suspicious
       OR OLD.suspicious_reason IS DISTINCT FROM NEW.suspicious_reason
       OR OLD.created_at IS DISTINCT FROM NEW.created_at
       OR OLD.partition_date IS DISTINCT FROM NEW.partition_date
    THEN
        RAISE EXCEPTION 'user_activity_logs: direct modification of log payload is not allowed (integrity enforcement)';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_user_activity_logs_immutable ON user_activity_logs;
CREATE TRIGGER trg_user_activity_logs_immutable
    BEFORE UPDATE ON user_activity_logs
    FOR EACH ROW
    EXECUTE FUNCTION check_user_activity_logs_immutable();

-- ── maintenance_vacuum_analyze() ─────────────────────────────
-- 定期 ANALYZE 高頻寫入表（由 cron job 呼叫）
CREATE OR REPLACE FUNCTION maintenance_vacuum_analyze() RETURNS void AS $$
BEGIN
    ANALYZE animals;
    ANALYZE animal_observations;
    ANALYZE animal_surgeries;
    ANALYZE animal_weights;
    ANALYZE animal_vaccinations;
    ANALYZE vet_recommendations;
    ANALYZE notifications;
    ANALYZE user_activity_logs;
    ANALYZE attachments;
    ANALYZE protocols;
    ANALYZE audit_logs;
    RAISE NOTICE 'maintenance_vacuum_analyze completed at %', NOW();
END;
$$ LANGUAGE plpgsql;

-- ── fn_expiry_alerts() ───────────────────────────────────────
-- 可傳入動態參數的效期查詢函數（排程器使用；v_expiry_alerts view 供 UI 直接查詢）
CREATE OR REPLACE FUNCTION fn_expiry_alerts(
    p_warn_days   INT DEFAULT 60,
    p_cutoff_days INT DEFAULT 90
)
RETURNS TABLE (
    product_id        UUID,
    sku               VARCHAR,
    product_name      VARCHAR,
    spec              TEXT,
    category_code     VARCHAR,
    warehouse_id      UUID,
    warehouse_code    VARCHAR,
    warehouse_name    VARCHAR,
    batch_no          VARCHAR,
    expiry_date       DATE,
    on_hand_qty       NUMERIC,
    base_uom          VARCHAR,
    days_until_expiry INT,
    expiry_status     VARCHAR,
    total_qty         NUMERIC
)
LANGUAGE SQL STABLE AS $$
    SELECT
        p.id                                                          AS product_id,
        p.sku,
        p.name                                                        AS product_name,
        p.spec,
        p.category_code,
        sl.warehouse_id,
        w.code                                                        AS warehouse_code,
        w.name                                                        AS warehouse_name,
        sl.batch_no,
        sl.expiry_date,
        SUM(CASE
            WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
            ELSE -sl.qty_base
        END)                                                          AS on_hand_qty,
        p.base_uom,
        (sl.expiry_date - CURRENT_DATE)::INT                         AS days_until_expiry,
        CASE WHEN sl.expiry_date < CURRENT_DATE
             THEN 'expired'
             ELSE 'expiring_soon'
        END                                                           AS expiry_status,
        COALESCE(inv.on_hand_qty_base, 0)                            AS total_qty
    FROM stock_ledger sl
    JOIN products p    ON sl.product_id   = p.id
    JOIN warehouses w  ON sl.warehouse_id = w.id
    LEFT JOIN inventory_snapshots inv
           ON inv.product_id = p.id AND inv.warehouse_id = sl.warehouse_id
    WHERE p.track_expiry = true
      AND sl.expiry_date IS NOT NULL
      AND p.is_active    = true
      AND sl.expiry_date >= CURRENT_DATE - p_cutoff_days
    GROUP BY p.id, p.sku, p.name, p.spec, p.category_code,
             sl.warehouse_id, w.code, w.name, sl.batch_no, sl.expiry_date,
             p.base_uom, inv.on_hand_qty_base
    HAVING
        SUM(CASE
            WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
            ELSE -sl.qty_base
        END) > 0
        AND sl.expiry_date <= CURRENT_DATE + p_warn_days
$$;

COMMENT ON FUNCTION fn_expiry_alerts IS '效期預警查詢函數，支援動態傳入提前預警天數與截止天數';

-- ── slow_queries view ────────────────────────────────────────
CREATE OR REPLACE VIEW slow_queries AS
SELECT
    queryid,
    LEFT(query, 200) AS query_preview,
    calls,
    mean_exec_time   AS avg_ms,
    total_exec_time  AS total_ms,
    rows
FROM pg_stat_statements
WHERE mean_exec_time > 100
ORDER BY mean_exec_time DESC
LIMIT 50;
