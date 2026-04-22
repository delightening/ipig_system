# R26 Service-driven Audit Refactor — Full Plan (SDD)

> **SDD Template v1.0**
> **SDD = Service-Driven Design**：把 audit log 寫入責任從 handler 層下移到 service 層，與資料變更同 tx 原子提交。
> 此 Full Plan 為**回溯性 + 前瞻性混合**：3 個 PR 已 merge、8 個 PR 開出 review 中、剩餘約 ~44 sites 未啟動。
> 維護規則見文末 §16。

---

## 一、動機（Why）— 為什麼要 SDD

### 1. 發現了什麼

**2026-04-21 委託對 Rust backend 進行四階段全流程審查**（架構 / 並發 / **Phase 3 GLP 合規** / 維護），產出 [docs/reviews/2026-04-21-rust-backend-review.md](../reviews/2026-04-21-rust-backend-review.md)。Phase 3 GLP 合規階段抓出 5 個結構性問題，**全部指向同一技術根因**：audit log 寫入不在資料變更的 transaction 內、且無法完整保留變更內容。

具體證據（5 個 finding，均明確標記 GLP）：

| Finding | 觀察到的症狀 | 量化證據 |
|---|---|---|
| **CRIT-01** IACUC 編號 race condition | `generate_iacuc_no` 查 max + 1 + UPDATE 之間無鎖；併發 `change_status` 拿到相同 max_seq → UNIQUE 衝突 | 3 個 numbering generator（`backend/src/services/protocol/numbering.rs:30 / 91 / 142`）皆無 tx |
| **CRIT-02** 多表變更未包 transaction | service 層多步寫入沒包 tx，中途 panic 留下不一致狀態 | `pool.begin()` 僅出現於 **8 / 80 service 檔，覆蓋率 10%**（`accounting / auth/login / storage_location / system_settings / document/* / animal/vet_patrol`）|
| **CRIT-03** UPDATE 類稽核無 before/after snapshot | handler 寫 `log_activity` 時 `before_data: None, after_data: None`，稽核員無法還原變更內容 | 僅 IACUC 編號變更（`handlers/animal/animal_core.rs:251-267`）有正確填 before/after；其餘 ~50 個 UPDATE call sites 全空 |
| **WARN-06** audit 用 `tokio::spawn` fire-and-forget | spawn 後失敗只記 `tracing::error!`，shutdown 時 task 被丟棄 → audit 漏寫但業務變更已 commit | ~20 個 `tokio::spawn(async move { AuditService::log_activity(...) })`（`auth/login.rs:58 / amendment.rs:174 / document.rs:203 / user.rs:77 / etc.`）|
| **SUGG-03** HMAC chain 缺自動驗證 | `compute_and_store_hmac` 寫入鏈，但無 job 定期掃描；繞過 trigger 改 DB 會斷鏈但不會被發現 | `services/audit.rs:202-215` 有 hmac 計算函式，無對應 verify job |

審查報告結論（單人維護視角，第一條）：
> 「CRIT-01 / CRIT-02 — 這是 **GLP 稽核最可能被挑戰的地方**」

### 2. 欠缺什麼

對照 **21 CFR Part 11 §11.10** audit trail 4 項硬要求，目前能力 vs 目標：

| Part 11 要求 | 目前能力 | 欠缺 |
|---|---|---|
| **Secure**（不可竄改） | HMAC chain 已實作 | **缺自動驗證**：斷鏈不會被偵測（SUGG-03） |
| **Computer-generated, time-stamped** | DB `created_at` 自動產生 | ✓ 已達標 |
| **Independent**（不可被繞過、不會半成品） | handler 層 fire-and-forget 為主 | **核心缺口**：audit 與資料變更不在同 tx；spawn 失敗 audit 漏寫但業務 commit；shutdown 期間 spawn task 被丟棄（CRIT-02 + WARN-06） |
| **Records create / modify / delete**（含完整 before/after） | 多數 UPDATE 無 snapshot | **核心缺口**：稽核員無法從 `user_activity_logs` 還原變更內容（CRIT-03） |

**第二層缺口**（執行 SDD 過程中浮現）：
- **缺 Service-driven 的 ergonomics**：原 `log_activity(11 個位置參數)` API 不適合在 ~45 個 call sites 重複使用
- **缺 actor context 抽象**：原本 service 接 `Uuid` 無法區分 user / system / anonymous，audit log 的 `actor_user_id` 對 batch / scheduler 觸發都填同一個 user id
- **缺 redact 機制**：若把 entity 整個塞進 `before_data`，密碼雜湊 / token 等敏感欄位會明文進 audit log

### 3. 為什麼現在要做

**時機論證 — 這個月是窗口**：

1. **客戶在排隊**：本系統目標客戶含**藥廠 GLP 實驗室 / CRO / 異種器官移植研究機構**。GLP 客戶採購前會做 vendor 合規 due-diligence，21 CFR Part 11 §11.10 是基本門檻。沒過 → 直接出局。
2. **應用層 security 已飽和，剩 architectural gap**：2026-04-20 commit `e710265` 完成 [docs/security/SECURITY_COMPLETED.md](../security/SECURITY_COMPLETED.md) 彙整 — **46 項 application-layer 強化已 done**（含 P1-7 電子簽章 §11.50/.100 合規、SQL/XSS/CSRF、Rate Limit、Audit UI 匯出）。剩下未檢視的就是 architectural-layer audit pipeline。
3. **越拖越貴**：目前實測 **97 個 `AuditService::log_activity / ::log` call sites 跨 27 個 handler 檔**。每多一個新 handler 上線，遷移成本 +1。
4. **基礎設施剛 ready**：`tokio_util::sync::CancellationToken` / `pg_advisory_xact_lock` / `sqlx::Transaction` 都是現有依賴，不需引入新 crate。
5. **單人維護視角**：要 GLP 認證一次到位，不能拖到下次有 audit 案才做。

**反面論證**（為什麼不再等）：審查報告 P0 排程「本月內」對 CRIT-01/02/04 + Argon2 spawn_blocking 有具體工時估計（共 ~2.5 day）。INFRA 實際做下去發現延伸到 ~465h，但**不做的代價是 GLP 失敗 = 失去整個客戶族群**。

---

## 二、目標與範圍（What）

### 4. 預期成果（Definition of Done）

每條皆可被機械式檢驗：

