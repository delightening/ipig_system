# Cloudflare Tunnel 設定

專案提供兩種隧道方式，依需求擇一使用。

---

## 方案一：臨時隧道（Quick Tunnel）

**適用**：本機開發、快速對外分享、不需固定網域。  
**特點**：免登入、免設定檔；每次啟動會得到一個新的 `*.trycloudflare.com` 網址，關閉後即失效。

### 啟動

在專案根目錄執行：

```powershell
.\scripts\start_tunnel.ps1
```

預設轉發 **http://localhost:8080**。若本機服務埠號不同，可指定：

```powershell
.\scripts\start_tunnel.ps1 -Port 3000
```

### 腳本會自動做的事

- 啟動 cloudflared，取得臨時 URL（例如 `https://xxxx.trycloudflare.com`）
- 將 URL 複製到剪貼簿
- **更新 email 連結**：寫入 `.env`、`backend\.env` 的 `APP_URL`，並重啟 API 容器，因此**信中的登入／重設密碼／計畫等連結都會指向這次的隧道網址**
- 保持執行直到你按 Ctrl+C 結束

### 注意

- **不需** Cloudflare 帳號或 `cloudflared tunnel login`
- URL **每次重啟都會變**
- 關閉視窗或結束程序後隧道即消失
- **SSE（如安全警報即時推送）**：後端已做立即回應與週期心跳，經 Quick Tunnel 時可避免 Cloudflare 524；tunnel 腳本無需額外設定。

---

## 方案二：固定隧道（Named Tunnel）

**適用**：正式對外服務、固定網域（例如 `ipig.yourdomain.com`）。  
**特點**：需登入 Cloudflare 並完成一次性設定；網址固定、可綁自訂網域。

### 啟動

在專案根目錄執行：

```powershell
.\scripts\start_named_tunnel.ps1
```

- **若尚未登入**：腳本會自動執行 `cloudflared tunnel login`（會開瀏覽器），完成後再啟動隧道。
- **若已登入**：直接啟動隧道。

**Email 連結**：腳本會從 `deploy/cloudflared-config.yml` 讀取 `hostname`，自動寫入 `APP_URL=https://<hostname>` 並重啟 API，信中的連結會使用此網址。

### 一次性設定（僅首次或重裝時）

#### 1. 安裝 cloudflared

若尚未安裝：[Cloudflare Tunnel 下載](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/)

#### 2. 登入（可手動或交給腳本）

- **手動**：`cloudflared tunnel login`
- **或**：直接執行 `.\scripts\start_named_tunnel.ps1`，腳本偵測到未登入會自動跑 login

登入後會在 **`%USERPROFILE%\.cloudflared\cert.pem`** 寫入憑證。

#### 3. 建立 Named Tunnel

```powershell
cloudflared tunnel create ipig-system
```

- 會得到一個 **Tunnel ID**（UUID）
- 憑證會寫入 `%USERPROFILE%\.cloudflared\<TUNNEL_ID>.json`

#### 4. 編輯設定檔

編輯 **`deploy/cloudflared-config.yml`**：

1. 將 `tunnel: <TUNNEL_ID>` 的 `<TUNNEL_ID>` 換成上一步的 UUID
2. 將 `credentials-file` 設為上述 JSON 的實際路徑，例如：
   - `credentials-file: C:\Users\你的帳號\.cloudflared\<TUNNEL_ID>.json`
   - 或把該 JSON 複製到專案內（例如 `deploy/`）再填相對路徑
3. 將 `hostname` 改成你的實際網域（並在 Cloudflare DNS 新增 CNAME 指向該 tunnel）

#### 5. （可選）DNS CNAME

在 Cloudflare DNS 為你的網域新增一筆 CNAME：

- 名稱：例如 `ipig`（子網域）
- 目標：`<TUNNEL_ID>.cfargotunnel.com`

---

## 兩方案比較

