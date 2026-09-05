-- Migration 013：稽核鏈三張表補上 STATEMENT 層 TRUNCATE 擋板
--
-- 背景：journal_entries / journal_entry_lines / stock_ledger 已有 no_truncate 擋板
-- （002_schema.sql，R97-2），理由寫在該組函式註解上：TRUNCATE **不觸發** row-level
-- trigger，沒有 STATEMENT 層擋板的話，DELETE 防線可被一句 TRUNCATE 繞過。
--
-- 本 migration 把同一道防線補到另外三張同屬 append-only 真相源、但只有
-- immutable / no_delete 的表：
--   * user_activity_logs      稽核紀錄（GLP §11.10(e)(1)）
--   * electronic_signatures   電子簽章（GLP §11.70）
--   * animal_blood_test_items 血檢結果（GLP §11.70）
--
-- 這個缺口是可被利用的、不是理論風險：app 的 DB 連線帳號與這三張表的 owner 相同，
-- 而 owner 身分擁有不受 GRANT / REVOKE 限制的 TRUNCATE 權限，毋須另外取得特權帳號。

-- ============================================================================
-- 1. user_activity_logs（分割表）
-- ============================================================================
--
-- ⚠️ 這張表是 PARTITION BY RANGE (partition_date)，擋板必須同時建在 parent 與
-- 每一個分割區上，兩者缺一不可：
--   * row-level trigger（既有的 no_delete）建在 parent 上時，PostgreSQL 會自動
--     遞迴套用到所有分割區，所以 DELETE 那條防線本來就是完整的。
--   * **statement-level TRUNCATE trigger 沒有這個遞迴行為**。只建在 parent 上時，
--     `TRUNCATE user_activity_logs` 會被擋，但 `TRUNCATE user_activity_logs_2026_q3`
--     直接對分割區下手則完全不觸發 parent 的擋板——查一次 pg_inherits 就能列出所有
--     分割區逐一清空，稽核紀錄全滅而擋板一次都不會響。
--
-- 新分割區由 PartitionMaintenanceJob::create_quarterly_partition 建立，該處已一併
-- 補上建 trigger 的邏輯，否則每季新長出來的分割區都會是裸的。

CREATE FUNCTION public.check_user_activity_logs_no_truncate() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'user_activity_logs is append-only (GLP §11.10(e)(1))。不可 TRUNCATE。
若需排除舊資料，使用 partition drop（DETACH PARTITION 不觸發本擋板）'
        USING ERRCODE = 'P0001';
END;
$$;

COMMENT ON FUNCTION public.check_user_activity_logs_no_truncate() IS 'GLP §11.10(e)(1)：TRUNCATE 不觸發 row-level trigger，需獨立的 STATEMENT 層擋板，否則 DELETE 防線可被一句 TRUNCATE 繞過。statement trigger 不會自動套用到分割區，故 parent 與每個分割區都要建。';

CREATE TRIGGER check_user_activity_logs_no_truncate_trigger
    BEFORE TRUNCATE ON public.user_activity_logs
    FOR EACH STATEMENT
    EXECUTE FUNCTION public.check_user_activity_logs_no_truncate();

-- 現有分割區逐一補上。用 pg_inherits 動態列舉而非寫死 12 個名字，理由是各環境的
-- 分割區集合未必與 002_schema.sql 相同——scheduler 可能已經多建了後續季度。
DO $do$
DECLARE
    part regclass;
BEGIN
    FOR part IN
        SELECT inhrelid::regclass
        FROM pg_inherits
        WHERE inhparent = 'public.user_activity_logs'::regclass
        ORDER BY 1
    LOOP
        EXECUTE format(
            'CREATE TRIGGER check_user_activity_logs_no_truncate_trigger '
            'BEFORE TRUNCATE ON %s '
            'FOR EACH STATEMENT '
            'EXECUTE FUNCTION public.check_user_activity_logs_no_truncate()',
            part
        );
    END LOOP;
END;
$do$;

-- ============================================================================
-- 2. electronic_signatures
-- ============================================================================

CREATE FUNCTION public.check_electronic_signatures_no_truncate() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'electronic_signatures is append-only (GLP §11.70)。不可 TRUNCATE。
若需作廢簽章，使用 SignatureService::invalidate（軟失效，保留紀錄）。'
        USING ERRCODE = 'P0001';
END;
$$;

