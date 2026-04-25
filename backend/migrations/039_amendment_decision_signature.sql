-- C2 (B-min): Amendment 決定簽章 FK（21 CFR §11.50/§11.70 — 非否認性）
-- Why: amendment_status_history 只記 who/when/from→to，無 cryptographic signature。
--      當前審查決定 → APPROVED/REJECTED 的最終態無 electronic signature 記錄，
--      審查委員可否認自己的核准/否決決定（無法律效力的 audit trail）。
-- How:  amendments 加 approved_signature_id / rejected_signature_id，連結到
--      electronic_signatures 表。當 check_all_decisions 觸發終態時，於同一 tx 內
--      建立簽章記錄並回填 FK；classify(Minor → ADMIN_APPROVED) 與 change_status
--      到終態時亦同。
--
-- Scope（B-min，後續 PR 可擴）：
--   - 終態簽章由「最後 tipping reviewer」/「分類者」/「狀態變更者」當簽章主體
--   - 不在本 PR 處理 per-reviewer 簽章（amendment_review_assignments.decision_signature_id）
--     — 屬 B-full，需動 RecordAmendmentDecisionRequest 加 password 欄，前端同步調

-- ON DELETE RESTRICT（CodeRabbit review #205, Major）：簽章 FK 是 21 CFR §11.10(e)
-- 「audit trail 不得遮蔽先前記錄」的合規骨幹。SET NULL 會讓 electronic_signatures
-- 被(誤)刪後 FK 靜默清空，update guard 失效；RESTRICT 由 DB 強制阻擋刪除，
-- 符合非否認性語意。
ALTER TABLE amendments
    ADD COLUMN IF NOT EXISTS approved_signature_id UUID
        REFERENCES electronic_signatures(id) ON DELETE RESTRICT,
    ADD COLUMN IF NOT EXISTS rejected_signature_id UUID
        REFERENCES electronic_signatures(id) ON DELETE RESTRICT;

COMMENT ON COLUMN amendments.approved_signature_id IS
'GLP 核准簽章 FK。NOT NULL 後表示已被簽章核准，service 層 update guard 拒絕修改（21 CFR §11.10(e)(1)）。';
COMMENT ON COLUMN amendments.rejected_signature_id IS
'GLP 否決簽章 FK。語意同 approved_signature_id 但表示被否決。';

-- 索引：admin dashboard / 合規報表查詢已簽章記錄
CREATE INDEX IF NOT EXISTS idx_amendments_approved_signature
    ON amendments (approved_signature_id) WHERE approved_signature_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_amendments_rejected_signature
    ON amendments (rejected_signature_id) WHERE rejected_signature_id IS NOT NULL;
