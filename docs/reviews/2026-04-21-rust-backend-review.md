# Rust 後端架構審查報告

- **審查日期**：2026-04-21
- **審查範圍**：`backend/` (erp-backend v0.1.0, Rust 2021, axum 0.7 + tokio + sqlx 0.8)
- **審查模式**：Phase 1-4 全流程（架構 / 並發 / GLP 合規 / 長期維護）
- **程式碼規模**：單一 crate，`src/` 下約 230 個 `.rs` 檔案

---

## R26 修復進度（2026-04-21 更新）

本報告在同日啟動 **R26 Service-driven Audit 重構**，透過 3 個 PR 落地於長期整合分支 `integration/r26`（尚未合併回 main）：

| PR | 範圍 | 對應審查項目 | 狀態 |
|---|---|---|---|
| [#153](https://github.com/tanntwl/ipig_system/pull/153) | INFRA：`ActorContext` / `DataDiff`+`AuditRedact` / `log_activity_tx` / `CancellationToken` / Argon2 `spawn_blocking` / migrations 033-035 | CRIT-01(基礎) / CRIT-02(基礎) / CRIT-03(基礎) / CRIT-04 / WARN-02(Argon2) / WARN-04 | ✅ Merged |
| [#154](https://github.com/tanntwl/ipig_system/pull/154) | Review feedback：`HmacInput` struct、migration 036 `changed_fields` UNION 修正、HMAC error tracing、`ci.yml` 加 integration/**、Python shlex hook | 強化 CRIT-02 / CRIT-03 的正確性 | ✅ Merged |
| [#155](https://github.com/tanntwl/ipig_system/pull/155) | Pattern demo：`ProtocolService::submit` 完整 Service-driven 改寫、`generate_apig_no(&mut tx)` 加鎖 | CRIT-01(APIG) / CRIT-02(submit) / CRIT-03(submit) | 🟡 Open（CI pass 後合併） |

**R26 尚未落地的項目**（詳見 [docs/TODO.md](docs/TODO.md#r26)）：
- R26-1：長 scheduler job 升級 `tokio::select!` 接 cancellation（對應 WARN-04 延伸）
- R26-2：HMAC chain 每日驗證 cron（對應 SUGG-03）
- R26-3：~20 handler 遷移至 `log_activity_tx`（對應 CRIT-02 / WARN-06 收尾）
- R26-4：舊 `log_activity` 最終移除（Argon2 與 deprecated 下架）
- R26-6：HMAC 版本化 + 儲存後雜湊（Gemini G2 + CodeRabbit audit.rs:82）
- R26-7：services/mod.rs 11 處逐項 `#[allow(dead_code)]` cleanup
- R26-8：`change_status` 完整 Service-driven（submit pattern 複製到 animals/users/document 等）

**未啟動的審查項目**（WARN-01 / WARN-03 / WARN-05 / WARN-07 / WARN-08 / SUGG-01..05）維持在原建議優先級；下節內文於對應項目處加註 ✅ Landed / 🟡 Partial / ⏳ Pending 標記。

---

## 1. 架構總覽

### Workspace 結構

單一 crate（`erp-backend`），分層清楚：

```
backend/src/
├── main.rs          # 入口，初始化 AppState / Scheduler / 靜態服務
├── lib.rs           # AppState 定義（db / config / geoip / jwt_blacklist / templates / permission_cache）
├── config.rs        # 環境變數 → Config struct（含 read_secret 支援 *_FILE Docker Secrets）
├── constants.rs     # 全域常數
├── error.rs         # AppError enum + IntoResponse + sqlx code 分流
├── openapi.rs       # utoipa 集中註冊
├── bin/             # 14 個維運 CLI
├── handlers/        # HTTP 層（animal/auth/hr/protocol/signature 子目錄）
├── services/        # 業務邏輯（45+ 檔）
├── repositories/    # SQL 層（14 檔，大多僅數百 bytes）
├── models/          # DB entity + DTO
├── middleware/      # auth / csrf / rate_limiter / etag / ip_blocklist / jwt_blacklist
├── routes/          # 路由組裝
├── startup/         # database / migration / seed / permissions / server / tracing
└── utils/           # 4 個純函式檔
```

### 整體評價（3-5 行）

**整體品質中上、細節紮實，但分層執行不一致。** 錯誤處理 ([error.rs](backend/src/error.rs)) 是全專案最漂亮的部分 — 統一 `AppError`、`IntoResponse` 對 sqlx code 分流、PoolTimedOut 回 503 + Retry-After、完整測試。中介層設計精細（rate limiter 分 auth/write/upload/api 四檔、ETag、JWT 黑名單雙層記憶體+DB、IP blocklist 30s cache）。**主要架構缺陷集中在三點**：(1) Service 層直接寫 SQL (909 處) 架空了 repositories/ 分層，(2) 審計日誌散落於 handler 層且不在交易內，(3) 部分同步 I/O（Argon2、calamine、rust_xlsxwriter、std::fs）未包 spawn_blocking，高負載下會阻塞 tokio worker。

### 主要資料流

```
HTTP → ip_blocklist → etag → api_rate_limit →
       auth_rate_limit │ write_rate_limit │ upload_rate_limit →
       csrf (POST/PUT/DELETE) → auth (JWT) → handler →
       access::require_* (權限) → Service → sqlx::query(&pool) → AuditService::log_activity
```

Handler 規範良好（抽查 `handlers/animal/animal_core.rs`）：純解析 / 權限 / 轉 service / 回 JSON，**不寫 SQL**。但 service 層直接寫 SQL 是系統性破綻（見 §3 🟡 WARN-01）。

---

## 2. 🔴 Critical Issues

### CRIT-01｜IACUC 編號產生存在 race condition ✅ Partially Landed

> **PR #153 + PR #155**：已改為 advisory lock + tx 版本。`generate_apig_no(&mut tx)` 落地於 PR #155（key `protocol_iacuc_number_gen`）、`generate_iacuc_no` 與 `generate_apig_nos_batch` 已有對應 pool/tx 雙層 API。**剩餘**：現仍殘留的 `generate_iacuc_no_pool` 呼叫端遷移至 tx 版本歸 R26-8（`change_status` Service-driven）追蹤。

**位置**：[backend/src/services/protocol/numbering.rs:142-180](backend/src/services/protocol/numbering.rs:142)（`generate_iacuc_no`），相同模式亦見於 [numbering.rs:30](backend/src/services/protocol/numbering.rs:30)（`generate_apig_no`）與 [numbering.rs:91](backend/src/services/protocol/numbering.rs:91)（`generate_apig_nos_batch`）。

**問題程式碼**：
```rust
let iacuc_nos: Vec<String> = sqlx::query_scalar(
    "SELECT iacuc_no FROM protocols WHERE iacuc_no LIKE $1 AND iacuc_no IS NOT NULL",
)
.bind(format!("{}%", prefix))
.fetch_all(pool)  // ← 無 FOR UPDATE，無 transaction
.await?;

let max_seq = iacuc_nos.iter().filter_map(|no| parse_no_sequence(no, &prefix)).max();
let seq = max_seq.map(|s| s + 1).unwrap_or(1);
// …接著呼叫端 UPDATE protocols SET iacuc_no = $3
```

**成因**：查最大序號 → +1 → UPDATE 寫回，**三個步驟之間沒有鎖**。兩個同時發生的 `change_status` 呼叫（核准兩份計畫）會拿到相同的 `max_seq`，其中一筆 UPDATE 會因 `UNIQUE(iacuc_no)` 失敗，使用者看到 409 錯誤。GLP 情境下，編號的連續性與唯一性是稽核重點，此處的「失敗重試」語意並不明確，且目前呼叫端（`services/protocol/status.rs:385`）沒有重試邏輯。

**建議**：改用 PostgreSQL sequence 或 advisory lock，且包在 transaction：

```rust
pub(super) async fn generate_iacuc_no(tx: &mut Transaction<'_, Postgres>) -> Result<String> {
    // 選項 A：per-year sequence
    let seq: i64 = sqlx::query_scalar(
        "SELECT nextval('iacuc_seq_' || $1::text)"  // 每年一個 sequence
    ).bind(roc_year).fetch_one(&mut **tx).await?;

    // 選項 B：advisory lock + 現有查詢
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
        .bind(&prefix).execute(&mut **tx).await?;
    // …原本的 max+1 邏輯，但在同一 tx 內且 UPDATE 也在 tx 內
}
```

---

### CRIT-02｜多表變更未包 transaction，GLP 合規風險 🟡 Pattern Landed

> **PR #153 + PR #155**：`AuditService::log_activity_tx(tx, actor, entry)` 已提供 tx 版本（4 參數），內含 `pg_advisory_xact_lock('audit_log_chain')` 確保 HMAC chain 串行化。**Pattern demo**：`ProtocolService::submit` 已完整改寫為 Service-driven（`begin() → SELECT FOR UPDATE → INSERT/UPDATE → log_activity_tx → commit`）於 PR #155。**剩餘**：~20 個 handler 仍用舊 `log_activity`（已標記 `#[deprecated]`），遷移追蹤於 R26-3；`change_status` 完整改寫歸 R26-8。

**位置**：
- [backend/src/services/protocol/status.rs:104-176](backend/src/services/protocol/status.rs:104)（`submit`：INSERT protocol_versions → UPDATE protocols → record_status_change → log_activity）
- [backend/src/services/protocol/status.rs:390-405](backend/src/services/protocol/status.rs:390)（`change_status`：generate_iacuc_no → UPDATE protocols → record_status_change → log_activity）
- [backend/src/handlers/animal/animal_core.rs:193-215](backend/src/handlers/animal/animal_core.rs:193)（`create_animal`：AnimalService::create → AuditService::log_activity 分離）

**問題程式碼**（`submit` 精簡版）：
```rust
sqlx::query("INSERT INTO protocol_versions ...").execute(pool).await?;   // 步驟 1
let updated = sqlx::query_as("UPDATE protocols SET status = $2 ...").fetch_one(pool).await?;  // 步驟 2
Self::record_status_change(pool, id, ...).await?;                        // 步驟 3
AuditService::log_activity(pool, ...).await?;                            // 步驟 4
```

**成因**：任一步中間 panic / DB 斷線 / deploy 導致 process 被殺，都會留下不一致狀態（例如已進入 Submitted 狀態但沒有 protocol_versions 快照，或更新了狀態但 record_status_change 漏寫）。GLP 要求「資料變更必有完整稽核軌跡且可還原」，此設計違反此要求。

**全專案量化**：`pool.begin()` 僅出現於 [8 個檔案](backend/src/services/)（accounting、auth/login、storage_location、system_settings、document/crud、document/workflow、document/grn、animal/vet_patrol）。對 80+ 個 service 檔而言覆蓋率過低。

**建議**：重要的多表變更一律走 transaction：

```rust
pub async fn submit(pool: &PgPool, id: Uuid, submitted_by: Uuid) -> Result<Protocol> {
    let mut tx = pool.begin().await?;

    sqlx::query("INSERT INTO protocol_versions ...").execute(&mut *tx).await?;
    let updated = sqlx::query_as::<_, Protocol>("UPDATE protocols ... RETURNING *")
        .fetch_one(&mut *tx).await?;
    Self::record_status_change_tx(&mut tx, id, ...).await?;
    AuditService::log_activity_tx(&mut tx, ...).await?;

    tx.commit().await?;
    Ok(updated)
}
```

配合這個改動，`AuditService::log` / `log_activity` 需要新增接收 `&mut Transaction` 的版本。

---

### CRIT-03｜UPDATE 類稽核日誌無 before/after snapshot 🟡 Infrastructure Landed

> **PR #153 + PR #154**：`DataDiff` struct + `AuditRedact` trait 已落地（`backend/src/models/audit_diff.rs`），支援 length-prefix canonical encoding、JSON Pointer redact（如 `/password_hash`、`/sessions/0/token`）、自動計算 `changed_fields`。migration 035/036 擴充 `log_activity` stored proc 為 12 參數（`impersonated_by` + `changed_fields`）。**Pattern demo**：`ProtocolService::submit` 已示範 `DataDiff::compute(Some(&before), Some(&after))` 寫入前後快照於 PR #155。**剩餘**：`animals` / `users` / `document` 等核心 mutation 改用 `DataDiff` 歸 R26-3（隨 handler 遷移一併完成）。

**位置**：[backend/src/handlers/animal/animal_core.rs:278-294](backend/src/handlers/animal/animal_core.rs:278)（`update_animal` 的一般更新）以及 [animal_core.rs:195-215](backend/src/handlers/animal/animal_core.rs:195)（`create_animal` 的 before 為 None，after 僅部分欄位）。

**問題程式碼**：
```rust
// update_animal 的一般 UPDATE 分支
if let Err(e) = AuditService::log_activity(
    &state.db,
    current_user.id,
    "ANIMAL",
    "UPDATE",
    Some("animal"),
    Some(id),
    Some(&animal.ear_tag),
    None,   // ← before_data = None
    None,   // ← after_data = None
    None, None,
).await { tracing::error!(...); }
```

**成因**：GLP 要求對資料變更保留「變更前 vs 變更後」完整快照。目前只記「誰在何時對哪一筆做了 UPDATE」，稽核員無法從 `user_activity_logs` 還原變更內容。僅 IACUC 編號變更（[animal_core.rs:251-267](backend/src/handlers/animal/animal_core.rs:251)）有正確填 before/after。

**建議**：
- AnimalService::update 內先 `SELECT *` 原資料 → 執行 UPDATE → 用兩份資料計算 `changed_fields`，回傳 `(Animal, ChangedFields)`
- Handler 收到後呼叫 `log_activity` 傳入 before/after JSON。
- 更理想的做法是在 DB 用 trigger，對關鍵表（animals / protocols / protocol_versions / user_roles）自動寫 `user_activity_logs` — 但這會和現行 HMAC chain 設計衝突，需另外設計。

**嚴重性理由**：雖然資料沒遺失，但 GLP 稽核員要求「能從稽核軌跡還原任一時刻的資料狀態」，目前做不到 → Critical。

---

### CRIT-04｜services/mod.rs 違反 CLAUDE.md 規則 🟡 Crate-wide Landed

> **PR #153**：crate-wide `#![allow(dead_code)]` 已移除。為避免一次性破壞太多模組，暫保留 11 處 per-item `#[allow(dead_code)]` 並加註釋指向 R26-7。**剩餘**：11 處逐一 review 真偽、刪除或改 `pub use` 歸 R26-7。

**位置**：[backend/src/services/mod.rs:1](backend/src/services/mod.rs:1)

```rust
#![allow(dead_code)]
```

**成因**：CLAUDE.md 明確規定「禁止 `#[allow(dead_code)]`、`#[allow(unused)]`，未使用的程式碼直接刪除」。全專案僅此一處破例，等於對整個 services/ 樹解除了 dead code 告警，會積累隱性未使用程式碼。

**建議**：移除此 attribute，跑 `cargo clippy --all-targets -- -D warnings` 查出實際 dead code 並刪除，或改用 `pub use` 讓編譯器認得用途。

**嚴重性理由**：列為 Critical 是因它是明文規則違反而非效能/正確性問題，但修復成本低、後遺症高（未來新增死碼會看不到）。

---

## 3. 🟡 Warnings

### WARN-01｜Service 層直接寫 SQL，架空 repositories/ 分層

**量化證據**：全專案 `sqlx::query*` 共 1045 次
- `services/`：**909 次，80 個檔案**
- `repositories/`：**136 次，13 個檔案**

重災區：`services/equipment.rs` (55 SQL 呼叫, 75KB)、`services/facility.rs` (35)、`services/animal/blood_test.rs` (36)、`services/partner.rs` (17+calamine/xlsx)、`services/calendar.rs` (25)、`services/storage_location.rs` (25)、`services/sku.rs` (26)、`services/audit.rs` (31 — 合理，因為是 audit service 本體)。

相對地 `repositories/` 下絕大多數是薄殼，[repositories/user.rs](backend/src/repositories/user.rs) 只有 472 bytes、[role.rs](backend/src/repositories/role.rs) 470 bytes、[warehouse.rs](backend/src/repositories/warehouse.rs) 427 bytes — 幾乎不具實質意義。

**成因**：CLAUDE.md 要求「Service 層應呼叫 Repository 取得資料，而非直接寫 SQL」，實際上 service 層被當成 `service + repository` 合併層使用。對單人長期維護而言，這種合併並非災難，但會導致：
1. 相同 SQL 片段重複（如 `"SELECT * FROM animals WHERE id = $1"` 在多個 service 各自寫一次）— 欄位增減時要改多處
2. SQL 測試困難 — repository 層可用 sqlx::test 單獨跑，混在 service 裡要包整個 business flow
3. 新進開發者搞不清到底該加在哪

**建議**：採漸進式清理：
- **不要求全量遷移**（成本太高），但新程式碼一律寫在 repositories/
- 修改現有 service 時順手把其中 SQL 搬到對應 repository
- 先把前 5 大違規模組（equipment、facility、calendar、storage_location、sku）分批處理

**需討論**：若決定放棄 repositories/ 分層（完全接受 service + SQL 合併），則應修改 CLAUDE.md 並把薄殼 repositories 刪除；若要維持分層，則需投入重構時間。

---

### WARN-02｜CPU 密集/同步 I/O 未包 spawn_blocking，阻塞 tokio worker 🟡 Partial (Argon2)

> **PR #153**：Argon2 hash/verify 已改用 `tokio::task::spawn_blocking`（`services/auth/password.rs` + `auth/login.rs` + `auth/two_factor.rs`）。**剩餘**：calamine / rust_xlsxwriter / google_calendar credential 檔讀取，待匯入匯出或月報表出現阻塞實測後分批處理（維持 P1-3 優先級）。

**位置與類型**：

| 類型 | 檔案:行 | 影響 |
|---|---|---|
| Argon2 hash/verify | [backend/src/services/auth/password.rs:91](backend/src/services/auth/password.rs:91) / [:102](backend/src/services/auth/password.rs:102) / [auth/login.rs:124](backend/src/services/auth/login.rs:124) / [auth/two_factor.rs:300](backend/src/services/auth/two_factor.rs:300) / [:346](backend/src/services/auth/two_factor.rs:346) | CPU 密集 100-500ms，登入尖峰會造成 worker 排隊 |
| rust_xlsxwriter 輸出 | [services/hr/attendance.rs:128](backend/src/services/hr/attendance.rs:128) / [warehouse.rs:506](backend/src/services/warehouse.rs:506) / [partner.rs:733](backend/src/services/partner.rs:733) / [product/import.rs:285](backend/src/services/product/import.rs:285) / [animal/import_export.rs:542](backend/src/services/animal/import_export.rs:542) / [:605](backend/src/services/animal/import_export.rs:605) | 數十筆資料 ~100ms，上千筆可達 2-5 秒 |
| calamine Excel 讀入 | [services/warehouse.rs:433](backend/src/services/warehouse.rs:433) / [partner.rs:638](backend/src/services/partner.rs:638) / [product_parser.rs:327](backend/src/services/product_parser.rs:327) / [animal/import_export.rs:95](backend/src/services/animal/import_export.rs:95) / [:107](backend/src/services/animal/import_export.rs:107) | 匯入大檔會阻塞 |
| 同步檔案 I/O | [services/google_calendar.rs:477](backend/src/services/google_calendar.rs:477)（`std::fs::read_to_string` 讀 OAuth credentials）／ [services/pdf/service.rs:78](backend/src/services/pdf/service.rs:78)（讀字型檔）／ [services/animal/import_export.rs:641](backend/src/services/animal/import_export.rs:641)（讀 PDF 附件）／ [config.rs:38](backend/src/config.rs:38)（read_secret，啟動期一次，可忽略） | 中到低風險，依呼叫頻率 |

**全專案 grep `spawn_blocking` → 0 個結果。**

**成因**：tokio 預設 worker threads = CPU 核心數。一個 worker 跑 Argon2 的 200ms 期間，該 worker 上其他在 `.await` 的 futures 全部被延遲。對單人維護小流量系統影響有限，但以下情境會明顯感受：
- 登入尖峰（例如早上上班打卡集中登入 50 人）
- 批次匯入匯出（動物 / 採購 / SKU / 會計）
- 背景任務月報表、balance_expiration 同時執行

**建議（Argon2 範例）**：
```rust
pub async fn hash_password(password: String) -> Result<String> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))
    })
    .await
    .map_err(|e| AppError::Internal(format!("spawn_blocking joined error: {}", e)))?
}
```

**需討論**：若流量確認恆低（單位內部使用、同時線上人數 <20），此項可降為 🔵。請確認典型尖峰併發數。

---

### WARN-03｜clippy.toml 閾值遠寬鬆於 CLAUDE.md 規範

**位置**：[backend/clippy.toml](backend/clippy.toml)

```toml
too-many-lines-threshold = 200          # CLAUDE.md 規定 ≤50
too-many-arguments-threshold = 10       # CLAUDE.md 規定 ≤5
cognitive-complexity-threshold = 30     # CLAUDE.md 規定 圈複雜度 ≤10
```

**成因**：CLAUDE.md 明訂函數長度 ≤50 行、參數 ≤5 個、圈複雜度 ≤10，但 clippy 設定的閾值是 4 倍到 3 倍寬鬆。結果是 clippy 不會告警違反專案規則的程式碼。[audit.rs:172](backend/src/services/audit.rs:172) `log_activity` 有 10 個參數（本專案上限 5，clippy 通過）就是個例子。

**建議**：對齊兩者。如果真的需要長函式、多參數，在特定檔案用 `#[allow(clippy::too_many_arguments)]` 個別標註（需審查），而非全域放寬。

---

### WARN-04｜背景任務缺少 graceful shutdown 協調 🟡 Partial

> **PR #153**：`AppState` 加入 `shutdown_token: CancellationToken`，`main.rs` 關機流程改為 `cancel → join(jwt_cleanup, timeout=10s) → axum graceful shutdown`；`SchedulerService` 14 jobs 全面插入 `is_cancelled()` 檢查。**剩餘**：長跑 scheduler job 升級至 `tokio::select!` 精準中斷歸 R26-1；fire-and-forget `tokio::spawn(audit)` 的全量修正歸 R26-3（見 WARN-06）。

**位置**：
- [backend/src/main.rs:93](backend/src/main.rs:93)（`let _scheduler = ...`，無 shutdown 信號路徑）
- [backend/src/middleware/jwt_blacklist.rs:158-167](backend/src/middleware/jwt_blacklist.rs:158)（`start_cleanup_task` 無 cancellation）
- [backend/src/services/ip_blocklist.rs:83](backend/src/services/ip_blocklist.rs:83)（`spawn_record_hit`，fire-and-forget）
- 另有 40+ 個 `tokio::spawn` 散落在 handlers（見下方並發模型專項）

**問題**：`startup/server.rs:127-149` 的 `shutdown_signal()` 接到 Ctrl+C/SIGTERM 時，只會停止 axum 收新連線，in-flight request 會被等完。但：
1. `SchedulerService` 被 `let _scheduler` 綁住，main return 時直接 drop，正在執行的 cron job 可能被中斷
2. `JwtBlacklist::start_cleanup_task` 的 `loop { interval.tick().await; … }` 會在 tokio runtime shutdown 時被強制取消，若剛好在清理 DB 中途，INSERT/DELETE 可能斷在一半
3. Handler 內 `tokio::spawn` 做 audit / notification fire-and-forget（例如 [handlers/auth/login.rs:58](backend/src/handlers/auth/login.rs:58)、[handlers/document.rs:203](backend/src/handlers/document.rs:203)），若 shutdown 時還沒完成就會被丟棄 → **可能遺失 audit log**

**建議**：
- 在 `AppState` 加一個 `tokio_util::sync::CancellationToken` 或 `tokio::sync::watch::Sender<bool>`
- 所有背景任務用 `tokio::select! { _ = token.cancelled() => break, _ = work => {} }` 接取消訊號
- `shutdown_signal()` 觸發後先 cancel token，等背景任務 join 完再關 server
- 對 fire-and-forget audit，改用 channel + background worker 而非 spawn — 或 handler 內直接 await 完 audit 再回傳 response（寧可慢一點也不要掉 log）

---

### WARN-05｜緩存穿透（cache stampede）風險

**位置**：
- [backend/src/services/alert_threshold.rs:22-56](backend/src/services/alert_threshold.rs:22)（60s cache）
- [backend/src/services/security_notifier.rs:77-113](backend/src/services/security_notifier.rs:77)（channels cache）
- [backend/src/services/ip_blocklist.rs:48-72](backend/src/services/ip_blocklist.rs:48)（30s cache）

**問題**：三個快取都是相同模式：`lock → read; if expired → release lock → query DB → lock → write`。若 cache 過期瞬間有 N 個併發請求，每個都會發現 cache 過期，N 個同時查 DB。雖然本專案流量不算高，但 IP blocklist 是 middleware 高頻路徑（每個 request 跑一次）。

**建議**：改用 `tokio::sync::OnceCell` + `get_or_init` / moka / 或者 singleflight 模式。簡化版：

```rust
static CACHE: LazyLock<Mutex<(Option<Vec<_>>, Instant)>> = ...;
static REFRESH_LOCK: LazyLock<Mutex<()>> = ...;  // 只允許一人查 DB

async fn load() -> Vec<_> {
    // 先試 cache
    // 過期時：先拿 REFRESH_LOCK，再查 CACHE 一次（雙重檢查）— 若別人已刷新就直接用
    // 否則自己查 DB 並寫回
}
```

**需討論**：若 DB 負載不是瓶頸，此項可延後處理。

---

### WARN-06｜audit 使用 `tokio::spawn` fire-and-forget，可能遺失日誌 🟡 Pattern Available

> **PR #153 + PR #155**：`log_activity_tx` 提供「同 tx 內同步寫入 audit」的正確模式，Protocol submit 已採用。**剩餘**：~20 個 handler 的 `tokio::spawn(async { log_activity })` 改為同步 `log_activity_tx` 歸 R26-3（隨 mutation Service-driven 遷移）。

**位置**：散見於 handlers，例如：
- [backend/src/handlers/auth/login.rs:58](backend/src/handlers/auth/login.rs:58)、[:114](backend/src/handlers/auth/login.rs:114)
- [backend/src/handlers/auth/password.rs:58](backend/src/handlers/auth/password.rs:58)、[:106](backend/src/handlers/auth/password.rs:106)
- [backend/src/handlers/amendment.rs:174](backend/src/handlers/amendment.rs:174)、[:234](backend/src/handlers/amendment.rs:234)、[:329](backend/src/handlers/amendment.rs:329)
- [backend/src/handlers/document.rs](backend/src/handlers/document.rs):203/267/327/390/454（5 處）
- [backend/src/handlers/user.rs](backend/src/handlers/user.rs):77/207/225/377/446（5 處）

**問題**：大量 handler 用 `tokio::spawn(async move { AuditService::log_activity(...).await; })` 讓稽核寫入在後台進行，以加速 response。但：
1. spawn 的 task 在 server shutdown 時被丟棄 → 遺失 audit（結合 WARN-04）
2. spawn 失敗（task panic、DB error）只有 `tracing::error!` 記錄，稽核員查不到
3. GLP 要求稽核寫入可靠性，fire-and-forget 不符合「可靠寫入」

**建議**：
- 關鍵稽核（登入、權限變更、資料 CRUD）**同步寫入** — 多出的 5-10ms 延遲換取可靠性
- 非關鍵 telemetry（例如 user agent 分析）才用 spawn
- 若仍要非同步，改用 MPSC channel → background worker 流程，worker 有 buffer + retry + drain-on-shutdown

---

### WARN-07｜equipment.rs 單檔 75KB、service 檔多數破 20KB

**位置**：
- [backend/src/services/equipment.rs](backend/src/services/equipment.rs)（75.5KB）
- [services/scheduler.rs](backend/src/services/scheduler.rs)（39.6KB，14 個 cron job 混在一檔）
- [services/partner.rs](backend/src/services/partner.rs)（32.5KB）
- [services/data_import.rs](backend/src/services/data_import.rs)（32.0KB）
- [services/sku.rs](backend/src/services/sku.rs)（31.1KB）
- [services/audit.rs](backend/src/services/audit.rs)（29.7KB）
- [services/report.rs](backend/src/services/report.rs)（26.1KB）
- [services/calendar.rs](backend/src/services/calendar.rs)（25.3KB）
- [repositories/glp_compliance.rs](backend/src/repositories/glp_compliance.rs)（30.9KB）

**問題**：CLAUDE.md 要求「每個檔案只做一件事。超過 300 行應考慮拆分」。equipment.rs 至少 2000+ 行（75KB ÷ ~40 bytes/行）。單一大檔的成本：
- Rust incremental compile 的最小單位是 crate + module；單檔變動必須重編整個檔
- Code review 時難以掌握
- 單人維護久了會產生「這裡應該有邏輯但找不到在哪」

**建議**：已經有 [animal/](backend/src/services/animal/)、[protocol/](backend/src/services/protocol/)、[auth/](backend/src/services/auth/)、[hr/](backend/src/services/hr/)、[product/](backend/src/services/product/)、[document/](backend/src/services/document/)、[stock/](backend/src/services/stock/)、[signature/](backend/src/services/signature/)、[amendment/](backend/src/services/amendment/)、[notification/](backend/src/services/notification/)、[mcp/](backend/src/services/mcp/) 等子目錄，照相同模式把 equipment / scheduler / partner / sku / calendar / audit 拆成子目錄。scheduler.rs 最好拆：每個 cron job 一個檔，`scheduler/mod.rs` 只做 JobScheduler 組裝。

---

### WARN-08｜權限字串硬編碼散落在 handler

**位置**：全專案 `has_permission("xxx.yyy.zzz")` 與 `require_permission!` 的字面值。範例：
- [handlers/animal/animal_core.rs:38](backend/src/handlers/animal/animal_core.rs:38): `"animal.animal.view_all"`, `"animal.animal.view_project"`
- [handlers/animal/animal_core.rs:114](backend/src/handlers/animal/animal_core.rs:114): `"animal.animal.view_all"`（重複）
- [services/access.rs:30](backend/src/services/access.rs:30): VIEW_ALL_ROLES 用了 constants 但 [services/access.rs:157](backend/src/services/access.rs:157) `has_permission("aup.protocol.view_all")` 又硬編碼

**問題**：改權限名稱時必須跨 handler / service 全專案搜尋取代，容易漏；打錯字時編譯器不會警告。CLAUDE.md「魔術字串必須定義為 const 或 enum」— 此處違反。

**建議**：產生 `permissions.rs`（已在 [startup/permissions.rs](backend/src/startup/permissions.rs) 30.7KB 有定義資料）對應的 `pub const PERM_ANIMAL_VIEW_ALL: &str = "animal.animal.view_all";` 常數；或更好的做法是用 macro + enum 生成。

---

## 4. 🔵 Suggestions

### SUGG-01｜DB pool 預設值適合小到中流量，高併發時需調校

- [backend/src/config.rs:159-170](backend/src/config.rs:159): `DATABASE_MAX_CONNECTIONS=40`、`min=5`、`acquire_timeout=30s`、`statement_timeout=30s`
- 與 tokio worker 數（預設 = CPU 核心）的比例一般建議是 worker × 2~4。若在 8 核機器上，40 是合理的；若 4 核機器上偏多，可能會造成 DB 端連線壓力
- `statement_timeout=30s` 對匯入匯出（大 INSERT）可能嫌短，特定操作可在 SQL 前跑 `SET LOCAL statement_timeout` 調整

**建議**：加一份 `docs/runbook/database-tuning.md`，記錄不同規模建議值與理由。

---

### SUGG-02｜考慮拆 workspace，改善編譯時間

目前單一 crate 230+ 檔，incremental rebuild 估計 10-30 秒。拆成：

```
backend/
├── Cargo.toml (workspace)
├── crates/
│   ├── erp-core/           # AppError, Config, Constants, models (零業務依賴)
│   ├── erp-db/             # sqlx + repositories + migrations
│   ├── erp-services/       # business logic
│   ├── erp-http/           # axum handlers + routes + middleware
│   └── erp-bin/            # main + bin/
```

修 handler 不用重編 services，修一個 service 不用重編 http 層。但要注意 utoipa 的 derive 需要在同一 crate 看到 schema — 可能需要把 model 放 core 讓大家共用。

**需討論**：單人維護的專案拆 workspace 有時會因「為了模組化而增加 Cargo.toml 維護」得不償失。建議若一週編譯時間 > 2h 才考慮拆。

---

### SUGG-03｜audit HMAC chain 的 tamper 檢測未自動化

[services/audit.rs:202-215](backend/src/services/audit.rs:202) 有 `compute_and_store_hmac`，但沒看到定期驗證 chain 完整性的 job。GLP 稽核時，若有人繞過 trigger 直接改 DB，HMAC chain 會斷 — 但得有人主動驗證才發現。

**建議**：加一個每日 cron，對當天寫入的 user_activity_logs 驗證 HMAC chain，斷鏈時發 SecurityNotifier。

---

### SUGG-04｜`database_statement_timeout_ms` 用 format! 拼接 SQL

[backend/src/startup/database.rs:99-105](backend/src/startup/database.rs:99):

```rust
sqlx::query(&format!("SET statement_timeout = {statement_timeout_ms}"))
```

值來自 config（u64），非使用者輸入 → 實務安全。但 sqlx-macro 檢查器不會驗證這段。建議改用 sqlx 的整數 parameter 或 `SET LOCAL` 配合 bind：

```rust
sqlx::query("SET statement_timeout = $1::int").bind(statement_timeout_ms as i64)
```

不過 Postgres `SET` 命令不接受 parameter，所以這個只能靠 code review 保證。保持現狀但加註解說明為什麼是安全的即可。

---

### SUGG-05｜test fixture 缺少 parallel 保護

[backend/tests/common/mod.rs](backend/tests/common/mod.rs) 每個 TestApp 共享同一個 DATABASE_URL + env var 設定。`serial_test` 已在 dev-dependency，但沒見到 `#[serial]` 標註。若真的 run 在一起，env var 會互相覆蓋。

**建議**：整合測試檔頭加 `#[serial]`，或用 testcontainers 每次起獨立 DB。

---

## 5. 並發模型專項評估

### 做得好的部分

1. **`reqwest::Client` 全數共用 + 有 timeout**（[pdf_service_client.rs:21-31](backend/src/services/pdf_service_client.rs:21)、[security_notifier.rs:17](backend/src/services/security_notifier.rs:17) 的 `LazyLock<reqwest::Client>`、`GotenbergClient` 也走相同 pattern）— TCP 連線池正確重用。
2. **std::sync::Mutex 在 cache 路徑的使用都正確**（[alert_threshold.rs:22](backend/src/services/alert_threshold.rs:22)、[security_notifier.rs:78](backend/src/services/security_notifier.rs:78)、[ip_blocklist.rs:50](backend/src/services/ip_blocklist.rs:50)）— guard 都在 await 前 drop，比 tokio::sync::Mutex 更輕量。
3. **JwtBlacklist 雙層架構**（[middleware/jwt_blacklist.rs:20](backend/src/middleware/jwt_blacklist.rs:20)）用 `Arc<RwLock<HashMap>>` 讓 is_revoked 可並發讀，寫入才互斥；RwLock 中毒時 fail-closed 拒絕 token — 正確的安全設計。
4. **適切的 `tokio::try_join!`**（[services/protocol/numbering.rs:104-115](backend/src/services/protocol/numbering.rs:104) 把 apig_nos / iacuc_nos 兩個獨立查詢並行）— 有看到正確利用並發。
5. **Graceful shutdown 對 axum 層設計正確**（[startup/server.rs:127-149](backend/src/startup/server.rs:127)）監聽 ctrl_c + SIGTERM 觸發 `with_graceful_shutdown`。
6. **tokio-cron-scheduler 包裝背景任務**（[services/scheduler.rs](backend/src/services/scheduler.rs) 14 個 job）而非自己寫 interval 迴圈 — 正確選擇。

### 可改善

（歸納自 WARN-02 / WARN-04 / WARN-06，不重複）

- **沒有任何 spawn_blocking**：Argon2 / Excel / 同步 fs 都在 async context 直接跑（🟡 WARN-02）
- **背景任務沒接 cancellation**：shutdown 時可能丟失 audit / notification（🟡 WARN-04）
- **fire-and-forget audit**：~20 個 `tokio::spawn(async { audit... })` 點（🟡 WARN-06）

### 並發並未浪費的證據

- Protocol 狀態轉換的權限檢查、PI 關聯、reviewer 關聯等多個 boolean 查詢 — [access.rs:164-196](backend/src/services/access.rs:164) 把原本 3 次查詢合併為單一 4-way UNION（程式碼註解明確記 HIGH-03）— 展現正向最佳化的思維
- Numbering 的 try_join! 如前述

這個專案並不是「全部序列執行」那種反模式，問題集中在**阻塞 I/O 的包裝**與**背景任務的生命週期管理**。

---

## 6. 優先修復順序

### P0（本月內）— 狀態更新於 2026-04-21

| # | 項目 | 對照 | 工時估計 | 狀態 |
|---|---|---|---|---|
| P0-1 | IACUC / APIG 編號產生加鎖 + transaction | CRIT-01 | 0.5 day | ✅ PR #153/155（剩 change_status 呼叫端 → R26-8） |
| P0-2 | 移除 `services/mod.rs:1` 的 `#![allow(dead_code)]` 並清乾淨 | CRIT-04 | 0.5 day | 🟡 crate 指令已移除（PR #153），剩 11 處 per-item → R26-7 |
| P0-3 | Protocol submit / change_status 包 transaction（含 audit） | CRIT-02 | 1 day | 🟡 submit 完成（PR #155）；change_status → R26-8 |
| P0-4 | Argon2 hash/verify 包 spawn_blocking | WARN-02 局部 | 0.5 day | ✅ PR #153 |

### P1（本季內）— 狀態更新於 2026-04-21

| # | 項目 | 對照 | 工時估計 | 狀態 |
|---|---|---|---|---|
| P1-1 | UPDATE 類操作補 before/after snapshot（從 animal + protocol 先開始） | CRIT-03 | 2-3 days | 🟡 基礎設施完成，遷移歸 R26-3 |
| P1-2 | 背景任務接 CancellationToken，保證 shutdown 不遺失 audit | WARN-04 + WARN-06 | 1-2 days | 🟡 CancellationToken 落地（PR #153），audit 同步化歸 R26-3 |
| P1-3 | Excel / calamine / PDF 同步呼叫包 spawn_blocking | WARN-02 | 1-2 days | ⏳ Pending |
| P1-4 | 動物 / 計畫等核心 mutation 類 API 全面檢視 transaction 邊界 | CRIT-02 延伸 | 3-5 days | 🟡 Pattern 就緒，R26-3（animals/users/document）陸續遷移 |
| P1-5 | clippy.toml 閾值對齊 CLAUDE.md，補 allow 標註個案 | WARN-03 | 1-2 days | ⏳ Pending |

### P2（半年內）— 狀態更新於 2026-04-21

| # | 項目 | 對照 | 工時估計 | 狀態 |
|---|---|---|---|---|
| P2-1 | 拆 scheduler.rs 為子目錄（每 job 一檔） | WARN-07 | 0.5 day | ⏳ Pending |
| P2-2 | 拆 equipment.rs / sku.rs / partner.rs 大檔 | WARN-07 | 2-3 days | ⏳ Pending |
| P2-3 | 權限字串改 constants 或 enum | WARN-08 | 1 day | ⏳ Pending |
| P2-4 | 新程式碼一律寫 repositories/，既有 SQL 漸進遷移 | WARN-01 | 持續 | 🟢 On-going policy |
| P2-5 | HMAC chain 每日驗證 job | SUGG-03 | 0.5 day | ⏳ R26-2 |
| P2-6 | 快取改 singleflight | WARN-05 | 1 day | ⏳ Pending |
| P2-7 | HMAC 版本化 + 儲存後雜湊 | SUGG-03 延伸 / Gemini G2 | 1-2 days | ⏳ R26-6 |

### P3（視需要）

- Workspace 拆分（SUGG-02，僅在編譯時間成為痛點時）
- tests 加 `#[serial]`（SUGG-05）
- DB tuning runbook（SUGG-01）

---

## 附錄：審查時未深入但值得後續關注

以下是本次審查中有發現但沒有深入的項目，供後續追查：

1. **[handlers/hr/leave.rs](backend/src/handlers/hr/leave.rs) 的 33 個 sqlx::query** — hr 的請假邏輯是常見 business logic 黑洞，值得專項 review
2. **[services/equipment.rs](backend/src/services/equipment.rs) 75KB 中的 55 個 sqlx** — 設備管理是完全沒拆的單檔巨獸
3. **[middleware/rate_limiter.rs](backend/src/middleware/rate_limiter.rs) 17KB + dashmap** — 沒看實作但 dashmap 用在 rate limit 的邊界條件（清理 expired key 的頻率）值得檢查
4. **[middleware/csrf.rs](backend/src/middleware/csrf.rs) 13.8KB** — CSRF 的 double-submit token 與 SameSite 設定的互動
5. **migrations/ 有 30+ 個 .sql**，漸進式 schema 沒 squash — 新環境初始化跑全部 migration 的時間
6. **`.sqlx/` 有 54 個 offline query cache 檔** — sqlx macro 離線模式有用，但若 master DB schema 漂移 CI 會炸

---

## 總結

本系統的**錯誤處理、中介層分層、安全設計**（JWT ES256、HMAC audit chain、IP blocklist、rate limiter、CSP）明顯高於一般 Rust 後端的平均水準，顯示開發者對生產環境的風險意識充足。主要弱點集中在**資料一致性保證**（transaction 使用不足 + audit 散落）與**並發 runtime 禮貌性**（沒有 spawn_blocking）。這兩類問題都是「現階段能運作但未來會出事」的類型，建議依 P0/P1 順序在本季內修復。

對單人長期維護而言，**最重要的三件事**：
1. CRIT-01 / CRIT-02 — 這是 GLP 稽核最可能被挑戰的地方
2. CRIT-04 + WARN-01 — 把 `#[allow(dead_code)]` 移掉後，會暴露多少未用碼是個信號 — 若很多，代表 SQL 散在 service 層已經造成重複程式碼
3. WARN-02（Argon2）— 登入這條路是最多人同時用的路徑，是效能退化最容易被感受到的地方

---

**報告結束**