| 項目           | 方案一：臨時隧道           | 方案二：固定隧道                 |
|----------------|----------------------------|----------------------------------|
| 腳本           | `.\scripts\start_tunnel.ps1` | `.\scripts\start_named_tunnel.ps1` |
| 登入/設定      | 不需要                     | 需登入 + 編輯 config             |
| 網址           | `*.trycloudflare.com`，每次變 | 固定，可自訂網域                 |
| **Email 連結** | **會**：腳本自動更新 `APP_URL` 並重啟 API | **會**：腳本自動更新 `APP_URL` 並重啟 API（依 config 的 hostname） |
| 適用情境       | 開發、臨時分享             | 正式對外、固定網域               |

**SSE / 長連線**：安全警報即時推送（`/api/admin/audit/alerts/sse`）等 SSE 連線，兩種隧道皆適用；後端已實作立即回應與 30 秒心跳，避免 Cloudflare 524，**無需修改 tunnel 腳本或 config**。

---

## 錯誤排除（方案二）

| 錯誤 | 處理方式 |
|------|----------|
| `Cannot determine default origin certificate path` | 執行一次 `cloudflared tunnel login`（或讓腳本自動執行） |
| `cert.pem` 不在預設路徑 | 在設定檔加上 `origincert: 你的cert.pem路徑`，或設定環境變數 `TUNNEL_ORIGIN_CERT` |
| `error parsing tunnel ID` | 確認 `deploy/cloudflared-config.yml` 裡的 `tunnel`、`credentials-file` 已改成實際的 ID 與 JSON 路徑 |

---

## 常見錯誤（隧道與 API）

透過隧道存取時，若瀏覽器出現下列狀況，可依下表排查：

| 現象 | 可能原因 | 建議作法 |
|------|----------|----------|
| **502 Bad Gateway**（例如 `/assets/xxx.js` 或任意請求） | 隧道連不到本機服務，或本機服務未啟動 | 確認本機 backend / 前端已啟動且埠號與 tunnel 轉發一致；重啟 tunnel 或本機服務後再試 |
| **500 Internal Server Error**（例如 `GET /api/v1/animals?breed=minipig&keyword=00&page=1&per_page=50`） | 後端處理該請求時發生錯誤（DB、decode、邏輯等） | 查看**執行 backend 的終端機或 log**，會印出 `list_animals failed: ... error=...`；依錯誤訊息修正後端或資料 |
| **524**（例如 `/api/admin/audit/alerts/sse`） | Cloudflare 長連線逾時 | 後端已做心跳；若仍 524，可改為輪詢或考慮 Cloudflare 付費方案較長逾時 |

**如何取得 500 的真實原因**：在專案根目錄執行 backend（例如 `cargo run` 或透過 Docker），觸發會 500 的那個 API，後端 log 會出現 `list_animals failed:` 與完整錯誤內容，依此除錯即可。

### 用 curl / PowerShell 帶認證測試動物列表

API 需要登入（Cookie），直接 `curl` 會得到 401。可先登入再帶同一 session 打動物列表：

**PowerShell（先登入，再打動物列表）：**

```powershell
# 請改成你的帳號與密碼（例如 admin@ipig.local 或 DEV_USER_PASSWORD 對應的 dev 帳號）
$email = "admin@ipig.local"
$password = "你的密碼"
$base = "http://localhost:8080/api/v1"

# 登入並保留 Cookie 到 $session
$loginBody = @{ email = $email; password = $password } | ConvertTo-Json
Invoke-RestMethod -Uri "$base/auth/login" -Method POST -Body $loginBody -ContentType "application/json" -SessionVariable session

# 用同一 session 打動物列表（會帶 Cookie）
Invoke-RestMethod -Uri "$base/animals?breed=minipig&keyword=00&page=1&per_page=50" -WebSession $session
```

若第二段回傳 200 且為 JSON 列表，代表本機後端正常；若回傳 500，請看執行 backend 的終端機裡的 `list_animals failed:` 錯誤內容。
