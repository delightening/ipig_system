# Audit Log HMAC Versioning（R28-M1 + R28-M2）

> **目的**：澄清 `user_activity_logs.hmac_version` 的設計、verifier 行為、backfill plan、以及 actor class residual risk acceptance。
>
> **背景**：R26-6 引入 length-prefix canonical encoding (v2) 取代舊的 string-concat (v1)，避免欄位串接碰撞風險。Migration 037 加 `hmac_version SMALLINT` column 標記版本。R28 review (M1+M2) 發現 migration 註解與 verifier 實際行為不一致，且 Anonymous→SYSTEM HMAC 替代讓 actor 類別在鏈中無法區分。

## 1. 兩種 HMAC 編碼

| Version | 名稱 | 引擎 | 寫入路徑 |
|---|---|---|---|
| **v1** | Legacy string-concat | `format!` 串接後 SHA-256 | R26 前 `log_activity` / `compute_and_store_hmac`（已移除 R26-4） |
| **v2** | Length-prefix canonical | 每欄位 `8-byte BE length + UTF-8 bytes` | R26+ `log_activity_tx` / `compute_and_store_hmac_tx` |

v2 解決 v1 的欄位邊界碰撞風險（如 `before="{a:1}"` + `after="{}"` vs `before="{a:1}{}"` + `after=""` 在 v1 編碼下產生同一字串）。

## 2. Verifier 行為（R28-M1 修正後）

`AuditService::verify_chain_rows` 對每筆 row：

```
hmac_version Some(v) → compute_hmac(v)，比對 stored
hmac_version None    → try-both 策略
                       ├─ 試 v2 (canonical) → 若 match → OK
                       └─ 試 v1 (legacy) → 若 match → OK
                       └─ 兩者都不 match → 斷鏈
```

**為什麼 NULL 採 try-both 而非「假設 v=1」**：
Migration 037 前的時間區間內，`log_activity_tx` 已使用 v2 編碼但 `hmac_version` column 不存在。這些 row 是 **v2 但 hmac_version=NULL**。若 verifier 假設 NULL=v1，會對這批 row 產生 false positive 斷鏈告警。

backfill 完成後（hmac_version 全表 NOT NULL），verifier 只走 explicit version 路徑，try-both fallback 可移除。

## 3. Backfill Plan

### 階段 A（已完成）— Migration 037
- 加 `hmac_version SMALLINT NULL` column
- 加 partial index `WHERE integrity_hash IS NOT NULL`
- 不 UPDATE 既有 row（避免全表 lock）

### 階段 B（待執行）— Idempotent Backfill
獨立 binary（暫缺）跑：

```sql
-- v2 row（migration 037 前 log_activity_tx 寫入；canonical encoding match）
UPDATE user_activity_logs
   SET hmac_version = 2
 WHERE hmac_version IS NULL
   AND integrity_hash IS NOT NULL
   AND <verifier 確認 v2 計算等於 stored>;

-- 剩下的 v1 row（pre-R26 legacy）
UPDATE user_activity_logs
   SET hmac_version = 1
 WHERE hmac_version IS NULL
   AND integrity_hash IS NOT NULL
   AND <verifier 確認 v1 計算等於 stored>;
```

**監控指標**（待 R28-future 加）：

```promql
# 尚未 backfill 的 row 數量（目標 → 0）
ipig_audit_hmac_legacy_rows{version="null"}
```

### 階段 C（backfill 完成後）— 移除 try-both fallback

當 `ipig_audit_hmac_legacy_rows{version="null"} = 0` **持續 30 天**後：
1. 修改 `verify_chain_rows`：刪除 `None` 分支的 try-both，改為 hard error（NULL 表示資料異常）
2. 修改 migration 037 comment：刪除 try-both 說明
3. （可選）`ALTER TABLE user_activity_logs ALTER COLUMN hmac_version SET NOT NULL`

## 4. Actor Class Residual Risk（R28-M2）

### 問題

`HmacInput.actor_user_id` 用 `Uuid` 編碼。`ActorContext::Anonymous` 寫入時 `actor_user_id` 在 DB 為 `NULL`，**但 HMAC 計算時用 `SYSTEM_USER_ID` 替代**：

```rust
// audit.rs writer L460-462
let hash_actor = actor
    .actor_user_id()
    .unwrap_or(crate::middleware::SYSTEM_USER_ID);
```

verifier 也用同一 fallback。

### 攻擊情境

理論上：若攻擊者能直接寫 `user_activity_logs` 並計算 HMAC，他可以：
1. 寫一筆 `actor_user_id=NULL` (Anonymous) 的 row，HMAC 用 SYSTEM_USER_ID 簽
2. 改 row 的 `actor_user_id=NULL → SYSTEM_USER_ID`，HMAC 仍 valid（因為 fallback 回相同 UUID）
3. 反之亦可

「Anonymous」與「真 SYSTEM」事件在鏈中**無法區分**。

### Residual Risk Acceptance

**accept** 此 residual risk 而不修補 v3 編碼，理由：

1. **威脅模型**：trigger `trg_user_activity_logs_immutable` 已禁止任何 UPDATE。攻擊者要繞過此 trigger 需要 superuser 權限；有 superuser 已能直接 DROP 表，actor class 區分無實質防護。
2. **HMAC 主要目的是 tamper detection**（內容竄改），不是 actor 認定。Actor 認定靠：
   - `event_type` 命名慣例（SECURITY 類事件無 actor）
   - DB 欄位 `actor_user_id` (NULL = Anonymous, vs UUID)
   - JWT issued by IDP 而非 HMAC chain
3. **修補成本高**：
   - 新增 v3 encoding：HmacInput 加 `actor_class: ActorClass` enum
   - migration 增加 backfill stage
   - verifier 三向 try-both（v3 → v2 → v1）
   - 預估 ~200 LOC + 一個 release cycle

### 未來 v3（若需提升）

若有合規要求（如 21 CFR §11.10(e) 強化），實作步驟：
1. `HmacInput` 加 `actor_class: ActorClass`（User / System / Anonymous）
2. canonical_bytes 加長一個 length-prefixed 欄位
3. 建立 `HMAC_VERSION_V3` 常數
4. writer 預設用 v3
5. verifier 試 v3 → v2 → v1
6. backfill 階段 D：v2 row → v3（新計算後 UPDATE）
7. 階段 C 之後再執行階段 D

## 5. 維護記錄

| Date | Change | By |
|---|---|---|
| 2026-04-27 | Initial — R28-M1 註解修正 + R28-M2 residual risk doc | Claude (PR-E) |
