# Code Review：豬博士 iPig 系統（第一層 — 安全與架構）

**倉庫**：https://github.com/delightening/ipig_system
**審查日期**：2026-03-15
**審查範圍**：倉庫衛生、部署配置、認證授權、資料庫安全、環境變數處理
**審查版本**：main 分支 `9c5ae3d`

---

## 總結

iPig 系統的安全架構**令人印象深刻**。對於一個實驗動物管理平台而言，安全設計的完整度已超出同類專案的平均水準。JWT 黑名單、CSRF 防護、多層速率限制、稽核日誌防篡改（HMAC）、GDPR 合規、2FA 支援等功能齊全，中間件分層清晰，CI 管線包含 cargo audit、cargo deny、Trivy 掃描、SQL 注入守衛等多重安全檢查。

但有幾個倉庫衛生和配置層面的問題需要優先處理。

---

## Critical Issues（必須修復）

| # | 位置 | 問題 | 嚴重度 |
|---|------|------|--------|
| 1 | `.venv/` 目錄 | **Python 虛擬環境被 commit 到倉庫**。`.gitignore` 中未包含 `.venv`，導致整個虛擬環境（含所有 pip 套件）進入版本控制。這嚴重膨脹了倉庫體積，且可能洩漏環境細節。 | 🔴 Critical |
| 2 | 根目錄 `old_ipig.dump` | **資料庫 dump 檔被 commit**。雖然 `.gitignore` 已加入 `*.dump`，但此檔案是在 gitignore 規則加入之前就已被 commit，所以仍然留在 git 歷史中。可能包含敏感資料（使用者密碼 hash、個人資料）。 | 🔴 Critical |
| 3 | 根目錄 `test-login.json`、`last_error.json` | **執行時產物已 commit**。同樣是在 gitignore 規則加入前就被追蹤。`test-login.json` 可能含有測試用憑證。 | 🟡 High |
| 4 | 根目錄 `product_import_from_stocklist.csv`、`匯入用.csv` | **業務資料檔案已 commit**。gitignore 有 `*.csv` 規則但這些檔案是先前 commit 的。 | 🟡 High |

### 修復建議

```bash
# 1. 將 .venv 加入 gitignore 並從追蹤中移除
echo ".venv/" >> .gitignore
git rm -r --cached .venv/

# 2. 移除已追蹤的敏感檔案
git rm --cached old_ipig.dump
git rm --cached test-login.json last_error.json
git rm --cached product_import_from_stocklist.csv 匯入用.csv

# 3. 若 old_ipig.dump 含敏感資料，考慮用 git-filter-repo 徹底清除歷史
# pip install git-filter-repo
# git filter-repo --path old_ipig.dump --invert-paths
```

---

## Suggestions（建議改進）

| # | 位置 | 建議 | 類別 |
|---|------|------|------|
| 1 | `backend/src/middleware/auth.rs` | `auth_middleware` 只查記憶體黑名單 (`is_revoked`)，未用 `is_revoked_with_db`。多實例部署時，A 實例撤銷的 JWT 在 B 實例的記憶體中不存在，會被放行。建議改用 `is_revoked_with_db` 或引入 Redis 共享黑名單。 | 安全性 |
| 2 | `backend/src/middleware/rate_limiter.rs` | Rate limiter 使用記憶體 `DashMap`（`OnceLock` + `static`），多實例部署時各自獨立計數，無法實現全局速率限制。若計畫水平擴展，需考慮 Redis-based 方案。 | 架構 |
| 3 | `docker-compose.yml` | API 服務的 `ports` 映射為 `"${API_PORT:-8000}:8000"`，預設綁定所有介面（0.0.0.0）。建議與 DB 一樣限制為 `127.0.0.1:${API_PORT:-8000}:8000`，因為通常只有 Nginx 需要存取 API。 | 安全性 |
| 4 | `frontend/nginx.conf` | CSP 中的 `script-src 'unsafe-eval'` 降低了 XSS 防護力。註解中提到這是 Vite 的已知限制，但建議追蹤 Vite issue 並在修復後移除。 | 安全性 |
| 5 | `.github/workflows/ci.yml` | CI 只能手動觸發（`workflow_dispatch`），`push` 和 `pull_request` 觸發器被註解掉了。這代表 PR 合併前不會自動運行安全掃描和測試。建議至少啟用 PR 觸發。 | 流程 |
| 6 | `backend/src/middleware/jwt_blacklist.rs` | `is_revoked_with_db` 在 DB miss 回填快取時硬編碼 `exp = now + 3600`，而非使用實際的 JWT 過期時間。若 JWT 有效期超過 1 小時（目前預設 6 小時），可能導致快取過早失效。 | 正確性 |
| 7 | `backend/src/handlers/auth.rs` | `forgot_password` 中，當帳號不存在時只記了 info log 但回傳相同訊息（防列舉，很好）。但 timing 可能不同——存在的帳號會做 DB 寫入 + 發信，不存在的直接回傳。攻擊者可能透過回應時間差異進行帳號列舉。建議加入固定延遲。 | 安全性 |
| 8 | `backend/src/config.rs` | `database_max_connections` 預設值在 `from_env()` 中是 `40`，但 `.env.example` 寫的是 `10`，docker-compose 也傳的是 `10`。代碼預設值與文件不一致。 | 可維護性 |

