-- AI API Keys 表：儲存 AI 系統存取 iPig 的 API 金鑰
CREATE TABLE IF NOT EXISTS ai_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    -- SHA-256 hash of the key (不儲存明文)
    key_hash VARCHAR(64) NOT NULL UNIQUE,
    -- key 前綴（用於辨識，如 "ipig_ai_abc1..."）
    key_prefix VARCHAR(12) NOT NULL,
    -- 建立此 key 的管理員
    created_by UUID NOT NULL REFERENCES users(id),
    -- 允許的權限範圍（JSON 陣列，如 ["animal.read", "protocol.read"]）
    scopes JSONB NOT NULL DEFAULT '["read"]'::jsonb,
    -- 是否啟用
    is_active BOOLEAN NOT NULL DEFAULT true,
    -- 過期時間（NULL 表示不過期）
    expires_at TIMESTAMPTZ,
    -- 最後使用時間
    last_used_at TIMESTAMPTZ,
    -- 使用次數
    usage_count BIGINT NOT NULL DEFAULT 0,
    -- 每分鐘請求上限
    rate_limit_per_minute INT NOT NULL DEFAULT 60,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_ai_api_keys_key_hash ON ai_api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_ai_api_keys_active ON ai_api_keys(is_active) WHERE is_active = true;

-- AI 查詢日誌（記錄 AI 的每次查詢）
CREATE TABLE IF NOT EXISTS ai_query_logs (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL,
    -- 查詢的端點
    endpoint VARCHAR(200) NOT NULL,
    -- 查詢方法
    method VARCHAR(10) NOT NULL DEFAULT 'GET',
    -- 查詢參數摘要（不含敏感資料）
    query_summary JSONB,
    -- 回應狀態碼
    response_status SMALLINT NOT NULL,
    -- 處理時間（毫秒）
    duration_ms INT NOT NULL DEFAULT 0,
    -- 來源 IP
    source_ip VARCHAR(45),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- 建立初始分區（當月 + 下個月）
DO $$
DECLARE
    curr_start DATE := date_trunc('month', CURRENT_DATE);
    next_start DATE := date_trunc('month', CURRENT_DATE + INTERVAL '1 month');
    after_next DATE := date_trunc('month', CURRENT_DATE + INTERVAL '2 months');
    curr_suffix TEXT := to_char(CURRENT_DATE, 'YYYY_MM');
    next_suffix TEXT := to_char(CURRENT_DATE + INTERVAL '1 month', 'YYYY_MM');
BEGIN
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS ai_query_logs_%s PARTITION OF ai_query_logs FOR VALUES FROM (%L) TO (%L)',
        curr_suffix, curr_start, next_start
    );
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS ai_query_logs_%s PARTITION OF ai_query_logs FOR VALUES FROM (%L) TO (%L)',
        next_suffix, next_start, after_next
    );
END $$;

CREATE INDEX IF NOT EXISTS idx_ai_query_logs_api_key ON ai_query_logs(api_key_id, created_at);
