# iPig System — VPS Cheatsheet

## 首次部署 Checklist

```bash
# 1. 建立 secrets 目錄並填入必要檔案
mkdir -p secrets

# JWT 金鑰
openssl ecparam -name prime256v1 -genkey -noout \
  | openssl pkcs8 -topk8 -nocrypt > secrets/jwt_ec_private_key.pem
openssl ec -in secrets/jwt_ec_private_key.pem -pubout > secrets/jwt_ec_public_key.pem

# DB 密碼 & URL
echo "your_strong_db_password" > secrets/db_password.txt
echo "postgres://postgres:your_strong_db_password@db:5432/ipig_db" > secrets/db_url.txt

# SMTP 密碼（Gmail App Password）
echo "your_gmail_app_password" > secrets/smtp_password.txt

# Grafana Postgres readonly 密碼（CREATE USER grafana_readonly PASSWORD 的密碼）
echo "your_grafana_pg_password" > secrets/grafana_pg_password.txt

# Alertmanager webhook token（與 .env ALERTMANAGER_WEBHOOK_TOKEN 相同）
echo "your_webhook_token" > secrets/alertmanager_webhook_token.txt

# METRICS_TOKEN（Prometheus 抓 /metrics 的 Bearer token）
# 若留空，/metrics 無認證（僅限 Docker 內網）
openssl rand -hex 32 > secrets/metrics_token.txt
# 同步填入 .env: METRICS_TOKEN=（與 metrics_token.txt 內容相同）
```

---

## Stack 架構

```
Nginx (反向代理)
  ├── :8080 → ipig-web    (React frontend)
  ├── :8000 → ipig-api    (Rust backend)
  ├── :9090 → Prometheus
  ├── :3001 → Grafana
  └── :9093 → Alertmanager

Docker Networks:
  frontend  → web ↔ browser
  backend   → api, web, prometheus, grafana, loki
  database  → api, db, db-backup, grafana
```

---

## Compose 檔組合

| 環境 | 指令 |
|------|------|
| **本機開發** | `docker compose up -d` |
| **本機 + Loki** | `docker compose -f docker-compose.yml -f docker-compose.logging.yml up -d` |
| **本機 + Monitoring** | `docker compose -f docker-compose.yml -f docker-compose.monitoring.yml up -d` |
| **生產（GHCR image）** | `docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --no-build` |
| **生產 + Logging** | `docker compose -f docker-compose.yml -f docker-compose.prod.yml -f docker-compose.logging.yml up -d --no-build` |

---

## 日常操作

### 啟動 / 停止

```bash
# 全部啟動
docker compose up -d

# 只重啟某個服務（env var 有變更用 up -d，不是 restart）
docker compose up -d grafana
docker compose up -d api

# 停止全部
docker compose down

# 停止並刪除 volume（⚠️ 資料會消失）
docker compose down -v
```

### 查看狀態

```bash
# 所有容器狀態
docker ps

# 查看某容器的 log（最後 100 行）
docker logs ipig-api --tail 100
docker logs ipig-web --tail 100
docker logs ipig-db  --tail 100

# 即時追蹤 log
docker logs -f ipig-api

# 查看容器使用的環境變數（確認 env var 有沒有套用）
docker inspect ipig-grafana | grep -A 30 '"Env"'
```

### 重新 build

```bash
# 重新 build api（Rust backend）
docker compose build api
docker compose up -d api

# 重新 build web（React frontend）
docker compose build web
docker compose up -d web

# 全部重新 build
docker compose build
docker compose up -d
```

---

## Loki（集中式 Log）

```bash
# 啟動 Loki + Promtail（本機）
docker compose -f docker-compose.yml -f docker-compose.logging.yml up -d loki promtail

# 只重啟 Loki（config 有變更時）
docker compose -f docker-compose.yml -f docker-compose.logging.yml up -d loki

# 確認 Loki 健康
curl http://localhost:3100/ready

# 確認 Loki 有收到 label
curl http://localhost:3100/loki/api/v1/labels
```

**Grafana Explore 常用 LogQL：**

```logql
# API 所有 log
{container_name="ipig-api"}

# 只看 ERROR
{container_name="ipig-api"} |= "ERROR"

# 所有容器
{job="docker"}

# 前端 nginx
{container_name="ipig-web"}

# 近 1 小時 WARN 以上
{container_name="ipig-api"} |= "WARN" | json
```

**Loki config 位置：** `monitoring/loki/config.yml`（retention 30 天）
**Promtail config 位置：** `monitoring/promtail/config.yml`

---

## Grafana

| 項目 | 值 |
|------|-----|
| URL | http://localhost:3001 |
| 預設帳號 | `admin` |
| 密碼 | `.env` 的 `GRAFANA_ADMIN_PASSWORD` |

```bash
# 重啟 Grafana（env var 變更後必須用 up -d）
docker compose up -d grafana

# 查看 Grafana log
docker logs ipig-grafana --tail 50
```

**已設定的 Datasource：**
- Prometheus → `http://prometheus:9090`
- Postgres (readonly) → `db:5432`（密碼從 `secrets/grafana_pg_password.txt`）
- Loki → `http://loki:3100`（需手動在 UI 新增）