---

## What Looks Good（做得好的部分）

- **Dockerfile 多階段建構**：5 階段建構（Chef → Planner → Builder → Runtime → Distroless），最終映像用 `gcr.io/distroless/cc-debian12`，攻擊面極小，以非 root 使用者 (`appuser`) 運行。
- **網路隔離**：docker-compose 中 frontend / backend / database 三層網路隔離，DB 只綁定 `127.0.0.1`。
- **Nginx 安全標頭**：完整的 CSP、HSTS（2 年）、X-Frame-Options DENY、X-Content-Type-Options nosniff、Permissions-Policy，且禁止直接存取 `/uploads` 目錄。
- **JWT 安全設計**：Cookie HttpOnly + SameSite=Lax + 可選 Secure、Access/Refresh Token 分離、JWT 黑名單（記憶體+DB 雙層）、JTI 追蹤、最小 32 字元密鑰驗證。
- **CSRF 防護**：Double Submit Cookie 模式，寫入方法驗證 X-CSRF-Token，公開端點豁免，有完整單元測試。
- **多層速率限制**：認證端點（30/min）、一般 API（600/min）、寫入端點（120/min）、檔案上傳（30/min），分層精細。
- **CI 安全掃描**：cargo audit + cargo deny + SQL 注入正則守衛 + unsafe 守衛 + npm audit + Trivy 容器掃描 + 覆蓋率門檻 25%。
- **密碼重設防列舉**：不論帳號存不存在都回傳相同訊息。
- **帳號鎖定機制**：SEC-20 登入失敗鎖定，可配置最大嘗試次數和鎖定時長。
- **敏感操作二級認證**：SEC-33 reauth token 機制，密碼確認後才能執行帳號刪除等操作。
- **模擬登入安全**：impersonated_by 記錄在 JWT 中，有完整的 audit trail。
- **GDPR 合規**：匯出個人資料 + 軟刪除帳號 + 審計日誌。
- **Docker Secrets 支援**：`read_secret()` 優先讀 `_FILE` 環境變數路徑，支援 Docker Secrets。
- **資料庫 Schema**：密碼存儲為 hash、refresh token 存儲為 hash、有適當索引、有帳號鎖定欄位。

---

## Verdict

**Request Changes** — 需要修復倉庫衛生問題後才建議部署到生產環境。

核心安全架構設計得非常好，但有 4 個倉庫衛生問題（`.venv` commit、DB dump、測試憑證、業務資料）需要優先處理。建議修復順序：

1. **立即**：從 git 追蹤中移除 `.venv`、`old_ipig.dump` 等敏感檔案
2. **短期**：啟用 CI 自動觸發（至少 PR 觸發）、修復 auth middleware 的黑名單查詢
3. **中期**：考慮多實例場景的共享狀態（Redis）、消除 timing attack 風險

---

---

# 第二層 Code Review：業務邏輯、前端架構與測試品質

**審查範圍**：後端 handlers/services 抽查、前端架構與狀態管理、錯誤處理模式、測試品質

---

## 總結

第二層審查聚焦在代碼品質和可維護性。整體而言，專案架構分層清晰（handlers → services → repositories），前端採用 React + Zustand + React Query + TypeScript，技術選型成熟。但在代碼重複、下載安全、和測試覆蓋上有改進空間。

---

## Critical Issues（必須修復）

