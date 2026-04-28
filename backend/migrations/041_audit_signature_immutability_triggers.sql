-- R30-20 + R30-21: DB-level immutability triggers
--
-- Why: GLP §11.10(e)(1) + §11.70 要求 audit log 與 electronic signature 不可
--      被竄改。應用層已用 services/audit + services/signature 控制寫入路徑，
--      但 DB 層缺最後一道防線：直接的 SQL UPDATE/DELETE（DBA、誤操作、SQL
--      injection 漏網之魚）會繞過應用層保護。
--
-- 既有狀態（migration 012:90-131）：
--   - user_activity_logs 已有 BEFORE UPDATE trigger 阻擋更新
--   - 缺 user_activity_logs BEFORE DELETE trigger
--   - electronic_signatures 完全無 trigger
--
-- 本 migration 補上：
--   R30-20: user_activity_logs BEFORE DELETE trigger（雙保險，防誤刪）
--   R30-21: electronic_signatures BEFORE UPDATE trigger（只允許 invalidate 流程）
--   R30-21: electronic_signatures BEFORE DELETE trigger（完全禁刪）
--
-- ⚠️ STAGING-ONLY MERGE：trigger 一旦 deploy 即生效；production 直接 merge 前
--    必須在 staging 跑 ≥24h 確認既有 audit / signature 寫入流程不被誤擋。

-- =========================================================================
-- R30-20: user_activity_logs BEFORE DELETE trigger
-- =========================================================================
-- partition table 的 retention drop 會走 ALTER TABLE ... DETACH PARTITION，
-- 不觸發 ROW-level trigger，因此 retention policy 不受影響。
-- 真正觸發此 trigger 的只有：手動 DELETE FROM user_activity_logs WHERE id = ...
-- 這正是要擋的。
CREATE OR REPLACE FUNCTION check_user_activity_logs_no_delete()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'user_activity_logs is append-only (GLP §11.10(e)(1))。不可 DELETE。
若需排除舊資料，使用 partition drop（DETACH PARTITION 不觸發 ROW trigger）'
        USING ERRCODE = 'P0001';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS check_user_activity_logs_no_delete_trigger ON user_activity_logs;
CREATE TRIGGER check_user_activity_logs_no_delete_trigger
    BEFORE DELETE ON user_activity_logs
    FOR EACH ROW
    EXECUTE FUNCTION check_user_activity_logs_no_delete();

COMMENT ON FUNCTION check_user_activity_logs_no_delete() IS
'GLP §11.10(e)(1) 雙保險：阻擋任何 DELETE FROM user_activity_logs。partition retention drop 走 DETACH PARTITION 不觸發。';

-- =========================================================================
-- R30-21: electronic_signatures BEFORE UPDATE trigger
-- =========================================================================
-- 只允許 services/signature/mod.rs::invalidate 走的「軟失效」UPDATE：
-- 設定 is_valid + invalidated_reason + invalidated_at + invalidated_by。
-- 其他欄位變動全擋。
CREATE OR REPLACE FUNCTION check_electronic_signatures_immutable()
RETURNS TRIGGER AS $$
BEGIN
    -- core fields (簽章本身的內容) 不可動
    IF OLD.entity_type IS DISTINCT FROM NEW.entity_type
       OR OLD.entity_id IS DISTINCT FROM NEW.entity_id
       OR OLD.signer_id IS DISTINCT FROM NEW.signer_id
       OR OLD.signature_type IS DISTINCT FROM NEW.signature_type
       OR OLD.content_hash IS DISTINCT FROM NEW.content_hash
       OR OLD.signature_data IS DISTINCT FROM NEW.signature_data
       OR OLD.signature_method IS DISTINCT FROM NEW.signature_method
       OR OLD.handwriting_svg IS DISTINCT FROM NEW.handwriting_svg
       OR OLD.stroke_data IS DISTINCT FROM NEW.stroke_data
       OR OLD.signed_at IS DISTINCT FROM NEW.signed_at THEN
        RAISE EXCEPTION 'electronic_signatures core fields immutable (GLP §11.70)。
僅 is_valid / invalidated_reason / invalidated_at / invalidated_by 可由 SignatureService::invalidate 修改。'
            USING ERRCODE = 'P0001';
    END IF;
    -- 走到這裡代表 core fields 都沒動 → 通過（is_valid + invalidated_* 可改）
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS check_electronic_signatures_immutable_trigger ON electronic_signatures;
CREATE TRIGGER check_electronic_signatures_immutable_trigger
    BEFORE UPDATE ON electronic_signatures
    FOR EACH ROW
    EXECUTE FUNCTION check_electronic_signatures_immutable();

COMMENT ON FUNCTION check_electronic_signatures_immutable() IS
'GLP §11.70：簽章 core fields (entity / signer / hash / signature_data / handwriting / signed_at) 不可動。僅允許軟失效（is_valid + invalidated_* 4 欄）。';

-- =========================================================================
-- R30-21: electronic_signatures BEFORE DELETE trigger
-- =========================================================================
CREATE OR REPLACE FUNCTION check_electronic_signatures_no_delete()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'electronic_signatures is append-only (GLP §11.70)。不可 DELETE。
若需作廢簽章，使用 SignatureService::invalidate（軟失效，保留紀錄）。'
        USING ERRCODE = 'P0001';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS check_electronic_signatures_no_delete_trigger ON electronic_signatures;
CREATE TRIGGER check_electronic_signatures_no_delete_trigger
    BEFORE DELETE ON electronic_signatures
    FOR EACH ROW
    EXECUTE FUNCTION check_electronic_signatures_no_delete();

COMMENT ON FUNCTION check_electronic_signatures_no_delete() IS
'GLP §11.70：electronic_signatures 一律不可 DELETE。作廢簽章須走軟失效流程。';
