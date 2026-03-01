# 專案 Walkthrough 紀錄

---

## 登入 Cookie 過大導致無法登入（2026-03-01）

### 背景

登入時出現：
- `Set-Cookie header is ignored... The combined size of the name and value must be less than or equal to 4096 characters`
- 後續 heartbeat、amendments、me 等 API 回傳 401 Unauthorized
- refresh token 回傳 400 Bad Request

### 原因

JWT access_token 將 roles 與 permissions 完整放入 claims。Admin 角色擁有全部權限（約 130+ 個），導致 JWT 過大，Set-Cookie 超過瀏覽器 4096 字元限制，cookie 被忽略，session 無法建立。

### 實作內容

在 `auth.rs` 的 `generate_access_token` 中：
- 若使用者具 admin 或 SYSTEM_ADMIN 角色，JWT claims 的 `permissions` 改為空陣列
- `has_permission` 對 admin 會直接回傳 true，故不需在 JWT 內含 permissions
- 可大幅縮小 admin 的 JWT，避免 cookie 超限

---

## SSE 安全警報 504 Gateway Timeout 修正（2026-03-01）

### 背景

`GET /api/admin/audit/alerts/sse` 出現 504 Gateway Timeout。SSE（Server-Sent Events）為長連線，會持續保持開啟；代理層預設 timeout 導致逾時。

### 原因

1. **Nginx**：`proxy_read_timeout` 預設 60 秒，SSE 長連線逾時
2. **proxy_buffering** 預設 on，可能阻礙 SSE 即時串流
3. **Vite dev proxy**：http-proxy 預設亦有讀取逾時

### 實作內容

1. **nginx.conf / nginx-ci.conf**：新增 `/api/admin/audit/alerts/sse` 專用 location（置於 `/api` 之前）：
   - `proxy_buffering off`、`proxy_cache off`
   - `proxy_read_timeout 86400s`、`proxy_send_timeout 86400s`
   - `chunked_transfer_encoding off`
   - 不套用 `limit_req`（SSE 需長連線）

2. **vite.config.ts**：新增 SSE 路徑專用 proxy（置於 `/api` 之前）：
   - `timeout: 0`、`proxyTimeout: 0` 避免 dev 環境逾時

---

## IDXF 匯入排除管理員內容（2026-03-01）

### 背景

匯入時排除管理員相關資料，避免覆蓋目標庫的管理員帳號與角色。

### 實作內容

在 `data_import.rs` 新增 `filter_admin_content`：

1. **users**：略過 email = admin@ipig.local（或 env `ADMIN_EMAIL`）
2. **roles**：略過 code = 'admin' 或 'SYSTEM_ADMIN'
3. **user_roles**：略過 user_id 或 role_id 為上述管理員者
4. **role_permissions**：略過 role_id 為管理員角色者

排除時建立 source_id -> target_id 對應，供子表（如 change_reasons.changed_by）FK remap。略過項目會出現在 `skipped_details`，reason 為「管理員相關，已略過」。

---

## IDXF 匯入略過項目說明（2026-03-01）

### 背景

使用者匯入 IDXF 時，需要清楚知道哪些項目被略過及原因。

### 實作內容

1. **Backend**（`data_import.rs`）：
   - 新增 `SkippedDetail` 結構：`table`、`reason`、`count`
   - `ImportResult` 新增 `skipped_details: Vec<SkippedDetail>`
   - 未知表：改為加入 `skipped_details`（reason: "未知表，已略過"），不再放入 `errors`
   - 重複鍵：每表若有 `skip > 0`，加入 `skipped_details`（reason: "重複鍵", count: N）

2. **Frontend**（`SettingsPage.tsx`）：
   - 匯入結果 Dialog 改為顯示「錯誤」與「略過項目」兩區塊
   - 有 `errors` 或 `skipped_details` 時開啟 Dialog

3. **Script**（`import-idxf.ps1`）：輸出略過項目明細

---

## IDXF 匯入 ID 對應機制（2026-03-01）

