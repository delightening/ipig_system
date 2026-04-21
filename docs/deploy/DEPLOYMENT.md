# iPig 系統部署與維運手冊

> 版本：1.2 | 更新日期：2026-04-13

本手冊說明**正式環境**的系統需求、首次部署、日常維運、監控、故障排除與容器自動更新。  
**與其他文件的關係：**

| 文件 | 用途 |
|------|------|
| [README.md](../../README.md) | 專案總覽、子系統、技術架構、角色權限、API 摘要 |
| [../user/QUICK_START.md](../user/QUICK_START.md) | 本地/測試環境快速啟動（Docker 或本地開發） |
| [../user/USER_GUIDE.md](../user/USER_GUIDE.md) | 使用者操作手冊（登入、AUP、動物管理、ERP） |
| **DEPLOYMENT.md**（本手冊） | 部署、備份、還原、監控、安全性維護、GHCR/Watchtower |

---

## 1. 系統需求

| 項目 | 最低需求 | 建議 | 備註 |
|------|----------|------|------|
| **OS** | Linux x86-64 | Ubuntu 22.04+ / Debian 12+ | DSM 7.2+（Synology）亦可 |
| **CPU** | 2 核 x86-64 | 4 核+ | Rust 編譯多核加速；執行期輕量 |
| **RAM（含 build）** | 8 GB | **16 GB** | Rust build 峰值約 6–7 GB；容器執行約 1–2 GB |
| **RAM（僅執行）** | 2 GB | 4 GB | 使用預建 image，不在機器上 build |
| **磁碟空間** | 50 GB | 250 GB+ | Docker images ~3 GB；另計 DB、logs、照片附件 |
| **Docker** | 24.0+ | 最新穩定版 | |
| **Docker Compose** | 2.20+ | 最新穩定版 | |
| **網路頻寬** | 10 Mbps | 100 Mbps+ | |

> **⚠ RAM 注意**：若直接在伺服器上執行 `docker compose up -d --build`（含 Rust 編譯），RAM 低於 8 GB 可能 OOM 導致 build 失敗。建議策略：在開發機 build 完 push image，伺服器僅執行 `docker compose up -d`（pull image），可將伺服器 RAM 需求降至 2 GB。

---

## 2. 首次部署

### 2.1 取得原始碼

```bash
git clone <repository-url> ipig_system
cd ipig_system
```

### 2.2 設定環境變數

```bash
cp .env.example .env
nano .env
```

**必填項目：**

| 變數 | 說明 | 範例 |
|------|------|------|
| `POSTGRES_PASSWORD` | 資料庫密碼 | 強密碼，≥16 字元 |
| `JWT_SECRET` | JWT 簽名密鑰 | `openssl rand -base64 64` |
| `ADMIN_INITIAL_PASSWORD` | 管理員初始密碼 | 強密碼，≥12 字元 |

**安全相關（正式環境必須設定）：**

| 變數 | 正式環境值 | 說明 |
|------|-----------|------|
| `COOKIE_SECURE` | `true` | 僅 HTTPS 傳送 Cookie |
| `SEED_DEV_USERS` | `false` | 禁用開發帳號 |
| `CORS_ALLOWED_ORIGINS` | 實際域名 | 如 `https://ipig.example.com` |

### 2.3 啟動服務

```bash
# 正式環境（使用 Nginx 靜態服務）
docker compose up -d db api web db-backup

# 開發環境（使用 Vite 熱更新）
docker compose up -d db api web-dev
```

### 2.4 驗證部署

```bash
# 檢查所有容器狀態
docker compose ps

# 健康檢查
curl http://localhost:8080/api/health
# 預期回應：{"status":"healthy","version":"0.1.0","checks":{"database":{"status":"up",...}}}

# 查看 API 日誌
docker compose logs api --tail 20
```

---

## 3. 日常維運

### 3.1 資料庫備份

備份由 `db-backup` 容器自動執行（預設每日凌晨 2:00）。使用 **prod overlay** 時，DB 密碼改由 Docker Secret `db_password` 提供（`POSTGRES_PASSWORD_FILE`），詳見 [scripts/backup/BACKUP.md](scripts/backup/BACKUP.md)。

```bash
# 查看備份日誌
docker compose logs db-backup --tail 20

# 手動觸發備份
docker compose exec db-backup /usr/local/bin/pg_backup.sh

# 列出備份檔案
docker compose exec db-backup ls -lah /backups/
```

**GPG 加密備份**：使用 **prod overlay** 時為**強制**（未設定 `BACKUP_GPG_RECIPIENT` 時備份腳本會 exit 1）。開發/測試為選配。

```bash
# 1. 匯入 GPG 公鑰到備份容器
docker compose exec db-backup gpg --import /path/to/public.key

# 2. 設定 .env 中的收件者
BACKUP_GPG_RECIPIENT=backup@example.com

# 3. 重建容器
docker compose up -d db-backup
```

### 3.2 資料庫還原

```bash
# 從 gzip 備份還原
gunzip -c backup.sql.gz | docker compose exec -T db psql -U postgres ipig_db

# 從 GPG 加密備份還原
gpg --decrypt backup.sql.gz.gpg | gunzip | docker compose exec -T db psql -U postgres ipig_db
```

### 3.3 GeoIP 資料庫更新

```bash
# 設定 MaxMind License Key
export MAXMIND_LICENSE_KEY="your_license_key"

# 執行更新
bash scripts/update_geoip.sh

# 重啟 API 載入新資料
docker compose restart api
```

> MaxMind License Key 取得：https://www.maxmind.com/en/accounts/current/license-key

### 3.4 依賴安全更新

```bash
# 後端 Rust 依賴更新
cd backend && cargo update && cd ..

# 前端 npm 依賴更新
cd frontend && npm update && cd ..

# 重建容器
docker compose build api web
docker compose up -d api web
```

