# Docker Compose 文件總覽

本專案使用多個 `docker-compose` 檔案以支援不同情境（開發、生產、CI、監控等）。各檔透過 **override 合併** 方式組合使用。

---

## 檔案一覽

| 檔案 | 用途 |
|------|------|
| `docker-compose.yml` | **核心**：PostgreSQL、API、web、web-dev、db-backup |
| `docker-compose.prod.yml` | **生產**：GHCR images、Watchtower、Docker Secrets、資源限制 |
| ~~`docker-compose.waf.yml`~~ | **已移除**：WAF 改由 Cloudflare WAF 處理（流量經 Cloudflare Tunnel） |
| `docker-compose.logging.yml` | **日誌**：Loki + Promtail |
| `docker-compose.monitoring.yml` | **監控**：Prometheus、Alertmanager、Grafana |
| `docker-compose.test.yml` | **CI 測試**：db-test、api-test、web-test（獨立 stack） |

---

## 核心服務 (`docker-compose.yml`)

| 服務 | 說明 | 預設埠 |
|------|------|--------|
| db | PostgreSQL 16，綁定 localhost | 5433 |
| api | Rust backend API | 8000 |
| web | React 生產版（nginx） | 8080 |
| web-dev | React 開發版（Vite） | 5173 |
| db-backup | 排程備份（pg_dump + GPG + rsync） | — |

**網路隔離**：`frontend`、`backend`、`database` 三層 bridge。

---

## 使用情境與指令

### 開發環境（預設）

```bash
# 啟動核心服務
docker compose up -d

# 含開發用 web-dev（Vite HMR）
docker compose up -d web-dev
```

### 生產環境

```bash
# 使用 GHCR 預建 image，無本地 build
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --no-build
```

**必要 .env**：`GHCR_OWNER`、`IMAGE_TAG`（預設 latest）  
**前置**：`docker login ghcr.io`

### CI / E2E 測試

> 流程參照：[docs/e2e/FLOW.md](e2e/FLOW.md)

```bash
# 獨立 stack，不與主 compose 共用 network
docker compose -f docker-compose.test.yml up -d

# 結束後
docker compose -f docker-compose.test.yml down
```

測試用 API 埠：`localhost:8000`，前端：`localhost:8080`（與主 compose 埠相同時會衝突，CI 環境通常單獨執行）。

### 監控堆疊

使用 monitoring overlay 前，**必須在 .env 設定 `GRAFANA_ADMIN_PASSWORD`**（無預設值，未設定將導致 Grafana 無法登入）。

```bash
# 先啟動核心服務建立 network
docker compose up -d

# 再啟動監控（依賴 ipig_system_backend network）
docker compose -f docker-compose.monitoring.yml up -d
```

| 服務 | 埠 | 說明 |
|------|-----|------|
| Prometheus | 127.0.0.1:9090 | metrics |
| Alertmanager | 127.0.0.1:9093 | 告警 |
| Grafana | 127.0.0.1:3001 | 儀表板 |

### 日誌堆疊（Loki + Promtail）

```bash
# 需先建立 external network ipig-net
docker network create ipig-net

docker compose -f docker-compose.logging.yml up -d
```

---

## 依賴關係

```
docker-compose.yml (核心)
├── docker-compose.prod.yml   → override api/web/db，加 watchtower
├── (docker-compose.waf.yml)   → 已移除，WAF 改由 Cloudflare WAF
├── docker-compose.monitoring.yml → 需 backend network (由核心建立)
└── docker-compose.logging.yml   → 獨立，需 ipig-net

docker-compose.test.yml → 完全獨立，db-test / api-test / web-test
```

---

## 常用指令速查

| 情境 | 指令 |
|------|------|
| 開發啟動 | `docker compose up -d` |
| 開發停止 | `docker compose down` |
| 生產啟動 | `docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --no-build` |
| 生產 + WAF | WAF 改由 Cloudflare WAF 處理，無需額外 compose 檔 |
| CI 測試 | `docker compose -f docker-compose.test.yml up -d` |
| 查看日誌 | `docker compose logs -f [service]` |

---

## 相關文件

- [QUICK_START.md](QUICK_START.md) — 本地開發、Docker 快速啟動、E2E 測試
- [DEPLOYMENT.md](DEPLOYMENT.md) — 正式部署、備份、監控、故障排除
