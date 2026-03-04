# scripts 目錄說明

本目錄集中存放 iPig 系統的維運、CI、備份、部署與監控腳本。以下依用途分類，便於查找。

---

## 目錄結構

```
scripts/
├── README.md                 # 本說明
├── backup/                   # 資料庫備份與還原
│   ├── BACKUP.md             # 備份流程與環境變數說明
│   ├── Dockerfile.backup     # 備份容器映像
│   ├── entrypoint.sh         # 備份容器 entrypoint（cron 排程）
│   ├── pg_backup.sh          # 手動/排程 pg_dump 備份（含驗證、SHA256）
│   └── pg_restore.sh         # 從備份還原
├── deploy/                   # 正式環境部署
│   ├── healthcheck.sh        # 部署後健康檢查
│   ├── rollback.sh           # 回滾至指定 image tag
│   └── setup-server.sh       # 一次性生產環境設定（GHCR、Watchtower）
├── k6/                       # 負載測試
│   ├── README.md             # k6 安裝與執行說明
│   └── load-test.js          # k6 腳本
├── monitor/                  # 監控與維運
│   ├── check_credential_rotation.sh  # 憑證輪換提醒
│   ├── check_disk_space.sh           # 磁碟與 uploads 空間檢查（含 Prometheus）
│   └── record_credential_rotation.sh # 輪換後更新狀態檔
└── （根目錄腳本見下方分類）
```

---

## 分類索引

### 啟動與隧道

| 腳本 | 說明 | 用法 |
|------|------|------|
| `start.ps1` | Docker Compose 啟動（本機開發） | `.\scripts\start.ps1` |
| `start_tunnel.ps1` | Cloudflare Quick Tunnel，並更新 `.env` 的 `APP_URL` | `.\scripts\start_tunnel.ps1` 或 `-Port 8080` |
| `start_named_tunnel.ps1` | Cloudflare 具名隧道（生產用） | `.\scripts\start_named_tunnel.ps1`，設定見 `docs/operations/TUNNEL.md` |

### CI / 測試

| 腳本 | 說明 | 用法 |
|------|------|------|
| `run-ci-local.ps1` | 本機完整 CI（Backend/Frontend/E2E/Security/Trivy） | `.\scripts\run-ci-local.ps1`，可加 `-SkipE2E` 等 |
| `run-ci-backend-tests.ps1` | 複現 CI 環境跑後端 `cargo test` | `.\scripts\run-ci-backend-tests.ps1` 或 `-UseExistingDb` |
| `run-ci-e2e-tests.ps1` | 複現 CI 環境跑 Playwright E2E | `.\scripts\run-ci-e2e-tests.ps1` |
| `run-integration-tests.ps1` / `.sh` | 後端 API 整合測試（需本機 Postgres） | `.\scripts\run-integration-tests.ps1` / `./scripts/run-integration-tests.sh` |
| `verify-deps.ps1` / `verify-deps.sh` | 並行驗證前端 + 後端依賴與建置 | `.\scripts\verify-deps.ps1` / `./scripts/verify-deps.sh` |
| `analyze-e2e-logs.sh` | E2E 除錯：分析後端日誌（401、JWT、Session） | `bash scripts/analyze-e2e-logs.sh` |

### 資料庫與資料

| 腳本 | 說明 | 用法 |
|------|------|------|
| `sync_migrations.ps1` | Migration 同步（Clear / FixChecksums / Manual） | `.\scripts\sync_migrations.ps1` 或 `-Method FixChecksums` |
| `restore_old_dump.ps1` | 舊 dump 還原（含可選備份、migration 同步） | `.\scripts\restore_old_dump.ps1 [-DumpPath <路徑>] [-SkipBackup] [-SkipMigrationSync]` |
| `import-idxf.ps1` | 全庫 IDXF 匯入（JSON 匯出檔） | `.\scripts\import-idxf.ps1 -FilePath "路徑"` 可選 `-Email` `-Password` |
| `fix_audit_remove_prefix.sql` | 一次性修復：操作日誌 `entity_display_name` 移除類型前綴 | 手動在 DB 執行（依需求使用） |

**說明**：若還原或遷移後出現 migration checksum 錯誤，請使用 `.\scripts\sync_migrations.ps1 -Method FixChecksums`（文件曾稱 `fix_migration_checksums.ps1`，功能已整合於此）。

### 備份

詳見 **`backup/BACKUP.md`**。摘要：

| 腳本 | 說明 | 用法 |
|------|------|------|
| `backup/pg_backup.sh` | 備份（gzip + 驗證 + SHA256 + 保留天數） | 容器內 cron 或手動執行 |
| `backup/pg_restore.sh` | 從 `.sql.gz` 還原 | `./scripts/backup/pg_restore.sh /path/to/ipig_YYYYMMDD_HHMMSS.sql.gz` |

### 部署

| 腳本 | 說明 | 用法 |
|------|------|------|
| `deploy/setup-server.sh` | 生產環境一次性設定（GHCR 登入、Watchtower token、.env） | `bash scripts/deploy/setup-server.sh` |
| `deploy/healthcheck.sh` | 部署後 API/Web 健康檢查 | `bash scripts/deploy/healthcheck.sh [MAX_WAIT_SECONDS] [RETRIES]` |
| `deploy/rollback.sh` | 回滾至指定 commit 映像 | `bash scripts/deploy/rollback.sh <commit-sha>` |

### 環境與維運

| 腳本 | 說明 | 用法 |
|------|------|------|
| `validate-env.sh` | 啟動前環境變數檢查（必填/選填、HMAC 長度等） | `./scripts/validate-env.sh`，CI 或 Docker 前執行 |
| `generate-alertmanager-config.sh` | 由環境變數產生 Alertmanager 設定 | `source .env; ./scripts/generate-alertmanager-config.sh` |
| `update_geoip.sh` | MaxMind GeoLite2-City 更新（需 `MAXMIND_LICENSE_KEY`） | `bash scripts/update_geoip.sh` |

### Windows 建置（Rust link.exe）

| 腳本 | 說明 | 用法 |
|------|------|------|
| `load-msvc-env.ps1` | 載入 MSVC 環境（供 `cargo build`） | `. .\scripts\load-msvc-env.ps1`（source 後再 cargo） |
| `build-backend.ps1` | 在 MSVC 環境下一鍵編譯後端 | `.\scripts\build-backend.ps1` 或 `--release` |
| `install-msvc-buildtools.ps1` | 下載安裝 VS Build Tools（C++ workload） | 需管理員，`.\scripts\install-msvc-buildtools.ps1` |
| `install-msvc-buildtools.cmd` | 同上，CMD 入口 | 管理員執行 |

---

## 相關文件

- 備份流程：`scripts/backup/BACKUP.md`
- 部署：`docs/DEPLOYMENT.md`、`docs/operations/infrastructure.md`
- 隧道：`docs/operations/TUNNEL.md`
- 本機 CI：`docs/development/ci-local.md`
- 舊 dump 還原：`docs/database/RESTORE_OLD_DUMP.md`
- 憑證輪換：`docs/security-compliance/CREDENTIAL_ROTATION.md`
- k6 壓測：`scripts/k6/README.md`、`docs/assessments/PERFORMANCE_BENCHMARK.md`
