# iPig 系統修復實作計畫書（v2 — 已修復項目更新）

**基於三層 Code Review 發現** | **日期**：2026-03-15（v2 更新）
**倉庫**：https://github.com/delightening/ipig_system
**執行者**：Claude Opus 4.6 + 開發者協作

---

## 已確認修復（無需處理）

| 原始項目 | 修復方式 | 節省 Token |
|----------|----------|-----------|
| ✅ L2-Critical #1：IDOR 防護 | `check_attachment_permission` 已存在 | ~40,000 |
| ✅ L2-Suggestion #2：DB 錯誤碼分類 | `error.rs` 已正確回傳 409/400 | ~8,000 |
| ✅ L2-Suggestion #1：Upload Handler 去重 | `handle_upload` 通用函式已存在 | ~18,000 |
| **合計節省** | | **~66,000** |

---

## 計畫總覽（更新後）

| 階段 | 名稱 | 涉及檔案數 | 預估 Token | 預估耗時 | 風險等級 |
|------|------|-----------|------------|----------|----------|
| Phase 1 | 倉庫衛生清理 | 2-3 | ~15,000 | 10 分鐘 | 低 |
| Phase 2 | 資料庫效能優化 | 3-4 | ~80,000 | 40 分鐘 | **高** |
| Phase 3 | 安全與 API 修正 | 3-4 | ~40,000 | 20 分鐘 | 中 |
| Phase 4 | 程式碼重構 | 3-4 | ~45,000 | 25 分鐘 | 中 |
| Phase 5 | 前端架構強化 | 2-3 | ~30,000 | 15 分鐘 | 低 |
| Phase 6 | CI/CD 與基礎設施 | 2-3 | ~20,000 | 10 分鐘 | 低 |
| **合計** | | **~15-21 檔案** | **~230,000** | **~2 小時** | |

> 相比 v1 計畫，移除 3 個已修復項目後節省 ~66,000 tokens（~75,000 含緩衝），總預算降低 22%。

---

## Phase 1：倉庫衛生清理

**對應 Review 項目**：L1-Critical #3-4, L3-Suggestion #9
**風險**：低（不影響功能）
**前置條件**：無

### 待辦清單

- [ ] 1.1 將以下檔案加入 `.gitignore` 並從追蹤移除：
  - `backend/build_final.txt`、`backend/build_output.txt`、`backend/ci-fail.txt`
  - `backend/error.log`、`backend/error.txt`、`backend/test_output.txt`
  - `last_error.json`、`test-login.json`
  - `product_import_from_stocklist.csv`、`匯入用.csv`
- [ ] 1.2 驗證 `.gitignore` 規則完整性

### Token 預估：**~15,000**

### Rollback

```bash
git revert <commit-hash>  # 純 .gitignore 變更，安全 revert
```

---

## Phase 2：資料庫效能優化（最高優先級）

**對應 Review 項目**：L3-Critical #1-2, L3-Suggestion #1-3
**風險**：**高**（修改核心庫存邏輯和 DB 視圖）
**前置條件**：Phase 1 完成、開發環境可測試庫存流程

### 待辦清單

#### 2A — 新增索引（零風險，立即執行）

- [ ] 2A.1 建立新 migration `012_stock_performance.sql`
- [ ] 2A.2 加入以下索引：
  ```sql
  CREATE INDEX CONCURRENTLY idx_stock_ledger_wh_prod_dir
    ON stock_ledger(warehouse_id, product_id, direction);
  CREATE INDEX CONCURRENTLY idx_stock_ledger_wh_prod_date
    ON stock_ledger(warehouse_id, product_id, trx_date DESC);
  CREATE INDEX CONCURRENTLY idx_stock_ledger_doc_id
    ON stock_ledger(doc_id);
  CREATE INDEX CONCURRENTLY idx_document_lines_product_id
    ON document_lines(product_id);
  ```
- [ ] 2A.3 驗證 `EXPLAIN ANALYZE` 確認索引被使用

#### 2B — check_stock_available 切換到 snapshot

- [ ] 2B.1 讀取 `services/stock.rs` 完整原始碼
- [ ] 2B.2 修改 `check_stock_available()` 改查 `inventory_snapshots`
- [ ] 2B.3 確認 `inventory_snapshots` 的更新機制存在且正確（核准單據時應同步更新 snapshot）
- [ ] 2B.4 若 snapshot 更新機制缺失，需在 `DocumentService::approve` 中加入 snapshot upsert 邏輯

#### 2C — 重寫 SQL 視圖

