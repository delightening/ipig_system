-- `protocol_activity_type` 新增 `SD_ASSIGNED`（裁定 21）
--
-- 問題：SD 變更目前只落成通用的 `PROTOCOL_UPDATE`，稽核報表上跟改標題、改日期
-- 長得一模一樣，要靠人去比對 before/after 的 JSON 才知道改的是 SD。
-- 而 GLP 稽核會問「這份計畫的 SD 換過幾次、誰換的」。
--
-- 實測依據（正式庫）：`user_activity_logs` 裡 `changed_fields` 真的含
-- `study_director` 的只有個位數，其餘是整份快照剛好帶到這個欄位——
-- 也就是說現況連「換過幾次」都得靠人工判讀。
--
-- ⚠️ 為什麼需要這支 migration：`ProtocolActivityType` 是 **PostgreSQL enum**
-- （`#[sqlx(type_name = "protocol_activity_type")]`），不是 Rust-only 的型別。
-- 只在 Rust 端加 variant 的話 `cargo check` 會過、`cargo check --tests` 也會過，
-- 但寫入時 runtime 才炸：
--     invalid input value for enum protocol_activity_type: "SD_ASSIGNED"
-- 「編譯成功」在這裡完全不代表能用。

ALTER TYPE public.protocol_activity_type ADD VALUE IF NOT EXISTS 'SD_ASSIGNED';

-- ⚠️ `ALTER TYPE ... ADD VALUE` 的三個限制，寫在這裡免得日後有人照抄出事：
--
-- 1. PostgreSQL 12 之前不能在 transaction 內執行。本專案要求 PG 16，
--    而 PG 12+ 已允許，所以不需要 `-- no-transaction` 標記。
-- 2. **新值在同一個 transaction 內不能馬上使用**。這支 migration 只加值、
--    不寫入任何使用該值的資料，所以不受影響；日後若要在同一支 migration 裡
--    回填帶新值的列，必須拆成兩支。
-- 3. 加進去的值**無法移除**（PostgreSQL 不支援 DROP VALUE）。
--    所以命名要一次到位——這裡跟 Rust 端 `as_str()` 的輸出對齊為 `SD_ASSIGNED`，
--    與既有的 `REVIEWER_ASSIGNED` / `VET_ASSIGNED` 同一種構詞。