| # | 位置 | 問題 | 嚴重度 |
|---|------|------|--------|
| 1 | `handlers/upload.rs` → `download_attachment` | ~~**附件下載缺少資源級權限檢查（IDOR 風險）**~~ ✅ **已修復** — `check_attachment_permission` 已存在，根據 entity_type 做資源級權限檢查。 | ~~🔴 Critical~~ ✅ |

### 修復建議

```rust
// download_attachment 應加入資源級權限檢查
pub async fn download_attachment(...) -> Result<Response> {
    let attachment = /* fetch attachment */;

    // 根據 entity_type 檢查對應的權限
    match attachment.entity_type.as_str() {
        "protocol" => require_permission!(current_user, "aup.protocol.view"),
        "animal" => require_permission!(current_user, "animal.animal.view"),
        "leave_request" => {
            // 只有本人或 HR 可以下載
            if attachment.uploaded_by != current_user.id {
                require_permission!(current_user, "hr.leave.manage");
            }
        }
        _ => {}
    }
    // ... proceed with download
}
```

---

## Suggestions（建議改進）

| # | 位置 | 建議 | 類別 |
|---|------|------|------|
| 1 | `handlers/upload.rs` | ~~**大量重複代碼**~~ ✅ **已修復** — `handle_upload` 通用函式已存在，7 個上傳 handler 已整合。 | ~~可維護性~~ ✅ |
| 2 | `error.rs` → `Database` 分支 | ~~**DB 錯誤預設回傳 500**~~ ✅ **已修復** — unique violation（23505）已回傳 409、FK violation（23503）已回傳 400，前端可正確區分。 | ~~正確性~~ ✅ |
| 3 | `stores/auth.ts` | **auth state 持久化到 localStorage 含 `isAuthenticated`**。頁面刷新時先從 localStorage 讀到 `isAuthenticated: true`，再由 `checkAuth` 驗證。若後端 token 已過期，會有短暫的「已登入」假象。目前有 `isInitialized` 標記緩解了這點，但更乾淨的做法是不持久化 `isAuthenticated`，而是每次從 `user !== null` 派生。 | 前端品質 |
| 4 | `lib/api/client.ts` | **deleteResource 用 POST 迴避 DELETE 405**。這是反 REST 模式，會導致 HTTP cache、代理、API 文件工具（如 Swagger）行為異常。建議先修復代理配置（Nginx 或 tunnel），而非在客戶端繞過。 | 架構 |
| 5 | `lib/api/client.ts` | **refresh token 競態處理中，import 使用動態 `await import()`**。refresh 成功後用 `await import('@/stores/auth')` 更新 session expiry——但 `useAuthStore` 已在檔案頂部靜態 import。這個動態 import 是不必要的，直接用已有的 `useAuthStore` 即可。 | 代碼品質 |
| 6 | `backend/tests/` | **整合測試覆蓋面偏窄**。目前有 auth、users、animals、protocols、reports、health、etag 的整合測試，但缺少 ERP（stock、product、warehouse）、HR（attendance、leave）、notification 等模組的測試。建議按業務重要性逐步補充。 | 測試 |
| 7 | `frontend/e2e/` | **E2E 測試集中在認證流程**。login、auth-refresh、profile、admin-users、dashboard 都有覆蓋，但 ERP 流程（採購、庫存）、動物管理 CRUD、協議審批等核心業務流程缺少 E2E 覆蓋。 | 測試 |
| 8 | `handlers/user.rs` → `create_user` | **歡迎郵件含明文密碼**。`create_user` 將 `plain_password` 傳給 `send_welcome_email`，意味著密碼以明文出現在郵件中。雖然 `must_change_password` 標記會強制使用者登入後修改，但密碼經郵件傳輸仍不安全（SMTP 通常非 E2E 加密）。建議改用「密碼重設連結」模式。 | 安全性 |

---

## What Looks Good（做得好的部分）