- [ ] **DoD-1（合規）**：所有資料 mutation（CREATE / UPDATE / DELETE / 狀態轉換）在同一 transaction 內寫 audit log，**handler 層 0 個 `tokio::spawn(... log_activity ...)`**
  - 檢驗：`grep -rn "tokio::spawn" backend/src/handlers/ | grep -v notification | wc -l == 0`
- [ ] **DoD-2（snapshot）**：所有 UPDATE 類 audit row 含完整 before / after JSON
  - 檢驗：SQL `SELECT count(*) FROM user_activity_logs WHERE event_type LIKE '%_UPDATE' AND (before_data IS NULL OR after_data IS NULL) == 0`（限新寫入 row）
- [ ] **DoD-3（HMAC chain 完整性）**：每日 02:00 UTC 自動驗證昨日 chain，斷鏈時 INSERT `security_alerts` + 觸發 `SecurityNotifier::dispatch`
  - 檢驗：`SELECT count(*) FROM security_alerts WHERE alert_type='audit_chain_broken' AND created_at >= NOW() - INTERVAL '7 day'` 可查到 cron 每日執行紀錄（即使是 0 broken 也應有 verify success log）
- [ ] **DoD-4（API 收斂）**：舊 `AuditService::log_activity(&pool, ...)` 完全移除；`cargo build` 無 `use of deprecated` warning
  - 檢驗：`cargo build 2>&1 | grep "use of deprecated" | wc -l == 0`
- [ ] **DoD-5（CI 守門）**：CI 加 grep guard，未來新 PR 若引入 `tokio::spawn(async move { AuditService::log_*` 會 fail
  - 檢驗：`.github/workflows/ci.yml` 含對應 grep step
- [ ] **DoD-6（IACUC race 完整修復）**：`change_status` 流程內所有 numbering 寫入用 `&mut tx` + advisory lock；併發 `change_status` 不會撞 UNIQUE
  - 檢驗：load test 模擬 10 並發 `change_status` 全成功（或測試填好的整合測試）
- [ ] **DoD-7（敏感欄位 redact）**：`User` / `UserSession` / `JwtBlacklist` / `TwoFactorSecret` / `McpKey` / `OAuthCredential` 等含敏感欄位的型別覆寫 `AuditRedact::redacted_fields()`；`SELECT before_data, after_data FROM user_activity_logs WHERE event_type='USER_UPDATE'` 看不到 password_hash 等明文
  - 檢驗：手動抽查 + 測試覆蓋
- [ ] **DoD-8（合流回 main）**：`integration/r26` 透過單一 PR 合入 `main`，CI all green
  - 檢驗：對應的 PR merged

### 5. 範圍邊界

**In scope**：
- 97 個 handler 層 `AuditService::log_activity / ::log` call sites 全數遷移
- `services/animal/`, `services/document/`, `services/hr/`, `services/user.rs`, `services/product*.rs`, `services/sku.rs`, `services/partner.rs`, `services/warehouse.rs`, `services/equipment.rs`, `services/role.rs`, `services/ai*.rs`, `services/auth/*.rs`, `services/two_factor.rs`, `services/data_export.rs` 14 個模組的 mutation 全數 SDD 化
- `AuditService::log_activity_tx` + `ActivityLogEntry` + `ActorContext` + `DataDiff` + `AuditRedact` infrastructure
- HMAC chain 每日驗證 cron + 版本化 + 儲存後雜湊
- Migration 033 / 034 / 035 / 036 + 預期 037（HMAC version column）
- `docs/PROGRESS.md` §9 + `docs/TODO.md` R26 章節 + 本 FullPlan.md 維護
- CI workflow（`integration/**` trigger、deprecated warning 過渡期、最終的 grep guard）

**Out of scope**（避免 scope creep）：
- **WARN-01** Service 層直接寫 SQL（909 處 / 80 檔）→ ongoing policy（新 code 寫 repositories/，舊 code 不強求）
- **WARN-02** 非 Argon2 的 spawn_blocking（calamine / rust_xlsxwriter / google_calendar credentials 讀檔）→ 待出現效能問題再處理
- **WARN-03** clippy.toml 閾值對齊 CLAUDE.md → 獨立 PR
- **WARN-05** Cache stampede（alert_threshold / security_notifier / ip_blocklist）→ 獨立 PR
- **WARN-07** equipment.rs 75KB / scheduler.rs 40KB 拆分 → 獨立 PR
- **WARN-08** 權限字串硬編碼 → 獨立 PR
- **SUGG-01 / SUGG-02 / SUGG-04 / SUGG-05** → 獨立 PR
- StockService / AccountingService 內部的 stock ledger / journal entry 自身 audit → R26-3 延伸（PR #6+）
- electronic signature 改寫（已於 P1-7 完成 §11.50/.100，不重做）

### 6. 假設、限制、依賴

**Assumptions**（不成立就整個計畫要重來）：
- (A1) 21 CFR Part 11 §11.10 是合規門檻 — 若客戶都不是 FDA 受管，整個動機消失
- (A2) PostgreSQL advisory lock + sqlx Transaction 行為穩定（`pg_advisory_xact_lock` 自動隨 tx commit / rollback 釋放）
- (A3) `tokio::task::spawn_blocking` 對 Argon2 的延遲改善有效（~50-200ms / 登入）
- (A4) handler 全部用 axum extractor 拿到 `CurrentUser` → 可包成 `ActorContext::User(...)`；非 handler 觸發走 `ActorContext::System { reason: &'static str }`

**Constraints**（硬性限制）：
- (C1) 單人維護：所有 PR 只能由一人 review + merge；批次 review 容量有限
- (C2) 不可 break main：integration/r26 自始至終隔離；epic 完成才合 main
- (C3) 不可 break GLP P1-7 已合規的部分（電子簽章 / IQ-OQ-PQ）— SDD 改 audit log 路徑時不能讓簽章功能 regress
- (C4) HMAC 演算法不換（仍 HMAC-SHA256）；hmac key 不換（仍 `AUDIT_HMAC_KEY` env / secret 檔）

