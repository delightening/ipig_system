-- 036_audit_log_activity_v3.sql
-- R26-5：修正 log_activity 的 changed_fields fallback 漏偵測「被刪除的 key」
--
-- 背景（對應 PR #153 review: Gemini + CodeRabbit）：
-- - migration 035 的 stored proc 當 p_changed_fields 為 NULL 時，用 JSONB EXCEPT
--   算 changed_fields — 但只比對 after_data 的 key，遺漏了 before_data 有而
--   after_data 沒有的 key（被刪除欄位）
-- - 例：改 user 把 bio 欄位刪掉，原 stored proc 的 v_changed_fields 不會含 "bio"
--
-- 本 migration 取聯集：從兩個 object 的所有 key 中找出值不同（或一邊不存在）的。

DROP FUNCTION IF EXISTS log_activity(
    UUID, VARCHAR, VARCHAR, VARCHAR, UUID, VARCHAR,
    JSONB, JSONB, INET, TEXT, UUID, TEXT[]
);

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
    p_impersonated_by_user_id UUID   DEFAULT NULL,
    p_changed_fields   TEXT[]  DEFAULT NULL
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
    --   2. 未提供但 before/after 都有 → 取「兩邊 key 的聯集，值不同者」
    --   3. 其他情況 → NULL
    --
    -- R26-5 修正：舊版只掃 after 的 key（漏掉 before 有但 after 沒有的刪除欄位）
    IF p_changed_fields IS NOT NULL THEN
        v_changed_fields := p_changed_fields;
    ELSIF p_before_data IS NOT NULL AND p_after_data IS NOT NULL THEN
        -- jsonb_object_keys 只能用於 JSON object；scalar/array 會 runtime error。
        -- 實務上所有 entity 都序列化為 object，但加 typeof guard 防呆 —
        -- 非 object 型別跳過 fallback 計算（v_changed_fields 保持 NULL）。
        IF jsonb_typeof(p_before_data) = 'object' AND jsonb_typeof(p_after_data) = 'object' THEN
            -- 聯集 before/after 的所有 key，過濾掉值相同的 → 得到真正變動的 key 集合
            SELECT array_agg(DISTINCT key ORDER BY key)
            INTO   v_changed_fields
            FROM (
                SELECT jsonb_object_keys(p_before_data) AS key
                UNION
                SELECT jsonb_object_keys(p_after_data) AS key
            ) all_keys
            WHERE  (p_before_data->key) IS DISTINCT FROM (p_after_data->key);
        END IF;
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
    'v3: 修正 changed_fields fallback 漏掉被刪除的欄位 (R26-5)。
     app 層仍建議傳 p_changed_fields 以支援巢狀 redact 欄位名。';
