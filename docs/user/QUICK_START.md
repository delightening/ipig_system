# 快速啟動指南

> 本指南適用於**本地或測試環境**。正式環境、備份還原、監控與維運請見 [../deploy/DEPLOYMENT.md](../deploy/DEPLOYMENT.md)。專案總覽與架構見 [README.md](../../README.md)。  
> **JWT 預設過期**：6 小時（360 分鐘），可於 .env 設定 `JWT_EXPIRATION_MINUTES`。

## 方式 1: Docker Compose（推薦）

### 前置條件
- Docker 與 Docker Compose 已安裝（建議 Docker 24+、Compose 2.20+）
- 啟動 Docker Desktop（Windows/Mac）

### 啟動步驟

```powershell
# 在專案根目錄執行
cp .env.example .env
# 編輯 .env，至少設定：POSTGRES_PASSWORD、JWT_SECRET、ADMIN_INITIAL_PASSWORD

docker compose up -d

# 查看日誌
docker compose logs -f

# 停止服務
docker compose down
```

### 服務入口
| 服務 | 網址 |
|------|------|
| 前端 | http://localhost:8080 |
| API | http://localhost:8000 |
| 資料庫 | localhost:5433 |

### 驗證
```powershell
curl http://localhost:8080/api/health
# 預期：{"status":"healthy",...}
```

---

## 方式 2: 本地開發模式

### 前置條件
- PostgreSQL 已安裝並運行
- 專案根目錄有 `.env`，或 `backend/.env` 含 `DATABASE_URL`、`JWT_SECRET`

### 啟動後端（終端機 1）

```powershell
cd backend
cp env.sample .env
# 編輯 .env 設定資料庫連線
cargo install sqlx-cli
sqlx database create
sqlx migrate run
cargo run
```

**Windows 若出現 `link.exe not found`**：需先載入 MSVC 環境，再執行 cargo：
```powershell
. .\scripts\load-msvc-env.ps1
cd backend
cargo run
```
或使用 `.\scripts\build-backend.ps1` 一鍵編譯。詳見 [WINDOWS_BUILD.md](../ops/WINDOWS_BUILD.md)。

後端預設：http://localhost:3000（或依 .env 的 PORT）

### 啟動前端（終端機 2）

```powershell
cd frontend
npm install
npm run dev
```

前端：http://localhost:5173

---

## 環境變數（必填）

| 變數 | 說明 |
|------|------|
| `POSTGRES_PASSWORD` | 資料庫密碼（≥16 字元建議） |
| `JWT_SECRET` | JWT 簽名密鑰（須安全隨機值） |
| `ADMIN_INITIAL_PASSWORD` | 管理員初始密碼（Docker 首次登入用） |

### 選填但建議設定

| 變數 | 說明 | 預設值 |
|------|------|--------|
| `REDIS_URL` | Redis 連線（Rate Limiter/Session 用）| `redis://127.0.0.1:6379` |
| `TRUST_PROXY` | 是否信任反向代理標頭 | `false` |
| `COOKIE_DOMAIN` | Cookie 網域 | 自動偵測 |
| `TWO_FACTOR_ISSUER` | 2FA TOTP 發行者名稱 | `iPig System` |
| `JWT_EXPIRATION_MINUTES` | JWT 過期時間（分鐘）| `360` |
| `SESSION_TIMEOUT_MINUTES` | Session 逾時（分鐘）| DB 設定優先 |

完整列表請參考 `.env.example` 及 [ENV_AND_DB.md](../ops/ENV_AND_DB.md)。

生成 JWT_SECRET（PowerShell）：
```powershell
$jwt = [Convert]::ToBase64String((1..64 | ForEach-Object { Get-Random -Minimum 0 -Maximum 256 }))
Write-Output $jwt
```

---

## 預設帳號

| 帳號 | 密碼 |
|------|------|
| admin@ipig.local | admin123 |

（正式環境請於 .env 設定 `ADMIN_INITIAL_PASSWORD` 並關閉 `SEED_DEV_USERS`，見 [../deploy/DEPLOYMENT.md](../deploy/DEPLOYMENT.md)。）

---

## 本機跑 Playwright E2E

