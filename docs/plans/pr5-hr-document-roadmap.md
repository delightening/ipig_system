# PR #5 系列（HR + Document）— Service-driven Audit Migration 路線圖

- **基於**：`integration/r26` + PR #155 pattern + animals 系列完成（PR #4 a-e）
- **對應 R26 Section**：R26-3 的 hr / document 子集（與 animals 平行或後續）
- **產出**：HR（20 mutations）和 Document（10 mutations）的完整 Service-driven 遷移
- **總工時估計**：~170 person-hours（hr ~109h + document ~58h + 3 個 AuditRedact stub）

---

## 背景：兩模組的差異特性

| 維度 | HR | Document |
|---|---|---|
| 檔案數 | 5（leave / overtime / balance / attendance / dashboard） | 5（crud / workflow / grn / stocktake / mod） |
| Public mutation 數 | 20 | 10 |
| 目前 `pool.begin()` 覆蓋 | **0 個**（全 pool-level） | **8/10**（crud.rs / workflow.rs / grn.rs 多半已有 tx） |
| Audit 目前模式 | leave.rs / overtime.rs 用 `AuditService::log`（舊 API） | handler 層 `audit_document()` fire-and-forget |
| 跨 service 依賴 | 涉及 `annual_leave_entitlements` / `comp_time_balances` | 呼叫 `StockService` + `AccountingService`（**最複雜**）|
| AuditRedact 現況 | 5 entity 全缺 | 3 entity 全缺 |

---

## PR #5a — Document 模組（建議先做）

**為何先做 Document**：8/10 mutation 已有 tx，主要工作是「swap log_activity → log_activity_tx + 加 DataDiff」，不是從頭寫 tx。可驗證 log_activity_tx 在 cross-service（Stock / Accounting）情境下的正確性。

### Scope
- **Simple（1）**：`delete`（soft-delete）
- **Moderate（2）**：`create`、`update`
- **Complex（7）**：`submit`、`approve`（呼叫 StockService + AccountingService）、`admin_approve`、`admin_reject`、`cancel`、`grn::create_additional_grn`、`grn::recalculate_po_receipt_status`

### 關鍵設計問題（PR 開前須決策）

1. **Cross-service tx 語意**：`DocumentService::approve` 目前呼叫 `StockService::process_document(&tx, ...)` + `AccountingService::post_document(&tx, ...)`。這兩個 service **是否也已接受 `&mut tx`？** 若否，需同步為它們新增 tx 版本，否則 tx 無法跨 service 流動。
2. **Stock / Accounting 本身也要 audit**：若 `StockService::process_document` 產生的 stock ledger 變更也要走 `log_activity_tx`，則那兩個 service 也需要 actor + tx。這會把 PR #5a scope 擴張到 3 個 service 同時改動。
3. **建議**：PR #5a 只處理 DocumentService 的 audit，不擴張到 Stock / Accounting 本身的 audit；後者歸 PR #6 或 R26 繼任者。

### 工時估計
- Simple (1) + Moderate (2) + Complex (7 已有 tx, swap only): 2 + 6 + 50 = ~58h
- 加 AuditRedact stubs（Document / DocumentLine / PoReceiptStatus）：~0.5h

---

## PR #5b — HR Leave 子模組

**為何拆 leave 獨立**：leave.rs 820 行，7 mutations，涉及 `annual_leave_entitlements` / `leave_balance_usage` / `comp_time_balances` 3 個關聯表，且審核流程（submit/approve/reject/cancel）有複雜的 balance 扣除與還原邏輯。單獨成 PR 便於 review。

### Scope（7 mutations）
- Simple（3）：`create_leave` / `update_leave` / `delete_leave`
- Moderate（1）：`submit_leave`（狀態轉換 + balance 預扣）
- Complex（3）：`approve_leave`（多步審核 + balance 確認）、`reject_leave`（balance 還原）、`cancel_leave`（retroactive 還原）

