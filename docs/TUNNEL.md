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

---

## 錯誤排除（方案二）

| 錯誤 | 處理方式 |
|------|----------|
| `Cannot determine default origin certificate path` | 執行一次 `cloudflared tunnel login`（或讓腳本自動執行） |
| `cert.pem` 不在預設路徑 | 在設定檔加上 `origincert: 你的cert.pem路徑`，或設定環境變數 `TUNNEL_ORIGIN_CERT` |
| `error parsing tunnel ID` | 確認 `deploy/cloudflared-config.yml` 裡的 `tunnel`、`credentials-file` 已改成實際的 ID 與 JSON 路徑 |
