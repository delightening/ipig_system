# R22-14: Log Aggregation Evaluation

> **Date:** 2026-04-15
> **Context:** R22 攻擊偵測功能需要 tracing log 持久化，目前 Docker container 重啟後 stdout log 會遺失。

## Current State

- Backend 使用 `tracing` crate + `tracing-subscriber` 寫入 stdout
- Docker 預設 `json-file` log driver，無 rotation 設定
- Container 重啟後 log 保留在 host filesystem，但無集中化搜尋能力
- Prometheus + Grafana 已部署（`docker-compose.monitoring.yml`），僅收集 metrics

## Options Comparison

| Criteria | Loki + Promtail | Elasticsearch + Filebeat | Structured JSON + S3 |
|----------|----------------|------------------------|---------------------|
| **Grafana 整合** | 原生支援（同一 stack） | 需 Kibana 或 Grafana ES plugin | 需自建 query layer |
| **資源消耗** | 低（~100MB RAM） | 高（~1-2GB RAM for ES） | 極低（僅 writer） |
| **查詢語言** | LogQL（類 PromQL） | KQL / Lucene | N/A（需另建） |
| **部署複雜度** | 低（2 containers） | 高（3+ containers） | 低（S3 client） |
| **全文搜尋** | 有限（label + filter） | 強（inverted index） | 無 |
| **成本（NAS 部署）** | 低 | 高（磁碟 I/O 密集） | 低 |
| **現有 infra 複用** | Grafana 已有 | 無 | 無 |

## Recommendation: Loki + Promtail

**理由：**
1. Grafana 已部署，Loki 是同一 stack，無需額外 UI
2. 資源消耗適合 DS923+ NAS（4GB RAM 環境）
3. LogQL 與 PromQL 語法類似，學習成本低
4. Docker log driver 模式可直接將 container log 推送到 Loki

## Implementation Plan

### Phase 1: Docker log rotation (R22-18, immediate)
- `docker-compose.prod.yml` 設定 `json-file` driver + rotation
- 確保重啟前 log 不佔滿磁碟

### Phase 2: Loki 部署 (future)
- 新增 `loki` + `promtail` services 到 `docker-compose.monitoring.yml`
- Promtail 收集 `/var/lib/docker/containers/*/*.log`
- Grafana 新增 Loki data source
- 建立 R22-15 Security Dashboard（LogQL queries）

### Phase 3: Docker Loki driver (optional)
- 替換 `json-file` → `loki` log driver
- 直接 push，不需 Promtail

## Decision

Phase 1（Docker log rotation）立即執行。Phase 2/3 在 R21 環境監控完成後視需求排程。
