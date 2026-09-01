-- 結案雙簽（設計 A）：`protocols` 加兩個結案簽章外鍵。
--
-- 需求：計畫要結案必須 PI 與 SD 各簽一次，兩簽齊備才允許轉 CLOSED。
-- 設計文件：docs/design/protocol-sd-close-and-handover.md §5
--
-- 為什麼加在 protocols 而不是開新表：一份計畫只結案一次，不需要一對多；
-- 與既有的 `equipment_disposals` / `equipment_idle_requests` 兩處做法一致。
--
-- ⚠️ **這兩個欄位不是判斷「可以結案了」的完整依據。**
-- 外鍵只保證「那一列簽章存在」，不保證它是**這份計畫的**、**結案用的**、
-- **還有效的**簽章。完整的 gate 有 7 條條件（entity_type / entity_id /
-- signature_type / is_valid / signer 對應 / 兩簽不同人），實作在
-- `services/protocol/closure.rs`，且必須在已 FOR UPDATE 鎖住該 protocol 的
-- 同一個 transaction 內求值。
-- 只驗這兩欄非空的話，任何能把既有簽章 id 寫進來的路徑都能繞過雙簽。
--
-- ⚠️ 為什麼 migration 編號是 008 而不是 007：007 已被 PR #28
-- （`007_protocol_activity_sd_assigned.sql`）佔用，該 PR 尚未合併。
-- 依 RULES_BACKEND §9 的選號程序，取尚未被任何 open PR 使用的最小編號。

ALTER TABLE public.protocols
    ADD COLUMN close_pi_signature_id UUID REFERENCES public.electronic_signatures(id),
    ADD COLUMN close_sd_signature_id UUID REFERENCES public.electronic_signatures(id);

COMMENT ON COLUMN public.protocols.close_pi_signature_id IS
    '結案雙簽：PI 那一簽（entity_type=protocol_closure）。非空不等於有效，完整 gate 見 services/protocol/closure.rs';
COMMENT ON COLUMN public.protocols.close_sd_signature_id IS
    '結案雙簽：SD 那一簽（entity_type=protocol_closure）。非空不等於有效，完整 gate 見 services/protocol/closure.rs';

-- 查「這份計畫簽到哪了」用。部分索引：絕大多數計畫沒有結案簽章，
-- 全欄位索引等於為 NULL 建索引，沒有意義。
CREATE INDEX protocols_closure_signed_idx
    ON public.protocols USING btree (id)
    WHERE close_pi_signature_id IS NOT NULL OR close_sd_signature_id IS NOT NULL;