**Dependencies**：
- (D1) `integration/r26` 長期分支（已建於 PR #153）
- (D2) `.github/workflows/ci.yml` `integration/**` trigger（已加於 PR #154）
- (D3) `tokio_util::sync::CancellationToken`（既有依賴）
- (D4) `sqlx::Transaction<'_, Postgres>`（既有依賴）
- (D5) AI reviewer 服務可用性：Gemini Code Assist + CodeRabbit（CodeRabbit 對非預設 base 需手動 `@coderabbitai review` 觸發）
- (D6) 手動 hook `.claude/hooks/block-dangerous.sh`（已加於 PR #154，避免危險命令誤觸）

---

## 三、方案（How）

### 7. 如何完善

**核心方案：Service-Driven Design**。每個 service mutation 採固定模板：

```rust
pub async fn mutate(pool: &PgPool, actor: &ActorContext, /* args */) -> Result<Entity> {
    let _user = actor.require_user()?;          // 或 actor_user_id().unwrap_or(SYSTEM_USER_ID) — 視操作性質
    let mut tx = pool.begin().await?;

    // 1. 取 before（含 row lock 防併發）
    let before = sqlx::query_as::<_, Entity>("SELECT * FROM ... WHERE id = $1 FOR UPDATE")
        .bind(id).fetch_one(&mut *tx).await?;

    // 2. validation + business logic

    // 3. 主 mutation
    let after = sqlx::query_as::<_, Entity>("UPDATE ... RETURNING *")
        .fetch_one(&mut *tx).await?;

    // 4. audit 同 tx 寫入（advisory lock 保證 HMAC chain 串行）
    AuditService::log_activity_tx(&mut tx, actor, ActivityLogEntry::update(
        "MODULE", "ACTION",
        AuditEntity::new("entity_type", after.id, &after.display_name),
        DataDiff::compute(Some(&before), Some(&after)),
    )).await?;

    tx.commit().await?;
    Ok(after)
}
```

**為什麼 SDD 而不是別的**（4 個替代方案的致命缺陷）：

| 方案 | 為什麼捨棄 |
|---|---|
| DB trigger 自動寫 audit | 和現有 HMAC chain 衝突（trigger 無法存取 actor / 無 redact） |
| Handler 層 `tokio::spawn(audit).await` | 仍非同 tx；CRIT-02 不解 |
| Middleware wrap handler 攔截 | 拿不到 mutation 前後的 entity snapshot；CRIT-03 不解 |
| **SDD（採用）** | ✅ 同 tx 原子；✅ 完整 before/after；✅ 失敗自動 rollback；✅ 與業務邏輯零距離 |

**支援設計決策**（時間序，永不刪除舊條目，被推翻者標 ⚠️ Superseded）：

| # | 決策 | 理由 |
|---|---|---|
| D-01 | 長期 integration branch `integration/r26` | 多 PR 系列數週，避免污染 main；可隨時放棄 |
| D-02 | INFRA（PR #153）先行，pattern demo（PR #155）後發 | 確定 API shape 穩定再大規模複製 |
| D-03 ⚠️ | hybrid actor guard：CRUD 用 `unwrap_or(SYSTEM_USER_ID)` | 後被 D-09 取代（不應靜默降級 Anonymous）|
| D-04 ⚠️ | AuditRedact 用 doc warning，不用 compile-time macro | 後被 D-10 強化（醫療/自由文字 entity 應 allowlist）|
| D-05 | upsert 拆 `create_or_update` | 取得完整 before/after diff |
| D-06 | 舊 `log_activity` 標 `#[deprecated]` 漸進遷移 | 一次刪 97 處不可能 review |
| D-07 | `ActivityLogEntry::{update,create,delete,simple}` constructors | 減少 ~45 sites × 4 行樣板 |
| D-08 | soft_delete 留在 simple PR | `change_reasons` 是 audit trail 的一部分 |
| D-09 | Anonymous actor 必須明確拒絕 | CodeRabbit PR #156：靜默降級風險 |
| D-10 | 醫療/自由文字 entity 走 allowlist 或 summary log | CodeRabbit PR #156：CareRecord / VetAdvice 空 impl 過寬 |
| D-11 | document approve 跨 service tx 串接保留，內部 audit 延後 | scope 控制；StockLedger 自 audit 歸 R26-3 延伸 |
| D-12 | Batch operation 採 N+1 audit（per-row + summary） | per-row only / summary only 都各有缺陷 |
| D-13 | PR #159 用 struct literal 不用 constructor | 避免 PR #156 rebase 衝突 |
| D-14 | 4/22 同日開 4 個並行 PR | pattern 已 proven、模組獨立 |

### 7.3 兩個不同度量單位的澄清（PR #165 review follow-up）

本文件**同時出現**兩個不同的計量單位，讀者容易誤以為可以加總，特此澄清：

- **97 call sites**（handler-level）= `grep -rn "AuditService::log_activity\|AuditService::log" backend/src/handlers/ | wc -l`，統計 handler 中每一次「呼叫 audit 的位置」。一個 handler function 可能有 2-3 個 call site（create / update 各一次）。
- **mutations** = service 層實際需要 audit 的資料變更函式（`pub async fn create / update / delete / approve ...`）。一個 mutation 可能對應 0-3 個 handler call sites。

兩者**不能直接加總或比對**。下方 §8 表格的「17 mutations 已遷」與 §1 標頭「97 call sites」是不同度量，這是刻意保留兩個數字：一個看程式碼改動量（mutations），一個看 reviewer 可 grep 驗證的數字（call sites）。

進度摘要表（以 merged-into-integration/r26 為準，隨 PR merge 動態更新）：

