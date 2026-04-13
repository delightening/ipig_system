# iPig 系統部署指南 — Synology DS923+

> 適用：Synology DS923+ · DSM 7.2+ · Cloudflare Tunnel · Docker Compose

---

## 1. 硬體需求

| 項目 | 原廠配置 | 建議升級 | 原因 |
|------|----------|----------|------|
| **RAM** | 4 GB DDR4 ECC | **16 GB**（必要） | Rust 編譯 + PostgreSQL + 多容器同跑約 6–8 GB，4 GB 會 OOM build 失敗 |
| **M.2 NVMe** | 無 | 1–2 顆（建議） | 做 SSD 快取或系統碟，提升 DB 讀寫效能；DS923+ 有 2 個 M.2 槽 |
| **HDD** | 依現有 | 依現有 | 存 uploads、備份、DB volume |

> 相容 RAM 型號：Kingston KSM32SES8/16MF 或 Crucial CT16G4SFD832A（DDR4-3200 SO-DIMM ECC）

---

## 2. DSM 軟體準備

1. 更新 DSM 至 **7.2+**
2. 套件中心安裝 **Container Manager**（取代舊版 Docker 套件）
3. 開啟 SSH：控制台 → 終端機與 SNMP → 啟用 SSH 服務
4. 以 SSH 登入後確認 Docker Compose 可用：
   ```bash
   docker compose version
   # Docker Compose version v2.x.x
   ```

---

## 3. 網路設定（Cloudflare Tunnel）

不需要對外開放任何 Port，全程透過 Cloudflare Tunnel 代理，無需設定 Port forwarding 或固定 IP。

### 3.1 安裝 cloudflared

```bash
# 在 DS923+ SSH 內執行
docker run -d \
  --name cloudflared \
  --restart unless-stopped \
  cloudflare/cloudflared:latest \
  tunnel --no-autoupdate run --token YOUR_TUNNEL_TOKEN
```

或在 Container Manager 直接建立容器。

### 3.2 Cloudflare Dashboard 設定

1. Zero Trust → Networks → Tunnels → 建立新 Tunnel
2. 複製 Token，填入上方指令的 `YOUR_TUNNEL_TOKEN`
3. 新增 Public Hostname：

| Subdomain | Service |
|-----------|---------|
| `ipig.yourdomain.com` | `http://localhost:8080`（前端） |
| `ipig-api.yourdomain.com`（選填） | `http://localhost:8000`（API，內部使用） |

4. SSL/TLS 模式設定為 **Full (strict)**

### 3.3 修改 `.env`

```env
APP_URL=https://ipig.yourdomain.com
CORS_ALLOWED_ORIGINS=https://ipig.yourdomain.com
COOKIE_SECURE=true
TRUST_PROXY_HEADERS=true
```

---

## 4. 儲存路徑規劃

建議在 NAS 上建立專用共享資料夾：

```
/volume1/
  ipig/
    uploads/      ← 照片與附件
    backups/      ← 資料庫備份（自動）
    repo/         ← Git 程式碼
```

在 `.env` 設定：

```env
UPLOAD_VOLUME=/volume1/ipig/uploads
```

備份 volume 也對應修改（`docker-compose.yml` 的 `db_backups` volume 可改為 bind mount 到 `/volume1/ipig/backups`）。

---

## 5. 首次部署步驟

```bash
# 1. Clone repo
cd /volume1/ipig
git clone https://github.com/delightening/ipig_system repo
cd repo

# 2. 複製並編輯 .env
cp .env.example .env
# 修改以下關鍵欄位：
#   POSTGRES_PASSWORD
#   JWT_SECRET
#   AUDIT_HMAC_KEY
#   SMTP_* 郵件設定
#   IMAGE_PROCESSOR_TOKEN（任意隨機字串）
#   APP_URL=https://ipig.yourdomain.com
#   UPLOAD_VOLUME=/volume1/ipig/uploads

# 3. 建立 uploads 資料夾
mkdir -p /volume1/ipig/uploads

# 4. Build 並啟動
docker compose up -d --build

# 5. 確認服務正常
docker compose ps
```

---

## 6. 從舊主機遷移

### 6.1 匯出現有資料庫

在**舊主機**執行：

```bash
docker exec ipig-db pg_dump -U postgres ipig_db > ipig_backup_$(date +%Y%m%d).sql
```

### 6.2 搬移照片與附件

```bash
# 使用 rsync 或 scp 將 uploads 資料夾複製到 NAS
rsync -avz ./uploads/ nas_user@ds923:/volume1/ipig/uploads/
```

### 6.3 在 DS923+ 還原資料庫

```bash
# 先啟動 DB
docker compose up -d db

# 等 DB ready 後還原
docker exec -i ipig-db psql -U postgres ipig_db < ipig_backup_20260413.sql

# 啟動其餘服務
docker compose up -d
```

---

## 7. 環境變數速查（NAS 專屬）

| 變數 | 說明 | 範例值 |
|------|------|--------|
| `UPLOAD_VOLUME` | 照片儲存路徑 | `/volume1/ipig/uploads` |
| `APP_URL` | 對外 URL | `https://ipig.yourdomain.com` |
| `CORS_ALLOWED_ORIGINS` | CORS 白名單 | `https://ipig.yourdomain.com` |
| `IMAGE_PROCESSOR_TOKEN` | 圖片微服務內部 token | 任意隨機字串 |
| `COOKIE_SECURE` | HTTPS 限定 Cookie | `true` |
| `TRUST_PROXY_HEADERS` | 信任 Cloudflare IP header | `true` |

---

## 8. 注意事項

- **不需要開放任何 Port 給外部**，Cloudflare Tunnel 走 outbound 連線，防火牆不需額外設定
- DS923+ 預設的 Container Manager 支援 `docker compose`，不需另外安裝
- RAM 不升級到 16 GB 前，請勿在 NAS 上執行 `docker compose build`，改在開發機 build 好 image 再推到 NAS
- 建議開啟 DSM 的**快照（Snapshot）**功能保護 `/volume1/ipig` 資料夾

---

*最後更新：2026-04-13*