- **檔案上傳安全**：FileService 的防護做得很完整——路徑穿越檢查（`../` 和 `canonicalize`）、Magic Number 驗證（SEC-14）、MIME 類型白名單、檔案大小限制（分類別）、UUID 重命名防覆蓋。測試覆蓋也很充分。
- **錯誤處理精細**：`AppError` 枚舉涵蓋了所有場景，`DuplicateWarning` 的 non-blocking 409 設計體貼前端；DB pool timeout 回 503 + Retry-After 非常專業；`JsonRejection` 翻譯成中文友善訊息。
- **前端 API Client**：axios interceptor 設計優秀——CSRF token 自動附加、503 自動重試（最多 2 次 + Retry-After）、refresh token 競態鎖（`isRefreshing` + subscriber queue）、防重複登出鎖。
- **前端狀態管理**：Zustand + persist 簡潔高效，`useShallow` 優化 re-render，selector 拆分（`useAuthUser`、`useAuthActions`）降低耦合。`checkAuth` 僅在 401 時清除狀態，503 保留登入避免誤登出。
- **IDOR 防護（部分）**：`get_user` 檢查 `current_user.id != id` 才要求 admin 權限；`delete_attachment` 檢查 `uploaded_by` 或 admin；敏感操作（delete user、reset password、impersonate）都要求 reauth token。
- **XSS 防護**：DOMPurify 用於 SVG sanitize，且有嚴格的 FORBID_TAGS/FORBID_ATTR 配置。
- **代碼組織**：前端 types 目錄將業務型別按模組拆分（auth、erp、animal、aup、report、audit、notification、amendment、upload），API client 也按模組拆分，可維護性好。
- **審計日誌**：user 模組的 create/update/delete/password reset/impersonate 都有完整審計，且對角色變更和帳號狀態變更做了專門的 security-level 審計追蹤。

---

## Verdict（第二層）

**Approve with Comments** — 業務邏輯和代碼品質整體良好，有一個 IDOR 風險需要修復。

優先修復：
1. **立即**：`download_attachment` 和 `list_attachments` 加入資源級權限檢查
2. **短期**：抽取上傳 handler 重複代碼、修正 DB 錯誤的 HTTP status、考慮替換歡迎郵件中的明文密碼
3. **中期**：補充 ERP/HR 模組的整合測試和 E2E 測試覆蓋

---

## 整體評價（兩層合計）

這是一個**安全意識非常高的專案**。從 SEC 編號系統可以看出，安全需求被系統性地追蹤和實作。在同類中小型企業應用中，這個安全設計水準屬於上乘。

主要風險集中在：附件下載的 IDOR 漏洞（唯一的 Critical）、多實例部署的狀態同步、以及測試覆蓋的業務模組空白。這些都是可以漸進修復的，不影響核心架構的穩健性。

---

---

---

# 第三層 Code Review：效能、查詢優化、前端架構、API 一致性與技術債務

**審查範圍**：資料庫查詢效能、索引覆蓋、前端路由與 bundle 策略、資料匯出邏輯、API 設計一致性、程式碼重複掃描

---

## 總結

第三層深入檢視了系統的效能瓶頸和長期可維護性。最顯著的問題是**庫存查詢的 O(n²) 模式**——CROSS JOIN warehouses × products 的笛卡爾積會隨資料量增長急劇惡化。前端的 code-splitting 和 lazy loading 策略設計得非常好，分三批優先級預載。資料匯出模組（IDXF）設計合理但存在記憶體爆炸風險。API 設計整體一致，但審計日誌存在嚴重的程式碼重複。

---

## Critical Issues（必須修復）

| # | 位置 | 問題 | 嚴重度 |
|---|------|------|--------|
| 1 | `services/stock.rs` → `get_on_hand()` | **CROSS JOIN 笛卡爾積模式**。查詢倉庫×產品所有組合，再 LEFT JOIN `stock_ledger` 聚合。若 50 個倉庫 × 2000 個產品 = 100,000 列結果，大部分為零庫存的無用列。`v_low_stock_alerts` 和 `v_inventory_summary` 視圖也使用相同模式。應改為只查 `inventory_snapshots` 中有記錄的組合。 | 🔴 Critical |
| 2 | `services/stock.rs` → `check_stock_available()` | **每次庫存操作都從完整帳本歷史重新計算**。核准單據時呼叫此函式，它 `SUM` 了 `stock_ledger` 表中該倉庫×產品的所有歷史紀錄。隨時間推移，這個查詢會越來越慢。應改用 `inventory_snapshots` 表（已存在但未被利用）。 | 🔴 Critical |
| 3 | `services/data_export.rs` → `export_full_database()` | **全庫匯出載入所有資料到記憶體**。`export_table()` 用 `json_agg(row_to_json(t))` 一次載入整張表的所有列為 JSON。對於大表（如 `stock_ledger`、`user_activity_logs`）可能導致 PostgreSQL OOM 或 Rust 行程記憶體暴增。Zip 模式雖有 NDJSON 分流但仍一次聚合。 | 🔴 Critical |

