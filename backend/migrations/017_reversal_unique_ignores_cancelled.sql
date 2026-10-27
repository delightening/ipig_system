-- 017: 作廢的沖銷單不再佔用原單「唯一的沖銷名額」
--
-- 背景（2026-10-27 使用者回報的 bug）
-- ============================================================================
-- 原單 A（已核准）→ 發起沖銷單 B（已提交、尚未核准）→ **作廢 B**。
-- 作廢 B 本身是安全的：B 還沒核准，沒有鏡射過任何庫存或傳票。
-- 但作廢之後 A 被卡死：
--   * 資料庫唯一索引 `idx_documents_reverses_doc_id_unique` 不分狀態，
--     作廢的 B 仍佔著 A 唯一的沖銷名額，A 無法再被沖銷；
--   * A 已核准，`cancel()` 禁止作廢已核准單據（要求改走沖銷）；
--   * 沖銷單也不在 A 的 source_doc_id 後代鏈上，硬刪 A 只會撞外鍵。
-- 結果 A 既不能沖銷、不能作廢、也不能刪——「進退維谷」。
--
-- 為什麼改索引而不是別的做法
-- ============================================================================
-- 系統在別處早已認定「只有 **已核准** 的沖銷才算沖銷」：`exclude_reversed_grn!`
-- 要求 `r.status = 'approved'` 才把原單排除在入庫進度之外。唯一索引、
-- `create_reversal` 的重複檢查、詳情頁的「被哪張單沖銷」三處卻沒有這個條件，
-- 與那條判準不一致——這是疏漏，不是設計。
--
-- 新條件：同一張原單最多一張**未作廢**的沖銷單（已提交或已核准）。
-- 仍然擋住並發下的重複沖銷（兩張都是 submitted/approved 時，後者撞唯一索引）；
-- 作廢的沖銷單保留在紀錄中（GLP append-only，不刪、不改），只是不再佔名額。
--
-- 安全性
-- ============================================================================
-- 新條件比舊條件**寬鬆**（舊：所有沖銷單；新：排除 cancelled 的），
-- 現有資料不可能讓新索引建立失敗。
--
-- 回復（down，手動）：
--   DROP INDEX IF EXISTS idx_documents_reverses_doc_id_unique;
--   CREATE UNIQUE INDEX idx_documents_reverses_doc_id_unique
--       ON documents (reverses_doc_id) WHERE reverses_doc_id IS NOT NULL;
-- ⚠️ 回復前要先確認沒有「同一原單有作廢加有效兩張沖銷單」的資料，否則舊索引會建立失敗。

DROP INDEX IF EXISTS idx_documents_reverses_doc_id_unique;

CREATE UNIQUE INDEX idx_documents_reverses_doc_id_unique
    ON documents USING btree (reverses_doc_id)
    WHERE reverses_doc_id IS NOT NULL AND status <> 'cancelled'::doc_status;

COMMENT ON COLUMN documents.reverses_doc_id IS
    'R84-5 紅字沖銷：本單為哪一張原始單據的沖銷單（放在沖銷單身上，指向原單；一般單據為 NULL）。partial unique index 保證一張原單最多一張「未作廢」的沖銷單（017：作廢的沖銷單不佔名額，原單可重新發起沖銷）。';
