-- 019: 既有沖銷單回填 doc_type = 'REV'，並加上一致性約束
--
-- 背景
-- ============================================================================
-- 見 018。從這支 migration 起，「是沖銷單」有兩個必須一致的表徵：
--   documents.reverses_doc_id IS NOT NULL   （指向原單，既有）
--   documents.doc_type = 'REV'              （類型，新）
-- 約束 documents_rev_type_consistent 保證兩者同進退，之後任何寫入路徑漏改其一都會被資料庫擋下，
-- 而不是悄悄產生「有指向卻不是 REV」或「是 REV 卻沒指向」的單據。
--
-- 回填範圍：所有 reverses_doc_id 不為空的單據（含已作廢的沖銷單；它們同樣是沖銷單）。
-- 不改的東西：
--   * doc_no：單號唯一且已進稽核紀錄，不回頭改；舊沖銷單維持原單前綴（例如 GRN-…）。
--     新建的沖銷單才使用 REV- 前綴。
--   * stock_ledger.doc_type：沖銷鏡射的庫存流水沿用**原單類型**，報表依 doc_type + direction
--     加總才會互抵，不可改成 REV。
--   * 既有稽核紀錄裡的單據快照：append-only，維持寫入當時的樣子。
--
-- 回復（down，手動）
-- ============================================================================
--   ALTER TABLE documents DROP CONSTRAINT IF EXISTS documents_rev_type_consistent;
--   UPDATE documents r SET doc_type = o.doc_type
--     FROM documents o WHERE r.reverses_doc_id = o.id AND r.doc_type = 'REV';
-- ⚠️ 回復前要確認沒有程式仍會寫入 REV；REV 列舉值本身無法移除（見 018）。

UPDATE documents
   SET doc_type = 'REV'
 WHERE reverses_doc_id IS NOT NULL
   AND doc_type <> 'REV';

ALTER TABLE documents
    ADD CONSTRAINT documents_rev_type_consistent
    CHECK ((doc_type = 'REV') = (reverses_doc_id IS NOT NULL));

COMMENT ON CONSTRAINT documents_rev_type_consistent ON documents IS
    '沖銷單的兩個表徵必須一致：doc_type = REV 若且唯若 reverses_doc_id 不為空（019）。';
