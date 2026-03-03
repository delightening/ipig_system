# E2E 測試完整指南

本文檔包含 ipig_system 專案 Playwright E2E 測試的完整說明，包括架構設計、配置檢查清單、故障排除指南和維護手冊。

---

## 📑 目錄

1. [E2E 流程說明](#e2e-流程說明)
2. [架構說明](#架構說明)
3. [配置檢查清單](#配置檢查清單)
4. [故障排除指南](#故障排除指南)
5. [維護指南](#維護指南)

---

# E2E 流程說明

> **CI 與本機差異**：GitHub Actions 使用 `docker-compose.test.yml`，API 設定 `CI=true` 時 admin 不需強制變更密碼；本機使用 `docker compose` 時 admin 首次登入會被導向 `/force-change-password`。Auth setup 會自動處理兩者差異。  
> **流程參考**：`docs/e2e/FLOW.md`（CI 可參照）

## Auth Setup 執行順序

```
1. authenticate as admin   → 登入 admin@ipig.local，完成 force-change（若需要），儲存 admin.json
2. authenticate as user    → 登入（使用 E2E_USER_* 或 admin），完成 force-change（若需要），儲存 user.json
```

**為何 admin 先於 user？**  
若兩者皆使用 admin 帳號，admin setup 完成 force-change 後密碼變為 `E2eTest123!`，user setup 須使用新密碼才能登入成功。

## 密碼與 force-change

| 環境 | admin 初始密碼 | 是否強制變更 | 變更後密碼 |
|------|----------------|--------------|------------|
| **CI** | `ci_test_admin_password_2024`（docker-compose.test.yml） | 否（`CI=true`） | — |
| **本機 fresh** | `ADMIN_INITIAL_PASSWORD` | 是 | `E2eTest123!` |
| **本機重跑** | 已變更為 `E2eTest123!` | 否 | — |

**Admin 登入重試**：auth setup 先嘗試 `ADMIN_INITIAL_PASSWORD`，若回傳 400（密碼已變更），則改用 `E2E_ADMIN_PASSWORD` 或 `E2eTest123!`。

## 關鍵程式與環境變數

| 程式 | 說明 |
|------|------|
| `auth.setup.ts` | 執行 admin → user 兩階段登入，呼叫 `completeForceChangePassword()` |
| `auth-helpers.ts` | `performLogin`、`completeForceChangePassword`、`getAdminCredentials`、`getCredentialsForUserSetup` |
| `getCredentialsForUserSetup` | user setup 與 login.spec 使用，回傳 admin 時密碼為 `E2E_ADMIN_PASSWORD \|\| E2eTest123!` |

**環境變數**（見 [配置檢查清單](#配置檢查清單)）：
- `ADMIN_INITIAL_PASSWORD`：Docker seed 與本機 admin 初始密碼
- `E2E_ADMIN_PASSWORD`：覆寫用，本機 force-change 後可設 `E2eTest123!`
- `E2E_BASE_URL`：預設 `http://localhost:8080`
- `E2E_USER_EMAIL`、`E2E_USER_PASSWORD`：user setup 專用（未設則 fallback admin）

## CI 執行流程（GitHub Actions）

```
1. cp .env.example .env
2. docker compose -f docker-compose.test.yml build
3. docker compose -f docker-compose.test.yml up -d --wait --wait-timeout 300
4. 確認 http://localhost:8080 與 http://localhost:8000/api/health 可連
5. cd frontend && npm ci && npx playwright install --with-deps
6. npm run test:e2e
   - env: E2E_BASE_URL, E2E_USER_EMAIL, E2E_USER_PASSWORD, E2E_ADMIN_EMAIL, E2E_ADMIN_PASSWORD
7. (failure) 上傳 playwright-report
8. docker compose -f docker-compose.test.yml down
```

CI 環境變數需與 `docker-compose.test.yml` 一致（`ci_test_admin_password_2024`）。

---

# 架構說明

## 📋 概述

本節說明 ipig_system 專案的 Playwright E2E 測試架構設計，特別是 **1 Worker + Shared Context** 的設計原則與實作細節。

## 🎯 設計原則

### 1. 為什麼使用 1 Worker？

**配置**（`playwright.config.ts:26-28`）：
```typescript
fullyParallel: false,
workers: 1,
```

**原因**：
- ✅ **模擬真實使用情境**：單一使用者登入後連續操作多個頁面，使用同一個活躍的 session
- ✅ **避免並行衝突**：多 worker 會導致同一帳號的多個 session 並行執行，可能觸發後端限制或 rate limiting
- ✅ **簡化 session 管理**：單一 worker 內的測試順序執行，session 狀態更可預測
- ✅ **符合專案需求**：後端允許 `MAX_SESSIONS_PER_USER=5`，但 E2E 測試不需要測試多 session 並行場景

**與真實操作的對照**：
| 面向 | 真實操作 | E2E（1 worker） | E2E（多 worker） |
|------|----------|-----------------|------------------|
| Session 數量 | 1 個 | 1 個 | N 個（同帳號） |
| 操作模式 | 連續 | 連續 | 並行 |
| Session 壓力 | 低 | 低 | 高 |
| Rate Limit 風險 | 低 | 低 | 高 |

### 2. 為什麼使用 Shared Context？

**實作**（`frontend/e2e/fixtures/admin-context.ts`）：
```typescript
export const test = base.extend<{}, { sharedAdminContext: BrowserContext }>({
    sharedAdminContext: [
        async ({ browser }, use) => {
            const ctx = await browser.newContext({ ... })
            // 登入一次
            const page = await ctx.newPage()
            await performLogin(page, email, password)
            await page.close()
            await use(ctx)
            await ctx.close()
        },
        { scope: 'worker' },  // 關鍵：worker 級別
    ],
    page: [
        async ({ sharedAdminContext }, use) => {
            const page = await sharedAdminContext.newPage()
            await use(page)
            await page.close()
        },
        { scope: 'test' },  // 每個測試新 page
    ],
})
```

**關鍵特性**：
- ✅ **Worker 級別共享**：同一 worker 內所有測試共享同一個 browser context
- ✅ **登入一次**：Worker 啟動時登入一次，之後所有測試都用這個已認證的 context
- ✅ **每測試新 page**：每個測試獲得新的 page，但 cookies/session 相同
- ✅ **避免 session 衝突**：不會出現「多個 context 載入同一 cookie 導致 session 失效」的問題

**生命週期**：
```
Worker 啟動
  ↓
建立 sharedAdminContext（登入一次）
  ↓
測試 1：新 page（共享 context）
測試 2：新 page（共享 context）
...
測試 N：新 page（共享 context）
  ↓
Worker 結束（關閉 context）
```

## 📊 Session 管理策略

### Session 生命週期

1. **Auth Setup**（`auth-setup` project）
   - 以 `admin@ipig.local` 登入
   - 將 cookie 儲存至 `frontend/e2e/.auth/admin.json`
   - **用途**：供 opt-in 的 firefox/webkit projects 使用（它們使用 storageState 而非 shared context）

2. **Chromium 主測試**（`chromium` project）
   - 使用 `sharedAdminContext` fixture
   - 忽略 `auth.setup.ts`、`login.spec.ts`
   - Worker 啟動時登入一次，所有測試共享

3. **登入測試**（`chromium-login` project）
   - 依賴 `chromium` project 完成後執行
   - 使用**空 storage**（不載入 cookie）
   - 每個登入測試都是「真實登入一次」

### Session 隔離策略

| Project | Session 來源 | 隔離方式 | 執行順序 |
|---------|-------------|----------|----------|
| auth-setup | 登入並存檔 | 獨立 project | 1 |
| chromium | Worker 級共享 context | 所有測試共享同一 session | 2 |
| chromium-login | 空 storage | 每測試獨立登入 | 3（最後） |
| firefox (opt-in) | storageState: admin.json | 每測試新 context，相同 cookie | 2 |
| webkit (opt-in) | storageState: user.json | 每測試新 context，相同 cookie | 2 |

> **跨瀏覽器 opt-in**：Firefox/WebKit 預設不啟用。需設 `PLAYWRIGHT_FIREFOX=1` / `PLAYWRIGHT_WEBKIT=1` 才會跑。
> 原因：workers=1 序列執行 100 tests 需 ~2 分鐘，storageState JWT 容易過期導致大量失敗。

**為什麼登入測試最後跑？**
- 避免「第二次登入踢掉第一個 session」（儘管後端允許多 session）
- 更接近真實情境：使用者先完成各項操作，最後才測試登入流程本身
- 減少 session 狀態干擾

## 🔧 配置清單

### 前端配置

**Playwright 配置**（`playwright.config.ts`）：
- `workers: 1`：單一 worker
- `fullyParallel: false`：禁用完全並行
- `timeout: 30_000`：預設 30 秒超時
- `baseURL: http://localhost:8080`：與後端同源

**環境變數**：
- `E2E_BASE_URL`：預設 `http://localhost:8080`
- `E2E_ADMIN_EMAIL`：測試用 admin 帳號（預設 `admin@ipig.local`）
- `E2E_ADMIN_PASSWORD`：測試用 admin 密碼（預設讀取 `.env` 中的 `ADMIN_INITIAL_PASSWORD`）

### 後端配置

**JWT 配置**（`.env`）：
- `JWT_EXPIRATION_MINUTES=15`：本機建議 >= 15 分鐘
- `JWT_EXPIRATION_MINUTES=60`：CI 環境（`docker-compose.test.yml`）

**Cookie 配置**（`.env`）：
- `COOKIE_SECURE=false`：本機開發必須為 false（使用 http）
- `COOKIE_DOMAIN`：未設定（使用請求 host，正確）
- `COOKIE_SAME_SITE=Lax`：預設值

**Session 配置**（後端 Rust）：
- `MAX_SESSIONS_PER_USER=5`：允許同帳號多 session（不會踢掉舊 session）

### CI 配置

**GitHub Actions**（`.github/workflows/ci.yml`）：
- 使用 `docker-compose.test.yml`
- JWT_EXPIRATION_MINUTES=60（足夠跑完所有測試）
- SEED_DEV_USERS=true（自動建立測試帳號）

## 📂 檔案結構

```
docs/e2e/
├── FLOW.md               # E2E 流程參考（CI 參照用）
├── README.md             # 本文件（完整指南）

frontend/e2e/
├── fixtures/
│   └── admin-context.ts        # Worker 級 shared context
├── helpers/
│   ├── auth-helpers.ts         # 登入邏輯與帳密讀取
│   ├── diagnostics.ts          # 診斷工具（新增）
│   └── session-monitor.ts      # Session 監控（新增）
├── scripts/
│   └── verify-config.ts        # 配置驗證腳本（新增）
├── .auth/
│   ├── admin.json              # Auth setup 產生的 admin cookie
│   └── user.json               # Auth setup 產生的 user cookie
├── auth.setup.ts               # Auth project（登入並存檔）
├── login.spec.ts               # 登入測試（使用空 storage）
├── animals.spec.ts             # 動物列表測試（使用 admin context）
├── dashboard.spec.ts           # Dashboard 測試（使用 admin context）
├── profile.spec.ts             # 個人資料測試（使用 admin context）
└── protocols.spec.ts           # 計畫書測試（使用 admin context）

playwright.config.ts            # Playwright 全局配置
```

## 🛡️ 安全與限制

### Session 安全

- ✅ **測試帳號隔離**：使用專用的測試帳號（`admin@ipig.local`），不使用生產帳號
- ✅ **本機限定**：E2E 測試僅在本機或 CI 環境執行，不對外暴露
- ✅ **Cookie 加密**：後端使用 JWT 簽名，cookie httpOnly

### 已知限制

1. **JWT TTL 需求**：
   - 本機：建議 >= 15 分鐘（E2E 全量約 1.5-2 分鐘，需預留緩衝）
   - CI：建議 >= 60 分鐘（CI 環境可能較慢）
   - **若 TTL 過短**：測試執行到一半 session 失效，被導向登入頁

2. **單一 worker 的代價**：
   - ❌ **執行時間較長**：測試順序執行，無法透過並行加速
   - ✅ **可接受**：目前 35 個測試約 1.5-2 分鐘，可接受範圍

3. **不適用場景**：
   - ❌ 測試「同帳號多 session 並行」場景（需改用多 worker + 獨立帳號）
   - ❌ 測試高並行負載（應使用壓力測試工具，非 E2E）

## 🔄 與其他測試的差異

### E2E vs Unit Tests

| 面向 | Unit Tests | E2E Tests |
|------|-----------|-----------|
| 範圍 | 單一函式/組件 | 完整使用者流程 |
| 執行環境 | Node.js | 真實瀏覽器 |
| Session | 無 | 真實 session |
| 速度 | 快（毫秒） | 慢（秒） |
| 並行 | 完全並行 | 單一 worker |

### E2E vs 手動測試

| 面向 | 手動測試 | E2E Tests |
|------|---------|-----------|
| 執行者 | 人工 | 自動化 |
| 一致性 | 低（可能遺漏步驟） | 高（完全相同） |
| 速度 | 慢（數分鐘） | 快（約 2 分鐘） |
| 可重現性 | 低 | 高 |
| Session 模式 | 單一 session | 單一 session（相同） |

## 🎓 架構總結

ipig_system 的 E2E 測試採用 **1 Worker + Shared Context** 設計，核心目標是：

✅ **模擬真實使用情境**：單一使用者登入後連續操作
✅ **避免 session 衝突**：Worker 級別共享 context，登入一次用到底
✅ **簡化 session 管理**：不需要處理多 session 並行的複雜性
✅ **提高測試穩定性**：session 狀態可預測，減少 flaky tests

---

# 配置檢查清單

## 執行前檢查

### 本機環境

- [ ] `.env` 中 `JWT_EXPIRATION_MINUTES >= 5`（建議 15）
- [ ] `.env` 中 `COOKIE_SECURE=false`
- [ ] `.env` 中 `ADMIN_INITIAL_PASSWORD` 已設定
- [ ] Docker containers 正常運行：`docker compose ps`
- [ ] 後端健康檢查通過：`curl http://localhost:8080/api/health`
- [ ] 前端可訪問：`curl http://localhost:8080`

### CI 環境

- [ ] `docker-compose.test.yml` 中 `JWT_EXPIRATION_MINUTES=60`
- [ ] `SEED_DEV_USERS=true`
- [ ] `ADMIN_INITIAL_PASSWORD` 與測試帳密一致

## 快速驗證

### 1. 配置驗證腳本

```powershell
cd frontend
npx tsx e2e/scripts/verify-config.ts
```

預期輸出：
```
=== E2E 配置驗證結果 ===

✅ JWT TTL: JWT_EXPIRATION_MINUTES = 15 (足夠)
✅ Cookie Secure: COOKIE_SECURE=false (正確)
✅ Admin Password: ADMIN_INITIAL_PASSWORD 已設定
✅ E2E Base URL: E2E_BASE_URL = http://localhost:8080

✅ 配置驗證通過
```

### 2. Docker 環境檢查

```powershell
# 檢查 containers 狀態
docker compose ps

# 檢查後端健康
curl http://localhost:8080/api/health

# 檢查前端
curl -I http://localhost:8080
```

### 3. 手動配置檢查

```powershell
# 檢查 JWT TTL
grep JWT_EXPIRATION_MINUTES .env

# 檢查 Cookie 設定
grep COOKIE_SECURE .env

# 檢查 Admin 密碼
grep ADMIN_INITIAL_PASSWORD .env
```

---

# 故障排除指南

## 常見問題

### 1. Session 失效（被導向登入頁）

**症狀**：測試中途失敗，錯誤訊息顯示 `Expected not to have URL /login`

**可能原因**：
- JWT TTL 過短
- Cookie 未正確設定
- 網路請求超時導致 refresh 失敗

**排查步驟**：

1. **檢查 JWT TTL**
   ```powershell
   grep JWT_EXPIRATION_MINUTES .env
   ```
   應該 >= 5（建議 15）

2. **檢查 Cookie**
   - 開啟瀏覽器開發者工具 > Application > Cookies
   - 確認 `access_token` cookie 存在且有值
   - 檢查 `Domain` 和 `SameSite` 屬性

3. **檢查後端日誌**
   ```powershell
   docker compose logs api | Select-String "401|JWT|expired" | Select -Last 20
   ```

4. **執行配置驗證**
   ```powershell
   cd frontend
   npx tsx e2e/scripts/verify-config.ts
   ```

**解決方法**：

- **增加 JWT TTL**：
  ```env
  JWT_EXPIRATION_MINUTES=15  # 本機
  JWT_EXPIRATION_MINUTES=60  # CI
  ```

- **確認 Cookie 設定**：
  ```env
  COOKIE_SECURE=false  # 本機開發
  # COOKIE_DOMAIN 應留空
  ```

- **檢查同源政策**：
  - 確認 `E2E_BASE_URL` 與後端 API 同源
  - 預設都是 `http://localhost:8080`

### 2. Networkidle 超時

**症狀**：測試超時，錯誤訊息 `Timeout 30000ms exceeded waiting for load state 'networkidle'`

**可能原因**：
- 頁面有輪詢請求（React Query refetchInterval）
- 多個並行 API 請求
- 後端響應慢

**排查步驟**：

1. **檢查是否使用 networkidle**
   ```powershell
   cd frontend/e2e
   Select-String "networkidle" *.spec.ts
   ```

2. **檢查 React Query 配置**
   - 查找 `refetchInterval` 設定
   - 確認沒有不必要的輪詢

3. **使用 Playwright Inspector 查看網路請求**
   ```powershell
   npx playwright test --debug
   ```

**解決方法**：

**移除 networkidle，改用元素等待**：

```typescript
// ❌ 不好：依賴 networkidle
await page.waitForLoadState('networkidle')

// ✅ 好：明確的元素等待
await page.waitForLoadState('domcontentloaded')
await expect(page.locator('.key-element')).toBeVisible()
```

**等待特定 API 響應**：

```typescript
// 等待特定 API 完成
await page.waitForResponse(
  resp => resp.url().includes('/api/animals') && resp.status() === 200,
  { timeout: 10_000 }
)
```

**已修復**（2024）：
- `auth-helpers.ts` 中的 `performLogin()` 已將 `networkidle` 改為 `domcontentloaded` + 元素等待
- 所有測試檔案（`animals.spec.ts`、`profile.spec.ts`、`protocols.spec.ts`）已使用 `ensureAdminOnPage()`，它使用 `domcontentloaded` 而非 `networkidle`

### 3. 登入測試未執行

**症狀**：報告顯示 `6 did not run`

**可能原因**：
- Project dependencies 設定錯誤
- 前置測試失敗導致跳過
- Playwright 版本不相容

**排查步驟**：

1. **檢查 playwright.config.ts**
   ```typescript
   {
     name: 'chromium-login',
     dependencies: ['chromium'],  // 確認依賴正確
     testMatch: /login\.spec\.ts/,
   }
   ```

2. **單獨執行登入測試**
   ```powershell
   npx playwright test e2e/login.spec.ts --project=chromium-login
   ```

3. **查看 HTML 報告**
   ```powershell
   npx playwright show-report
   ```
   檢查為何測試被跳過

**解決方法**：

**分步執行**：

```powershell
# 先執行 chromium
npx playwright test --project=chromium

# 再執行 chromium-login
npx playwright test --project=chromium-login
```

**完整執行**：

```powershell
# 一次執行所有 projects（依賴鏈會自動處理）
npm run test:e2e
```

### 4. Rate Limit (429) 錯誤

**症狀**：測試失敗，錯誤訊息包含 `429 Too Many Requests`

**可能原因**：
- 測試執行過快，觸發後端 rate limiting
- 登入請求頻率過高

**排查步驟**：

1. **檢查後端日誌**
   ```powershell
   docker compose logs api | Select-String "429|rate"
   ```

2. **檢查登入重試邏輯**
   - `auth-helpers.ts` 中應有 429 處理
   - 檢查 `Retry-After` header

**解決方法**：

`auth-helpers.ts` 已內建 429 處理：

```typescript
// 已實作：429 時自動重試（含隨機化）
if (loginResponse.status() === 429) {
  const retryAfter = parseInt(loginResponse.headers()['retry-after'] || '60')
  // 增加隨機化，避免所有重試同時發生（jitter: ±20%）
  const jitter = Math.random() * 0.4 - 0.2 // -0.2 到 +0.2
  const waitMs = Math.min(
    Math.floor((retryAfter * 1000 + 2000) * (1 + jitter)),
    65_000
  )
  await page.waitForTimeout(waitMs)
  // 重試...
}
```

**已改善**（2024）：
- 重試間隔加入隨機化（jitter），避免多個重試同時發生
- 根據 `Retry-After` header 動態調整等待時間
- 增加詳細的錯誤日誌，方便診斷

如果仍然失敗，可以：
- 增加重試次數（預設 5 次）
- 檢查後端 rate limit 設定（CI 環境建議 >= 100/min）

### 5. Session 過期導致大量登入與 429 連鎖失敗

**症狀**：測試執行時頻繁出現 `Login rate limited (429) after N retries`，或 `ensureLoggedIn()` 觸發大量重新登入後 Fixture setup timeout (30s)

**根因鏈**：
```
Session 過期（測試頻繁誤判）
    ↓
ensureLoggedIn() 觸發重新登入
    ↓
大量登入請求
    ↓
觸發後端 429 Rate Limiting
    ↓
Fixture setup timeout (30s)
```

**可能原因**：
- `isSessionExpired()` 過於敏感，誤判 cookie 已過期
- `context.cookies()` 在 worker 級 fixture 下讀取不穩定
- JWT TTL 過短，測試執行期間 token 真的過期
- 多個測試同時觸發 `ensureLoggedIn()` 導致登入請求集中

**排查步驟**：

1. **檢查 JWT TTL 是否足夠**
   ```powershell
   grep JWT_EXPIRATION_MINUTES .env
   # 本機建議 >= 15，CI 建議 >= 60
   ```

2. **檢查後端日誌中的 401/429**
   ```powershell
   docker compose logs api | Select-String "401|429|JWT|expired" | Select -Last 30
   ```

3. **執行配置驗證**
   ```powershell
   cd frontend
   npx tsx e2e/scripts/verify-config.ts
   ```

**緩解方法**：
- 增加 JWT TTL（CI 環境建議 60 分鐘）
- 確認後端 auth rate limit 已放寬（如 100/min）供 E2E 使用
- 檢查 `admin-context.ts` 中 `isSessionExpired()` 與 `context.cookies()` 的 URL 參數設定
- 若持續發生，需深入調查 shared context 的 cookie 持久化機制（見 `docs/TODO.md` P4 任務）

**已改善**（2024）：
- **Session 有效性緩存**：`isSessionExpired()` 使用 30 秒緩存，減少不必要的 cookie 讀取
- **防抖機制**：`ensureLoggedIn()` 增加防抖鎖定（2 秒窗口），避免多個測試同時觸發登入
- **容錯處理**：Cookie 讀取失敗時使用緩存或保守假設，避免誤判
- **詳細日誌**：增加登入過程的日誌輸出，方便追蹤問題

**實作細節**：
- Session 緩存：30 秒 TTL，減少重複檢查
- 防抖窗口：2 秒，避免並發登入請求
- Cookie 讀取容錯：失敗時使用緩存或保守假設

## 調試技巧

### 使用 Playwright Inspector

```powershell
# 開啟 Inspector 逐步執行
npx playwright test --debug

# 調試特定測試
npx playwright test e2e/animals.spec.ts --debug
```

### 查看詳細日誌

```powershell
# 顯示詳細 API 日誌
$env:DEBUG="pw:api"
npx playwright test

# 清除環境變數（執行後）
Remove-Item Env:\DEBUG
```

### 保留測試產物

```powershell
# Headed 模式 + 慢動作
npx playwright test --headed --slowMo=500

# 保留截圖和影片
npx playwright test --reporter=html
npx playwright show-report
```

### 分析後端日誌

```powershell
# 查看最近 5 分鐘的 API 日誌
docker compose logs api --since 5m

# 過濾特定錯誤
docker compose logs api | Select-String "401|JWT|expired|session"

# 統計 API 請求
docker compose logs api --since 5m | Select-String "GET|POST|PUT|DELETE" | Group-Object | Sort-Object Count -Descending
```

---

# 維護指南

## 日常維護

### 每週檢查

- [ ] 執行完整測試套件確認通過率 >= 95%
- [ ] 檢查是否有新的 flaky tests（連續 5 次執行）
- [ ] 更新 Playwright 到最新穩定版（如有需要）

```powershell
# 每週執行
cd frontend
npm run test:e2e

# 檢查通過率
# 目標：>= 33/35 通過（95%）
```

### 每月檢查

- [ ] 審查測試覆蓋率（是否有新功能未加 E2E 測試）
- [ ] 清理過時或重複的測試
- [ ] 更新本文檔（如有架構變更）

### 新增測試時

- [ ] 遵循現有的 fixture 模式（使用 admin-context）
- [ ] 避免使用 `networkidle`
- [ ] 使用元素等待而非固定 `waitForTimeout`
- [ ] 加入適當的 `data-testid`（方便定位元素）
- [ ] 測試通過後連續執行 3 次確認穩定性

## 最佳實踐

### ✅ 推薦做法

1. **使用 Shared Context**
   ```typescript
   // 使用 admin-context fixture
   import { test, expect } from './fixtures/admin-context'
   ```

2. **元素等待策略**
   ```typescript
   // 好：明確的元素等待
   await page.waitForLoadState('domcontentloaded')
   await expect(page.locator('.key-element')).toBeVisible({ timeout: 10_000 })
   ```

3. **使用 data-testid**
   ```typescript
   // 前端元件
   <button data-testid="submit-button">提交</button>

   // 測試
   await page.getByTestId('submit-button').click()
   ```

4. **等待 API 響應**
   ```typescript
   await page.waitForResponse(
     resp => resp.url().includes('/api/...') && resp.status() === 200
   )
   ```

5. **設定合理的 timeout**
   ```typescript
   // 一般操作：10 秒
   await expect(element).toBeVisible({ timeout: 10_000 })

   // 慢速操作（登入、API）：15-20 秒
   await performLogin(page, email, password) // 內建 retry
   ```

### ❌ 避免做法

1. **不要使用 networkidle**
   ```typescript
   // ❌ 不好：容易超時
   await page.waitForLoadState('networkidle')
   ```

2. **不要使用固定 waitForTimeout**
   ```typescript
   // ❌ 不好：不可靠且浪費時間
   await page.waitForTimeout(5000)

   // ✅ 好：等待特定條件
   await expect(element).toBeVisible()
   ```

3. **不要在測試間共享狀態**
   ```typescript
   // ❌ 不好：測試間共享變數
   let sharedData: any

   test('test 1', () => {
     sharedData = { ... }
   })

   test('test 2', () => {
     // 依賴 test 1 的 sharedData
   })
   ```

4. **不要依賴測試執行順序**
   ```typescript
   // ❌ 不好：假設 test 1 先執行
   test('test 2', () => {
     // 假設 test 1 已建立資料
   })

   // ✅ 好：每個測試獨立
   test.beforeEach(async ({ page }) => {
     // 建立測試所需資料
   })
   ```

## Session 管理原則

### Worker 級別 Fixture

```typescript
// sharedAdminContext: worker 啟動時登入一次
export const test = base.extend<{}, { sharedAdminContext: BrowserContext }>({
  sharedAdminContext: [
    async ({ browser }, use) => {
      // 登入邏輯
    },
    { scope: 'worker' },  // 關鍵
  ],
})
```

**優點**：
- ✅ 所有測試共享同一 session
- ✅ 只登入一次，減少測試時間
- ✅ 更接近真實使用情境

### Test 級別 Fixture

```typescript
// page: 每個測試獲得新 page
page: [
  async ({ sharedAdminContext }, use) => {
    const page = await sharedAdminContext.newPage()
    await use(page)
    await page.close()
  },
  { scope: 'test' },  // 每個測試新 page
],
```

**優點**：
- ✅ 測試間隔離（不同 page）
- ✅ 共享認證（相同 context/session）

### 注意事項

- JWT TTL 應足夠覆蓋整個測試 suite
  - 本機：>= 15 分鐘
  - CI：>= 60 分鐘

- 避免在測試中主動登出
  - 會影響後續測試（因為共享 context）

- 登入測試使用空 storage
  - 不影響其他測試的 session

## 升級指南

### 升級 Playwright

```powershell
cd frontend

# 升級到最新版本
npm install -D @playwright/test@latest

# 安裝瀏覽器
npx playwright install

# 驗證版本
npx playwright --version
```

### 升級後驗證

```powershell
# 1. 執行配置驗證
npx tsx e2e/scripts/verify-config.ts

# 2. 執行測試套件
npm run test:e2e

# 3. 檢查報告
npx playwright show-report
```

### 升級檢查清單

- [ ] Playwright 版本升級成功
- [ ] 瀏覽器安裝完成
- [ ] 配置驗證通過
- [ ] 所有測試通過（>= 95%）
- [ ] 無新的 deprecation warnings
- [ ] 更新 package.json 和 package-lock.json

---

## 📚 參考資料

- [Playwright 官方文檔](https://playwright.dev/docs/intro)
- [Playwright API Reference](https://playwright.dev/docs/api/class-playwright)
- [專案文檔 - e2e-vs-real-usage-discussion.md](../project/e2e-vs-real-usage-discussion.md)
- [專案文檔 - e2e-four-items-check.md](../project/e2e-four-items-check.md)

---

## 📞 支援

如遇問題，請：

1. 查看本文檔的「故障排除指南」
2. 執行配置驗證腳本：`npx tsx e2e/scripts/verify-config.ts`
3. 查看 Playwright HTML 報告：`npx playwright show-report`
4. 檢查後端日誌：`docker compose logs api`
5. 在專案 Issue 追蹤系統報告問題

---

最後更新：2026-02-27
