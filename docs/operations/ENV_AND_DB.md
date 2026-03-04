# .env 與 DB 設定分工說明

本文說明專案中 **環境變數（.env）** 與 **資料庫（system_settings）** 的正確使用方式，以及何者該填在 .env、何者適合放在 DB。

---

## 一、正確使用 .env

### 1. 基本原則

- **`.env`** 用來放「部署／環境」相關的設定與密鑰，**不**提交到 Git（已在 `.gitignore`）。
- 複製範本：`cp .env.example .env`，再依環境填入實際值。
- 後端啟動時會用 `dotenvy::dotenv().ok()` 載入 `.env`；Docker 也可用 `env_file: .env` 或 `environment:` 注入。

### 2. 適合放在 .env 的內容

| 類型 | 說明 | 範例 |
|------|------|------|
| **連線與密鑰** | 應用啟動時就需要、且不透過 UI 變更 | `DATABASE_URL`、`JWT_SECRET`、`AUDIT_HMAC_KEY`、`SMTP_PASSWORD` |
| **伺服器／基礎設施** | 依部署環境不同而不同 | `HOST`、`PORT`、`APP_URL`、`CORS_ALLOWED_ORIGINS`、`UPLOAD_DIR` |
| **功能開關／預設值** | 依環境（開發/正式）區分 | `COOKIE_SECURE`、`SEED_DEV_USERS`、`TRUST_PROXY_HEADERS`、`JWT_EXPIRATION_MINUTES` |
| **打卡／安全** | 與主機或網路環境綁定 | `ALLOWED_CLOCK_IP_RANGES`、`CLOCK_OFFICE_LATITUDE`、`CLOCK_GPS_RADIUS_METERS` |
| **初始帳號（僅首次）** | 僅在建立 admin 時使用 | `ADMIN_INITIAL_PASSWORD` |
| **測試／E2E** | 僅開發或 CI 使用 | `E2E_BASE_URL`、`E2E_ADMIN_PASSWORD`、`TEST_DATABASE_URL` |

以上都屬於「**部署時或環境級**」的設定，改動時通常會重啟服務或重新部署，因此放在 .env 是正確的。

---

## 二、何者應放在 DB（system_settings）而非僅 .env？

專案採用 **DB-first（資料庫優先）** 的系統設定：管理員可在後台修改的項目會存進 `system_settings` 表，**若 DB 有值則會覆蓋 .env 的對應設定**。

### 1. 目前已由 DB 覆蓋 .env 的項目（SMTP）

發信相關設定可由管理員在「系統設定」中修改，**優先使用 DB，沒有才用 .env**：

- `smtp_host`、`smtp_port`、`smtp_username`、`smtp_password`
- `smtp_from_email`、`smtp_from_name`

因此：

- **.env**：仍建議填寫一組預設 SMTP（例如開發或備援用），或至少讓程式能啟動。
- **DB**：正式環境若希望**不重啟、不改 .env 就改 SMTP**，應在後台「系統設定」裡填寫；存進 DB 後會覆蓋 .env。

其他可存在 `system_settings` 的業務設定（依實作而定）：如 `company_name`、`default_warehouse_id`、`cost_method`、`session_timeout_minutes` 等，屬於**營運／業務參數**，適合放 DB、由管理員在 UI 調整，而不是寫死在 .env。

### 2. 判斷表：放 .env 還是 DB？

| 情境 | 建議 |
|------|------|
| 密鑰、連線字串、僅部署時變更 | **.env** |
| 依環境不同（dev/staging/prod） | **.env** |
| 管理員要在後台改、且不想重啟服務 | **DB（system_settings）** |
| 業務參數（公司名稱、預設倉庫、逾時分鐘數等） | **DB** |
| 目前程式只從 .env 讀、沒有 DB 對應欄位 | 先放 **.env**；若要改由 UI 管理再擴充 DB 與 API |

---

## 三、實務建議

1. **.env 必填項**：至少設定 `DATABASE_URL`、`JWT_SECRET`（≥32 字元）、正式環境的 `ADMIN_INITIAL_PASSWORD`；其餘可參考 `.env.example`。
2. **SMTP**：  
   - 開發：在 .env 設好即可。  
   - 正式：可在 .env 留預設，實際使用以**後台系統設定（DB）**為主，方便更換信箱或密碼而不動部署。
3. **敏感資訊**：一律放在 .env（或 Docker Secrets / `*_FILE`），不要寫進 DB 的明文明細或日誌；DB 裡的 SMTP 密碼會由後端遮罩後再給前端。
4. **不確定時**：若某項是「部署／環境／密鑰」→ 放 .env；若是「營運可調、希望從後台改」→ 放 DB（需後端有對應 key 與解析邏輯）。

---

## 四、相關檔案

- 環境變數讀取：`backend/src/config.rs`（`Config::from_env()`）
- DB 系統設定與 SMTP 解析：`backend/src/services/system_settings.rs`（`resolve_smtp_config`，DB 優先、.env fallback）
- 發信使用解析結果：`backend/src/services/email/mod.rs`（`resolve_smtp`）
- 範本：專案根目錄 `.env.example`
