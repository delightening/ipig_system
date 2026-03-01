# 專案 Walkthrough 紀錄

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
