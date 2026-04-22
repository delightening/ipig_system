# PR #5a — Document 模組 Service-driven Audit Migration

- **基於**：PR #4a（#156）merged 後的 `integration/r26`
- **對應 R26 Section**：R26-3 的 document 子集
- **產出**：`backend/src/services/document/` 10 mutations 完整遷移至 Service-driven audit
- **工時估計**：~58 person-hours（比 animal 系列輕，因 8/10 已有 tx）
- **測試標準**（CLAUDE.md R26 執行紀律）：handler-touching → `cargo test --all-targets` 全綠

---

## 為何 Document 比 Animals 輕鬆

Animals 的 20+ simple mutation 全部從 `pool` 直接寫入 → 需要新增 `pool.begin()`。
Document 的 10 mutations 中 **8 個已有 `pool.begin()`**，只需：
1. Swap 舊 `AuditService::log_activity(&pool, ...)` → 新 `log_activity_tx(&mut tx, &actor, ActivityLogEntry::...)`
2. Wire `&ActorContext` 參數進 service signature（handler 層 wrap `CurrentUser`）
3. 補 `DataDiff::compute(Some(&before), Some(&after))`（需先 SELECT FOR UPDATE before）

剩下 2 個（`admin_reject` / `cancel`）需先確認目前 tx 狀態。

---

## Step 0 — 前置

- 確認 PR #156 已 merged：`git log --oneline origin/integration/r26 | head -3` 含 PR #4a
- 確認 CodeRabbit 對 PR #156 無殘留 🔴/🟠 issue
- `git checkout -b feat/document-service-driven-audit origin/integration/r26`

---

## Step 1 — AuditRedact empty impls（3 entities）

位置：`backend/src/models/document.rs`（若無則建立）

```rust
impl crate::models::audit_diff::AuditRedact for Document {}
impl crate::models::audit_diff::AuditRedact for DocumentLine {}
impl crate::models::audit_diff::AuditRedact for PoReceiptStatus {}
```

無敏感欄位。若 `Document` 含 `signed_by_cert` 類欄位（第三方簽章憑證），則改覆寫 `redacted_fields`。

**Commit 1**：`feat(models): AuditRedact empty impls for 3 document entities`

---

## Step 2 — 已有 tx 的 8 個 mutations（swap only）

**共用改寫模式**（已有 `pool.begin()`，只 swap audit + 補 DataDiff）：

```rust
// Before
pub async fn mutate(pool: &PgPool, /* args */, user_id: Uuid) -> Result<Entity> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query_as(...).fetch_one(&mut *tx).await?;
    // audit 目前可能在 handler 層 fire-and-forget 或直接 log_activity(&pool)
    tx.commit().await?;
    Ok(row)
}

// After
pub async fn mutate(pool: &PgPool, actor: &ActorContext, /* args */) -> Result<Entity> {
    let _user = actor.require_user()?;  // 或 actor_user_id().unwrap_or(SYSTEM_USER_ID)
    let mut tx = pool.begin().await?;
    let before: Option<Entity> = SELECT FOR UPDATE;  // 新增
    let after = sqlx::query_as(...).fetch_one(&mut *tx).await?;
    AuditService::log_activity_tx(&mut tx, actor, ActivityLogEntry::update(
        "ERP",
        "DOCUMENT_...",
        AuditEntity::new("document", after.id, &after.doc_no),
        DataDiff::compute(before.as_ref(), Some(&after)),
    )).await?;
    tx.commit().await?;
    Ok(after)
}
```

### Commit 2 — crud.rs::create / update / delete（3 mutations）

| 函數 | 現有 tx | action | 動作 |
|---|---|---|---|
| `DocumentService::create` (24:~75) | ✅ | `"DOCUMENT_CREATE"` | CREATE entry |
| `DocumentService::update` (222:~300) | ✅ | `"DOCUMENT_UPDATE"` | UPDATE entry（SELECT FOR UPDATE 取 before） |
| `DocumentService::delete` (344:~395, 8-table cascade) | ✅ | `"DOCUMENT_DELETE"` | DELETE entry（before = full doc） |