### 關鍵設計問題
- **Balance 扣除的 audit**：每次 balance 變動（從 10 天 → 7 天）應產生 audit log。目前 leave_balance_usage 表已記錄細節；需決定：
  - (A) 讓 `annual_leave_entitlements` 的 UPDATE 產生 audit（balance 快照 before/after）
  - (B) 只對 `leave_requests` 的 status 轉換產生 audit，balance 視為衍生狀態
  - 建議 (A)，GLP 稽核員會要求看到「誰在何時把誰的假單扣了 3 天」

### 工時估計
- Simple (3) + Moderate (1) + Complex (3): 6 + 3 + 45 = ~54h
- AuditRedact stubs（LeaveRequest + AnnualLeaveEntitlement + LeaveBalanceUsage）：~0.5h

---

## PR #5c — HR Overtime + Balance + Attendance

### Scope（13 mutations）
- Simple: `create_overtime` / `update_overtime` / `delete_overtime` / `create_entitlement` / `adjust_balance` / `clock_in` / `clock_out` / `correct_attendance`（8 個）
- Moderate: `submit_overtime`、`approve_overtime`（含 comp_time 授予）（2 個）
- Complex: `reject_overtime`、`batch_auto_calculate`（3+ 表批次）、`batch workflows`（3 個）

### 工時估計
- Simple (8) + Moderate (2) + Complex (3): 16 + 6 + 45 = ~67h（比 leave 略高，因批次計算）
- AuditRedact stubs（OvertimeRecord / CompTimeBalance / AttendanceRecord）：~0.5h

### 決策點
- `batch_auto_calculate`（批次算年資時數）**每筆個別 audit** vs **單筆 batch summary audit**？建議 batch summary，但在 before_data 放 `{ "affected_users": [...], "dry_run": false }` 等後設資料。

---

## 總路線圖建議排程

```
PR #4a (animals simple)  ─┐
PR #4b (animals moderate) ─┤
PR #4c (animals transfer) ─┼─ 平行執行
PR #4d (animals import)   ─┤   animals 系列 ~200h
PR #4e (animals complex)  ─┘

            ↓

PR #5a (document, swap-in) ─ ~60h（最快落地，驗證 cross-service）

            ↓

PR #5b (hr leave)      ─ ~55h
PR #5c (hr overtime)   ─ ~70h

            ↓

PR #6 (剩餘小模組: product / sku / partner / warehouse / equipment / role / ai / auth / user)
  - 按 R26-3 audit call site inventory 的「非 animal 小計 48 處」消化
```

**總工時**：animals ~200h + hr ~125h + document ~60h + 剩餘 ~80h = **~465 person-hours**

這是 R26 epic 的真實規模（原估「R26-3 ~20 handler」嚴重低估；實際 97 call sites 跨 27 檔）。

---

## 關鍵決策（PR #5 啟動前需與使用者對齊）

1. **Stock / Accounting service 是否納入 audit refactor scope**？如是，PR #5a 會多 ~40h。
2. **Balance 變動 audit 精度**：annual_leave_entitlements 的每次 UPDATE 都 audit？還是只 audit leave_requests 的 status？
3. **Batch operations（batch_auto_calculate、import_export）audit 粒度**：per-row or summary？
4. **PR #5 是否可以與 PR #4 平行執行**？從依賴看可以（HR 和 Document 不依賴 animal service），但 review capacity 可能吃緊。

---

## 延伸記錄

- 完整 audit call site inventory → [memory: project_r26_audit_call_sites.md](~/.claude/projects/C--System-Coding-ipig-system/memory/project_r26_audit_call_sites.md)
- 此路線圖實際執行前需更新 `docs/TODO.md` R26-3 的「~20 處」說明為「~97 處 / 跨 27 檔」
- PR #155 pattern 定型於 → [memory: project_r26_service_driven_pattern.md](~/.claude/projects/C--System-Coding-ipig-system/memory/project_r26_service_driven_pattern.md)