### 背景

匯入 IDXF 到**已有資料**的資料庫時，會發生 FK 違反錯誤。原因：
- 父表（roles, permissions, users 等）以 natural key（code, email）做 ON CONFLICT
- 目標庫已有同 code/email 的列，但 `id` 不同
- 子表匯入資料仍帶來源庫的 `id`，導致 FK 找不到對應父列

### 實作內容

在 `backend/src/services/data_import.rs` 中新增：

1. **`fetch_fk_config`**：從 `information_schema` 查詢 FK 關係，僅保留參照 `*.id` 且 ref_table 有 natural key 的表
2. **`build_id_mappings`**：匯入父表前，依 conflict key 查詢目標庫既有列，建立 `source_id -> target_id` 對應
3. **`remap_foreign_keys`**：子表匯入前，將 rows 中的 FK 欄位由 source_id 替換為 target_id
4. **JSON 匯入排序**：依 `EXPORT_TABLE_ORDER` 排序，確保父表先於子表處理

### 有 ID 對應的表

| 表 | Conflict Key | 說明 |
|----|--------------|------|
| roles, permissions | code | 角色、權限 |
| blood_test_templates, blood_test_panels | code / key | 血液檢查模板 |
| users | email | 使用者 |
| animal_sources, warehouses, partners | code | 基礎主檔 |
| product_categories, sku_categories, sku_subcategories | code | ERP 分類 |

### 流程

```
for each table in EXPORT_TABLE_ORDER:
  1. build_id_mappings(table, rows)  # 若為 natural key 表，查目標庫建立 mapping
  2. remap_foreign_keys(table, rows, fk_config, id_mapping)
  3. import_table(table, rows)
```

---

## SQLx 離線編譯（2026-03-01）

### 背景

sqlx 的 `query!`、`query_scalar!` 等巨集會在編譯時連線資料庫驗證 SQL。若本機無資料庫或無法連線，會出現「無法識別主機名稱」等錯誤。

### 修正方式

在 `backend/.cargo/config.toml` 新增 `[env]` 區塊，預設 `SQLX_OFFLINE=true`：

```toml
[env]
SQLX_OFFLINE = "true"
```

- 編譯時使用 `.sqlx/` 快取，無需連線資料庫
- 若環境已設 `SQLX_OFFLINE`（如 CI 的 backend-test 設為 false），不會覆寫
- 新增或修改 SQL 巨集後，需在有 DB 的環境執行 `cargo sqlx prepare` 更新快取

---

## PowerShell Migration 執行紀錄（2026-03-01）

### 背景

於 PowerShell 執行 `sqlx migrate run` 進行資料庫遷移。

### 嘗試 1：sqlx-cli

```powershell
cargo install sqlx-cli --no-default-features --features postgres
```

**結果**：失敗。錯誤 `linker 'link.exe' not found`，需安裝 Visual Studio Build Tools（含 C++ 工作負載）或 MSVC 工具鏈。

### 嘗試 2：Docker + psql 直接執行 SQL

```powershell
$env:DATABASE_URL = "postgres://postgres:ipig_password_123@localhost:543/ipig_db"
Get-Content "backend\migrations\001_types.sql" -Raw | docker exec -i ipig-db psql -U postgres -d ipig_db
```

**結果**：
1. **Schema 已存在**：資料庫已跑過舊 migrations，types/tables/indexes 多數已存在，產生大量 `already exists` 錯誤。
2. **編碼問題**：PowerShell 預設編碼導致 migration 內中文（如權限描述）變成 `??????`，造成 INSERT 語法錯誤。
3. **結論**：新 migrations（001~010）僅適用於**全新安裝**，既有環境請勿直接套用。詳見 `docs/MIGRATION_REFACTOR_2026-03-01.md`。

### 建議做法