### 修復建議

```sql
-- Issue #1: 改用 inventory_snapshots 取代 CROSS JOIN
-- 替換 v_low_stock_alerts 視圖
CREATE OR REPLACE VIEW v_low_stock_alerts AS
SELECT p.id AS product_id, p.sku, p.name AS product_name,
       inv.warehouse_id, w.code AS warehouse_code,
       inv.on_hand_qty_base AS on_hand_qty, p.base_uom,
       CASE
         WHEN inv.on_hand_qty_base <= 0 THEN 'out_of_stock'
         WHEN p.safety_stock IS NOT NULL AND inv.on_hand_qty_base < p.safety_stock THEN 'below_safety'
         WHEN p.reorder_point IS NOT NULL AND inv.on_hand_qty_base < p.reorder_point THEN 'below_reorder'
       END AS stock_status
FROM inventory_snapshots inv
JOIN products p ON inv.product_id = p.id
JOIN warehouses w ON inv.warehouse_id = w.id
WHERE p.is_active AND w.is_active
  AND (inv.on_hand_qty_base <= 0
       OR (p.safety_stock IS NOT NULL AND inv.on_hand_qty_base < p.safety_stock)
       OR (p.reorder_point IS NOT NULL AND inv.on_hand_qty_base < p.reorder_point));
```

```rust
// Issue #2: check_stock_available 改用 snapshot
pub async fn check_stock_available(pool: &PgPool, warehouse_id: Uuid, product_id: Uuid) -> Result<Decimal> {
    let row = sqlx::query_scalar!(
        "SELECT on_hand_qty_base FROM inventory_snapshots
         WHERE warehouse_id = $1 AND product_id = $2",
        warehouse_id, product_id
    ).fetch_optional(pool).await?;
    Ok(row.unwrap_or(Decimal::ZERO))
}
```

```rust
// Issue #3: 分頁匯出大表
async fn export_table(pool: &PgPool, table: &str) -> Result<TableExport> {
    // 改用 cursor-based pagination
    let sql = format!(
        "SELECT row_to_json(t) FROM (SELECT * FROM \"{}\" ORDER BY created_at) t
         LIMIT $1 OFFSET $2", table
    );
    // 分批取出，每批 5000 列
}
```

---

## Suggestions（建議改進）