---

## 4. 監控

### 4.1 健康檢查

| 端點 | 方法 | 說明 |
|------|------|------|
| `/api/health` | GET | DB 連通性 + 延遲量測 |

```bash
# 可整合至 Uptime 監控（如 UptimeRobot）
# 200 = healthy，503 = unhealthy
curl -s http://localhost:8080/api/health | jq .
```

### 4.2 日誌

```bash
# 正式環境建議啟用 JSON 日誌格式
RUST_LOG_FORMAT=json  # 在 .env 中設定

# 查看 API 日誌（包含 Request ID）
docker compose logs api --tail 50

# 追蹤即時日誌
docker compose logs -f api
```

### 4.3 Docker 健康檢查

```bash
# PostgreSQL 自帶 healthcheck（3 秒間隔）
docker inspect --format='{{.State.Health.Status}}' ipig-db
```

---

## 5. 故障排除

### 常見問題

| 問題 | 原因 | 解決方案 |
|------|------|---------|
| `502 Bad Gateway` | API 未啟動或 proxy buffer 不足 | `docker compose logs api` 查看錯誤 |
| `health` 回傳 503 | DB 連線失敗 | `docker compose ps db` 確認資料庫運行中 |
| JWT 相關錯誤 | JWT_SECRET 不一致 | 確認 `.env` 中 `JWT_SECRET` 未變更 |
| 打卡 GPS 失敗 | 未設定辦公室座標 | 設定 `CLOCK_OFFICE_LATITUDE/LONGITUDE` |
| 備份加密失敗 | GPG 公鑰未匯入 | 匯入公鑰至 db-backup 容器 |
| GeoIP 更新失敗 | License Key 過期 | 更新 `MAXMIND_LICENSE_KEY` |

### 緊急復原流程

```bash
# 1. 停止服務
docker compose down

# 2. 從備份還原資料庫
gunzip -c /path/to/latest_backup.sql.gz | \
  docker compose exec -T db psql -U postgres ipig_db

# 3. 還原上傳檔案（如有異地備份）
rsync -az user@nas:/backups/ipig/uploads/ ./uploads/

# 4. 重啟服務
docker compose up -d
```

---

## 6. 安全性維護

### 定期任務

| 任務 | 頻率 | 指令 |
|------|------|------|
| 依賴漏洞掃描 | 每週 | CI 自動（cargo audit + npm audit） |
| 容器映像掃描 | push 到 main | CI 自動（Trivy） |
| GeoIP 更新 | 每月 | `bash scripts/update_geoip.sh` |
| 密碼輪換 | 每季 | 修改 `.env` 中密碼/密鑰 |
| 備份還原演練 | 每季 | 從備份還原至測試環境 |

### 密碼/密鑰輪換

```bash
# 1. 更新 JWT_SECRET（會使所有現有 session 失效）
JWT_SECRET=$(openssl rand -base64 64)

# 2. 更新 .env 並重啟
docker compose restart api

# 3. 通知使用者重新登入
```

---

## 7. 容器自動更新（GHCR + Watchtower）

### 7.1 架構概覽

```
Push to main → CI 通過 → CD 建構映像 → 推送至 GHCR
  → Watchtower 偵測新映像（30 秒輪詢）→ 拉取 + 重啟 api/web → 健康檢查 → Email 通知
```

- **CI/CD 分離**：CI（`.github/workflows/ci.yml`）負責檢查，CD（`.github/workflows/cd.yml`）負責建構並推送映像
- **映像來源**：GitHub Container Registry（`ghcr.io/<owner>/ipig-api`、`ghcr.io/<owner>/ipig-web`）
- **自動更新**：Watchtower 僅監控標記 `watchtower.enable=true` 的容器（api、web）
- **DB 安全**：db 和 db-backup 明確排除自動更新

### 7.2 首次設定

```bash
# 執行一鍵設定腳本
bash scripts/deploy/setup-server.sh
```

腳本會自動：
1. 登入 GHCR（需 GitHub PAT，scope: `read:packages`）
2. 將 `GHCR_OWNER`、`IMAGE_TAG` 寫入 `.env`
3. 產生 Watchtower API token
4. 拉取映像並啟動服務

### 7.3 正式環境啟動

使用 **prod overlay** 時，前端 web 服務埠綁定於 **127.0.0.1**（僅本機存取），對外請透過 Nginx、Caddy 或 Cloudflare Tunnel 等反向代理提供服務。

```bash
# 正式環境（使用 GHCR 映像 + Watchtower）
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --no-build

# 開發環境（不變，使用本地建構）
docker compose up -d db api web-dev
```

### 7.4 手動觸發更新

```bash
# 透過 Watchtower HTTP API
curl -H "Authorization: Bearer $WATCHTOWER_API_TOKEN" http://localhost:8090/v1/update
```

### 7.5 回滾

```bash
# 回滾至特定 commit SHA
bash scripts/deploy/rollback.sh <commit-sha>

# 回滾後恢復自動更新
export IMAGE_TAG=latest
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d watchtower
```

### 7.6 健康檢查

```bash
# 手動執行健康檢查
bash scripts/deploy/healthcheck.sh

# API 健康端點
curl http://localhost:8000/api/health
```

### 7.7 注意事項

- **資料庫遷移**：SQLx 遷移在 API 啟動時自動執行。回滾 API 版本不會自動還原遷移，破壞性遷移需手動撰寫補償 SQL。
- **短暫停機**：單一 API 實例重啟約 5-10 秒。如需零停機，可考慮擴展為多副本。
- **Watchtower 通知**：使用現有 SMTP 設定發送部署通知。設定 `.env` 中的 `DEPLOY_NOTIFY_EMAIL`。
