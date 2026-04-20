# SMTP 憑證管理（方案 B：服務獨立 app password）

## 策略

1 個專用 Gmail 服務帳號（非個人信箱），4 組獨立 app password，一服務一組。
任一組洩漏只需撤銷該組，其他服務不受影響。

## 建議的 Gmail 服務帳號

- ❌ 不要用：`jason4617987@gmail.com`（個人帳號）
- ✅ 建議新建：例如 `ipig.sysalert@gmail.com` 或自有網域 `alerts@yourdomain.tld`

## 4 個服務對照表

| # | 服務 | 密碼儲存方式 | 路徑 | 讀取機制 |
|---|------|-------------|------|---------|
| 1 | **Backend**（主 App） | file | `./secrets/smtp_password.txt` | `SMTP_PASSWORD_FILE` env → `read_secret()` |
| 2 | **Alertmanager** | file | `./secrets/alert_smtp_password.txt` | `docker-entrypoint.sh` 啟動時 `cat` |
| 3 | **Grafana** | file | `./secrets/grafana_smtp_password.txt` | `GF_SMTP_PASSWORD__FILE` 原生支援 |
| 4 | **Watchtower** | file | `./secrets/watchtower_smtp_password.txt` | `watchtower-entrypoint.sh` `cat` + export |

所有檔案都在 `secrets/` 目錄，已由 `.gitignore` 保護不進 git。

## 重建流程

### Step 1：建立／登入 Gmail 服務帳號

開啟 [Google Account Apps Passwords](https://myaccount.google.com/apppasswords)。

### Step 2：產生 4 組 app password

每組取一個好記的 label，Google 會顯示 16 字元密碼（含空格，複製時保留空格或全部移除皆可，Gmail 都接受）：

| Label 建議 | 用途 |
|---|---|
| `ipig-backend` | 服務 #1 |
| `ipig-alertmanager` | 服務 #2 |
| `ipig-grafana` | 服務 #3 |
| `ipig-watchtower` | 服務 #4 |

### Step 3：寫入 4 個 secret 檔

⚠️ 檔案內容**不可有尾端換行**（會讓 SMTP auth 失敗）。用 `printf '%s'` 而非 `echo`。

```bash
# 在專案根目錄執行
mkdir -p secrets
printf '%s' 'xxxx xxxx xxxx xxxx' > secrets/smtp_password.txt              # 服務 #1
printf '%s' 'yyyy yyyy yyyy yyyy' > secrets/alert_smtp_password.txt        # 服務 #2
printf '%s' 'zzzz zzzz zzzz zzzz' > secrets/grafana_smtp_password.txt      # 服務 #3
printf '%s' 'wwww wwww wwww wwww' > secrets/watchtower_smtp_password.txt   # 服務 #4

# 限權（僅 owner 可讀）
chmod 600 secrets/*.txt
```

Windows PowerShell（如果不用 WSL）：

```powershell
mkdir -Force secrets | Out-Null
Set-Content -NoNewline -Path secrets/smtp_password.txt            -Value 'xxxx xxxx xxxx xxxx'
Set-Content -NoNewline -Path secrets/alert_smtp_password.txt      -Value 'yyyy yyyy yyyy yyyy'
Set-Content -NoNewline -Path secrets/grafana_smtp_password.txt    -Value 'zzzz zzzz zzzz zzzz'
Set-Content -NoNewline -Path secrets/watchtower_smtp_password.txt -Value 'wwww wwww wwww wwww'
```

### Step 4：更新 `.env` 的非秘密欄位

```bash
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=ipig.sysalert@gmail.com
SMTP_FROM_EMAIL=ipig.sysalert@gmail.com
SMTP_FROM_NAME=iPig System

ALERT_SMTP_HOST=smtp.gmail.com:587
ALERT_SMTP_USER=ipig.sysalert@gmail.com
ALERT_EMAIL_TO=your-inbox@example.com
ALERT_EMAIL_FROM=ipig.sysalert@gmail.com

GRAFANA_SMTP_HOST=smtp.gmail.com:587
GRAFANA_SMTP_USER=ipig.sysalert@gmail.com
```

### Step 5：重啟容器

```bash
docker compose up -d api alertmanager grafana watchtower
```

### Step 6：更新 DB 內的 SMTP 密碼（服務 #1 專用）

Backend 啟動時會優先使用 DB `system_settings` 的值覆蓋 `.env`。
重建後必須同步更新，否則 DB 舊值會被拿來用：

**方法 A（從 Admin UI）**：登入 → 管理員 → 系統設定 → 重填 SMTP 密碼 → 存檔。

**方法 B（SQL）**：
```sql
UPDATE system_settings
SET value = '"xxxx xxxx xxxx xxxx"', updated_at = NOW()
WHERE key = 'smtp_password';
```

### Step 7：驗證

| 服務 | 驗證方式 |
|---|---|
| Backend | 登入頁點「忘記密碼」→ 確認收到重設信 |
| Alertmanager | `docker compose logs alertmanager --tail 50`，無 `SMTP auth` 錯誤 |
| Grafana | 儀表板 → Alerting → Contact points → Test notification |
| Watchtower | `docker compose logs watchtower --tail 20`，無 auth 失敗 |

## 撤銷單一服務密碼

若某服務的 app password 疑似外洩：

1. 到 [apppasswords](https://myaccount.google.com/apppasswords) 找到對應 label 撤銷
2. 該 label 重新產生一組新密碼
3. 只改對應的 `secrets/*.txt` 一個檔案
4. 重啟該服務：`docker compose restart <service-name>`

其他 3 個服務不需動。

## 相關檔案

- [docker-compose.yml](../../docker-compose.yml)（主 compose）
- [docker-compose.monitoring.yml](../../docker-compose.monitoring.yml)（監控 overlay）
- [monitoring/alertmanager/docker-entrypoint.sh](../../monitoring/alertmanager/docker-entrypoint.sh)（Alertmanager 密碼載入）
- [scripts/watchtower-entrypoint.sh](../../scripts/watchtower-entrypoint.sh)（Watchtower 密碼載入）
- [.env.example](../../.env.example)（env 範本）