| 模組 | mutations (planned) | mutations (merged) | handler call sites (planned) | handler call sites (cleared) |
|---|---|---|---|---|
| protocol | 1 | 1 (PR #155) | 1 | 1 |
| animals | 49 | 0 ⚠️ PR #156 closed | 49 | 0 |
| document | 10 | 0 ⚠️ PR #159 closed | 8 | 0 |
| HR (leave/overtime/balance/attendance) | 21 | 21 (PR #160 + #161) | ~8 | ~8 |
| user | 4 | 4 (PR #162) | 6 | 6 |
| partner/warehouse/equipment | 13 | 13 (PR #166) | 12 | 12 |
| product/sku | 13 | 0 (PR #164 open) | 12 | 0 |
| role/ai/auth/two_factor | ~12 | 0 | ~12 | 0 |
| **合計** | **~123** | **39 已 merged** | **~108** | **27 已清** |

注意：
- 合計「~123 mutations」 > 原估「97 call sites」，是因 mutations 單位更細（一個 service method 即使對應 1 個 handler call site，也算獨立 mutation 需改）。
- PR #156 / #159 狀態為 **closed（非 merged）**，對應的 animals / document 需另案重開，不列入已落地數字。
- 本表為估計值與現況快照，隨 PR 進度每週同步（見 §16 維護規則）。

### 8. 具體要改什麼

| 類型 | 位置 | 改動 |
|---|---|---|
| **新增** | `backend/src/middleware/actor.rs` | `ActorContext` enum + `SYSTEM_USER_ID` const + 7 methods |
| **新增** | `backend/src/models/audit_diff.rs` | `DataDiff` struct + `AuditRedact` trait + `compute / create_only / delete_only / empty` |
| **新增** | `backend/src/services/audit_chain_verify.rs` | `verify_yesterday_chain` 函式 + `yesterday_range_utc` helper + 2 tests |
| **新增** | `backend/migrations/033_system_user.sql` | SYSTEM user row（FK 約束有效）|
| **新增** | `backend/migrations/034_audit_impersonation.sql` | `impersonated_by_user_id` column |
| **新增** | `backend/migrations/035_audit_log_activity_v2.sql` | stored proc 12 參數版本 |
| **新增** | `backend/migrations/036_audit_log_activity_v3.sql` | changed_fields fallback UNION + `IS DISTINCT FROM` + `jsonb_typeof` 守衛 |
| **新增** | `backend/migrations/037_audit_hmac_version.sql`（待做）| `user_activity_logs.hmac_version SMALLINT` for R26-6 |
| **新增** | `docs/plans/pr*.md`（5 份）| 各 PR 的執行計畫 |
| **修改** | `backend/src/services/audit.rs` | + `ActivityLogEntry` struct + `log_activity_tx` + `HmacInput` + `compute_and_store_hmac_tx` + `verify_chain_range` + `compute_hmac_for_fields`；舊 `log_activity` 標 `#[deprecated]` |
| **修改** | `backend/src/services/protocol/{numbering, history, status, core}.rs` | numbering 接 `&mut tx` + advisory lock；submit() Service-driven 完整改寫 |
| **修改** | `backend/src/services/animal/{source, weight, vet_advice, care_record, observation, surgery, medical, blood_test, transfer, import_export, field_correction, vet_patrol, sudden_death}.rs` | 17 simple mutations 已遷（PR #156）；剩 32 moderate/complex 待遷（PR #4b-e） |
| **修改** | `backend/src/services/document/{crud, workflow, grn}.rs` | 10 mutations 已遷（PR #159），含 cross-service approve 同 tx |
| **修改** | `backend/src/services/hr/{leave, overtime, balance, attendance}.rs` | 21 mutations 已遷（PR #160 + #161），含 7 balance helpers tx 化 |
| **修改** | `backend/src/services/user.rs` | 4 mutations + 7 audit sites 已遷（PR #162）；含首個非空 `redacted_fields()` |
| **修改** | `backend/src/services/{product, sku, partner, warehouse, equipment, role, ai, auth/*, two_factor}.rs` | 待遷（PR #6b/c/d，~44 sites） |
| **修改** | `backend/src/handlers/animal/{source, weight_vaccination, vet_advice, care_record, observation, surgery}.rs` 等 | 移除 fire-and-forget `tokio::spawn(audit)` + 改傳 `ActorContext` |
| **修改** | `backend/src/main.rs` | `CancellationToken` 串到 `AppState` / `JwtBlacklist::start_cleanup_task` / scheduler 14 jobs |
| **修改** | `backend/src/services/scheduler.rs` | 14 jobs 加 `is_cancelled()` 檢查；新增 `register_audit_chain_verify_job` cron |
| **修改** | `.github/workflows/ci.yml` | + `integration/**` trigger；+ `-A deprecated` 過渡 flag；最終加 grep guard |
| **修改** | `.coderabbit.yaml` | `language: zh-TW → zh`（schema 合法值） |
| **修改** | `.claude/hooks/block-dangerous.sh` | Python shlex tokenize（避免 bash grep 對 commit message 誤觸） |
| **修改** | `docs/PROGRESS.md` §9 / `docs/TODO.md` R26 / `docs/reviews/2026-04-21-rust-backend-review.md` | 進度 + 對照標記 |
| **刪除** | `backend/src/services/audit.rs::log_activity` 舊版 | R26-4（最後一個遷移 PR 完成後）|
| **刪除** | 11 處 `#[allow(dead_code)]` per-item allows | R26-7 |
| **刪除** | `.github/workflows/ci.yml` `-A deprecated` flag | R26-4 同步 |

---

## 四、執行（Execute）

### 9. 為什麼要分步驟

**4 個分步驟邏輯**：

1. **INFRA 必須先行**：`ActorContext` / `DataDiff` / `log_activity_tx` 是後續所有 PR 的依賴；若 API shape 設計錯，後面 10 個 PR 全要重做。先 PR #153 + #154 把 INFRA 穩定。
2. **Pattern demo 必須先驗證**：複製模板到 ~10 個模組前，先用 PR #155（Protocol::submit 一個函式）證明模板可行；reviewer 重點放在「pattern 好不好複製」。**這就是 SDD「先設計再實作」的精髓**。
3. **模組 PR 必須獨立可 merge**：若 5 個模組 PR 互相依賴，任一受 review block 全卡。所以模組 PR 從 `integration/r26` HEAD 各自 fork，**不互依**；只在 docs（PROGRESS / TODO）有預期 rebase 衝突，政策先寫好。
4. **清理 PR 最後做**：`R26-4 舊 log_activity 移除` 必須 99% 已遷完才能做（不然編譯 fail）；同理 `R26-7 dead code 11 處` 必須其他 module PR 都 merge 後才能正確判斷死碼。

### 10. 步驟拆解

每個步驟對應 1 個或多個 PR。**標 ✅ 已完成、🟡 開出 review、⏳ 未啟動**。

---

#### Step 1 ✅ INFRA 落地（PR #153 + #154）

- **Input**：2026-04-21 審查報告 5 個 GLP finding；4/20 SECURITY_COMPLETED 已知 application-layer 飽和
- **Action**：建 `integration/r26` branch；落地 `ActorContext` / `DataDiff` / `AuditRedact` / `log_activity_tx` / `CancellationToken`；migrations 033/034/035/036；PR #154 處理 PR #153 的 review feedback（HmacInput length-prefix / changed_fields fallback / shutdown log / CI integration trigger）
- **Output**：基礎 API 可被後續 service 函式使用；91 個舊 `log_activity` 仍有效（`#[deprecated]`）
- **預估時間**：實際 ~3 day（INFRA 1.5d + review fix 1d + docs 0.5d）
- **驗證**：見 §11 Step 1

---

#### Step 2 ✅ SDD Pattern Demo（PR #155）

- **Input**：Step 1 的 INFRA API；Codex pre-review 給 CONDITIONAL GO
- **Action**：把 `ProtocolService::submit` 完整改寫為 SDD（SELECT FOR UPDATE → INSERT versions → generate_apig_no(&mut tx) → UPDATE → record_status_change_tx → log_activity_tx → commit）；解 CRIT-01 IACUC race（3 numbering generators + advisory lock）；解 CRIT-04 crate-level allow_dead_code 移除；產出 `pr4a-animals-simple-mutations.md` + `pr5-hr-document-roadmap.md` 計畫文件
- **Output**：1 個 mutation 完整 SDD；template 可被複製
- **預估時間**：實際 ~1 day
- **驗證**：見 §11 Step 2

---

#### Step 3 🟡 模組漸進遷移（PR #156-#163，並行 8 個 PR）

- **Input**：Step 2 的 SDD template
- **Action**：
  - PR #156 animals simple（17 mutations + 16 AuditRedact stubs + ActivityLogEntry constructors）
  - PR #157 docs aftermath + R26-3 數字訂正（~20 → 97）+ pr5a 計畫
  - PR #158 R26-2 HMAC chain daily verify cron
  - PR #159 document（10 mutations，含 cross-service approve）
  - PR #160 hr/leave（7 mutations + 7 balance helpers tx 化）
  - PR #161 hr/overtime + balance + attendance（14 mutations）
  - PR #162 user（4 mutations + 7 audit sites + 首個非空 `redacted_fields()`）
  - PR #163 docs 4-PR 紀錄
- **Output**：53 / 97 mutations migrated；deprecated warnings 91 → ~50；HR epic closure
- **預估時間**：實際 ~2 day（含 4/22 同日 4 PR 並行）；剩 review + merge ~1-2 day
- **驗證**：見 §11 Step 3

---

#### Step 4 ⏳ animals moderate/complex 完成（PR #4b → #4e）

- **Input**：Step 3 的 PR #156 已 merge
- **Action**：
  - PR #4b：observation/surgery update（含 `record_versions` 巢狀表）
  - PR #4c：transfer state machine（initiate / vet_evaluate / approve / complete / reject）
  - PR #4d：import_export 批次（per-row + summary audit）
  - PR #4e：field_correction + vet_patrol + medical sudden_death + blood_test
- **Output**：animals 49 sites 全完成；deprecated warnings ~50 → ~20
- **預估時間**：~7 day（4 個 PR × 1-3 day each）
- **驗證**：見 §11 Step 4

---

#### Step 5 ⏳ R26-6 HMAC 版本化 + 儲存後雜湊

- **Input**：PR #158 merged（HMAC verify cron 已上線，但對 legacy row 會 false positive）
- **Action**：
  - 新增 migration 037：`user_activity_logs.hmac_version SMALLINT` column（`1`=string-concat legacy / `2`=length-prefix canonical）
  - `log_activity` 寫 v=1，`log_activity_tx` 寫 v=2
  - `verify_chain_range` 依 version 分流計算 expected hash
  - `log_activity_tx` 改為「stored proc 內 INSERT + 計算 final changed_fields → 後計算 HMAC → UPDATE row」（CodeRabbit PR #154 Major 議題）
- **Output**：HMAC verify cron 對所有 row 都能正確驗證；INSERT-then-UPDATE HMAC pattern 收斂為一次 round-trip
- **預估時間**：~1.5 day
- **驗證**：見 §11 Step 5

---

#### Step 6 ⏳ 其他模組遷移（PR #6b → #6d）

- **Input**：Step 4 完成
- **Action**：
  - PR #6b：product + sku（12 sites）
  - PR #6c：partner + warehouse + equipment（12 sites）
  - PR #6d：role + ai + auth + two_factor（11 sites，含 reset_password / impersonate 涉及 AuthService）
- **Output**：non-animal 48 sites 全完成；deprecated warnings ~20 → 0
- **預估時間**：~5 day
- **驗證**：見 §11 Step 6

---

#### Step 7 ⏳ 邊角清理（R26-9/-10/-11/-12）

- **Input**：Step 6 完成
- **Action**：
  - R26-9：CareRecord / VetAdviceRecord 等敏感 entity 走 redact allowlist 或 summary log
  - R26-10：upsert pattern 併發安全化（鎖父層 / 用 `INSERT ON CONFLICT` 原子 upsert + `xmax = 0 AS was_inserted` 判斷）
  - R26-11：service-layer 授權補強（delete / status_change 類）
  - R26-12：document_lines audit completeness（DOCUMENT_CREATE/UPDATE 含子表）
- **Output**：CodeRabbit PR #156 + Gemini PR #159 的 Major review 全 close
- **預估時間**：~3 day
- **驗證**：見 §11 Step 7

---

#### Step 8 ⏳ 結構性清理（R26-1, R26-7, R26-8）

- **Input**：Step 7 完成
- **Action**：
  - R26-8：`ProtocolService::change_status` 完整 SDD（300+ 行、跨 PartnerService、需 `PartnerService::create_tx`）
  - R26-1：scheduler 長 job（monthly_report / db_analyze / calendar_sync）升級 `tokio::select!` + shutdown grace period
  - R26-7：11 處 per-item `#[allow(dead_code)]` 逐項判斷（API DTO / 預留 utility / serde 被動欄位）
- **Output**：審查報告 P0/P1 全綠；無 dead code 標籤
- **預估時間**：~3 day
- **驗證**：見 §11 Step 8

---

#### Step 9 ⏳ 舊 API 最終移除（R26-4）

- **Input**：所有 mutation 都已遷移（Step 6 完成）
- **Action**：
  - 刪除 `AuditService::log_activity(&pool, ...)` 舊版 + 對應 `compute_and_store_hmac` 舊版
  - 移除 `.github/workflows/ci.yml` 的 `-A deprecated` flag
  - 加 CI grep guard：`grep -rn "tokio::spawn" backend/src/handlers/ | grep "AuditService::log"` 必須 0 match
- **Output**：deprecated warning 0；CI 防迴歸
- **預估時間**：~0.5 day
- **驗證**：見 §11 Step 9

---

#### Step 10 ⏳ Epic 收尾：integration/r26 → main

- **Input**：Step 9 完成；DoD §4 全綠
- **Action**：
  - 開單一 PR `integration/r26 → main`
  - PR description 引本 FullPlan + 列出所有合入的 sub PR + 量化指標
  - rebase / merge 衝突解決
  - main 上 CI 全綠
- **Output**：R26 epic 結束；audit log 完整 GLP §11.10 合規可展示
- **預估時間**：~0.5-1 day（看 main 累積的非-R26 變動量）
- **驗證**：見 §11 Step 10

### 11. 每步驟驗證標準（Acceptance Criteria）

#### Step 1 驗證 ✅
- [x] 1.1 `cargo check` / `cargo clippy --all-targets -- -D warnings -A deprecated`：0 errors
- [x] 1.2 `cargo test --lib`：423/423 pass（PR #153/#154 各自）
- [x] 1.3 `ActorContext` enum 含 User / System / Anonymous 三變體 + 7 methods + 8 tests
- [x] 1.4 `DataDiff::compute` 對 redact 欄位實際把值替換為 `"[REDACTED]"`、欄位名保留（11 tests）
- [x] 1.5 `log_activity_tx` 開頭呼叫 `pg_advisory_xact_lock(hashtext('audit_log_chain'))`
- [x] 1.6 prev_hash 查詢用 `(created_at, id) < ($1, $2)` tuple 比較 + 雙欄 DESC（避免同微秒併發碰撞）
- [x] 1.7 migrations 033/034/035/036 全部可正向 + 反向（spot check）
- [x] 1.8 `CancellationToken` 在 `AppState` 內可被 14 個 cron job 觀察 `is_cancelled()`

#### Step 2 驗證 ✅
- [x] 2.1 `ProtocolService::submit` 改用 `(pool, &ActorContext, id)` 簽名
- [x] 2.2 submit 函式內所有 DB 操作綁同一 `tx`（`begin → ... → commit`）
- [x] 2.3 IACUC `generate_apig_no(&mut tx)` 內呼叫 `pg_advisory_xact_lock(hashtext('protocol_iacuc_number_gen'))`
- [x] 2.4 `submit` 寫入 `user_activity_logs` 含完整 before / after JSON（手動執行測試確認）
- [x] 2.5 `services/mod.rs` 不再有 crate-level `#![allow(dead_code)]`
- [x] 2.6 11 處 per-item `#[allow(dead_code)]` 各自帶理由註解

#### Step 3 驗證 🟡（per-PR）
- 每個 PR 各自必須：
  - [ ] 3.x.1 `cargo test --lib` 423/423 pass
  - [ ] 3.x.2 `cargo clippy --all-targets -- -D warnings -A deprecated` 0 issues
  - [ ] 3.x.3 該模組所有 mutation 簽名改 `(pool, &ActorContext, ...)`
  - [ ] 3.x.4 該模組對應 handler 移除 `tokio::spawn(... AuditService::log_*)`
  - [ ] 3.x.5 deprecated warning 數量單調遞減
- 整體 Step 3：
  - [ ] 3.0 53 / 97 mutations migrated（量測：grep `log_activity_tx` 於 services/）

#### Step 4 驗證 ⏳
- [ ] 4.1 animals 49 sites 全綠
- [ ] 4.2 transfer state machine 任一中途失敗會 rollback animal status
- [ ] 4.3 import_export 批次 audit 採 N+1 粒度（per-row + summary）
- [ ] 4.4 deprecated warnings ≤ 50

#### Step 5 驗證 ⏳
- [ ] 5.1 `user_activity_logs.hmac_version` column 存在
- [ ] 5.2 舊 row（v=1）+ 新 row（v=2）皆可被 `verify_chain_range` 正確驗證（人工注入測試）
- [ ] 5.3 `log_activity_tx` 改為單 round-trip INSERT-with-hash（不再 INSERT-then-UPDATE）
- [ ] 5.4 PR #158 verify cron 對 legacy row 0 false positive

#### Step 6 驗證 ⏳
- [ ] 6.1 deprecated warnings ≤ 5（reset_password / impersonate 等特殊 case 視情況）
- [ ] 6.2 product / sku / partner / warehouse / equipment / role / ai / auth / two_factor mutation 全 SDD

#### Step 7 驗證 ⏳
- [ ] 7.1 CareRecord / VetAdviceRecord 等敏感 entity 的 audit row 經人工抽查無自由文字洩漏
- [ ] 7.2 upsert pattern 併發測試（10 並發）無 UNIQUE 衝突
- [ ] 7.3 delete / status_change 類 service mutation 內含 access check 或明確標 `// 由 handler 層保證`
- [ ] 7.4 DOCUMENT_CREATE / DOCUMENT_UPDATE 的 audit 含 document_lines 變動

#### Step 8 驗證 ⏳
- [ ] 8.1 `ProtocolService::change_status` 完整單 tx；併發 10 個 change_status 全成功
- [ ] 8.2 scheduler 長 job 收到 cancellation 後 ≤ 30s 退出
- [ ] 8.3 `services/mod.rs` 0 個 `#[allow(dead_code)]`

#### Step 9 驗證 ⏳
- [ ] 9.1 `cargo build 2>&1 | grep "use of deprecated" | wc -l == 0`
- [ ] 9.2 CI grep guard 已加（人工注入錯誤 PR 驗證 fail）
- [ ] 9.3 `.github/workflows/ci.yml` 不再含 `-A deprecated`

#### Step 10 驗證 ⏳
- [ ] 10.1 `git log main --oneline | head` 含 R26 epic merge commit
- [ ] 10.2 main CI all green（含 E2E）
- [ ] 10.3 DoD §4 1 ~ 8 全部 ✅
- [ ] 10.4 `docs/security/GLP_VALIDATION.md` 加 R26 後 OQ 補測 row（TC-XX：audit log 完整性）

### 12. 回滾策略

| 步驟 | 失敗徵兆 | 回滾動作 |
|------|----------|----------|
| Step 1 | INFRA API design 錯（後續 PR 才發現） | 修 INFRA → 推 follow-up PR（如 PR #154 對 PR #153）；不需 revert，因為舊 `log_activity` 仍可用 |
| Step 2 | submit 改寫 break Protocol 流程 | revert PR #155 個別 commit；不影響 INFRA |
| Step 3 | 某模組 PR break tests | revert 該模組 PR；其他並行 PR 不受影響（因模組獨立） |
| Step 4 | animals moderate PR break record_versions | revert 該 PR；animals simple（PR #156）已 merged 不受影響 |
| Step 5 | HMAC version migration 不可正向 | migration 037 down 並回滾；HMAC verify cron 暫停執行（手動 disable） |
| Step 6 | 其他模組 PR 互相依賴失敗 | revert 該 PR；獨立模組 PR 不受影響 |
| Step 7 | upsert pattern 修法錯 | revert R26-10 PR；舊 SELECT FOR UPDATE 模式仍能用（雖有 race） |
| Step 8 | change_status 單 tx 化 break PartnerService 互動 | revert R26-8 PR；舊 mini-tx wrapper 仍提供 80% 解 |
| Step 9 | 移除舊 `log_activity` 後 cargo build fail | revert R26-4 PR；恢復 `#[deprecated]` 標籤 + `-A deprecated` |
| Step 10 | `integration/r26 → main` 衝突或 CI fail | rebase + 解衝突 → 重推；極端情況可保留 integration/r26 直到下次 main 合適時機 |

**整體 nuclear option**：放棄 `integration/r26` 整個分支，main 不受影響；損失 R26 工時但系統繼續用舊 audit 模式運作（GLP 合規未升級但未 regression）。

---

## 五、風險與後續

### 13. 已知風險與緩解

來自 reviewer feedback（70 則 review comments：2🔴 / 23🟠 / 43🟡 / 1🔵 / 1 note）+ 設計層面風險：

| 風險 | 可能性 | 影響 | 緩解措施 |
|------|--------|------|----------|
| **PR #158 verifier vs writer NULL hash 分歧** 🔴 | 高 | 高（cron 報錯告警 → 失去信任） | PR #158 自身追加 commit 修正 `ChainRow.actor_user_id: Uuid` panic + verifier prev_hash 推進規則對齊 |
| **R26-6 不做 / 拖太久** | 中 | 高（PR #158 cron 對 legacy row false positive，告警噪音）| Step 5 排在 Step 4 之後立即執行；不允許進 Step 6 前還未做 |
| **CodeRabbit Major：Anonymous 靜默降級為 System**（CRIT-D-09）| 已發生 | 中（audit log 失真）| D-09 採納；後續 PR 統一改明確 `match` reject Anonymous |
| **CodeRabbit Major：CareRecord / VetAdvice 空 redact**（CRIT-D-10）| 已發生 | 中（醫療紀錄外洩）| R26-9 處理；目前已知有風險的 entity 列入 Step 7 |
| **upsert pattern race**（CRIT-D-05）| 中 | 中（罕見併發 → UNIQUE 衝突）| R26-10 處理；目前已知 `AnimalVetAdviceService::create_or_update` 受影響 |
| **跨 service tx 串接（document approve）失敗回滾範圍變大** | 低 | 中（以前 audit 失敗不擋業務，現在會擋） | 已於 PR #159 description 明文揭示；運維文件補對應 runbook |
| **integration/r26 與 main 偏離過大難合併** | 中 | 高（epic 收尾痛苦） | 每 1-2 週手動 `git merge main → integration/r26` 一次；保持兩邊新 commits 數差距 ≤ 100 |
| **AI reviewer 不可用** | 中 | 低（人工 review 仍可繼續） | Codex shared runtime 不穩 → fallback 用 general-purpose agent；CodeRabbit non-default base 需手動觸發 |
| **R26 命名碰撞（與資安強化 epic）** | 已發生 | 低（文件混亂） | FullPlan §1 + §2.1 警告；文件明寫 R26-N 在不同 epic 不可換算 |
| **scope 又被低估** | 低 | 中（再多一輪 4.85× ?）| 已從 ~20 訂為 97，含 buffer；新發現的 R26-9 ~ R26-12 已記入；後續若再發現新 site，先補 §10 步驟再做 |
| **單人 review capacity 不足** | 中 | 中（PR backlog） | 同日最多開 4 個並行 PR（D-14）；docs PR 與 code PR 分流 |
| **HMAC key 輪替時 chain 斷** | 低 | 高 | R26-6 加 hmac_version 後可分流計算；key 輪替前先建 R26 runbook |

### 14. 後續追蹤事項（Out-of-scope but Remember）

R26 範圍**之外**的延伸工作（不要在本 epic 內做，但記下來避免遺忘）：

- **WARN-01 服務層 SQL 散落**（909 處）：ongoing policy 新 code 寫 repositories/，舊 code 不強求；R26 後可做專項 epic
- **WARN-02 spawn_blocking 全面化**（calamine / xlsx / google_calendar credentials）：等出現效能問題再做
- **WARN-03 clippy.toml 對齊 CLAUDE.md**：獨立 PR
- **WARN-05 cache stampede 修法**：alert_threshold / security_notifier / ip_blocklist 改 singleflight；獨立 PR
- **WARN-07 大檔拆分**：equipment.rs 75KB / scheduler.rs 40KB；獨立 epic
- **WARN-08 權限字串 const 化**：`const PERM_ANIMAL_VIEW_ALL: &str = "..."`；獨立 PR
- **SUGG-01 DB pool tuning runbook**：`docs/runbook/database-tuning.md`
- **SUGG-02 backend workspace 拆分**：等編譯時間 > 2h 才考慮
- **SUGG-04 SQL format! 註解**：startup/database.rs `SET statement_timeout` 改用 const + 註解
- **SUGG-05 test fixture parallel 保護**：integration tests 加 `#[serial]` 或 testcontainers
- **StockService / AccountingService 內部 audit**：本 R26 只做 DocumentService 層；StockLedger / JournalEntry 自身的 audit 留 R26 後處理
- **電子簽章與 R26 audit 整合**：簽章已是 §11.50/.100 合規（P1-7）；R26 完成後可考慮把簽章事件也走 `log_activity_tx` 統一
- **GLP_VALIDATION.md 更新**：R26 完成後在 OQ 表加 row（TC-XX：audit log 完整性 / TC-XX：HMAC chain 自動驗證）

### 15. 結語

R26 epic 完成後，世界長這樣：

- **GLP 客戶 due-diligence 通過**：21 CFR Part 11 §11.10 4 項硬要求皆可從程式碼 + audit log + cron 紀錄展示
- **任意資料變更可被稽核員從 audit log 完整還原**（before / after JSON + changed_fields + integrity_hash chain）
- **任意 audit 寫入失敗會 rollback 業務變更**（不再有「資料已變但 audit 沒寫」的 case）
- **HMAC chain 每天自動驗證**，斷鏈 24h 內告警
- **未來新增 mutation 自動沿用 SDD pattern**：CI grep guard 阻擋 fire-and-forget audit 寫法重新出現
- **單一 FullPlan.md 維持事實來源**：新人 / 半年後的我可以從本文件還原所有決策脈絡

如果回頭看 R26，最重要的學到：**SDD 的本質是「設計責任歸屬權」的搬移** — 不是寫法問題，是 audit 屬於 service 還是 handler 的歸屬問題。寫法跟著歸屬走，就自然單 tx + 完整 snapshot。

---

## 六、附錄：執行追蹤資料

### 16. 維護規則

- 新 PR 開出 → §10 步驟拆解新增 Step；§13 風險視需要新增；§14 後續若有新發現則加
- PR merged → 對應 Step 標 ✅；§11 驗證標準勾選對應項目
- 新決策 → §7 表格底部追加 D-NN（編號連續遞增、絕不重用、絕不刪除舊條目）
- Reviewer 新意見 → §13 風險表加列；若需獨立追蹤項則加 §14
- 進度數字（mutations migrated / deprecated warnings count）每週手動同步一次
- 本文件路徑固定 `docs/plans/R26_FullPlan.md`，不要分裂為多檔

### 17. PR Catalog（11 個 PR 概覽）

| PR | 對應 Step | 類型 | Mutations | 狀態 |
|----|-----------|------|-----------|------|
| [#153](https://github.com/delightening/ipig_system/pull/153) | Step 1 | INFRA | 0 | ✅ Merged |
| [#154](https://github.com/delightening/ipig_system/pull/154) | Step 1 | INFRA fix | 0 | ✅ Merged |
| [#155](https://github.com/delightening/ipig_system/pull/155) | Step 2 | Pattern Demo | 1 | ✅ Merged |
| [#156](https://github.com/delightening/ipig_system/pull/156) | Step 3 | animals simple | 17 | 🟡 Open |
| [#157](https://github.com/delightening/ipig_system/pull/157) | Step 3 | docs aftermath | - | 🟡 Open |
| [#158](https://github.com/delightening/ipig_system/pull/158) | Step 3 | HMAC verify cron | 0 | 🟡 Open（含 2 🔴 Critical 待修） |
| [#159](https://github.com/delightening/ipig_system/pull/159) | Step 3 | document | 10 | 🟡 Open |
| [#160](https://github.com/delightening/ipig_system/pull/160) | Step 3 | hr/leave | 7 + 7 helpers | 🟡 Open |
| [#161](https://github.com/delightening/ipig_system/pull/161) | Step 3 | hr/overtime+balance+attendance | 14 | 🟡 Open |
| [#162](https://github.com/delightening/ipig_system/pull/162) | Step 3 | user | 4 | 🟡 Open |
| [#163](https://github.com/delightening/ipig_system/pull/163) | Step 3 | docs 4-PR record | - | 🟡 Open |
| #165（本 PR）| 維護 | docs FullPlan | - | 🟡 Open |

### 18. Review feedback 響應矩陣

詳細的 review response（2🔴 / 23🟠 / 43🟡 / 1🔵）對應原 FullPlan.md v1.0 §5.2 / §5.3 表格內容；本版簡化僅以下 5 個 critical 風險入 §13。完整矩陣由 PR description + GitHub PR comments 互相對照即可重建，不重複維護於本文件。

---

## Pre-Execute Checklist

開始執行**下一個 Step**前逐項確認：

- [ ] 只看 §4 預期成果，能判斷整個 epic 成功與否？
  - ✅ DoD-1 ~ DoD-8 全部可機械式檢驗
- [ ] 執行到任一步驟失敗，能從前一步乾淨接回去？
  - ✅ §12 回滾策略對 Step 1-10 各有對策；nuclear option = 放棄 integration/r26
- [ ] §11 所有驗證項目加總，能覆蓋 §4 的所有成功標準？
  - ✅ Step 9 驗證 9.1/9.2 = DoD-1+DoD-4+DoD-5；Step 5 驗證 5.x = DoD-3 補強；其他類推
- [ ] §5 In scope / Out of scope 明確到不會爭議？
  - ✅ Out of scope 明列 8 個 WARN/SUGG + cross-service audit 延伸
- [ ] §6 Assumptions 若有一條不成立，知道要停下來重新評估？
  - ✅ A1（GLP 客戶存在）若不成立 → epic 動機消失，整個停做

---

## Post-Execute Review

epic 結束後（Step 10 完成）回填，作為下次 SDD 的經驗參考：

- **實際耗時 vs 預估**：（待填）原估 ~465h / 實際 ___h
- **哪些步驟比預期順利**：（待填）
- **哪些步驟比預期卡**：（待填）
- **Spec 有哪些漏洞是執行時才發現的**：（已知部分）
  - R26-3 scope 從 ~20 訂為 97（4.85×）— 初版 spec 沒有實測 grep
  - PR #156 R26-9/-10/-11 三個新追蹤項在 review 時才發現（CodeRabbit 抓到）
  - PR #158 verifier 與 writer NULL hash 分歧 — INFRA 階段沒驗證 verifier 路徑
- **下次類似任務的改進**：（待填）
  - 啟動前先做 grep-based scope inventory（不要相信原估）
  - 對於有「獨立 reimplementation」的設計（如 verifier 之於 writer），spec 要明列「兩端必須對等」的 invariant 並寫對等測試
  - 預期 4 個並行 PR 同日 review 的 capacity 上限（單人 ~4-6 PR/day）

---

**版本**：v2.0（2026-04-22 — SDD Template v1.0 重寫；前版 v1.0 為自由格式）
**維護者**：單人（Jason Wang）+ AI 協作（Claude Code + Gemini + CodeRabbit + Codex）