Handler：`backend/src/handlers/document.rs` 移除所有 `audit_document()` fire-and-forget，改傳 `ActorContext::User`。

### Commit 3 — workflow.rs::submit / approve（2 mutations，含 cross-service）

| 函數 | 現有 tx | action | 備註 |
|---|---|---|---|
| `DocumentService::submit` (47:~85) | ✅ | `"DOCUMENT_SUBMIT"` | simple（狀態轉換） |
| `DocumentService::approve` (184:~265) | ✅ | `"DOCUMENT_APPROVE"` | **呼叫 StockService::process_document + AccountingService::post_document（tx 已串接）**；本 PR 只 audit DocumentService 的狀態變更，Stock/Accounting 內部 audit 歸 PR #6 |

**決策點**：`approve` 的 audit 是否要記錄「連帶 stock/accounting 變動」的 summary？

⚠️ **基礎建設限制**：目前 `DataDiff`（`backend/src/models/audit_diff.rs`）的 `after` 直接由實體 `serde_json::to_value` 產生，沒有預留 meta 欄位；`ActivityLogEntry` 也只有 `before_data` / `after_data` / `request_context` / `changed_fields`，沒有獨立的 `metadata` slot。如要記錄「此 approve 連帶觸發 stock_processed / accounting_posted」的 summary，**只能在本 PR 執行前先決定一個路徑**：

- (a) 寫成獨立 audit event（例如 `DOCUMENT_APPROVE_SIDE_EFFECTS`），`after_data` 放 `{"stock_processed": true, "accounting_posted": true, "approved_doc_id": ...}`
- (b) 擴充 `ActivityLogEntry` 增加 `metadata: Option<serde_json::Value>` 欄位（跨模組影響，不建議在本 PR 做）
- (c) **本 PR 選擇略過**：stock ledger / GL entries 由那兩個 service 自行 audit 足矣（R26-3 延伸），不在 DocumentService 層記 side effect summary

**本 PR 採 (c)**：最小侵入，等 stock/accounting 的 Service-driven 遷移時自然有完整記錄。

### Commit 4 — workflow.rs::admin_approve / admin_reject / cancel（3 mutations）

| 函數 | 現有 tx | action |
|---|---|---|
| `DocumentService::admin_approve` (269:~335) | ✅ | `"DOCUMENT_ADMIN_APPROVE"` |
| `DocumentService::admin_reject` | **需確認** | `"DOCUMENT_ADMIN_REJECT"` |
| `DocumentService::cancel` | **需確認** | `"DOCUMENT_CANCEL"` |

admin_reject 和 cancel 需先 grep 確認是否已有 `pool.begin()`；若無則歸 Commit 5 一起處理。

### Commit 5 — grn.rs::create_additional_grn / recalculate_po_receipt_status（2 mutations）

| 函數 | 現有 tx | action |
|---|---|---|
| `DocumentService::create_additional_grn` (199:~265) | ✅ | `"DOCUMENT_GRN_CREATE"` |
| `DocumentService::recalculate_all_po_receipt_status` (377:~425) | ✅ | 批次 — 需決策 audit 粒度 |

**recalculate** 若受影響 N 個 PO，是：
- (A) N 筆 `"PO_RECEIPT_RECALC"` audit row（per-row）
- (B) 1 筆 batch summary audit with `data_diff.after = {"affected_po_count": N, "updated_at": ...}`

建議 (B)，因實際業務語意是一次性 reconciliation job。per-PO 細節可從 PO 自身 audit 歷史回查。

⚠️ **實作注意**：summary audit **不套用 `DataDiff::compute`**（該 helper 是為單一實體 before/after 比對設計的；對 batch job，before/after 是無意義的 entity snapshot）。改為直接建構 `serde_json::Value`：

