# iPig System — NAS 遷移 SOP

> 本機 Docker → Synology DS925+ + Cloudflare Tunnel

## 總覽

```
本機 (Windows)                    NAS (DS925+)                 Cloudflare
┌──────────────┐   tar/scp   ┌──────────────────┐   Tunnel   ┌─────────────┐
│ PostgreSQL   │ ──────────> │ PostgreSQL       │            │             │
│ uploads/     │ ──────────> │ uploads/         │            │ app.domain  │
│ configs/     │ ──────────> │ configs/         │            │     ↓       │
│ secrets/     │ ──(手動)──> │ secrets/         │            │  Tunnel     │
└──────────────┘             │ cloudflared ─────│───────────>│     ↓       │
                             │ web (nginx) ←────│────────────│  web:8080   │
                             │ api ← gotenberg  │            │             │
                             │     ← img-proc   │            └─────────────┘
                             └──────────────────┘
```

---

## Step 0: 前置準備

### NAS 端

```bash
# 1. 安裝 Container Manager (Docker)
#    DSM → 套件中心 → Container Manager

# 2. 開啟 SSH
#    DSM → 控制台 → 終端機和 SNMP → 啟動 SSH

# 3. SSH 進 NAS，確認 docker compose 可用
ssh admin@<NAS_IP>
sudo docker compose version
# 預期: Docker Compose version v2.x.x

# 4. 登入 GHCR（拉 private image）
sudo docker login ghcr.io -u <GITHUB_USERNAME> --password-stdin < token.txt
```

### 本機端

```bash
# 確認 secrets/ 目錄完整
ls -la secrets/
# 應有: jwt_secret.txt, db_url.txt, db_password.txt, smtp_password.txt,
#       google-service-account.json, watchtower_api_token.txt, watchtower_smtp_password.txt
```

---

## Step 1: NAS 建立目錄結構

```bash
# SSH 進 NAS
ssh admin@<NAS_IP>

sudo mkdir -p /volume1/docker/ipig/{config,data,secrets}
sudo mkdir -p /volume1/docker/ipig/config/{geoip,prometheus,alertmanager}
sudo mkdir -p /volume1/docker/ipig/config/grafana/{provisioning,dashboards}
sudo mkdir -p /volume1/docker/ipig/data/{postgres,uploads,backups}

# 最終結構：
# /volume1/docker/ipig/
# ├── docker-compose.yml      ← docker-compose.nas.yml 改名
# ├── .env
# ├── secrets/
# │   ├── cloudflare_tunnel_token.txt   ← 新增
# │   ├── jwt_secret.txt
# │   ├── db_url.txt                    ← 需修改 host
# │   ├── db_password.txt
# │   ├── smtp_password.txt
# │   ├── google-service-account.json
# │   ├── watchtower_api_token.txt
# │   └── watchtower_smtp_password.txt
# ├── config/
# │   ├── geoip/GeoLite2-City.mmdb
# │   ├── prometheus/prometheus.yml
# │   ├── alertmanager/alertmanager.yml
# │   ├── grafana/provisioning/...
# │   ├── grafana/dashboards/ipig-overview.json
# │   └── watchtower-entrypoint.sh
# └── data/
#     ├── postgres/             ← DB 資料目錄
#     ├── uploads/              ← 使用者上傳檔
#     └── backups/              ← DB 備份
```

---

## Step 2: 本機匯出資料

```bash
# 在本機 ipig_system 專案根目錄執行

# 方法 A: 自動腳本（推薦）
chmod +x docs/nas-migration/data-migration.sh
./docs/nas-migration/data-migration.sh admin <NAS_IP>

# 方法 B: 手動操作（見下方）
```

### 手動匯出 PostgreSQL

```bash
# 停止寫入，確保一致性
docker compose stop api web

# 匯出完整 DB
docker compose exec -T db pg_dumpall -U postgres --clean > pg_dump.sql

# 確認檔案大小合理
ls -lh pg_dump.sql

# 完成後停止所有服務
docker compose down
```