| # | 位置 | 建議 | 類別 |
|---|------|------|------|
| 1 | `007_audit_erp.sql` → `stock_ledger` 索引 | **缺少關鍵複合索引**。目前只有 `idx_stock_ledger_warehouse_product(warehouse_id, product_id)` 和 `idx_stock_ledger_trx_date(trx_date)`。`check_stock_available` 查詢需要 `(warehouse_id, product_id, direction)` 複合索引；帳本查詢需要 `(warehouse_id, product_id, trx_date DESC)` 複合索引。`doc_id` 也缺少索引。 | 效能 |
| 2 | `services/stock.rs` → `get_ledger()` | **硬編碼 LIMIT 1000 無分頁支援**。查詢結尾 `LIMIT 1000` 但無 offset/cursor 參數，前端無法載入超過 1000 筆的帳本紀錄，也無法實現無限捲動。 | 效能 |
| 3 | `services/stock.rs` | **SQL 大量重複**。`get_on_hand()` 和 `get_low_stock_alerts()` 含有近乎相同的庫存聚合 SQL（CROSS JOIN + LEFT JOIN stock_ledger + CASE direction SUM），僅差 WHERE 條件和回傳欄位。應抽取為共用查詢或直接利用 `inventory_snapshots`。 | 可維護性 |
| 4 | `handlers/document.rs` | **審計日誌呼叫高度重複**。create、update、submit、approve、cancel、delete 六個 handler 都有幾乎一樣的 `AuditService::log_activity(...)` 呼叫，只差 event_type 和 after_data。建議抽取為 `fn audit_document(db, user_id, event_type, doc_id, doc_no, extra_data)` 或用 middleware / macro。 | 可維護性 |
| 5 | `services/data_export.rs` → `get_schema_version()` | **fallback 邏輯不安全**。當 `_sqlx_migrations` 表不存在或查詢失敗時，fallback 為 `"010"`（最新版）。這意味著若匯入到全新的資料庫，schema_mapping 會認為版本是 010 而跳過所有 migration。應改為 fallback `"000"` 或回傳 `Err`。 | 正確性 |
| 6 | `services/data_export.rs` → `export_table()` | **SQL 注入防護依賴常數陣列**。`format!("SELECT ... FROM \"{}\"", table)` 中的 table 名來自 `EXPORT_TABLE_ORDER` 常數，因此目前安全。但若未來有人修改為接受動態輸入，就會有 SQL 注入風險。建議加上 `assert!(EXPORT_TABLE_ORDER.contains(&table))` 或使用白名單檢查函式。 | 防禦性程式設計 |
| 7 | `App.tsx` → `prefetch` 效果 | **預載清單與 lazy 宣告不完全同步**。例如 `PurchaseSalesSummaryPage`、`AccountingReportPage`、`QAUDashboardPage`、`FacilitiesPage`、`TrainingRecordsPage` 在 lazy 宣告中存在但不在預載清單中。低頻頁面不預載是合理的，但建議加註解說明為何排除。 | 可維護性 |
| 8 | `App.tsx` → 路由結構 | **部分路由缺少 `PageErrorBoundary`**。`DashboardPage`、`ProtocolEditPage` 有包裹 `<PageErrorBoundary>`，但其他大多數頁面沒有。應統一在 `MainLayout` 層或 `ProtectedRoute` 層包裹，而非逐頁手動加。 | 可靠性 |
| 9 | 根目錄雜項檔案 | **開發產物被 commit**。`backend/build_final.txt`、`backend/build_output.txt`、`backend/ci-fail.txt`、`backend/error.log`、`backend/error.txt`、`backend/test_output.txt`、`last_error.json` 等應加入 `.gitignore` 並從追蹤移除。 | 倉庫衛生 |
| 10 | `handlers/` 全域 | **handler 與 service 職責邊界模糊**。`document.rs` 的 handler 直接做 `check_document_access` 權限判斷和 `AuditService` 呼叫；而 `stock.rs` 的 handler 則非常薄，只做 `require_permission!` + 委派。建議統一：handler 負責認證/授權/HTTP，service 負責業務邏輯+審計。 | 架構一致性 |

---

## 資料庫索引建議

```sql
-- stock_ledger 效能優化索引
CREATE INDEX idx_stock_ledger_wh_prod_dir
  ON stock_ledger(warehouse_id, product_id, direction);

CREATE INDEX idx_stock_ledger_wh_prod_date
  ON stock_ledger(warehouse_id, product_id, trx_date DESC);

CREATE INDEX idx_stock_ledger_doc_id
  ON stock_ledger(doc_id);

-- document_lines 效能優化
CREATE INDEX idx_document_lines_product_id
  ON document_lines(product_id);
```

---

## What Looks Good（做得好的部分）

- **前端 Code-Splitting 策略**：所有 65+ 頁面元件都使用 `React.lazy` 動態載入，Layouts 保持靜態 import。三批次 idle-time 預載策略（高頻→次要→低頻）設計精良，使用 `requestIdleCallback` + fallback `setTimeout(2000)` 確保不影響首次渲染效能。
- **路由權限設計**：`ProtectedRoute`、`DashboardRoute`、`AdminRoute`、`RequirePermission` 多層路由守衛，支援 `role`、`permission`、`anyOf` 三種權限模式，`fallback="redirect"` 設計貼心。
- **IDXF 匯出格式設計**：自訂 iPig Data Exchange Format，含 schema_version 語意化版本、FK 依賴順序匯出（確保匯入時不會違反外鍵約束）、大表自動切換 NDJSON 格式、稽核資料可選擇包含/排除。
- **SSE 實作品質**：`AlertBroadcaster` 使用 `tokio::sync::broadcast(64)` 頻道，SSE 端點有 15 秒 keepalive 防止連線逾時，`Lagged` 錯誤優雅處理（重新訂閱），clean shutdown 時發送 `[DONE]` 事件。
- **Document Handler 完整度**：完整的單據生命週期（建立→提交→核准/取消→刪除），每個狀態變更都有審計日誌、非同步通知、IDOR 防護（`check_document_access` 檢查建立者/倉管/管理員）。`utoipa` 標註提供 OpenAPI 文件自動生成。
- **Stock Handler 精簡設計**：handler 層極度精簡（每個函式 ~5 行），只做 `require_permission! + 委派 service`，職責分離教科書級。
- **Partitioned Audit Logs**：`user_activity_logs` 使用 PostgreSQL RANGE 分區（按季），已建好 2026Q1-2027Q4 共 8 個分區，並有 `partition_maintenance.rs` 自動維護。索引設計完整（actor、entity、category、event_type、suspicious、ip、date、integrity）。
- **inventory_snapshots 表已存在**：materialized 快照表的 schema 設計正確（warehouse_id + product_id 複合主鍵、on_hand_qty_base、avg_cost），只是在查詢端尚未被充分利用。
- **公開頁面優化**：`checkAuth` 在公開路由（login、forgot-password 等）不呼叫 `/api/me`，直接標記 `isInitialized`，避免無謂的認證請求。

