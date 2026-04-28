-- R30-A: euthanasia 模組三軸補強 — optimistic lock version 欄
-- Why: pi_approve / pi_appeal / chair_decide / execute 為多角色協作流程，
--      若兩人同時點擊（PI 同時開兩個 tab、CHAIR 同時收到兩個獸醫的 appeal）
--      會出現 lost update。需與 services/animal/core/update.rs 同模式以
--      `version INTEGER` 做 optimistic lock：UPDATE 時帶 `WHERE version = $n`，
--      命中 0 row 即回 409 Conflict（「此記錄已被其他人修改」）。
-- How:  euthanasia_orders / euthanasia_appeals 各加 `version INTEGER NOT NULL
--      DEFAULT 1`，service 層每次 UPDATE 同步 `version = version + 1`。

ALTER TABLE euthanasia_orders
    ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;

ALTER TABLE euthanasia_appeals
    ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;

COMMENT ON COLUMN euthanasia_orders.version IS
'Optimistic lock version。每次 UPDATE 時 +1，service 層 WHERE version = $n 命中 0 row 即回 409 Conflict。';
COMMENT ON COLUMN euthanasia_appeals.version IS
'Optimistic lock version。語意同 euthanasia_orders.version。';