**安全警報 SQL（Grafana Explore）：**

```sql
-- 各類型待處理數量
SELECT alert_type, COUNT(*)
FROM security_alerts
WHERE status = 'open'
GROUP BY alert_type ORDER BY COUNT(*) DESC;

-- 近 1 小時登入失敗
SELECT COUNT(*) AS value FROM login_events
WHERE event_type = 'login_failure'
  AND created_at > NOW() - INTERVAL '1 hour';
```

---

## Prometheus

| 項目 | 值 |
|------|-----|
| URL | http://localhost:9090 |
| 認證 | `monitoring/prometheus/web.yml`（bcrypt） |
| Config | `monitoring/prometheus/prometheus.yml` |
| Retention | 30 天 |

```bash
# 重載 Prometheus config（不重啟）
curl -X POST http://localhost:9090/-/reload

# 確認 targets 狀態
curl http://localhost:9090/api/v1/targets
```

---

## 備份

```bash
# 手動觸發備份
docker exec ipig-db-backup /scripts/backup.sh

# 查看備份 volume 內容
docker exec ipig-db-backup ls -lh /backups

# 查看備份 log
docker logs ipig-db-backup --tail 30
```

**備份排程：** `.env` 的 `BACKUP_SCHEDULE`（預設每天 02:00）
**保留天數：** `BACKUP_RETENTION_DAYS`（預設 30 天）
**異地備份：** `RSYNC_TARGET`（空值則不啟用）

---

## Secrets 管理

所有 secret 檔案放在 `./secrets/`，不進版本控制。

```bash
# 首次部署：產生 JWT 金鑰
openssl ecparam -name prime256v1 -genkey -noout \
  | openssl pkcs8 -topk8 -nocrypt > secrets/jwt_ec_private_key.pem
openssl ec -in secrets/jwt_ec_private_key.pem -pubout > secrets/jwt_ec_public_key.pem

# 檢查 secret 檔案是否齊全
ls secrets/
# 應有: db_password.txt, db_url.txt, smtp_password.txt,
#       jwt_ec_private_key.pem, jwt_ec_public_key.pem,
#       grafana_pg_password.txt, alertmanager_webhook_token.txt
```

---

## 生產部署（GHCR）

```bash
# 1. 登入 GHCR
echo $GITHUB_TOKEN | docker login ghcr.io -u <GHCR_OWNER> --password-stdin

# 2. Pull 最新 image
docker compose -f docker-compose.yml -f docker-compose.prod.yml pull

# 3. 無停機更新
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --no-build

# 4. 指定版本回滾（.env 設 IMAGE_TAG）
IMAGE_TAG=<commit-sha> docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --no-build

# 手動觸發 Watchtower 更新
curl -H "Authorization: Bearer $(cat secrets/watchtower_api_token.txt)" \
  http://localhost:8090/v1/update
```

---

## Prometheus 密碼更換（Task 5 — 生產前必做）

目前 `monitoring/prometheus/web.yml` 使用開發密碼 `prometheus-dev`，部署前需換強密碼：

```bash
# 方法 A：用 Docker 內的 promtool（不用安裝任何工具）
docker run --rm -it prom/prometheus:v2.51.0 promtool web-ui-hash-password
# 輸入新密碼 → 複製輸出的 $2y$... 雜湊

# 方法 B：用 htpasswd（需安裝 apache2-utils）
htpasswd -nBC 12 prometheus
```

然後更新 `monitoring/prometheus/web.yml`：
```yaml
basic_auth_users:
  prometheus: "$2y$12$你的新雜湊..."
```

同步更新 `deploy/grafana/provisioning/datasources/prometheus.yml`：
```yaml
secureJsonData:
  basicAuthPassword: 你的新密碼（明文，Grafana 負責加密存儲）
```

重啟套用：
```bash
docker compose up -d prometheus grafana
```

---

## 常見問題

### env var 改了沒生效
```bash
# ❌ 這個不會重載 env var
docker compose restart grafana

# ✅ 這個才會
docker compose up -d grafana
```

### 容器一直重啟
```bash
docker logs ipig-api --tail 50   # 看錯誤訊息
docker inspect ipig-api          # 看 exit code
```

### DB 連不上
```bash
# 確認 DB 健康
docker exec ipig-db pg_isready -U postgres -d ipig_db

# 確認 db_url secret
cat secrets/db_url.txt
```

### 磁碟空間不足
```bash
# 查看 volume 使用量
docker system df

# 清除未使用的 image/container（⚠️ 確認後再執行）
docker system prune -f

# 查看 Loki data 大小
docker exec ipig-loki du -sh /loki
```

---

## 服務 Port 對照

| 服務 | Host Port | 說明 |
|------|-----------|------|
| Web (nginx) | 8080 | 前端，對外由反向代理轉發 |
| API (Rust) | 8000 | 後端 API |
| PostgreSQL | 5433 | DB，僅 localhost |
| Grafana | 3001 | 監控儀表板 |
| Prometheus | 9090 | 指標收集 |
| Alertmanager | 9093 | 告警管理 |
| Loki | 3100 | Log 聚合 |
| Watchtower | 8090 | 自動更新 API |