---

## Verdict（第三層）

**Request Changes** — 有 3 個效能相關的 Critical 問題需要修復。

優先修復：
1. **立即**：將 `check_stock_available()` 改為查詢 `inventory_snapshots`，消除每次操作的全歷史重算
2. **立即**：新增 stock_ledger 複合索引（`wh_prod_dir`、`wh_prod_date`、`doc_id`）
3. **短期**：重寫 `v_low_stock_alerts` 和 `v_inventory_summary` 視圖，移除 CROSS JOIN 改用 inventory_snapshots
4. **短期**：data_export 加入串流/分頁匯出機制，避免大表 OOM
5. **中期**：抽取審計日誌通用函式、統一 handler/service 職責邊界、統一 PageErrorBoundary

---

---

# 三層審查總結

## 問題統計

| 嚴重度 | 第一層 | 第二層 | 第三層 | 合計 | 已修復 | 剩餘 |
|--------|--------|--------|--------|------|--------|------|
| 🔴 Critical | 2 | 1 | 3 | 6 | 3 | **3** |
| 🟡 High | 2 | 0 | 0 | 2 | 2 | **0** |
| 💡 Suggestion | 8 | 8 | 10 | 26 | 2 | **24** |

## 修復優先級路線圖

### P0 — 立即修復（影響安全/效能）
1. ~~`.venv` 和 `old_ipig.dump` 從 git 歷史清除~~（✅ 已完成）
2. ~~`download_attachment` IDOR 修復~~（✅ 已修復 — `check_attachment_permission` 已存在）
3. `check_stock_available()` 改用 `inventory_snapshots`
4. stock_ledger 加索引

### P1 — 短期修復（1-2 週）
5. 移除 CROSS JOIN 視圖，改用 inventory_snapshots
6. data_export 串流匯出
7. 啟用 CI push/PR 觸發器
8. ~~DB unique/FK 錯誤回傳正確 HTTP status~~（✅ 已修復）
9. 歡迎郵件改用重設連結

### P2 — 中期改善（1-2 月）
10. ~~抽取上傳 handler 重複碼~~（✅ 已修復）/ 審計日誌重複碼
11. 統一 handler/service 職責邊界
12. `get_ledger()` 加分頁
13. 全局 PageErrorBoundary
14. 清理 backend 開發產物檔案
15. 補充 ERP/HR 測試覆蓋

### P3 — 長期考量
16. 多實例狀態同步（Redis for rate limiter + JWT blacklist）
17. `forgot_password` timing attack 防護
18. 前端 localStorage auth state 重構

## 整體評價

這是一個**架構成熟度很高的專案**，尤其在安全設計方面（SEC 編號系統、多層防護、GDPR 合規、HMAC 審計）已超越大多數同規模的企業應用。前端的 code-splitting 策略和權限路由設計也很專業。

主要的技術債集中在兩個方面：**庫存查詢效能**（CROSS JOIN + 全歷史重算，會隨資料量線性惡化）和**程式碼重複**（upload handlers、audit logging）。好消息是 `inventory_snapshots` 表已經存在且 schema 正確，只需要在查詢端切換過去，這是最高 ROI 的改進。

---

## 審查成本紀錄

| 項目 | 估計值 |
|------|--------|
| 審查日期 | 2026-03-15 |
| 使用模型 | Claude Opus 4.6 |
| 瀏覽器操作次數 | ~90 次 |
| 讀取原始碼檔案數 | ~25 個 |
| 估計總消耗 Token | ~300,000 |
| 審查耗時 | 單次對話（含上下文續接），三層完成 |
| 審查覆蓋 | 第一層（安全/架構）全面 + 第二層（業務/品質）抽查 + 第三層（效能/一致性）深度 |
