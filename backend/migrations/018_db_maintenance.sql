-- 018: 資料庫維護自動化
-- pg_stat_statements 擴展 + VACUUM ANALYZE 排程函數

-- 啟用 pg_stat_statements（需要在 postgresql.conf 預載，Docker 映像一般已含）
-- 若擴展不存在，CREATE EXTENSION 會自動跳過
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- 定期重置統計快照的函數（可透過 cron 或應用排程呼叫）
CREATE OR REPLACE FUNCTION maintenance_vacuum_analyze()
RETURNS void AS $$
BEGIN
    -- 高頻寫入表
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

-- 查詢慢查詢的輔助 View
CREATE OR REPLACE VIEW slow_queries AS
SELECT
    queryid,
    LEFT(query, 200) AS query_preview,
    calls,
    mean_exec_time AS avg_ms,
    total_exec_time AS total_ms,
    rows
FROM pg_stat_statements
WHERE mean_exec_time > 100
ORDER BY mean_exec_time DESC
LIMIT 50;
