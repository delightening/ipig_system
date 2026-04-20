-- R24-1: IP 黑名單（自動封鎖攻擊者來源 IP）
-- 延伸 R22-6（user 停用）→ 同步封 IP；R22-1（auth ratelimit）→ 封 IP 1h；R22-16（honeypot）→ 永久封。
-- 由 middleware/ip_blocklist.rs 於最外層短路。

CREATE TABLE IF NOT EXISTS ip_blocklist (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address       INET NOT NULL,
    reason           TEXT NOT NULL,
    source           TEXT NOT NULL,           -- 'R22-6_idor' / 'R22-1_ratelimit' / 'honeypot' / 'manual'
    alert_id         UUID,                    -- 關聯 security_alerts.id（auto source 時填）
    blocked_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    blocked_until    TIMESTAMPTZ,             -- NULL = permanent
    blocked_by       UUID REFERENCES users(id), -- NULL = auto
    hit_count        BIGINT NOT NULL DEFAULT 0,
    last_hit_at      TIMESTAMPTZ,
    unblocked_at     TIMESTAMPTZ,
    unblocked_by     UUID REFERENCES users(id),
    unblocked_reason TEXT,
    metadata         JSONB,
    CONSTRAINT chk_ip_blocklist_source
        CHECK (source IN ('R22-6_idor', 'R22-1_ratelimit', 'honeypot', 'manual'))
);

-- 僅允許一筆 active entry per IP（middleware 最熱查詢路徑）
CREATE UNIQUE INDEX IF NOT EXISTS idx_ip_blocklist_active
    ON ip_blocklist (ip_address)
    WHERE unblocked_at IS NULL;

-- 過期清理查詢（若未來加 cron，或 Grafana 面板）
CREATE INDEX IF NOT EXISTS idx_ip_blocklist_expiring
    ON ip_blocklist (blocked_until)
    WHERE blocked_until IS NOT NULL AND unblocked_at IS NULL;

-- 歷史查詢：最近 blocked_at 排序
CREATE INDEX IF NOT EXISTS idx_ip_blocklist_recent
    ON ip_blocklist (blocked_at DESC);
