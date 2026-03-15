# iPIG 資料庫備份指南

## 架構

```
┌─────────────┐     pg_dump     ┌──────────┐     rsync      ┌──────┐
│  PostgreSQL │ ──────────────► │  /backups │ ─────────────► │  NAS │
│   (db)      │                 │  (volume) │                │      │
└─────────────┘                 └──────────┘                └──────┘
       ↑                              ↑
       │                              │
   db-backup 容器（cron 排程）
```

## 快速開始

### 環境變數（.env）

```env
# 備份排程（cron 格式，預設每天凌晨 2:00）
BACKUP_SCHEDULE=0 2 * * *

# 備份保留天數（預設 30 天）
BACKUP_RETENTION_DAYS=30

# 異地備份目標（rsync 格式，留空則跳過）
RSYNC_TARGET=user@nas:/volume1/backups/ipig

# 啟動時立即執行一次備份
BACKUP_ON_START=false
```

### 啟動備份服務

```bash
docker compose up -d db-backup
```

**生產環境**：使用 `docker-compose.prod.yml` 時，db-backup 改由 Docker Secret `db_password` 提供密碼（`POSTGRES_PASSWORD_FILE=/run/secrets/db_password`），腳本會優先從該檔讀取，密碼不會出現在 process listing。

## 手動備份

```bash
# 進入容器執行備份
docker compose exec db-backup /usr/local/bin/pg_backup.sh

# 或直接在主機執行 pg_dump
docker compose exec db pg_dump -U postgres ipig_db | gzip > backup.sql.gz
```

## 還原

### 自動化腳本（推薦）

```bash
# 從備份容器執行（備份檔在 volume 內）
docker compose exec db-backup /usr/local/bin/pg_restore.sh /backups/ipig_YYYYMMDD_HHMMSS.sql.gz

# 從主機執行（需掛載備份目錄）
./scripts/backup/pg_restore.sh /path/to/ipig_YYYYMMDD_HHMMSS.sql.gz
```

環境變數：`DB_HOST`、`DB_USER`、`DB_NAME`、`DB_PASSWORD`（同備份腳本）

### 手動還原

```bash
# 方法一：從容器 volume（custom format 需用 pg_restore）
gunzip -c /backups/ipig_YYYYMMDD_HHMMSS.sql.gz | docker compose exec -T db pg_restore -U postgres -d ipig_db --clean --if-exists --no-owner -

# 方法二：從主機檔案
gunzip -c backup.sql.gz | docker compose exec -T db pg_restore -U postgres -d ipig_db --clean --if-exists --no-owner -
```

## 異地備份（rsync 到 NAS）

### SSH 金鑰設定

```bash
# 1. 產生金鑰（在主機上）
ssh-keygen -t ed25519 -f ~/.ssh/ipig_backup -N ""

# 2. 將公鑰加入 NAS
ssh-copy-id -i ~/.ssh/ipig_backup.pub user@nas

# 3. 將私鑰掛載到容器（docker-compose.yml）
volumes:
  - ~/.ssh/ipig_backup:/root/.ssh/id_ed25519:ro
  - ~/.ssh/known_hosts:/root/.ssh/known_hosts:ro
```

## 查看備份日誌

```bash
docker compose logs db-backup --tail 50
```

## 備份檔案位置

備份檔案儲存在 Docker volume `db_backups` 中：

```bash
# 列出所有備份
docker compose exec db-backup ls -lah /backups/

# 複製到主機
docker cp ipig-db-backup:/backups/ipig_db_YYYYMMDD_HHMMSS.sql.gz ./
```
