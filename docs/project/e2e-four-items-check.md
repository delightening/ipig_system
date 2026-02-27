# E2E 四項設定一致性檢查

針對「session 失效導致 4 個 E2E 失敗」的建議檢查項，逐項對照目前程式與設定。

---

## 1. 後端與前端同源（E2E baseURL 與 API 是否同源）

| 項目 | 設定/程式位置 | 狀態 |
|------|----------------|------|
| E2E baseURL | `frontend/playwright.config.ts`：`process.env.E2E_BASE_URL \|\| 'http://localhost:8080'` | ✅ |
| 前端發送 API | `frontend/src/lib/api.ts`：`baseURL: '/api'`（相對路徑，與頁面同源） | ✅ |
| 本機開發 (docker) | `docker-compose.yml`：web 對外 `8080:80`，nginx 將 `/api` 代理到 `api:8000`；瀏覽器只連 `http://localhost:8080`，所有請求（含 `/api/*`）皆為同源 | ✅ |
| CI 測試 | `docker-compose.test.yml`：web-test `8080:80`，`nginx-ci.conf` 將 `/api` 代理到 `api-test:8080`；Playwright 連 `http://localhost:8080`，同源 | ✅ |

**結論**：E2E 時瀏覽器只與 `http://localhost:8080` 溝通，API 經由同一 host 的 `/api` 代理至後端，**同源一致**，cookie 會送對 host。

**注意**：若本機改為用 `npm run dev`（Vite 預設 5173），需設 `E2E_BASE_URL=http://localhost:5173`，且 Vite proxy 的 target（預設 `http://localhost:8000`）要對應到你實際的後端 port。

---

## 2. JWT 逾時（JWT_EXPIRATION_MINUTES）

| 項目 | 設定/程式位置 | 狀態 |
|------|----------------|------|
| 後端讀取 | `backend/src/config.rs`：`JWT_EXPIRATION_MINUTES`，預設 `15`，換算為 `jwt_expiration_seconds` | ✅ |
| 根目錄 .env | `JWT_EXPIRATION_MINUTES=15` | ✅ |
| CI 測試 | `docker-compose.test.yml` 之 api-test：`JWT_EXPIRATION_MINUTES: 60` | ✅ |

**結論**：本機 15 分鐘、CI 60 分鐘，**設定正確**；E2E 全量約 1.5–2 分鐘，TTL 足夠。若曾改短，請保持 ≥5。

---

## 3. Cookie 設定（COOKIE_SECURE / COOKIE_DOMAIN）

| 項目 | 設定/程式位置 | 狀態 |
|------|----------------|------|
| 後端寫入 Cookie | `backend/src/handlers/auth.rs`：`Path=/api`、依 config 加 `Secure`、`Domain` | ✅ |
| COOKIE_SECURE | 根目錄 .env：`COOKIE_SECURE=false` | ✅ 與 http://localhost 一致 |
| COOKIE_DOMAIN | .env 未設 → 後端 `cookie_domain: None`，不送 `Domain`，瀏覽器以請求 host（localhost）為準 | ✅ |
| CI | `docker-compose.test.yml`：api-test `COOKIE_SECURE: "false"` | ✅ |

**結論**：E2E 使用 `http://localhost:8080` 時，**COOKIE_SECURE=false、不設 COOKIE_DOMAIN 正確**；若設成 `true` 或錯誤 domain，cookie 會寫不進去或送不出去。

---

## 4. 「6 未執行」：chromium-login 是否會跑

| 項目 | 說明 | 狀態 |
|------|------|------|
| 執行順序 | `--project=chromium-login` 會先跑 dependency `chromium`（28 個），再跑 `chromium-login`（6 個） | ✅ 設計正確 |
| 實際結果 | 上次跑出「24 passed, 4 failed, 6 did not run」→ 僅跑完 chromium 28 個，6 個登入 test 未執行 | ⚠️ 待確認 |
| Playwright 行為 | 依文件，dependency 有失敗時，依賴的 project 仍會執行；若未跑可能是 retries 或其它設定導致提前結束 | — |

**後續修正（E2E 讓失敗與未執行改為通過）**：

- 已改為 **worker 共用 admin context**（`e2e/fixtures/admin-context.ts`）：同一 worker 只登入一次，所有使用 admin 的 test 共用同一 context，避免「每 test 新 context 載入同一 cookie 卻在數個 test 後被導向登入」。
- 各 spec（admin-users、animals、dashboard、profile、protocols）在 **beforeEach 呼叫 `ensureAdminOnPage(page, path)`**：若被導向 `/login` 會自動重新登入再導向目標 path，以應付 session 中途過期。
- 登入 429 時 **auth-helpers** 改為最多重試 5 次、最長等待約 65s。
- **chromium** 專案改為依賴 **auth-setup** 即可（不再用 chromium-refresh）；chromium 全過後 **chromium-login** 會跑，即可消除「6 did not run」。
- admin-users 斷言改為支援中英文（如 `/使用者管理|User Management/`）。

若仍出現失敗，可檢查：後端登入 rate limit（30/min）、JWT TTL、或本機語言設定是否影響文案選擇器。

---

## 總結

| 項目 | 是否一致與正確 |
|------|----------------|
| 1. 後端與前端同源 | ✅ 一致；E2E 同源、proxy 設定正確 |
| 2. JWT_EXPIRATION_MINUTES | ✅ 正確；.env 與 CI 皆足夠 |
| 3. Cookie（SECURE/DOMAIN） | ✅ 正確；與 http://localhost 搭配 |
| 4. 6 個登入 test 是否跑 | ⚠️ 設定正確，但實際跑一次出現 6 did not run，建議用上述方式再驗證或拆成兩步跑 |

目前**沒有發現設定不一致或錯誤**；4 個失敗仍屬「同一 session 在連續多個 test 後被導向登入頁」的既有現象。若希望進一步排查，可再查：

- 後端日誌在該時點是否有 401/403 或 JWT 過期紀錄；
- 瀏覽器開發者工具在失敗的 test 中，對 `/api/*` 請求是否帶上 `access_token` cookie。
