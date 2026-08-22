# 部署與儀表板目錄說明

本目錄存放**正式或測試環境**所需的部署設定與 Grafana 儀表板。

---

## 目錄結構

```
deploy/
├── README.md                    # 本說明
└── cloudflared-config.yml       # Cloudflare 具名隧道設定（生產用）
```

Grafana provisioning 與儀表板已移至 `monitoring/grafana/`，由 `docker-compose.yml` 掛載。

---

## 分類說明

| 類型 | 檔案 | 說明 |
|------|------|------|
| **隧道 + WAF** | `cloudflared-config.yml` | 生產環境對外暴露用，WAF 由 Cloudflare Dashboard 管理；詳見 `docs/operations/TUNNEL.md` |

---

## 相關文件

- Docker Compose 總覽：`docs/operations/COMPOSE.md`
- 維運手冊：`docs/operations/OPERATIONS.md`
- 隧道設定：`docs/operations/TUNNEL.md`
- 監控設定檔：`monitoring/` 目錄（見 [monitoring/README.md](../monitoring/README.md)）