```rust
let summary = serde_json::json!({
    "affected_po_count": n,
    "recalculated_at": Utc::now().to_rfc3339(),
});
AuditService::log_activity_tx(
    &mut tx,
    actor,
    ActivityLogEntry {
        category: "ERP",
        event_type: "DOCUMENT_PO_RECEIPT_RECALC",
        entity: None,
        data_diff: None,            // 不用 DataDiff
        changed_fields: vec![],
        request_context: Some(summary),
    },
).await?;
```

避免強套 `DataDiff::compute(Some(&()), Some(&summary_struct))` 之類的 hack。

---

## Step 3 — 無 tx 的 2 個 mutations（若存在）

從 C agent 回報，10 個 mutations 中 8 個有 tx。剩下 2 個若也是 complex 類，歸入 Commit 4 或 5 處理；若是 simple 類獨立成 Commit 6。

執行前 grep 確認：
```bash
rtk grep -n "pool.begin()" backend/src/services/document/*.rs
```

---

## Step 4 — 測試

- `cargo test --lib`（~2.85 sec）
- `cargo test --all-targets`（含整合測試，需本地 test DB）
- 特別檢查：
  1. 每個 document mutation 產生一筆 `user_activity_logs` row
  2. `approve` 的 audit row 與 stock/accounting 在同一 tx（非驗證順序）— ⚠️ **PostgreSQL 同一 tx 內所有 `NOW()` / `CURRENT_TIMESTAMP` 均回傳 tx 起始時間戳，因此 `created_at` **無法**區分同 tx 內的先後；且本專案 `id` 用 UUID v4（無時間排序）。若需要順序驗證，改用以下任一：
     - 檢查 `commit` 成功後兩個 log 都存在（原子性）
     - 使用 `statement_timestamp()` 或 `clock_timestamp()` 替代 `NOW()`（已非推薦，會破壞 advisory lock 假設）
     - 引入 UUID v7 或加顯式 `sequence` 欄位（非本 PR 範疇）
  3. Stock ledger 仍正確（未因 audit 改動而破壞現有 flow）

---

## Step 5 — Clippy + 格式

```bash
cargo clippy --all-targets -- -D warnings -A deprecated
cargo fmt --check
```

新遷移應減少 10 個 deprecated warning（97 → ~70）。

---

## Step 6 — PR 描述模板

- **標題**：`feat(document): Service-driven audit for 10 mutations`
- **要點**：
  - 3 AuditRedact empty impls
  - 10 mutations 完整 Service-driven（8 已有 tx，swap only；2 未確認）
  - 跨 service（StockService + AccountingService）tx 串接保留，本 PR 只 audit DocumentService 層
  - deprecated log_activity 減少 10 處
- **base**：`integration/r26`
- **CI**：需手動 `@coderabbitai review`（非 main base）

---

## 關鍵決策點（PR 前與使用者對齊）

1. **admin_reject / cancel 是否已有 tx？**：grep 確認後調整 commit 切分
2. **recalculate batch 粒度**：per-row vs summary（建議 summary）
3. **Stock / Accounting 內部 audit 是否納入本 PR？**：建議延後到 PR #6，保持本 PR scope 聚焦
4. **approve audit 如何記錄 cross-service 變動？**：建議 `data_diff.after` 加 meta，但不重複寫 stock/accounting 細節

---

## 風險

1. **document.rs 是核心業務邏輯**（含入庫、會計 posting），any bug 會導致庫存/帳本錯誤。建議本 PR 前先手動跑一次完整採購 → 入庫 → 付款流程（smoke test）
2. **現有 tx 與新 audit 互動**：若 `approve` 內的 `StockService::process_document(&mut tx, ...)` 在某路徑沒接 `&mut tx`，audit 可能和實際 stock 變動不同 tx（現有 code 若有此問題，本 PR 修掉）

---

## 後續 PR

- **PR #5b**：HR leave 子模組（~55h；complex 3 個狀態機）
- **PR #5c**：HR overtime + balance + attendance（~70h；13 mutations + batch）
- **PR #6**：user / product / sku / partner / warehouse / equipment / role / ai / auth / two_factor（~48 call sites，~80h）

合流後 R26-3 即可關閉（97 → 0 deprecated warnings）。