> **流程參照**：詳見 [dev/e2e/FLOW.md](../dev/e2e/FLOW.md)（CI 與本機流程）、[dev/e2e/README.md](../dev/e2e/README.md)（完整指南）。  
> 本機 admin 首次登入會強制變更密碼，auth setup 會自動完成此流程（變更為 `E2eTest123!`）。

### 前置條件

1. **服務已啟動**：`docker compose up -d`，前端為 http://localhost:8080
2. **環境變數已設定**：`.env` 中包含 `ADMIN_INITIAL_PASSWORD`
3. **JWT TTL 足夠**：`.env` 中 `JWT_EXPIRATION_MINUTES >= 5`（建議 15）

### 配置驗證（建議先執行）

```powershell
cd frontend
npx tsx e2e/scripts/verify-config.ts
```

預期輸出：
```
✅ JWT TTL: JWT_EXPIRATION_MINUTES = 15 分鐘（足夠）
✅ Cookie Secure: COOKIE_SECURE=false（正確）
✅ Admin Password: ADMIN_INITIAL_PASSWORD 已設定
✅ E2E Base URL: E2E_BASE_URL = http://localhost:8080（正確）
✅ 配置驗證完全通過
```

如有 ❌ 或 ⚠️，請參考 [dev/e2e/README.md](../dev/e2e/README.md) 修正。

### 執行測試

- **workers=1**：已固定為單 worker，貼近真實「單一使用者連續操作」。
- **登入 test 最後跑**：各瀏覽器皆拆成「主 project + -login」。完整單一瀏覽器請用：
  - Chromium 34 個：`npx playwright test --project=chromium-login`
  - Firefox 34 個：`npx playwright test --project=firefox-login`
  - WebKit 34 個：`npx playwright test --project=webkit-login`

```powershell
cd frontend
pnpm install --frozen-lockfile
pnpm exec playwright install

# 執行 Chromium 測試（推薦）
pnpm exec playwright test --project=chromium-login

# 或執行所有瀏覽器
pnpm run test:e2e
```

若跑全部瀏覽器：`npx playwright test`（會跑 auth-setup → chromium → chromium-login → firefox → firefox-login → webkit → webkit-login）。

**登入帳密**：會自動從專案根目錄 **`.env`** 讀取：

- 未設 `E2E_ADMIN_*` 時，使用 **`ADMIN_INITIAL_PASSWORD`** 與 **admin@ipig.local** 登入。
- 未設 `E2E_BASE_URL` 時，使用 **http://localhost:8080**。

因此只要 `.env` 有正確的 `ADMIN_INITIAL_PASSWORD`，本機不需再設環境變數即可跑 E2E。若要覆寫，可設 `E2E_ADMIN_EMAIL`、`E2E_ADMIN_PASSWORD`、`E2E_BASE_URL`。

**若登入失敗**，可明確設定 E2E 環境變數後再跑：

```powershell
cd frontend
$env:E2E_USER_EMAIL="admin@ipig.local"; $env:E2E_USER_PASSWORD="AdminIpig2026!"
$env:E2E_ADMIN_EMAIL="admin@ipig.local"; $env:E2E_ADMIN_PASSWORD="AdminIpig2026!"
npx playwright test --project=chromium-login
```

（密碼請與你 `.env` 的 `ADMIN_INITIAL_PASSWORD` 一致。）

**後端 Session（JWT）**：E2E 跑約 1.5–2 分鐘，建議 `.env` 中 `JWT_EXPIRATION_MINUTES` ≥ 5（預設 15 即可）。

**讓 .env 與 DB 的 admin 密碼一致（建議本機／E2E 使用）：**

1. 在專案根目錄 `.env` 中設定：`ADMIN_INITIAL_PASSWORD=<你的密碼>`
2. 執行以下指令，將資料庫內 admin@ipig.local 的密碼同步為同一組：
   ```powershell
   docker compose run --rm api /app/create_admin
   ```
   完成後即可直接執行 `npx playwright test --project=chromium-login`（完整 Chromium 34 個 test）。

開 UI 除錯：`npx playwright test --ui`。

---

## 下一步

- **使用系統**：請參考 [USER_GUIDE.md](USER_GUIDE.md)（登入、AUP、動物管理、ERP）。
- **正式部署與維運**：請參考 [../deploy/DEPLOYMENT.md](../deploy/DEPLOYMENT.md)（系統需求、備份、監控、故障排除、容器自動更新）。