### 手動打包 uploads

```bash
tar czf uploads.tar.gz -C . uploads/
```

---

## Step 3: 傳輸到 NAS

```bash
NAS="admin@<NAS_IP>"
BASE="/volume1/docker/ipig"

# DB dump
scp pg_dump.sql ${NAS}:${BASE}/data/

# Uploads
scp uploads.tar.gz ${NAS}:${BASE}/data/
ssh ${NAS} "cd ${BASE}/data && tar xzf uploads.tar.gz && rm uploads.tar.gz"

# Config files
scp -r geoip/ ${NAS}:${BASE}/config/geoip/
scp -r monitoring/prometheus/ ${NAS}:${BASE}/config/prometheus/
scp -r monitoring/alertmanager/ ${NAS}:${BASE}/config/alertmanager/
scp -r deploy/grafana/provisioning/ ${NAS}:${BASE}/config/grafana/provisioning/
scp deploy/grafana_dashboard.json ${NAS}:${BASE}/config/grafana/dashboards/ipig-overview.json
scp scripts/watchtower-entrypoint.sh ${NAS}:${BASE}/config/watchtower-entrypoint.sh

# Secrets（⚠️ 敏感資料，傳完後確認權限）
scp -r secrets/ ${NAS}:${BASE}/secrets/
ssh ${NAS} "sudo chmod 600 ${BASE}/secrets/*"

# Compose + env
scp docs/nas-migration/docker-compose.nas.yml ${NAS}:${BASE}/docker-compose.yml
scp .env ${NAS}:${BASE}/.env
```

---

## Step 4: NAS 端設定

### 4a. 修改 .env

SSH 進 NAS，編輯 `/volume1/docker/ipig/.env`：

```bash
cd /volume1/docker/ipig
nano .env
```

必須修改的項目：

```env
# 生產環境關鍵設定
COOKIE_SECURE=true
SEED_DEV_USERS=false

# CORS — 改為你的正式 domain
CORS_ALLOWED_ORIGINS=https://app.yourdomain.com

# APP_URL — 郵件中的連結
APP_URL=https://app.yourdomain.com

# GHCR（拉 image 用）
GHCR_OWNER=your-github-username
IMAGE_TAG=latest
```

### 4b. 修改 db_url secret

```bash
# db_url.txt 中的 host 仍然是 'db'（Docker 內部 DNS），不需要改
# 但請確認格式正確：
cat secrets/db_url.txt
# 應為: postgres://postgres:<password>@db:5432/ipig_db
```

### 4c. 新增 Cloudflare Tunnel token

```bash
# 從 Cloudflare Dashboard 取得 tunnel token
# Zero Trust → Networks → Tunnels → 你的 tunnel → Install → 複製 token
echo "your-tunnel-token-here" > secrets/cloudflare_tunnel_token.txt
chmod 600 secrets/cloudflare_tunnel_token.txt
```

### 4d. Cloudflare Dashboard 設定 Public Hostname

在 Cloudflare Zero Trust Dashboard：
1. **Networks → Tunnels** → 選擇你的 tunnel
2. **Public Hostname** → Add
3. 設定：
   - Subdomain: `app`（或你想要的）
   - Domain: `yourdomain.com`
   - Type: `HTTP`
   - URL: `ipig-web:8080`

---

## Step 5: 匯入資料庫

```bash
cd /volume1/docker/ipig

# 先只啟動 DB
sudo docker compose up -d db

# 等待 DB 就緒
sudo docker compose exec db pg_isready -U postgres
# 反覆執行直到回傳 "accepting connections"

# 匯入 dump
sudo docker compose exec -T db psql -U postgres < data/pg_dump.sql

# 驗證
sudo docker compose exec db psql -U postgres -d ipig_db -c "\dt" | head -20
# 應該看到所有 table

# 清理 dump 檔
rm data/pg_dump.sql
```

---