- [ ] 2C.1 重寫 `v_low_stock_alerts`：移除 CROSS JOIN，改用 `inventory_snapshots JOIN`
- [ ] 2C.2 重寫 `v_inventory_summary`：同上
- [ ] 2C.3 修改 `get_on_hand()`：查詢 `inventory_snapshots` 而非聚合 `stock_ledger`
- [ ] 2C.4 修改 `get_low_stock_alerts()`：使用重寫後的視圖或直接查 snapshot

#### 2D — get_ledger 分頁

- [ ] 2D.1 為 `get_ledger()` 加入 `offset` + `limit` 參數
- [ ] 2D.2 回傳總筆數供前端分頁 UI

### Token 預估：**~80,000**

| 步驟 | Token |
|------|-------|
| 讀 stock.rs + 007_audit_erp.sql + document service | 22,000 |
| 建立 012 migration + 修改查詢 + 重寫視圖 | 26,000 |
| 加入 snapshot upsert（如缺失） | 12,000 |
| EXPLAIN ANALYZE 驗證 | 3,000 |
| 瀏覽器操作開銷 | 17,000 |

### Deploy 檢查點

- [ ] `inventory_snapshots` 有正確的初始資料（可手動 backfill 一次）
- [ ] 核准單據後 snapshot 同步更新
- [ ] 庫存頁面載入時間 < 500ms（目前可能 > 2s）
- [ ] 低庫存警告正常觸發
- [ ] 帳本查詢支援翻頁

### Rollback 觸發條件

- 庫存數量計算與帳本歷史不一致
- stock_ledger 與 inventory_snapshots 資料不同步
- 核准單據後庫存數字未更新

---

## Phase 3：安全與 API 修正

**對應 Review 項目**：L1-Suggestion #7, L2-Suggestion #4-5, L2-Suggestion #8
**風險**：中
**前置條件**：無

> ~~原 Phase 3 的 DB 錯誤碼修正（L2-Suggestion #2）已確認修復，移除。~~

### 待辦清單

- [ ] 3.1 修改 `handlers/auth.rs` → `forgot_password`：加入固定延遲（~200ms）防 timing attack
- [ ] 3.2 修改 `handlers/user.rs` → `create_user`：歡迎郵件改為「重設密碼連結」而非明文密碼
  - 需要讀取 email service 模板
  - 需要修改 `send_welcome_email` 參數
- [ ] 3.3 修改 `lib/api/client.ts`：移除 `deleteResource` 的 POST workaround，改用 DELETE
  - 需先確認 Nginx 已支援 DELETE 方法
- [ ] 3.4 修改 `lib/api/client.ts`：移除 refresh handler 中不必要的動態 `await import()`

### Token 預估：**~40,000**

| 步驟 | Token |
|------|-------|
| 讀 auth.rs + 修改 timing | 7,000 |
| 讀 email service + user handler + 修改歡迎郵件 | 14,000 |
| 讀 + 修改 client.ts（DELETE + dynamic import） | 8,000 |
| 瀏覽器操作開銷 | 11,000 |

### Deploy 檢查點

- [ ] 新建使用者收到的郵件不含明文密碼
- [ ] DELETE API 請求正常通過 Nginx
- [ ] forgot_password 回應時間對存在/不存在帳號一致

---

## Phase 4：程式碼重構

**對應 Review 項目**：L3-Suggestion #3-4, #10
**風險**：中
**前置條件**：Phase 2 完成（Stock SQL 去重依賴 snapshot 切換）

> ~~原 Phase 4A Upload Handler 重構（L2-Suggestion #1）已確認修復，移除。~~

### 待辦清單

#### 4A — 審計日誌抽取

- [ ] 4A.1 建立 `fn audit_document(db, user_id, event, doc_id, doc_no, extra)` 通用函式
- [ ] 4A.2 替換 document.rs 中 6 處重複的 `AuditService::log_activity` 呼叫
- [ ] 4A.3 考慮是否用 Rust macro 進一步簡化

#### 4B — Stock SQL 去重

- [ ] 4B.1 若 Phase 2 已切換到 inventory_snapshots，確認重複 SQL 已自動消除
- [ ] 4B.2 確認 `get_on_hand` 和 `get_low_stock_alerts` 不再有重複 SQL

#### 4C — Handler/Service 職責統一

- [ ] 4C.1 審查所有 handler，確認模式一致：handler → 認證/授權/HTTP、service → 業務+審計
- [ ] 4C.2 將 `check_document_access` 從 handler 移入 DocumentService

### Token 預估：**~45,000**

| 步驟 | Token |
|------|-------|
| 讀 document.rs + 建立 audit_document | 11,000 |
| 替換 6 處審計呼叫 | 4,000 |
| Stock SQL 驗證 | 5,000 |
| Handler/Service 職責調整 | 14,000 |
| 瀏覽器操作開銷 | 11,000 |

### Deploy 檢查點

