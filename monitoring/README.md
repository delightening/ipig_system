# 監控目錄說明

本目錄存放 iPig 系統的**監控與告警**設定檔，供 Docker Compose 監控堆疊使用。

---

## 目錄結構

```
monitoring/
├── README.md                    # 本說明
├── prometheus/
│   ├── prometheus.yml           # Prometheus 抓取設定（API metrics）
│   └── alert_rules.yml          # 告警規則
├── alertmanager/
│   ├── alertmanager.yml         # 告警路由與通知（實際使用）
│   ├── alertmanager.example.yml # 範本（含環境變數佔位）
│   ├── entrypoint.sh            # 啟動時替換變數
│   └── docker-entrypoint.sh     # Docker 入口
└── promtail/
    └── config.yml               # Loki 日誌收集（Loki + Promtail 已整合於 docker-compose.yml）
```

---

## 用途說明

| 元件 | 用途 | 相關 compose |
|------|------|--------------|
| **Prometheus** | 抓取 `/api/metrics`、儲存時序資料 | `docker-compose.yml`（base 已整合） |
| **Alertmanager** | 接收 Prometheus 告警、依規則通知（Email / Webhook） | 同上 |
| **Grafana** | 儀表板與告警視覺化 | 同上 |
| **Loki / Promtail** | 收集容器日誌 → Loki | 同上 |

---

## 啟用方式

監控與日誌堆疊（Prometheus + Grafana + Alertmanager + Loki + Promtail + node-exporter）
**已全部整合於 base `docker-compose.yml`**，隨主 stack 一起啟動，無需額外 overlay：

```bash
docker compose up -d
```

> 2026-06-10：原 `docker-compose.monitoring.yml` overlay 已移除——其 prometheus /
> alertmanager / grafana 與 base 重複（base 已是實際部署的唯一來源）。

---

## 相關文件

- 部署與監控章節：`docs/DEPLOYMENT.md`
- Grafana 儀表板與 datasource：`deploy/` 目錄（見 [deploy/README.md](../deploy/README.md)）
- 告警規則與 Alertmanager 範本：`docs/` 或 `.env.example` 中的 `ALERT_*` 變數說明
