# iPig 專案 Infrastructure 總覽

> 本文件記錄專案所使用的基礎設施類別、數量與必要性說明。  
> **最後更新：** 2026-03-04

---

## 1. 數量總覽

專案中「infrastructure」依類別統計如下。

| 類別 | 數量 | 內容 |
|------|------|------|
| **GitHub Actions 工作** | **15 個** | CI 11 個 + CD 4 個（見下方清單） |
| **Docker Compose 檔案** | **6 個** | 主線、prod、test、CI 本地、monitoring、logging |
| **監控／可觀測元件** | **5 個** | Prometheus、Grafana、Alertmanager、Promtail、Loki |
| **Deploy／對外與安全** | **2 類** | Cloudflare Tunnel（含 Cloudflare WAF）、Grafana/Prometheus 設定 |
| **Dockerfile** | **3 個** | backend、frontend、scripts/backup |
| **維運腳本** | 多個 | backup、deploy、monitor、k6 壓測、CI 本地／整合測試等 |

---

## 2. CI/CD（GitHub Actions）

- **Workflow 檔案：** `.github/workflows/ci.yml`、`.github/workflows/cd.yml`、`.github/dependabot.yml`

### 2.1 CI 工作（11 個）

| Job 名稱 | 說明 |
|----------|------|
| `security-audit` | 使用 cargo-audit 檢查 Rust 依賴漏洞（RustSec Advisory DB） |
| `cargo-deny` | 供應鏈檢查：授權、漏洞、來源 |
| `sql-injection-guard` | 檢查是否有 `format!` 拼接 SQL，防 SQL 注入 |
| `unsafe-guard` | 檢查後端是否出現 `unsafe` 程式碼 |
| `backend-check` | 後端 `cargo check` 編譯檢查 |
| `backend-test` | 後端單元／整合測試（含 DB） |
| `backend-lint` | 後端 `cargo clippy`、`cargo fmt` |
| `frontend-check` | 前端建置與檢查 |
| `npm-audit` | 前端依賴漏洞檢查（high/critical） |
| `trivy-scan` | 後端與前端 Docker 映像的 Trivy 容器掃描 |
| `e2e-test` | Playwright E2E 測試（依賴 `docker-compose.test.yml`） |

### 2.2 CD 工作（4 個）

| Job 名稱 | 說明 |
|----------|------|
| `gate` | 確認 CI 通過後才繼續 |
| `build-api` | 建置並推送 API 映像至 GHCR |
| `build-web` | 建置並推送 Web 映像至 GHCR |
| `summary` | 彙總 CD 結果 |

---

## 3. Docker Compose 情境（7 個）

| 檔案 | 用途 |
|------|------|
| `docker-compose.yml` | 主線：本機開發／預設服務（db、api、web 等） |
| `docker-compose.prod.yml` | 正式環境覆寫 |
| `docker-compose.test.yml` | 測試用環境（CI E2E 使用） |
| `docker-compose.test.ci-local.yml` | 本地跑 CI 測試用 |
| `docker-compose.monitoring.yml` | 監控：Prometheus、Alertmanager、Grafana |
| `docker-compose.logging.yml` | 日誌：Loki、Promtail |
| ~~`docker-compose.waf.yml`~~ | 已移除，WAF 改由 Cloudflare WAF 處理 |

---

## 4. 監控與可觀測（5 個元件）

| 元件 | 說明 | 設定位置 |
|------|------|----------|
| **Prometheus** | 指標抓取與儲存 | `monitoring/prometheus/`、`deploy/prometheus.yml` |
| **Grafana** | 儀表板與查詢 | `deploy/grafana/`（provisioning、dashboard） |
| **Alertmanager** | 告警路由與通知 | `monitoring/alertmanager/` |
| **Promtail** | 日誌收集（送 Loki） | `monitoring/promtail/config.yml` |
| **Loki** | 日誌儲存與查詢 | 由 `docker-compose.logging.yml` 定義 |

---

## 5. Deploy 與安全

| 項目 | 說明 | 位置 |
|------|------|------|
| **Cloudflare Tunnel + WAF** | 具名隧道對外暴露（無直開 22/80/443），WAF 由 Cloudflare Managed Ruleset 提供 | `deploy/cloudflared-config.yml`、`scripts/start_named_tunnel.ps1`、Cloudflare Dashboard |
| **Grafana / Prometheus 設定** | 資料源、儀表板 provisioning | `deploy/grafana/`、`deploy/prometheus.yml` |

---

## 6. Dockerfile（3 個）

| 映像 | 路徑 |
|------|------|
| 後端 API | `backend/Dockerfile` |
| 前端 Web | `frontend/Dockerfile` |
| 備份用 | `scripts/backup/Dockerfile.backup` |

---

## 7. 維運腳本（摘要）

| 類別 | 代表腳本／目錄 |
|------|----------------|
| **備份／還原** | `scripts/backup/pg_backup.sh`、`pg_restore.sh`、`BACKUP.md` |
| **部署** | `scripts/deploy/healthcheck.sh`、`rollback.sh`、`setup-server.sh` |
| **監控／維運** | `scripts/monitor/`（憑證輪替、磁碟檢查等） |
| **壓測** | `scripts/k6/`（k6 負載測試） |
| **CI 本地／隧道** | `scripts/run-ci-*.ps1`、`start_tunnel.ps1`、`start_named_tunnel.ps1` |

---

## 8. 必要性說明

整體而言，這些 infrastructure 對「正式上線」與 `docs/PROGRESS.md` 中的 **Production Readiness** 目標是對齊的；必要性與可調整處如下。

| 面向 | 必要性 | 備註 |
|------|--------|------|
| **CI 多個 job** | ✅ 核心必要 | 後端 check/test/lint、前端 check、E2E、cargo audit、cargo-deny、npm audit、Trivy 皆與品質／安全直接相關。sql-injection-guard、unsafe-guard 可視團隊紀律考慮合併或簡化以縮短 CI 時間。 |
| **7 個 docker-compose** | ✅ 多數必要 | 主線、prod、test、E2E 用為必要；monitoring、logging、WAF 可先保留為選用，待正式需要時再啟用。 |
| **監控 5 元件** | ✅ 目標必要 | 與可觀測性、上線準備度一致。若資源有限，可先上 Prometheus + Grafana，Alertmanager／Promtail／Loki 分階段啟用。 |
| **Cloudflare Tunnel + WAF** | ✅ 已啟用 | 流量經 Cloudflare Tunnel，WAF 由 Cloudflare Dashboard 啟用 Managed Ruleset |

**一句話結論：** 數量多來自「同一套產品」的不同面向（建置、測試、部署、監控、日誌、安全），多數對上線與維運有必要；可依團隊資源在「CI 守衛精簡」與「monitoring／logging／WAF 延後啟用」上做取捨。

---

## 9. 相關文件

- 總體進度與上線準備度：`docs/PROGRESS.md`
- E2E 流程與說明：`docs/e2e/FLOW.md`、`docs/e2e/README.md`
- 安全與 CVE 說明：`docs/security/security.md`（若存在）
- 備份流程：`scripts/backup/BACKUP.md`
