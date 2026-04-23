-- 035_audit_log_activity_v2.sql
-- R26-3：log_activity stored proc 擴充至 v2，支援 impersonated_by_user_id 與 app 層提供的 changed_fields
--
-- 動機（對應審查報告議題 3 疑慮 3）：
-- - v1 的 log_activity 只接 10 參數，impersonation 與 app 算好的 changed_fields
--   只能靠事後 UPDATE 寫入，但 **UPDATE 在 HMAC 計算之後**，這兩欄不進 HMAC
-- - 攻擊者若取得 DB write 權限可清空 impersonated_by_user_id 而 HMAC chain 仍完整
-- - 本 migration 讓兩欄成為 INSERT 參數的一部分，HMAC 計算納入它們
--
-- 相容性：新增的兩個參數都有 DEFAULT NULL，既有 10-arg 呼叫者不受影響

-- 先 DROP 再 CREATE（CREATE OR REPLACE 不支援變更參數簽名）
DROP FUNCTION IF EXISTS log_activity(UUID, VARCHAR, VARCHAR, VARCHAR, UUID, VARCHAR, JSONB, JSONB, INET, TEXT);

CREATE FUNCTION log_activity(
    p_actor_user_id    UUID,
    p_event_category   VARCHAR(50),
    p_event_type       VARCHAR(100),
    p_entity_type      VARCHAR(50),
    p_entity_id        UUID,
    p_entity_display_name VARCHAR(255),
    p_before_data      JSONB   DEFAULT NULL,
    p_after_data       JSONB   DEFAULT NULL,
    p_ip_address       INET    DEFAULT NULL,
    p_user_agent       TEXT    DEFAULT NULL,
    p_impersonated_by_user_id UUID   DEFAULT NULL,  -- R26-3 新增
    p_changed_fields   TEXT[]  DEFAULT NULL         -- R26-3 新增（app 層提供；NULL 則用 JSONB EXCEPT 算）
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

    -- changed_fields 來源優先級：
    --   1. app 層提供（已含 redact 欄位名，例如 password_hash）→ 直接用
    --   2. 未提供但 before/after 都有 → 用 JSONB EXCEPT 自動算（但看不到 redact 欄位）
    --   3. 其他情況 → NULL
    IF p_changed_fields IS NOT NULL THEN
        v_changed_fields := p_changed_fields;
    ELSIF p_before_data IS NOT NULL AND p_after_data IS NOT NULL THEN
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
        ip_address, user_agent,
        impersonated_by_user_id
    ) VALUES (
        p_actor_user_id, v_actor_email, v_actor_display_name, v_actor_roles,
        p_event_category, p_event_type,
        p_entity_type, p_entity_id, p_entity_display_name,
        p_before_data, p_after_data, v_changed_fields,
        p_ip_address, p_user_agent,
        p_impersonated_by_user_id
    ) RETURNING id INTO v_id;

    RETURN v_id;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION log_activity IS
    'v2: 擴充 impersonated_by_user_id 與 changed_fields 參數（R26-3）。兩者 DEFAULT NULL，既有 10-arg 呼叫者相容。';