| 情境 | 做法 |
|------|------|
| **全新安裝** | 1. 安裝 MSVC 工具鏈後 `cargo install sqlx-cli`<br>2. `$env:DATABASE_URL="postgres://postgres:ipig_password_123@localhost:543/ipig_db"`<br>3. `cd backend; sqlx migrate run` |
| **既有 DB** | 維持現狀，或備份後 drop database 重建再執行 migrations |
| **CI（Linux）** | GitHub Actions 已使用 `cargo install sqlx-cli`，無 MSVC 問題 |

---

## reviewdog 設定與專案檢查報告

## 1. 已完成項目

### 1.1 本地執行 reviewdog

僅在本地使用 reviewdog，不透過 GitHub Actions。需先安裝 reviewdog：

```powershell
# Windows (Scoop)
scoop install reviewdog

# 或使用 x-cmd
x install reviewdog
```

**Rust 後端：**
```powershell
cd backend
cargo clippy --all-targets --message-format=json 2>/dev/null | reviewdog -reporter=local -filter-mode=diff_context -efm="%f:%l:%c: %m"
# 或直接看 clippy 輸出
cargo clippy --all-targets -- -D warnings -W clippy::unwrap_used
```

**React 前端：**
```powershell
cd frontend
npm run lint 2>&1 | reviewdog -reporter=local -filter-mode=diff_context -f=eslint
```

### 1.2 ESLint 設定調整

在 `frontend/eslint.config.js` 的 `ignores` 中新增 `storybook-static`，避免對 Storybook 建置產物執行 lint（該目錄為第三方產生檔，會產生誤報）。

---

## 2. 專案檢查結果摘要

### 2.1 Rust 後端 (Clippy) — 12 個錯誤

| 檔案 | 問題類型 | 說明 |
|------|----------|------|
| `handlers/auth.rs` | `needless_question_mark` | 多處 `Ok(...?)` 可簡化為直接回傳 |
| `handlers/auth.rs` | `needless_question_mark` | Response builder 的 `Ok(...?)` 可簡化 |
| `handlers/two_factor.rs` | `needless_question_mark` | 同上 |
| `handlers/sse.rs` | `new_without_default` | `AlertBroadcaster::new()` 建議實作 `Default` |
| `handlers/system_settings.rs` | `unnecessary_map_or` | `map_or(false, \|s\| !s.is_empty())` 可改為 `is_some_and(\|s\| !s.is_empty())` |
| `middleware/jwt_blacklist.rs` | `new_without_default` | `JwtBlacklist::new()` 建議實作 `Default` |
| `services/animal/core.rs` | `clone_on_copy` | `status.clone()` 可改為 `*status`（`AnimalStatus` 為 Copy） |
| `services/system_settings.rs` | `if_same_then_else` | if/else 兩分支回傳相同值，可簡化 |

### 2.2 React 前端 (ESLint) — 約 254 個 warnings、16 個 errors

**主要問題類型：**

1. **`@typescript-eslint/no-unused-vars`**：未使用的變數、import、參數（建議以 `_` 前綴標記）
2. **`@typescript-eslint/no-explicit-any`**：使用 `any` 型別，建議改為明確型別
3. **`react-hooks/exhaustive-deps`**：`useEffect` / `useMemo` 依賴陣列不完整
4. **`@typescript-eslint/no-empty-object-type`**：空介面可改為 type alias

**storybook-static 相關 errors**：已透過 `ignores` 排除，不再影響 lint 結果。

---

## 3. 建議後續處理

1. **Rust**：依 clippy 建議逐一修正，可讓 CI 的 `backend-lint` 與 reviewdog 的 clippy 檢查通過。
2. **前端**：可採漸進式修復，優先處理 `error` 等級，再處理 `warning`；部分可透過 `npm run lint -- --fix` 自動修復。
3. **reviewdog 本地執行**：見上方「1.1 本地執行 reviewdog」。

---

## 4. 參考連結

- [reviewdog](https://github.com/reviewdog/reviewdog)
- [giraffate/clippy-action](https://github.com/giraffate/clippy-action)
- [reviewdog/action-eslint](https://github.com/reviewdog/action-eslint)
