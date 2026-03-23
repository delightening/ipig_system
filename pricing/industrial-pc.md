# iPig ERP 系統 — 工業電腦自建方案與定價分析

> 更新日期：2026-03-22
> 價格以 TWD 計算，來源為台灣通路公開資訊

---

## 目錄

1. [工作負載需求](#1-工作負載需求)
2. [現成工業電腦品牌參考](#2-現成工業電腦品牌參考)
3. [消費級迷你電腦替代方案](#3-消費級迷你電腦替代方案)
4. [自組工業電腦零件清單](#4-自組工業電腦零件清單)
5. [Docker 推薦資源配置](#5-docker-推薦資源配置)
6. [PostgreSQL 調優建議](#6-postgresql-調優建議)
7. [磁碟配置建議](#7-磁碟配置建議)
8. [作業系統與可靠性建議](#8-作業系統與可靠性建議)

---

## 1. 工作負載需求

### 服務清單

| 服務 | CPU 特性 | 記憶體特性 | I/O 特性 |
|------|----------|-----------|----------|
| PostgreSQL | 中等，查詢併發 | 重要，shared_buffers 需大 | SSD 必須，IOPS 關鍵 |
| Rust API (Axum) | 低～中，async 多工 | 低（Rust 很省） | 低 |
| Gotenberg/Chromium | 突發高，PDF 生成 | 中高，每次渲染 ~200MB | 低 |
| Prometheus + Loki | 低，持續寫入 | 中 | 中，持續寫 TSDB |
| Nginx | 極低 | 極低 | 低 |

### 最低硬體需求

| 項目 | 最低 | 建議 |
|------|------|------|
| CPU | 4 核心 | 4C/8T 以上 |
| RAM | 16GB | 32GB |
| 系統碟 | 256GB SSD | 512GB NVMe |
| 資料碟 | 256GB SSD | 1TB NVMe（PostgreSQL 專用） |
| 網路 | 1× GbE | 2× GbE（服務 + 管理） |

---

## 2. 現成工業電腦品牌參考

> 工業電腦普遍不公開定價，以下為經銷商詢價估算範圍。

### 研華 Advantech

| 型號 | CPU | 特色 | 估價 (TWD) | 連結 |
|------|-----|------|-----------|------|
| ARK-1125H | Intel N200 (4C) | 超迷你、無風扇、-20~60°C 寬溫 | ~15,000–25,000 | [官網](https://www.advantech.com/en-us/products/1-2jkbyz/ark-1125h/mod_bc65cf57-e6ae-48e7-bcbb-e1afe4a11c06) |
| ARK-2251 | i5-1335UE (10C) | 模組化、雙 LAN、6 USB | ~30,000–50,000 | [官網](https://www.advantech.com/en-us/products/ark-2000_series_embedded_box_pcs/ark-2251/mod_de661626-e644-4813-a0f0-be7f006923c1) |
| ARK-3533 | i5/i7/i9 LGA1700 (12~14 代) | 高效能、可擴充、雙 M.2 | ~40,000–70,000 | [官網](https://www.advantech.com/en-us/products/1-2jkd2d/ark-3533/mod_e6e1a36b-df85-40fc-ba2a-98a64f39216f) |

詢價管道：[研華 IoTMart 台灣](https://iotmart.advantech.com.tw/)

### 超恩 Vecow

| 型號 | CPU | 特色 | 估價 (TWD) | 連結 |
|------|-----|------|-----------|------|
| ECX-3000 | i5/i7/i9 (12~13 代) | 2.5GbE LAN、工業擴充 | ~35,000–60,000 | [官網](https://www.vecow.com/dispPageBox/vecow/VecowCP.aspx?ddsPageID=ECX-3000_TW) |

### 宸曜 Neousys

| 型號 | CPU | 特色 | 估價 (TWD) | 連結 |
|------|-----|------|-----------|------|
| Nuvo-9000 系列 | i5/i7/i9 (12~14 代) | 無風扇、多 LAN、MezIO 擴充 | ~40,000–80,000 | [官網](https://www.neousys-tech.com/tw/product/product-lines/industrial-computers/product/listing/fanless-industrial-pc) |

### Supermicro（伺服器級）

| 型號 | CPU | 特色 | 估價 (TWD) | 連結 |
|------|-----|------|-----------|------|
| E300-12D-4CN6P | Xeon D-1718T (4C/8T) | ECC RAM、雙 25GbE、IPMI | ~35,000–50,000 | [eStore](https://store.supermicro.com/us_en/iot-edge-embedded-sys-e300-12d-4cn6p.html) |
| E300-12D-8CN6P | Xeon D-1736NT (8C/16T) | 同上，更多核心 | ~50,000–70,000 | [PChome](https://24h.pchome.com.tw/store/DSAM5J) |

---

## 3. 消費級迷你電腦替代方案

適合辦公室恆溫環境，非嚴格工業級但性價比高。

| 品牌型號 | CPU | RAM 支援 | 參考價 (TWD) | 來源 |
|----------|-----|---------|-------------|------|
| ASUS NUC 13 Pro i5（準系統） | i5-1340P (12C) | 最高 64GB DDR4 | ~9,500–12,000 | PChome |
| ASUS NUC 13 Pro i5（含 8G+500G） | i5-1340P | 8GB（可擴充） | ~16,900–23,300 | PChome/BigGo |
| MSI Cubi 5 i5（準系統） | i5-1235U (10C) | 最高 64GB DDR4 | ~7,500–9,000 | MSI 旗艦館 |
| MSI Cubi 5 i5（含 8G+512G+Win11） | i5-1235U | 8GB | ~22,900–25,900 | PChome |

### 推薦 DIY 配置：ASUS NUC 13 Pro i5

| 項目 | 規格 | 預估價格 |
|------|------|---------|
| 準系統 | NUC 13 Pro i5-1340P | ~10,000 |
| RAM | 32GB DDR4 SO-DIMM（自購） | ~2,500 |
| 系統碟 | 512GB NVMe（內建 M.2） | ~1,500 |
| 資料碟 | 1TB NVMe（外接 USB4/TB4 SSD） | ~2,500 |
| **合計** | | **~16,500** |

---

## 4. 自組工業電腦零件清單

### 方案 A：性價比優先

| 零件 | 推薦 | 預算 (TWD) | 說明 |
|------|------|-----------|------|
| CPU | Intel i5-13500 / i5-14500 | ~6,000–7,000 | 6P+8E 核，ERP 綽綽有餘 |
| 主機板 | Intel Q670/B660 工業級 ITX/mATX | ~4,000–8,000 | 選有雙 LAN 的 |
| RAM | 32GB DDR4/DDR5 ECC（如主板支援） | ~3,000–5,000 | ECC 防 bit flip |
| 系統碟 | 512GB NVMe SSD | ~2,000 | OS + Docker |
| 資料碟 | 1TB NVMe SSD | ~2,500 | PostgreSQL 專用 |
| 備份碟 | 2TB 2.5" SATA SSD 或 HDD | ~2,000–3,500 | db-backup volume |
| 機殼 | 工業級無風扇/低噪音機殼 | ~3,000–8,000 | 依需求選擇 |
| 電源 | 200W~300W 工業電源 | ~2,000–3,000 | ERP 不吃電 |
| **合計** | | **~25,000–40,000** | |

### 方案 B：可靠性優先

| 零件 | 推薦 | 說明 |
|------|------|------|
| CPU | Intel Xeon E-2434 / E-2436 | 支援 ECC，伺服器等級 |
| 主機板 | Supermicro X13SAE / X13SEM | IPMI 遠端管理 |
| RAM | 32–64GB DDR5 ECC UDIMM | 資料完整性保障 |
| 磁碟 | 2x 1TB NVMe SSD（RAID 1） | 資料冗餘 |
| UPS | 在線式 600VA~1000VA | 斷電保護 |
| **預算** | **~50,000–70,000 TWD** | |

---

## 5. Docker 推薦資源配置

### 核心服務

| 服務 | CPU limit | CPU reserve | Memory limit | Memory reserve |
|------|-----------|-------------|--------------|----------------|
| db (PostgreSQL) | 1.5 | 0.5 | 2G | 512M |
| api (Rust/Axum) | 1.5 | 0.25 | 512M | 128M |
| web (nginx) | 0.5 | 0.1 | 128M | 32M |
| gotenberg (PDF) | 1.0 | 0.25 | 512M | 128M |
| db-backup | 0.5 | 0.1 | 256M | 64M |

### 監控服務

| 服務 | CPU limit | Memory limit |
|------|-----------|--------------|
| prometheus | 0.5 | 512M |
| alertmanager | 0.25 | 64M |
| grafana | 0.5 | 256M |
| loki | 0.5 | 256M |
| promtail | 0.25 | 64M |
| watchtower | 0.25 | 64M |

### 資源總計

| 項目 | 全部服務加總 |
|------|-------------|
| CPU limits | ~7.25 核 |
| Memory limits | ~4.5 GB |
| Memory reserves | ~1.3 GB |

---

## 6. PostgreSQL 調優建議

在 docker-compose 中加入 PostgreSQL 參數（以 2G container memory limit 為基準）：

```yaml
db:
  command:
    - "postgres"
    - "-c"
    - "shared_buffers=512MB"
    - "-c"
    - "effective_cache_size=1GB"
    - "-c"
    - "work_mem=16MB"
    - "-c"
    - "maintenance_work_mem=128MB"
    - "-c"
    - "max_connections=50"
    - "-c"
    - "wal_buffers=16MB"
    - "-c"
    - "checkpoint_completion_target=0.9"
```

---

## 7. 磁碟配置建議

```
SSD 1 (512GB NVMe)  →  OS + Docker Engine + Container images
SSD 2 (1TB NVMe)    →  PostgreSQL data volume (postgres_data)
                       Prometheus / Loki 時序資料
HDD/SSD 3 (2TB)     →  備份 (db_backups volume)
                       上傳檔案 (uploads volume)
```

docker-compose 中使用 bind mount 指向不同磁碟：

```yaml
volumes:
  postgres_data:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /mnt/data-ssd/postgres   # SSD 2

  db_backups:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /mnt/backup/db-backups   # 磁碟 3
```

---

## 8. 作業系統與可靠性建議

### 作業系統

| 選項 | 推薦度 | 說明 |
|------|--------|------|
| Ubuntu Server 24.04 LTS | 最推薦 | Docker 官方支援最好，5 年安全更新 |
| Debian 12 | 推薦 | 更穩定、更輕量 |
| Windows Server | 不建議 | Docker on Windows 開銷大，效能差 |

### 可靠性必備

| 項目 | 原因 |
|------|------|
| UPS 不斷電系統 | PostgreSQL 寫入中斷電 = 資料損毀風險 |
| ECC RAM（如預算許可） | 長時間運作防記憶體 bit flip |
| SMART 監控 | SSD 壽命預警，配合 Prometheus 監控 |
| 異地備份 | docker-compose.yml 已有 `RSYNC_TARGET` 設定，務必啟用 |

### 網路

- 選有雙 Gigabit LAN 的主機板
  - LAN 1：接內網，提供 ERP 服務
  - LAN 2：管理用 / 備份同步用
- 設定靜態 IP

### 散熱

- 無風扇被動散熱：完全靜音，適合辦公環境，CPU 限制在 35W~65W TDP
- 風扇主動散熱：可用更強 CPU，建議選工業級滾珠軸承風扇（壽命 5 萬小時+）

---

## 參考資料

- [研華 IoTMart 台灣](https://iotmart.advantech.com.tw/)
- [研華 ARK-2251](https://www.advantech.com/en-us/products/ark-2000_series_embedded_box_pcs/ark-2251/mod_de661626-e644-4813-a0f0-be7f006923c1)
- [超恩 ECX-3000](https://www.vecow.com/dispPageBox/vecow/VecowCP.aspx?ddsPageID=ECX-3000_TW)
- [宸曜科技](https://www.neousys-tech.com/tw/)
- [Supermicro eStore](https://store.supermicro.com/)
- [ASUS NUC 13 Pro - PChome](https://24h.pchome.com.tw/store/DSAUEW)
- [ASUS NUC 13 Pro 比價 - BigGo](https://biggo.com.tw/s/Intel+NUC+13+Pro/)
- [MSI Cubi 系列 - MSI 旗艦館](https://tw-store.msi.com/collections/dt-cubi)
- [Supermicro - PChome](https://24h.pchome.com.tw/store/DSAM5J)