COMMENT ON FUNCTION public.check_electronic_signatures_no_truncate() IS 'GLP §11.70：TRUNCATE 不觸發 row-level trigger，需獨立的 STATEMENT 層擋板；亦擋下他表 TRUNCATE ... CASCADE 的連帶清空。';

CREATE TRIGGER check_electronic_signatures_no_truncate_trigger
    BEFORE TRUNCATE ON public.electronic_signatures
    FOR EACH STATEMENT
    EXECUTE FUNCTION public.check_electronic_signatures_no_truncate();

-- ============================================================================
-- 3. animal_blood_test_items
-- ============================================================================
--
-- ⚠️ 這張表與另外兩張不同，擋板必須帶一個 escape hatch。
--
-- 成因：IDXF 全庫匯入前，DataImportService 會對 partial-unique 表下
-- `TRUNCATE TABLE "pens","zones","buildings","facilities" RESTART IDENTITY CASCADE`
-- （services/data_import.rs::cleanup_partial_unique_tables）。CASCADE 會**遞移**展開到
-- 所有直接與間接參照這四張表的表，而 FK 鏈
--     pens ← animals.pen_id ← animal_blood_tests.animal_id ← animal_blood_test_items.blood_test_id
-- 讓本表正好落在展開結果裡。裸的擋板會讓整句 TRUNCATE 失敗（TRUNCATE 是單一 statement，
-- 任一目標被擋就全句 rollback），IDXF 全庫匯入直接壞掉。
--
-- 解法沿用本表既有慣例：check_blood_test_items_no_delete 早就用同一個機制處理合法的
-- cascade 場景。GUC 另取一個名字（不與 delete 那個共用），避免打開 DELETE 例外時
-- 連帶把 TRUNCATE 也放行。bypass 走 SET LOCAL，出了 transaction 自動失效——
-- 它不是「取消擋板」，是具名、有紀錄、範圍限縮的例外。
--
-- ⚠️ 「有紀錄」指的是 **service 層**在同一個 tx 內寫進 user_activity_logs 的稽核事件
-- （DATA_IMPORT_TRUNCATE，進 HMAC 雜湊鏈），不是下面那句 RAISE NOTICE。
-- NOTICE 只是 DB 層的即時提示：PostgreSQL 預設 log_min_messages=warning，
-- NOTICE **不會**寫進 server log（client_min_messages=notice 只讓它送到連線端，
-- 而 sqlx 那端沒有 notice handler）。實測值見 PR #79 討論。
-- 要留下持久且可驗證的軌跡，靠的是 cleanup_partial_unique_tables 內的
-- AuditService::log_activity_tx，與 TRUNCATE 同生共死。

CREATE FUNCTION public.check_blood_test_items_no_truncate() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    bypass TEXT;
BEGIN
    -- 取 session-level GUC；未設定時為 NULL（current_setting with missing_ok = true）
    bypass := current_setting('app.bypass_blood_test_items_truncate', true);
    IF bypass = 'true' THEN
        RAISE NOTICE 'animal_blood_test_items TRUNCATE bypassed via app.bypass_blood_test_items_truncate';
        RETURN NULL;
    END IF;

    RAISE EXCEPTION 'animal_blood_test_items is append-only (GLP §11.70)。不可 TRUNCATE。
若需修正單筆結果，使用 BloodTestService::correct_item_with_reason 走 supersede 流程；
如為 IDXF 全庫匯入這類合法的整表重灌場景，service 層需在同 tx 內
SET LOCAL app.bypass_blood_test_items_truncate = ''true''.'
        USING ERRCODE = 'P0001';
END;
$$;

COMMENT ON FUNCTION public.check_blood_test_items_no_truncate() IS 'GLP §11.70：TRUNCATE 不觸發 row-level trigger，需獨立的 STATEMENT 層擋板。session GUC app.bypass_blood_test_items_truncate 為 escape hatch（IDXF 全庫匯入的 TRUNCATE ... CASCADE 會遞移波及本表）。軌跡由 service 層在同 tx 寫入 user_activity_logs（DATA_IMPORT_TRUNCATE，進 HMAC 鏈）；函式內的 RAISE NOTICE 僅為即時提示，預設 log_min_messages=warning 下不進 server log。';

CREATE TRIGGER check_blood_test_items_no_truncate_trigger
    BEFORE TRUNCATE ON public.animal_blood_test_items
    FOR EACH STATEMENT
    EXECUTE FUNCTION public.check_blood_test_items_no_truncate();