- [ ] 審計日誌仍正確記錄所有單據操作
- [ ] 重構前後行為完全一致（diff 驗證）

---

## Phase 5：前端架構強化

**對應 Review 項目**：L3-Suggestion #5-8, L2-Suggestion #3
**風險**：低
**前置條件**：無（可與其他 Phase 並行）

### 待辦清單

- [ ] 5.1 在 `ProtectedRoute` 或 `MainLayout` 層加入全局 `PageErrorBoundary`，移除各頁面的手動包裹
- [ ] 5.2 在 `App.tsx` prefetch 清單加入遺漏頁面的註解說明
- [ ] 5.3 修改 `stores/auth.ts`：不持久化 `isAuthenticated`，改為從 `user !== null` 派生
- [ ] 5.4 修改 `data_export.rs` → `get_schema_version()` fallback 改為 `"000"`
- [ ] 5.5 修改 `data_export.rs` → `export_table()` 加入白名單檢查防禦

### Token 預估：**~30,000**

---

## Phase 6：CI/CD 與基礎設施

**對應 Review 項目**：L1-Suggestion #3, #5
**風險**：低
**前置條件**：無

### 待辦清單

- [ ] 6.1 啟用 CI push/PR 觸發器（取消 ci.yml 中的註解）
- [ ] 6.2 docker-compose.yml API port 綁定 `127.0.0.1`
- [ ] 6.3 確認 Nginx 已正確代理，不影響外部存取

### Token 預估：**~20,000**

---

## Token 預算總表（v2 更新）

| 階段 | v1 預估 | v2 預估 | 變化 | 可由 Claude 自主完成？ |
|------|---------|---------|------|----------------------|
| Phase 1 — 倉庫衛生 | 15,000 | 15,000 | — | ⚠️ 需本地 git 操作 |
| ~~Phase 1(v1) — IDOR 修復~~ | ~~40,000~~ | ~~0~~ | ✅ 已修復 | — |
| Phase 2 — DB 效能 | 80,000 | 80,000 | — | ⚠️ 需本地測試驗證 |
| Phase 3 — 安全/API 修正 | 50,000 | 40,000 | -10,000 | ✅ 可遠端讀改 |
| Phase 4 — 程式碼重構 | 70,000 | 45,000 | -25,000 | ✅ 可遠端讀改 |
| Phase 5 — 前端強化 | 30,000 | 30,000 | — | ✅ 可遠端讀改 |
| Phase 6 — CI/CD | 20,000 | 20,000 | — | ✅ 可遠端讀改 |
| **合計** | **305,000** | **230,000** | **-75,000** | |

### 總預算預估（v2）

| 類別 | v1 | v2 | 變化 |
|------|-----|-----|------|
| 已完成（審查+計畫） | 330,000 | 360,000 | +30,000（含本次更新） |
| 待執行（6 個 Phase） | 305,000 | 230,000 | **-75,000** |
| 緩衝（+20%） | 61,000 | 46,000 | -15,000 |
| **專案總預估** | **696,000** | **636,000** | **-60,000** |

---

## 依賴關係圖（v2 更新）

```
Phase 1 (倉庫衛生)
  ↓
Phase 2 (DB 效能) ──────────┐
  ↓                         │── 可並行
Phase 3 (安全/API) ─────────┘
  ↓
Phase 4 (程式碼重構) ← 依賴 Phase 2 的 snapshot 切換結果
  ↓
Phase 5 (前端) ← 獨立，可隨時執行
Phase 6 (CI/CD) ← 獨立，可隨時執行
```

---

## 執行策略建議（推薦混合模式）

1. **Phase 1, 6** → 開發者自行完成（簡單的 git/config 操作）
2. **Phase 3, 4, 5** → Claude 透過瀏覽器讀改，開發者本地 push
3. **Phase 2** → 開發者本地用 Claude Code，可即時 cargo test + psql EXPLAIN

---

## Pre-Deploy Checklist（每個 Phase 部署前）

### 通用檢查

- [ ] cargo clippy 無警告
- [ ] cargo test 全部通過
- [ ] npm run build 成功
- [ ] npm run test 通過
- [ ] Docker build 成功
- [ ] 無新的 cargo audit / npm audit 漏洞

### 資料庫變更特定（Phase 2）

- [ ] Migration 可在空 DB 上重頭執行
- [ ] Migration 可在已有資料的 DB 上增量執行
- [ ] inventory_snapshots backfill 腳本已準備
- [ ] EXPLAIN ANALYZE 確認索引被使用
- [ ] 舊視圖定義已備份（rollback 用）

### Post-Deploy 監控

- [ ] 確認錯誤率未異常上升
- [ ] 確認 API 回應時間穩定
- [ ] 確認庫存數據正確
- [ ] 通知相關人員部署完成