## Step 6: 啟動所有服務

```bash
cd /volume1/docker/ipig

# 拉取所有 image
sudo docker compose pull

# 啟動
sudo docker compose up -d

# 檢查狀態
sudo docker compose ps

# 所有服務應該都是 Up (healthy)
# 給 30 秒讓所有 healthcheck 通過
```

---

## Step 7: 驗證

### 基本健康檢查

```bash
# 各 service 狀態
sudo docker compose ps

# API 健康
sudo docker compose exec api /app/healthcheck

# DB 連線
sudo docker compose exec db psql -U postgres -d ipig_db -c "SELECT count(*) FROM users;"

# Cloudflare Tunnel 連線
sudo docker compose logs cloudflared --tail 20
# 應看到 "Connection registered" 或類似成功訊息
```

### 外部存取測試

```bash
# 從外部瀏覽器測試
# 1. 開啟 https://app.yourdomain.com
# 2. 確認能看到登入頁
# 3. 用管理員帳號登入
# 4. 確認資料完整（使用者、動物、上傳的檔案等）
```

### 檔案上傳測試

```bash
# 登入後嘗試上傳一張圖片
# 確認 uploads volume 正確掛載
sudo docker compose exec api ls -la /app/uploads/
```

---

## Step 8: 本機善後

確認 NAS 一切正常後：

```bash
# 1. 本機停止服務（如果還在跑）
docker compose down

# 2. 觀察 NAS 運行 1-2 天確認穩定

# 3. （可選）本機資料保留作為備份，或日後清除
# ⚠️ 不要急著刪除本機的 postgres_data volume！
#    至少保留到確認 NAS 穩定運行 + 一次完整備份循環後
```

---

## 故障排除

### Cloudflare Tunnel 連不上

```bash
# 檢查 token 是否正確
sudo docker compose logs cloudflared

# 常見錯誤：token 過期或格式錯誤
# 解法：重新到 Dashboard 取得 token

# 檢查 network — cloudflared 能不能連到 web
sudo docker compose exec cloudflared wget -qO- http://ipig-web:8080/ 2>&1 | head -5
```

### API 啟動失敗

```bash
# 查看 log
sudo docker compose logs api --tail 50

# 常見原因：
# - DB 連線失敗 → 檢查 db_url.txt
# - secrets 檔案權限 → chmod 600 secrets/*
# - GHCR image 拉不到 → docker login ghcr.io
```

### DB 匯入失敗

```bash
# 如果 pg_dumpall 匯入有錯誤，可用 pg_dump 單庫匯入：
# 本機重新匯出（單庫）：
docker compose exec -T db pg_dump -U postgres -Fc ipig_db > ipig_db.dump
# NAS 匯入：
sudo docker compose exec -T db pg_restore -U postgres -d ipig_db --clean < ipig_db.dump
```

### uploads 權限問題

```bash
# API container 以非 root 用戶運行時可能無法寫入
# 檢查 container 內的 uid
sudo docker compose exec api id
# 設定 NAS 目錄權限
sudo chown -R 1000:1000 /volume1/docker/ipig/data/uploads/
```

---

## 回滾計畫

如果 NAS 出問題，需要退回本機：

```bash
# 1. NAS 停止服務
ssh admin@<NAS_IP> "cd /volume1/docker/ipig && sudo docker compose down"

# 2. Cloudflare Dashboard 把 tunnel hostname 指回本機（如果本機也有 tunnel）
#    或暫時移除 hostname

# 3. 本機重新啟動
docker compose up -d

# 4. 如果 NAS 上有新資料需要同步回本機：
#    NAS 端匯出 DB
ssh admin@<NAS_IP> "cd /volume1/docker/ipig && sudo docker compose exec -T db pg_dumpall -U postgres --clean > /tmp/nas_dump.sql"
scp admin@<NAS_IP>:/tmp/nas_dump.sql .
#    本機匯入
docker compose exec -T db psql -U postgres < nas_dump.sql
```
